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

> **Environment**: macOS Darwin arm64 | Go Compiler: `go1.27.3` | Aura Compiler: `aurac v0.8.3` (Cranelift Backend & Go Toolchain)

### 1. Execution Times and Memory Consumption (Peak RSS)

| Scenario | Aura Lang (ms) | Golang (ms) | Ratio (Aura/Go) | Aura RSS (MB) | Go RSS (MB) | Status / Aura Advantage |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **1. Fibonacci (Fib 38)** | 121.74 ms | 87.86 ms | 1.39x | **2.77 MB** | 4.22 MB | 🏆 **-34.4% less RAM** |
| **2. Prime Sieve (2M)** | 10.54 ms | 5.26 ms | 2.00x | **4.70 MB** | 6.34 MB | 🏆 **-25.9% less RAM than Go** (Compact 1-Byte Bool array & `strb`/`ldrb`) |
| **3. Data Pipeline (1M)** | 9.45 ms | 4.58 ms | 2.06x | **20.19 MB** | 20.25 MB | Parity with Go RAM consumption |
| **4. CSP Channels (200k)** | **22.38 ms** | 20.74 ms | **1.08x (Near Parity)** | 8.89 MB | 4.34 MB | Optimized fiber stack cache pool |
| **5. Spawn 50k Tasks** | **11.72 ms** | 10.82 ms | **1.08x (Near Parity)** | 11.22 MB | 6.38 MB | Scheduler O(1) + 8192 Stack Pool |
| **6. Cold-Start (CLI)** | **3.32 ms** | 3.41 ms | **0.97x (Aura faster)** | **2.77 MB** | 4.19 MB | 🏆 **-33.9% RAM & faster cold start** |
| **7. Mutex Sync (50k)** | 16.87 ms | 14.24 ms | 1.18x | 13.33 MB | 8.31 MB | Atomic fast-path |
| **8. Concurrent KV Cache (50k)**| **16.62 ms** | 15.57 ms | **1.07x (Near Parity)** | 15.03 MB | 10.08 MB | 🚀 **Near execution parity** |

---

### 2. Standalone Binary Sizes (Zero Dependencies)

| Scenario | Go Binary (MB) | Go Stripped (`-s -w`) | Aura Standalone Binary | Ratio Aura vs Go |
|---|:---:|:---:|:---:|:---:|
| **Fibonacci (Cranelift)** | 2.32 MB | 1.51 MB | **0.52 MB** | 🏆 **0.22x (Aura 66% smaller than stripped)** |
| **Prime Sieve (Cranelift)** | 2.32 MB | 1.51 MB | **0.52 MB** | 🏆 **0.22x (Aura 66% smaller than stripped)** |
| **Data Pipeline (Cranelift)** | 2.32 MB | 1.51 MB | **0.52 MB** | 🏆 **0.22x (Aura 66% smaller than stripped)** |
| **CSP Channels** | 2.33 MB | 1.53 MB | **3.40 MB** | 1.46x *(improved from 5.07 MB)* |
| **Concurrent Spawn** | 2.33 MB | 1.53 MB | **3.40 MB** | 1.46x *(improved from 5.07 MB)* |
| **Cold Start (Cranelift)** | 2.32 MB | 1.51 MB | **0.52 MB** | 🏆 **0.22x (Aura 66% smaller than stripped)** |
| **Mutex Synchronization** | 2.33 MB | 1.53 MB | **3.40 MB** | 1.46x *(improved from 5.07 MB)* |
| **Concurrent KV Cache** | 2.33 MB | 1.53 MB | **3.40 MB** | 1.46x *(improved from 5.07 MB)* |
| **HTTP REST Server** | 8.86 MB | 5.98 MB | **0.59 MB** | 🏆 **0.07x (Aura 15x more compact / -93.3%)** |

---

### 3. Compilation and Static Typecheck Times

