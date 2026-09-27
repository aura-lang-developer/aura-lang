package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"
)

type BenchmarkResult struct {
	Name       string  `json:"name"`
	Category   string  `json:"category"`
	AuraTimeMs float64 `json:"aura_time_ms"`
	GoTimeMs   float64 `json:"go_time_ms"`
	AuraRssMB  float64 `json:"aura_rss_mb"`
	GoRssMB    float64 `json:"go_rss_mb"`
	SpeedupGo  float64 `json:"speedup_go"`
	Notes      string  `json:"notes"`
}

type BinarySizeResult struct {
	Name             string  `json:"name"`
	GoSizeMB         float64 `json:"go_size_mb"`
	GoStrippedSizeMB float64 `json:"go_stripped_size_mb"`
	AuraSizeMB       float64 `json:"aura_size_mb"`
	RatioAuraToGo    float64 `json:"ratio_aura_to_go"`
}

type BuildTimeResult struct {
	Name           string  `json:"name"`
	AuracCheckMs   float64 `json:"aurac_check_ms"`
	AuracCompileMs float64 `json:"aurac_compile_ms"`
	AuracBuildMs   float64 `json:"aurac_build_ms"`
	GoBuildMs      float64 `json:"go_build_ms"`
}

type HttpBenchResult struct {
	Target        string  `json:"target"`
	TotalReqs     int     `json:"total_reqs"`
	Concurrency   int     `json:"concurrency"`
	DurationSec   float64 `json:"duration_sec"`
	ReqPerSec     float64 `json:"req_per_sec"`
	LatencyMeanMs float64 `json:"latency_mean_ms"`
	LatencyP50Ms  float64 `json:"latency_p50_ms"`
	LatencyP90Ms  float64 `json:"latency_p90_ms"`
	LatencyP95Ms  float64 `json:"latency_p95_ms"`
	LatencyP99Ms  float64 `json:"latency_p99_ms"`
	MemoryRssMB   float64 `json:"memory_rss_mb"`
}

type FullReport struct {
	SystemInfo    string             `json:"system_info"`
	GeneratedAt   string             `json:"generated_at"`
	BinarySizes   []BinarySizeResult `json:"binary_sizes"`
	BuildTimes    []BuildTimeResult  `json:"build_times"`
	Benchmarks    []BenchmarkResult  `json:"benchmarks"`
	HttpBenchmark []HttpBenchResult  `json:"http_benchmark"`
}

func runCmd(name string, args ...string) (string, error) {
	cmd := exec.Command(name, args...)
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	err := cmd.Run()
	if err != nil {
		return stdout.String() + stderr.String(), fmt.Errorf("%w: %s", err, stderr.String())
	}
	return stdout.String(), nil
}

func measureRss(cmdPath string, args ...string) (float64, error) {
	cmdArgs := append([]string{"-l", cmdPath}, args...)
	cmd := exec.Command("/usr/bin/time", cmdArgs...)
	var stderr bytes.Buffer
	cmd.Stderr = &stderr
	_ = cmd.Run()

	lines := strings.Split(stderr.String(), "\n")
	for _, l := range lines {
		if strings.Contains(l, "maximum resident set size") {
			parts := strings.Fields(l)
			if len(parts) > 0 {
				bytesVal, err := strconv.ParseFloat(parts[0], 64)
				if err == nil {
					return bytesVal / (1024.0 * 1024.0), nil
				}
			}
		}
	}
	return 0.0, nil
}

func measureDuration(fn func()) float64 {
	start := time.Now()
	fn()
	return float64(time.Since(start).Microseconds()) / 1000.0
}

func getFileSizeMB(path string) float64 {
	fi, err := os.Stat(path)
	if err != nil {
		return 0.0
	}
	return float64(fi.Size()) / (1024.0 * 1024.0)
}

