# ⚡ Comparative Benchmark Suite: Aura Lang vs Golang

This directory contains an automated, rigorous benchmark suite that performs end-to-end performance comparisons between **Aura Lang (`aurac`)** and **Golang (`go`)**.

---

## 🎯 Evaluated Scenarios

The suite evaluates critical aspects of backend development, concurrency, and systems programming:

1. **CPU & Deep Recursion (`fibonacci/`)**:
   - Recursive Fibonacci calculation ($N = 38$).
   - Evaluates stack frame overhead, function calls, and arithmetic optimization via Cranelift.
2. **Numerical Calculation & Arrays (`primes/`)**:
   - Sieve of Eratosthenes up to 2,000,000 prime numbers.
   - Evaluates fast array indexing, buffer mutability, and `while`/`for` loops.
3. **Functional Data Pipeline (`data_pipeline/`)**:
   - Transformation of 1,000,000 elements: `filter(evens) |> map(x * 3) |> sum()`.
   - Evaluates intermediate memory allocations, closures, and chained iteration.
4. **CSP Ping-Pong Concurrency (`concurrency_channels/`)**:
   - Coordinated bidirectional exchange of 200,000 messages between two tasks using channels (`Channel`).
   - Evaluates synchronization overhead, fiber scheduler vs goroutines, and channel latency.
5. **Massive Task Spawning (`concurrency_spawn/`)**:
   - Creation and synchronization of 50,000 fibers vs goroutines with `WaitGroup`.
   - Evaluates lightweight worker throughput and memory consumption.
6. **Cold-Start Latency (`startup/`)**:
   - Full process lifecycle from OS process spawn to clean exit.
   - Critical for Serverless functions, micro-lambdas, and CLI tools.
7. **Mutex Synchronization & Contention (`sync_mutex/`)**:
   - 50,000 atomic increment operations synchronized with `Mutex.new()`.
   - Evaluates lock contention, mutual exclusion, and atomicity under concurrency.
8. **Concurrent In-Memory KV Cache (`kv_store/`)**:
   - 50,000 concurrent queries (reads and writes) on an in-memory lock-protected key-value store.
   - Evaluates safe concurrent access and mutation of shared data structures.
9. **HTTP Server & REST Microservice (`http_server/`)**:
   - Load test with 10,000 HTTP requests at concurrency 50 against a JSON endpoint.
   - Metrics include throughput (req/s) and latency percentiles (p50, p90, p95, p99).

---

## 📊 Results Summary (Darwin arm64 / Apple Silicon)

> **Environment**: macOS Darwin arm64 | Go Compiler: `go1.27.2` | Aura Compiler: `aurac v0.6.0` (Cranelift Backend & Go Toolchain)

### 1. Execution Times and Memory Consumption (Peak RSS)

| Scenario | Aura Lang (ms) | Golang (ms) | Ratio (Aura/Go) | Aura RSS (MB) | Go RSS (MB) | Status / Aura Advantage |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **1. Fibonacci (Fib 38)** | **149.18 ms** | 116.49 ms | 1.28x *(improved from 1.32x)* | **2.83 MB** | 4.28 MB | 🏆 **-34.0% less RAM** |
| **2. Prime Sieve (2M)** | 16.25 ms | 9.25 ms | 1.76x | 18.14 MB | 6.28 MB | Optimized with SIMD `Array.repeat` |
| **3. Data Pipeline (1M)** | 13.01 ms | 8.39 ms | 1.55x | 20.28 MB | 20.42 MB | Parity with Go RAM consumption |
| **4. CSP Channels (200k)** | **29.57 ms** | 25.41 ms | **1.16x (Near Parity)** | 9.17 MB | 4.36 MB | Optimized fiber stack cache pool |
| **5. Spawn 50k Tasks** | 15.73 ms | 11.76 ms | 1.34x | 11.11 MB | 6.39 MB | Scheduler O(1) + 8192 Stack Pool |
| **6. Cold-Start (CLI)** | **6.11 ms** | 5.88 ms | **1.04x (Full Parity)** | **2.80 MB** | 4.25 MB | 🏆 **-34.1% RAM (2.80 MB vs 4.25 MB)** |
| **7. Mutex Sync (50k)** | 19.98 ms | 13.97 ms | 1.43x | 13.30 MB | 7.72 MB | Atomic fast-path |
| **8. Concurrent KV Cache (50k)**| **18.39 ms** | 18.14 ms | **1.01x (Absolute Parity)** | 15.42 MB | 10.27 MB | 🚀 **Absolute execution parity** |

---

### 2. Standalone Binary Sizes (Zero Dependencies)