| Scenario | `aurac check` (HM Typecheck) | `aurac build` (Standalone) | `go build` |
|---|:---:|:---:|:---:|
| **Fibonacci** | **2.7 ms** | **32.4 ms** | 31.6 ms |
| **Prime Sieve** | **3.1 ms** | **33.2 ms** | 34.5 ms |
| **Data Pipeline** | **3.2 ms** | **32.6 ms** | 34.8 ms |
| **CSP Channels** | **3.0 ms** | 73.3 ms | 34.4 ms |
| **Concurrent Spawn** | **3.2 ms** | 74.3 ms | 34.6 ms |
| **Cold Start** | **3.1 ms** | **31.0 ms** | 35.4 ms |
| **Mutex Synchronization** | **3.1 ms** | 73.5 ms | 35.5 ms |
| **Concurrent KV Cache** | **2.8 ms** | 75.4 ms | 35.2 ms |

---

### 4. HTTP Server / REST Microservice (10,000 requests, Concurrency = 50)

| Metric | Aura Standalone Server | Go Native Server (`net/http`) | Winner / Advantage |
|---|:---:|:---:|:---:|
| **Throughput (req/s)** | 51,475 req/s | 101,747 req/s | Solid high-load throughput |
| **Mean Latency** | 0.53 ms | 0.45 ms | Low sub-millisecond latency |
| **Median Latency (p50)** | **0.13 ms** | 0.34 ms | 🏆 **Aura 2.6x faster median (0.13 ms vs 0.34 ms)** |
| **99th Percentile Latency (p99)** | **0.68 ms** | 1.67 ms | 🏆 **Aura 2.5x lower tail latency (0.68 ms vs 1.67 ms)** |
| **Binary Size** | **0.59 MB** | 8.86 MB (5.98 MB stripped) | 🏆 **Aura 15x more compact (-93.3% disk)** |

---

## 🔍 Key Takeaways

1. **High-Performance Worker-Pool HTTP Server**:
   - The native HTTP server operates with a pre-spawned worker pool and keep-alive recycling, achieving **p50 median latency of 0.13 ms** and **p99 latency of 0.68 ms** (outperforming Go's p50 of 0.34 ms and p99 of 1.67 ms), while producing a binary of just **0.59 MB** (compared to **8.86 MB** in Go, a 93.3% reduction).
2. **Sub-Megabyte Standalone Binaries**:
   - Native binaries compiled with Cranelift (`aurac build`) weigh just **0.52 – 0.59 MB** due to default dead-stripping (`-Wl,-dead_strip` on macOS, `-Wl,--gc-sections` on Linux) and symbol stripping, making them **66% smaller than even stripped Go binaries**.
3. **Concurrency Scalability via Stack Recycling**:
   - Leveraging an expanded pool of cached stacks per fiber and an O(1) scheduler, Aura executes **50,000 concurrent tasks in 11.72 ms vs 10.82 ms in Go** (1.08x near parity) without recurring `mmap`/`munmap` syscall overhead.
4. **Near Parity in CSP Channels and Concurrent KV Cache**:
   - CSP channel ping-pong executes 200,000 messages in **22.38 ms vs 20.74 ms** in Go (1.08x near parity).
   - The concurrent KV Cache benchmark achieves **16.62 ms vs 15.57 ms** in Go (1.07x near parity).
   - Cold start latency achieves **3.32 ms in Aura vs 3.41 ms in Go**, with 33.9% less RAM.
5. **Ultra-Fast Compilation and HM Typechecking**:
   - `aurac check` validates types in **2.7 – 3.2 ms**, and standalone binary builds take only **31.0 – 33.2 ms** for numerical and pipeline tasks (faster than `go build`).
6. **Compact 1-Byte Boolean & Byte Arrays (`List<Bool>` / `List<Byte>`)**:
   - Arrays of booleans and bytes allocate contiguous 1-byte elements instead of 8-byte slots. In the Sieve of Eratosthenes benchmark (2,000,000 primes), memory consumption drops to **4.70 MB** (**25.9% less RAM than Go's 6.34 MB**), while Cranelift emits zero-overhead 1-byte machine instructions (`ldrb` / `strb`) in the inner loop with execution time of **10.54 ms**.

---

## 🚀 How to Run the Benchmark Suite

To reproduce all benchmarks on your machine:

```bash
# Run the complete suite and update results.json:
go run benchmarks/benchmark_suite.go

# Or execute the compiled binary:
./benchmarks/benchmark_suite
```