func benchmarkHttp(url string, totalRequests int, concurrency int) (HttpBenchResult, error) {
	client := &http.Client{
		Transport: &http.Transport{
			MaxIdleConns:        1000,
			MaxIdleConnsPerHost: 1000,
			IdleConnTimeout:     30 * time.Second,
		},
		Timeout: 5 * time.Second,
	}

	// Warmup
	for i := 0; i < 50; i++ {
		resp, err := client.Get(url)
		if err == nil {
			io.Copy(io.Discard, resp.Body)
			resp.Body.Close()
		}
	}

	reqsPerWorker := totalRequests / concurrency
	latencies := make([]float64, 0, totalRequests)
	var latMutex sync.Mutex

	start := time.Now()
	var wg sync.WaitGroup

	for c := 0; c < concurrency; c++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			localLats := make([]float64, 0, reqsPerWorker)
			for i := 0; i < reqsPerWorker; i++ {
				t0 := time.Now()
				resp, err := client.Get(url)
				dur := float64(time.Since(t0).Microseconds()) / 1000.0
				if err == nil {
					io.Copy(io.Discard, resp.Body)
					resp.Body.Close()
					localLats = append(localLats, dur)
				}
			}
			latMutex.Lock()
			latencies = append(latencies, localLats...)
			latMutex.Unlock()
		}()
	}

	wg.Wait()
	totalDuration := time.Since(start).Seconds()

	validCount := len(latencies)
	if validCount == 0 {
		return HttpBenchResult{}, fmt.Errorf("no successful requests")
	}

	sort.Float64s(latencies)
	sum := 0.0
	for _, l := range latencies {
		sum += l
	}
	mean := sum / float64(validCount)
	p50 := latencies[int(float64(validCount)*0.50)]
	p90 := latencies[int(float64(validCount)*0.90)]
	p95 := latencies[int(float64(validCount)*0.95)]
	p99 := latencies[int(float64(validCount)*0.99)]

	rps := float64(validCount) / totalDuration

	return HttpBenchResult{
		TotalReqs:     validCount,
		Concurrency:   concurrency,
		DurationSec:   totalDuration,
		ReqPerSec:     rps,
		LatencyMeanMs: mean,
		LatencyP50Ms:  p50,
		LatencyP90Ms:  p90,
		LatencyP95Ms:  p95,
		LatencyP99Ms:  p99,
	}, nil
}