| Scenario | Go Binary (MB) | Go Stripped (`-s -w`) | Aura Standalone Binary | Ratio Aura vs Go |
|---|:---:|:---:|:---:|:---:|
| **Fibonacci (Cranelift)** | 2.32 MB | 1.51 MB | **0.50 MB** | 🏆 **0.22x (Aura 67% smaller than stripped)** |
| **Prime Sieve (Cranelift)** | 2.32 MB | 1.51 MB | **0.50 MB** | 🏆 **0.22x (Aura 67% smaller than stripped)** |
| **Data Pipeline (Cranelift)** | 2.32 MB | 1.51 MB | **0.50 MB** | 🏆 **0.22x (Aura 67% smaller than stripped)** |
| **CSP Channels** | 2.33 MB | 1.53 MB | **3.40 MB** | 1.46x *(improved from 5.07 MB)* |
| **Concurrent Spawn** | 2.33 MB | 1.53 MB | **3.40 MB** | 1.46x *(improved from 5.07 MB)* |
| **Cold Start (Cranelift)** | 2.32 MB | 1.51 MB | **0.49 MB** | 🏆 **0.21x (Aura 68% smaller than stripped)** |
| **Mutex Synchronization** | 2.33 MB | 1.53 MB | **3.40 MB** | 1.46x *(improved from 5.07 MB)* |
| **Concurrent KV Cache** | 2.33 MB | 1.53 MB | **3.40 MB** | 1.46x *(improved from 5.07 MB)* |
| **HTTP REST Server** | 8.85 MB | 5.96 MB | **0.57 MB** | 🏆 **0.06x (Aura 15.5x more compact / -93.6%)** |

---

### 3. Compilation and Static Typecheck Times

| Scenario | `aurac check` (HM Typecheck) | `aurac build` (Standalone) | `go build` |
|---|:---:|:---:|:---:|
| **Fibonacci** | **12.8 ms** | **52.4 ms** | 95.6 ms |
| **Prime Sieve** | **9.2 ms** | **43.7 ms** | 86.4 ms |
| **Data Pipeline** | **7.4 ms** | **45.3 ms** | 87.5 ms |
| **CSP Channels** | **7.8 ms** | **206.7 ms** | 86.3 ms |
| **Concurrent Spawn** | **7.5 ms** | **165.1 ms** | 86.1 ms |
| **Cold Start** | **9.7 ms** | **43.1 ms** | 86.6 ms |
| **Mutex Synchronization** | **7.5 ms** | **163.9 ms** | 86.6 ms |
| **Concurrent KV Cache** | **8.8 ms** | **167.6 ms** | 86.4 ms |

---

### 4. HTTP Server / REST Microservice (10,000 requests, Concurrency = 50)

| Metric | Aura Standalone Server | Go Native Server (`net/http`) | Winner / Advantage |
|---|:---:|:---:|:---:|
| **Throughput (req/s)** | **96,092 req/s** | 88,744 req/s | 🏆 **Aura higher throughput (+8.3%)** |
| **Mean Latency** | **0.48 ms** | 0.51 ms | 🏆 **Aura lower mean latency** |
| **Median Latency (p50)** | **0.38 ms** | 0.42 ms | 🏆 **Aura faster median (0.38 ms vs 0.42 ms)** |
| **99th Percentile Latency (p99)** | **1.50 ms** | 1.72 ms | 🏆 **Aura lower tail latency** |
| **Binary Size** | **0.57 MB** | 8.85 MB (5.96 MB stripped) | 🏆 **Aura 15.5x more compact (-93.6% disk)** |

---

## 🔍 Key Takeaways

1. **High-Performance Worker-Pool HTTP Server**:
   - The native HTTP server operates with a pre-spawned pool of 256 workers and keep-alive recycling, achieving **96,092 req/s** with **0.48 ms mean latency**, outperforming Go's `net/http` (**88,744 req/s**) while producing a binary of just **0.57 MB** (compared to **8.85 MB** in Go).
2. **Sub-Megabyte Standalone Binaries**:
   - Native binaries compiled with Cranelift (`aurac build`) weigh just **0.49 – 0.57 MB** due to default dead-stripping (`-Wl,-dead_strip` on macOS, `-Wl,--gc-sections` on Linux) and symbol stripping, making them **67% smaller than even stripped Go binaries**.
3. **Concurrency Scalability via Stack Recycling**:
   - Leveraging an expanded pool of up to 8,192 cached stacks per fiber (16 KB on Apple Silicon, 8 KB on x86_64) and an O(1) hash table scheduler, Aura executes **50,000 concurrent tasks in 15.73 ms** without recurring `mmap`/`munmap` syscall overhead.
4. **Near Parity in CSP Channels and Parity in Concurrent KV Cache**:
   - CSP channel ping-pong executes 200,000 messages in **29.57 ms vs 25.41 ms** in Go.
   - The concurrent KV Cache benchmark achieves **18.39 ms vs 18.14 ms** (1.01x parity).
5. **Fast Compilation**:
   - `aurac check` validates types in **7.4 – 12.8 ms**, and standalone binary builds take only **43 – 52 ms** for numerical and pipeline tasks (nearly 2x faster than `go build`).

---

## 🚀 How to Run the Benchmark Suite

To reproduce all benchmarks on your machine:

```bash
# Run the complete suite and update results.json:
go run benchmarks/benchmark_suite.go

# Or execute the compiled binary:
./benchmarks/benchmark_suite
```