func main() {
	fmt.Println("================================================================================")
	fmt.Println("🚀 SUITE DE BENCHMARKS COMPARATIVA EXHAUSTIVA: AURA LANG vs GO LANG")
	fmt.Println("================================================================================")

	sysInfo, _ := runCmd("uname", "-sm")
	sysInfo = strings.TrimSpace(sysInfo)
	goVer, _ := runCmd("go", "version")
	goVer = strings.TrimSpace(goVer)
	fmt.Printf("Entorno: %s | Compilador Go: %s\n\n", sysInfo, goVer)

	scenarios := []struct {
		Name     string
		Dir      string
		AuraSrc  string
		GoSrc    string
		AuraBin  string
		GoBin    string
		Category string
		Runs     int
	}{
		{
			Name:     "1. Fibonacci Recursivo (Fib 38 - CPU & Call Stack)",
			Dir:      "benchmarks/fibonacci",
			AuraSrc:  "fib.aura",
			GoSrc:    "fib.go",
			AuraBin:  "fib_aura",
			GoBin:    "fib_go",
			Category: "CPU / Recursión",
			Runs:     5,
		},
		{
			Name:     "2. Criba de Eratóstenes (2,000,000 Primos - Memoria y Bucles)",
			Dir:      "benchmarks/primes",
			AuraSrc:  "primes.aura",
			GoSrc:    "primes.go",
			AuraBin:  "primes_aura",
			GoBin:    "primes_go",
			Category: "Cálculo Numérico / Arrays",
			Runs:     5,
		},
		{
			Name:     "3. Pipeline de Datos Funcional (1,000,000 Elementos: Filter + Map + Sum)",
			Dir:      "benchmarks/data_pipeline",
			AuraSrc:  "pipeline.aura",
			GoSrc:    "pipeline.go",
			AuraBin:  "pipeline_aura",
			GoBin:    "pipeline_go",
			Category: "Procesamiento de Colecciones",
			Runs:     5,
		},
		{
			Name:     "4. Concurrencia CSP: Channel Ping-Pong (200,000 Mensajes)",
			Dir:      "benchmarks/concurrency_channels",
			AuraSrc:  "channels.aura",
			GoSrc:    "channels.go",
			AuraBin:  "channels_aura",
			GoBin:    "channels_go",
			Category: "Concurrencia CSP / Canales",
			Runs:     5,
		},
		{
			Name:     "5. Spawn Masivo de Tareas (50,000 Fibers vs Goroutines)",
			Dir:      "benchmarks/concurrency_spawn",
			AuraSrc:  "spawn.aura",
			GoSrc:    "spawn.go",
			AuraBin:  "spawn_aura",
			GoBin:    "spawn_go",
			Category: "Concurrencia / Dispatch",
			Runs:     5,
		},
		{
			Name:     "6. Latencia Cold-Start (Inicio de Proceso a Salida)",
			Dir:      "benchmarks/startup",
			AuraSrc:  "startup.aura",
			GoSrc:    "startup.go",
			AuraBin:  "startup_aura",
			GoBin:    "startup_go",
			Category: "Tiempo de Arranque (CLI)",
			Runs:     10,
		},
		{
			Name:     "7. Sincronización Mutex (50,000 Operaciones Atómicas)",
			Dir:      "benchmarks/sync_mutex",
			AuraSrc:  "mutex.aura",
			GoSrc:    "mutex.go",
			AuraBin:  "mutex_aura",
			GoBin:    "mutex_go",
			Category: "Sincronización / Mutex",
			Runs:     5,
		},
		{
			Name:     "8. Almacenamiento KV Cache (50,000 Consultas Concurrentes)",
			Dir:      "benchmarks/kv_store",
			AuraSrc:  "kv.aura",
			GoSrc:    "kv.go",
			AuraBin:  "kv_aura",
			GoBin:    "kv_go",
			Category: "Estructuras de Datos / Cache",
			Runs:     5,
		},
	}

	var report FullReport
	report.SystemInfo = fmt.Sprintf("%s (%s)", sysInfo, goVer)
	report.GeneratedAt = time.Now().Format(time.RFC3339)

	fmt.Println("--------------------------------------------------------------------------------")
	fmt.Println("📦 FASE 1: MEDICIÓN DE PESOS DE BINARIOS Y TIEMPOS DE COMPILACIÓN")
	fmt.Println("--------------------------------------------------------------------------------")

	for _, sc := range scenarios {
		auraPath := filepath.Join(sc.Dir, sc.AuraSrc)
		goPath := filepath.Join(sc.Dir, sc.GoSrc)
		auraBin := filepath.Join(sc.Dir, sc.AuraBin)
		goBin := filepath.Join(sc.Dir, sc.GoBin)
		goStrippedBin := filepath.Join(sc.Dir, sc.GoBin+"_stripped")

		var checkMs, compileMs, auraBuildMs, goBuildMs float64

		checkMs = measureDuration(func() {
			_, _ = runCmd("aurac", "check", auraPath)
		})

		compileMs = measureDuration(func() {
			_, _ = runCmd("aurac", "compile", auraPath, "-o", filepath.Join(sc.Dir, "temp.js"))
		})
		_ = os.Remove(filepath.Join(sc.Dir, "temp.js"))
		_ = os.Remove(filepath.Join(sc.Dir, "temp.d.ts"))
		_ = os.Remove(filepath.Join(sc.Dir, "temp.js.map"))

		auraBuildMs = measureDuration(func() {
			_, _ = runCmd("aurac", "build", auraPath, "-o", auraBin)
		})

		goBuildMs = measureDuration(func() {
			_, _ = runCmd("go", "build", "-o", goBin, goPath)
		})

		_, _ = runCmd("go", "build", "-ldflags=-s -w", "-o", goStrippedBin, goPath)

		goSize := getFileSizeMB(goBin)
		goStrippedSize := getFileSizeMB(goStrippedBin)
		auraSize := getFileSizeMB(auraBin)

		ratio := 0.0
		if goSize > 0 {
			ratio = auraSize / goSize
		}

		bSize := BinarySizeResult{
			Name:             sc.Name,
			GoSizeMB:         goSize,
			GoStrippedSizeMB: goStrippedSize,
			AuraSizeMB:       auraSize,
			RatioAuraToGo:    ratio,
		}
		report.BinarySizes = append(report.BinarySizes, bSize)

		bTime := BuildTimeResult{
			Name:           sc.Name,
			AuracCheckMs:   checkMs,
			AuracCompileMs: compileMs,
			AuracBuildMs:   auraBuildMs,
			GoBuildMs:      goBuildMs,
		}
		report.BuildTimes = append(report.BuildTimes, bTime)

		fmt.Printf("• %s:\n", sc.Name)
		fmt.Printf("    Binarios: Go: %.2f MB (Stripped: %.2f MB) | Aura Standalone: %.2f MB (%.1fx Go)\n", goSize, goStrippedSize, auraSize, ratio)
		fmt.Printf("    Compilación: aurac check: %.1f ms | aurac build: %.1f ms | go build: %.1f ms\n", checkMs, auraBuildMs, goBuildMs)
	}

	{
		httpAuraBin := "benchmarks/http_server/server_aura"
		httpGoBin := "benchmarks/http_server/server_go"
		httpGoStrippedBin := "benchmarks/http_server/server_go_stripped"
		_, _ = runCmd("aurac", "build", "benchmarks/http_server/server.aura", "-o", httpAuraBin)
		_, _ = runCmd("go", "build", "-o", httpGoBin, "benchmarks/http_server/server.go")
		_, _ = runCmd("go", "build", "-ldflags=-s -w", "-o", httpGoStrippedBin, "benchmarks/http_server/server.go")
		goSize := getFileSizeMB(httpGoBin)
		goStripped := getFileSizeMB(httpGoStrippedBin)
		auraSize := getFileSizeMB(httpAuraBin)
		ratio := 0.0
		if goSize > 0 {
			ratio = auraSize / goSize
		}
		report.BinarySizes = append(report.BinarySizes, BinarySizeResult{
			Name:             "9. Servidor HTTP / Microservicio REST",
			GoSizeMB:         goSize,
			GoStrippedSizeMB: goStripped,
			AuraSizeMB:       auraSize,
			RatioAuraToGo:    ratio,
		})
		fmt.Printf("• %s:\n", "9. Servidor HTTP / Microservicio REST")
		fmt.Printf("    Binarios: Go: %.2f MB (Stripped: %.2f MB) | Aura Standalone: %.2f MB (%.1fx Go)\n", goSize, goStripped, auraSize, ratio)
	}

	fmt.Println("\n--------------------------------------------------------------------------------")
	fmt.Println("⚡ FASE 2: RENDIMIENTO EN EJECUCIÓN Y CONSUMO DE MEMORIA (RSS)")
	fmt.Println("--------------------------------------------------------------------------------")

	for _, sc := range scenarios {
		auraBin := filepath.Join(sc.Dir, sc.AuraBin)
		goBin := filepath.Join(sc.Dir, sc.GoBin)

		var auraTimes []float64
		for i := 0; i < sc.Runs; i++ {
			t := measureDuration(func() {
				_, _ = runCmd("./" + auraBin)
			})
			auraTimes = append(auraTimes, t)
		}
		sort.Float64s(auraTimes)
		auraMedian := auraTimes[len(auraTimes)/2]
		auraRss, _ := measureRss("./" + auraBin)

		var goTimes []float64
		for i := 0; i < sc.Runs; i++ {
			t := measureDuration(func() {
				_, _ = runCmd("./" + goBin)
			})
			goTimes = append(goTimes, t)
		}
		sort.Float64s(goTimes)
		goMedian := goTimes[len(goTimes)/2]
		goRss, _ := measureRss("./" + goBin)

		speedup := 0.0
		if goMedian > 0 {
			speedup = auraMedian / goMedian
		}

		res := BenchmarkResult{
			Name:       sc.Name,
			Category:   sc.Category,
			AuraTimeMs: auraMedian,
			GoTimeMs:   goMedian,
			AuraRssMB:  auraRss,
			GoRssMB:    goRss,
			SpeedupGo:  speedup,
		}
		report.Benchmarks = append(report.Benchmarks, res)

		fmt.Printf("• %s:\n", sc.Name)
		fmt.Printf("    Tiempo: Go: %.2f ms | Aura Standalone: %.2f ms | Ratio (Aura/Go): %.2fx\n", goMedian, auraMedian, speedup)
		fmt.Printf("    Memoria Peak RSS: Go: %.2f MB | Aura Standalone: %.2f MB\n", goRss, auraRss)
	}

	fmt.Println("\n--------------------------------------------------------------------------------")
	fmt.Println("🌐 FASE 3: SERVIDOR HTTP / MICROSERVICIOS (10,000 REQUESTS, CONCURRENCIA 50)")
	fmt.Println("--------------------------------------------------------------------------------")

	totalHttpReqs := 10000
	httpConcurrency := 50

	// 1. Bench Aura HTTP Server
	fmt.Println("Arrancando Servidor HTTP de Aura en :8091...")
	auraServerCmd := exec.Command("./benchmarks/http_server/server_aura")
	auraServerCmd.Env = append(os.Environ(), "PORT=8091")
	_ = auraServerCmd.Start()
	time.Sleep(1 * time.Second)

	auraHttpBench, err := benchmarkHttp("http://localhost:8091/bench", totalHttpReqs, httpConcurrency)
	if err != nil {
		fmt.Printf("Error testeando Aura HTTP server: %v\n", err)
	} else {
		auraHttpBench.Target = "Aura Standalone Server (:8091)"
		report.HttpBenchmark = append(report.HttpBenchmark, auraHttpBench)
		fmt.Printf("Aura HTTP: %.1f req/sec | Latencia Media: %.2f ms | p50: %.2f ms | p99: %.2f ms\n",
			auraHttpBench.ReqPerSec, auraHttpBench.LatencyMeanMs, auraHttpBench.LatencyP50Ms, auraHttpBench.LatencyP99Ms)
	}
	_ = auraServerCmd.Process.Kill()
	_ = auraServerCmd.Wait()

	// 2. Bench Go HTTP Server
	fmt.Println("Arrancando Servidor HTTP de Go en :8092...")
	goServerCmd := exec.Command("./benchmarks/http_server/server_go")
	goServerCmd.Env = append(os.Environ(), "PORT=8092")
	_ = goServerCmd.Start()
	time.Sleep(1 * time.Second)

	goHttpBench, err := benchmarkHttp("http://localhost:8092/bench", totalHttpReqs, httpConcurrency)
	if err != nil {
		fmt.Printf("Error testeando Go HTTP server: %v\n", err)
	} else {
		goHttpBench.Target = "Go Native Server (:8092)"
		report.HttpBenchmark = append(report.HttpBenchmark, goHttpBench)
		fmt.Printf("Go HTTP:   %.1f req/sec | Latencia Media: %.2f ms | p50: %.2f ms | p99: %.2f ms\n",
			goHttpBench.ReqPerSec, goHttpBench.LatencyMeanMs, goHttpBench.LatencyP50Ms, goHttpBench.LatencyP99Ms)
	}
	_ = goServerCmd.Process.Kill()
	_ = goServerCmd.Wait()

	data, _ := json.MarshalIndent(report, "", "  ")
	_ = os.WriteFile("benchmarks/results.json", data, 0644)
	fmt.Println("\n✨ Resultados guardados en 'benchmarks/results.json' exitosamente.")
}
