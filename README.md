> 🌍 **Languages / Idiomas:** **English** | [Español](README-ES.md)

# 🌟 Aura Language (`aurac`)

[![Rust CI](https://img.shields.io/badge/Rust_CI-250_tests_passing-brightgreen.svg)](tests)
[![Compiler Speed](https://img.shields.io/badge/Speed-850k%2B_LOC%2Fs-blue.svg)#-benchmarks-and-empirical-performance)
[![Native Backend](<https://img.shields.io/badge/Backend-Cranelift_Native_(Mach--O_/_ELF)-orange.svg>)#-compiler-and-runtime-architecture)
[![Go Backend](https://img.shields.io/badge/Backend-Golang_Toolchain_Transpiler-blue.svg)#-native-backend-and-golang-model)
[![Concurrency](https://img.shields.io/badge/Concurrency-Go--style_CSP_Fibers_%26_Channels-purple.svg)#8-csp-concurrency-go-style-model)
[![Self-Hosted](https://img.shields.io/badge/Self--Hosted-Stage_1_Bootstrapped-success.svg)#-self-hosted-compiler-and-bootstrapping)
[![Spanish Docs](https://img.shields.io/badge/Docs-Espa%C3%B1ol-yellow.svg)](README-ES.md)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Aura** is a high-performance, strongly typed systems and backend services programming language featuring default immutability and native machine code compilation. It harmoniously combines the modern, ergonomic syntax of **TypeScript and Go** with the formal static type safety, algebraic data types (ADTs), exhaustive pattern matching, and robust error handling of **Rust and ML**.

Aura compiles directly into **standalone native executable binaries** (Mach-O on macOS, ELF on Linux) with zero external runtime dependencies thanks to its native [**Cranelift**](https://cranelift.dev/) backend and ultra-lightweight M:N runtime (`aura-runtime`), or alternatively via direct transpilation targeting the **Golang** toolchain. Its compiler engine written in Rust processes more than **850,000 lines of code per second** with a cold-start time of under **2 ms**.

---

## 📑 Table of Contents

1. [🚀 Quick Start](#-quick-start)
   - [Building the Toolchain with Cargo](#1-building-the-aura-toolchain-with-cargo)
   - [Your First Aura Program](#2-your-first-aura-program)
   - [Compilation and Execution Modes](#3-compilation-and-execution-modes)
2. [🚀 What's New in v0.2.0: Unified Native Database Subsystem & NoSQL Engines](#-whats-new-in-v020-unified-native-database-subsystem--nosql-engines)
3. [✨ Key Features](#-key-features)
4. [⚡ Benchmarks and Empirical Performance](#-benchmarks-and-empirical-performance)
   - [Execution Times and Peak RSS Memory](#1-execution-times-and-memory-consumption-peak-rss)
   - [Standalone Binary Sizes](#2-standalone-binary-sizes-zero-dependencies)
   - [Compilation and Static Checking Times](#3-compilation-and-static-typecheck-times)
   - [HTTP Server and REST Microservice Benchmark](#4-http-server--rest-microservice-10000-requests-concurrency--50)
   - [Compilation Throughput vs TypeScript (`tsc`)](#5-compilation-throughput-vs-typescript-tsc)
   - [How to Reproduce the Benchmark Suite](#6-how-to-run-the-benchmark-suite)
5. [📁 Compiler and Runtime Architecture](#-compiler-and-runtime-architecture)
   - [Pipeline Flow Diagram](#pipeline-flow-diagram)
   - [Dual-Target Strategy: Native Binaries & JavaScript Interoperability](#dual-target-strategy-native-binaries--javascript-interoperability)
   - [Why Rust for the Implementation: Advantages and Trade-offs](#why-rust-for-the-implementation-advantages-and-trade-offs)
   - [Source Code Organization](#source-code-organization)
   - [The Standalone Native Runtime (`aura-runtime`)](#the-standalone-native-runtime-cratesaura-runtime)
6. [📘 Complete Aura Language Guide](#-complete-aura-language-guide)
   - [1. Comprehensive Type System](#1-comprehensive-type-system)
   - [2. Variables, Immutability, and Destructuring](#2-variables-immutability-and-destructuring)
   - [3. Control Flow, Expressions, and Labeled Loops](#3-control-flow-expressions-and-labeled-loops)
   - [4. Slices and Collection Segmentation (Golang Model)](#4-slices-and-collection-segmentation-golang-model)
   - [5. Sum Types (ADTs) and Exhaustive Pattern Matching](#5-sum-types-adts-and-exhaustive-pattern-matching)
   - [6. Error Handling with Result, Option, and `?` Operator](#6-error-handling-with-result-option-and--operator)
   - [7. Pipelines (`|>`) and Tail-Call Optimization (TCO)](#7-pipelines--and-tail-call-optimization-tco)
   - [8. CSP Concurrency (Fibers, Channels, Select, and Sync)](#8-csp-concurrency-go-style-model)
   - [9. Compile-Time Concurrency Safety (`Sendable`) and Deadlock Sentinel](#9-concurrency-safety-sendable-and-deadlock-sentinel)
   - [10. Guaranteed Cleanup: LIFO Defer and Conditional ErrDefer](#10-guaranteed-cleanup-lifo-defer-and-conditional-errdefer-zig)
   - [11. Receiver Methods, Visibility, and Implicit Interfaces](#11-receiver-methods-visibility-and-implicit-interfaces)
   - [12. Explicit Pointers, Packed Structs, and C FFI (`extern "C"`)](#12-explicit-pointers-packed-structs-and-c-ffi-extern-c)
   - [13. Automatic FFI Header Generator (`aurac bindgen`)](#13-automatic-ffi-header-generator-aurac-bindgen)
   - [14. Context and Propagated Cancellation (`Context`)](#14-context-and-propagated-cancellation-context)
   - [15. Generics and Parametric Polymorphism](#15-generics-and-parametric-polymorphism)
   - [16. Compilation Directives, Build Tags, and Struct Tags](#16-compilation-directives-build-tags-and-struct-tags)
   - [17. TypeScript Ingestion (`.d.ts`) and Static Embedding (`embed`)](#17-typescript-ingestion-dts-and-static-asset-embedding-embed)
   - [18. Testing Directives, Assertions, and Benchmarking](#18-testing-directives-assertions-and-benchmarking)
   - [19. Rust-Inspired Features: Derives, Must-Use, Auto-Fix, and Doctests](#19-rust-inspired-features-derives-must-use-auto-fix-and-doctests)
7. [🗄️ Standard Library, Web Server, and Databases](#️-standard-library-web-server-and-databases)
   - [HTTP Web Server & ServeMux Router (`net/http`)](#-http-web-server-and-routing-mux-nethttp)
   - [Prometheus Metrics and W3C Traceparent Tracing](#-prometheus-metrics-and-distributed-tracing)
   - [Unified Native Database Subsystem & Connection Pools](#-unified-native-database-subsystem--connection-pools)
   - [Native Embedded SQLite Driver (`sqlite`)](#-native-embedded-sqlite-driver-sqlite)
   - [Native PostgreSQL Driver (`postgres` / `pg`)](#-native-postgresql-driver-postgres--pg)
   - [Native MySQL Driver (`mysql`)](#-native-mysql-driver-mysql)
   - [Generic SQL Interface (`database/sql` / `sql`)](#-generic-sql-interface-databasesql--sql)
   - [Native Database Driver Example Suite](#-native-database-driver-example-suite)
   - [Native Redis Driver & In-Memory Cache (`redis` / `std/redis`)](#-native-redis-driver--in-memory-cache-redis--stdredis)
   - [Native MongoDB Driver & Document Store (`mongodb` / `mongo` / `std/mongodb`)](#-native-mongodb-driver--document-store-mongodb--mongo--stdmongodb)
   - [System Modules: OS, Time, Crypto, JWT, and JSON](#-system-modules-os-time-crypto-jwt-and-json)
   - [Production REST Microservice: `bookstore_api`](#-production-rest-microservice-bookstore_api)
8. [🐛 Interactive Debugging and Source Maps V3](#-interactive-debugging-and-source-maps-v3)
   - [Terminal Step-by-Step Debugger (`aurac step`)](#terminal-step-by-step-debugger-aurac-step)
   - [V8 / DAP Debugger Server (`aurac debug`)](#v8--dap-debugger-server-aurac-debug)
9. [🛠️ Full Tooling and CLI Reference](#️-full-tooling-and-cli-reference)
   - [Core Compiler (`aurac`)](#1-core-compiler-aurac)
   - [Test, Benchmark, and Coverage Runner (`auratest`)](#2-test-benchmark-and-coverage-runner-auratest)
   - [Opinionated Code Formatter (`aurafmt`)](#3-opinionated-code-formatter-aurafmt-gofmt-style)
   - [Language Server (`auralsp`)](#4-language-server-auralsp--aurac-lsp)
   - [Decentralized Module Manager (`auramod` / `aurac mod`)](#5-decentralized-module-manager-auramod--aurac-mod)
   - [C/Rust FFI Binding Generator (`aurabindgen`)](#6-c-rust-ffi-binding-generator-aurabindgen--aurac-bindgen)
   - [Interactive Web Playground (`aurac playground`)](#7-interactive-web-playground)
10. [🧩 Editor Configuration](#-editor-configuration)
11. [🔄 Self-Hosted Compiler and Bootstrapping](#-self-hosted-compiler-and-bootstrapping)
12. [🛡️ Security and Runtime Hardening (v0.8.1)](#️-security-and-runtime-hardening-v081)
13. [🗺️ Current Status, Roadmap, and Milestones](#️-current-status-roadmap-and-milestones)
14. [📄 License and Trademarks](#-license)

---

## 🚀 Quick Start

### 1. Building the Aura Toolchain with Cargo

Aura is implemented in Rust (2024 edition) and uses Cargo as the build system and package manager for the compiler engine and its native runtime ([`crates/aura-runtime`](crates/aura-runtime)).

#### A. Prerequisites

Ensure you have a modern Rust toolchain installed:

```bash
rustc --version # Requires Rust 1.85+ (2024 Edition)
cargo --version
```

#### B. Clone and Build the Complete Toolchain

```bash
git clone https://github.com/aura-lang-developer/aura-lang.git
cd aura-lang

# Optimized production compilation (Release with Thin LTO, 1 codegen-unit, and symbol stripping)
cargo build --release

# Or fast development compilation (Debug)
cargo build
```

The workspace is configured with centralized `[workspace.dependencies]`, unified `[workspace.lints]`, and optimized release profiles in `Cargo.toml`.

The compiled binaries will be placed in `target/release/` (or `target/debug/`):

- `aura` (or `aurac`): Unified CLI toolchain and core compiler, native builder, hot runner (`run`), test engine (`test` / `--test`), watcher, debugger, and playground.
- `auratest`: Complete unit test, benchmark (`ns/op`), and HTML coverage suite.
- `aurafmt`: Fast, idempotent, and opinionated code formatter (`gofmt` style).
- `auralsp`: Language Server Protocol (LSP) daemon compatible with any modern editor.
- `auramod` / `aurapkg`: Decentralized package manager (`aura.mod` and `aura.lock`).
- `aurabindgen`: Automated FFI binding generator from C header files (`.h`).

#### C. Selective Binary Compilation

You can compile individual binaries using the `--bin` flag:

```bash
# Compile the primary toolchain binary (or --bin aurac)
cargo build --release --bin aura

# Compile only the test and benchmark runner
cargo build --release --bin auratest

# Compile only the code formatter
cargo build --release --bin aurafmt

# Compile only the Language Server
cargo build --release --bin auralsp
```

#### D. Direct Invocation via Cargo (Without modifying PATH)

You can invoke the tools directly using `cargo run`, separating Aura flags with `--`:

```bash
# Build an Aura program into a standalone native binary:
cargo run --release --bin aurac -- build examples/hello.aura -o dist/hello

# Rapid hot execution:
cargo run --bin aurac -- run examples/hello.aura

# Static type checking (Hindley-Milner):
cargo run --bin aurac -- check examples/hello.aura

# Compile with conditional build tags:
cargo run --bin aurac -- build examples/build_tags_demo.aura -o dist/build_demo --tags "pro"

# Run Aura tests:
cargo run --bin auratest -- examples/tests/

# Format Aura files:
cargo run --bin aurafmt -- -w examples/
```

#### E. Global System Installation

To make all Aura commands (`aurac`, `auratest`, `aurafmt`, etc.) available globally in your shell:

```bash
cargo install --path .
```

This installs all binaries into `$HOME/.cargo/bin`. Make sure `$HOME/.cargo/bin` is in your `PATH`.

#### F. Internal Compiler Verification with Cargo

To verify the compiler implementation and its test suite:

```bash
# Run all unit tests for the compiler library
cargo test --lib

# Run end-to-end integration and bootstrapping tests
cargo test --test bootstrap_tests -- --test-threads=1

# Run concurrency and build tag evaluation tests
cargo test --test build_tags_and_race_tests
```

### 2. Your First Aura Program

Create a file named `hello.aura`:

```aura
export fn main(): Unit => {
    println("Hello from Aura Lang!");
}
```

Verify your installation:

```bash
aura --version
# Output: aura 0.8.2
aurac --version
# Output: aurac 0.8.2
```

### 3. Compilation and Execution Modes

#### A. Rapid Development Execution:

```bash
aurac run hello.aura
# Output: Hello from Aura Lang!
```

#### B. Standalone Native Binary Compilation (Cranelift):

Generates a standalone native binary without external runtime dependencies:

```bash
aurac build hello.aura -o hello --target native
./hello
# Output: Hello from Aura Lang!
```

#### C. Optimized Compilation with SIMD and Dead-Code Stripping:

```bash
aurac build hello.aura -o hello --release -O3
```

#### D. Compilation via the Go Toolchain:

```bash
aurac build hello.aura -o hello --target go
./hello
# Or emit pure, inspectable Go source code:
aurac emit-go hello.aura -o hello.go
```

---

## 🚀 What's New in v0.2.0: Unified Native Database Subsystem & NoSQL Engines

Aura **v0.2.0** introduces a high-performance, strongly-typed native database subsystem designed specifically for standalone binary executables compiled directly via **Cranelift**. It operates with **zero external dynamic runtime dependencies** (no `.so` or `.dylib` library prerequisites), bundles pure-Rust network wire protocols, and integrates embedded in-memory fallback execution:

### 🌟 Architectural Highlights

1. **Pure-Rust Embedded & Wire Protocols (Zero External Shared Libraries)**:
   - **Embedded SQLite 3**: Statically bundled SQLite engine supporting lightning-fast in-memory stores (`:memory:`) and persistent single-file databases.
   - **Pure-Rust PostgreSQL Wire Protocol (3.0)**: Parameterized extended query execution (`$1, $2`), cursor streaming, and MD5 authentication without requiring `libpq`.
   - **Pure-Rust MySQL Protocol (10)**: Prepared statement parameter binding (`?`), binary result set parsing, and SHA-1 authentication handshakes without `libmysqlclient`.
   - **Pure-Rust Redis Client (RESP)**: Native Redis serialization protocol engine with standalone in-memory cache fallback (`:memory:`).
   - **Pure-Rust MongoDB Document Engine**: Native document store supporting nested JSON/BSON records, dynamic query filters, atomic field updates, and multi-stage aggregation pipelines with standalone `:memory:` fallback.

2. **Polymorphic Generic SQL Interface (`database/sql` / `sql`)**:
   - Go-style connection pooling (`DB`) with configurable worker limits (`maxConns`), health checks (`db.ping()`), and deterministic cleanup (`db.close()`).
   - Atomic ACID Transactions (`Tx`) with automatic `commit()` and `rollback()` guarantees, fully compatible with transactional `errdefer`.
   - Strongly-typed cursor iteration (`Rows`) with zero-copy row scanners (`getInt`, `getString`, `getFloat`, `getBool`, `toRecord`, `toList`).

3. **High-Performance NoSQL & Key-Value Engines**:
   - **Redis (`std/database/redis` / `std/redis`)**: Full support for Strings (KV with TTL), Structured Hash maps (`hset`, `hget`, `hdel`, `hgetall`), Atomic counters (`incr`, `decr`), FIFO Queues and Lists (`lpush`, `rpop`, `lrange`), Key expiration (`expire`, `ttl`), and Health checks (`ping`).
   - **MongoDB (`std/database/mongodb` / `std/mongo`)**: Full CRUD support (`insertOne`, `insertMany`, `findOne`, `find`, `updateOne`, `updateMany`, `deleteOne`, `deleteMany`, `count`), rich filter operators (`$gt`, `$gte`, `$lt`, `$lte`, `$ne`, `$in`), atomic modifiers (`$set`, `$inc`, `$unset`), and multi-stage aggregation pipelines (`$match`, `$group`, `$sum`, `$avg`).

4. **Direct Cranelift Native Machine Code Emission**:
   - 33+ C ABI runtime entrypoints registered in `libaura_runtime.a` and mapped directly into Cranelift native machine code.
   - Transparent member access and seamless interoperability between synchronous Go-style syntax (`let res = col.insertOne({...});`) and asynchronous TypeScript-style syntax (`let res = await col.insertOne({...});`).

5. **7 Production-Grade Standalone Examples**:
   - Complete suite of runnable examples in [`examples/`](examples) demonstrating SQLite CRUD, ACID bank transactions, generic SQL connection pools, PostgreSQL order processing, MySQL inventory management, all-in-one SQL showcase, and Redis + MongoDB NoSQL workflows.

---

## ✨ Key Features

- **🏎️ Native Cranelift Backend**: Generates native machine code (Mach-O on macOS, ELF on Linux) statically linked with the lightweight Rust runtime (`libaura_runtime.a`), without interpreters or dynamic dependencies.
- **🔒 Bidirectional Hindley-Milner Type Inference**: Robust type resolution, complete parametric generics (`<T, E>`), and total elimination of null pointer exceptions (`null` / `undefined`) through standard algebraic types `Option<T>` (`Some(v)` / `None`) and `Result<T, E>` (`Ok(v)` / `Err(e)`).
- **🔪 Slices and Dynamic Collections (Go Model)**: Concise `[]T` syntax (equivalent to `List<T>`), half-open slicing expressions `s[low:high]`, `s[:high]`, `s[low:]`, `s[:]`, and 3-index slicing `s[low:high:max]`, with built-in functions `len`, `cap`, `append`, `make`, and full backend support.
- **🔄 CSP Concurrency (Go Style)**: Lightweight fibers on an M:N scheduler with stack switching (`spawn`), strongly typed channels with configurable buffers (`Channel<T>`), directional channels (`SendChannel<T>`, `RecvChannel<T>`), channel iteration (`for val in ch`), selective multiplexing (`select`) with timeout and default branches, plus synchronization primitives (`Mutex`, `RWMutex`, `WaitGroup`, `Once`, `Pool`).
- **🛡️ Compile-Time Concurrency Safety (`Sendable`)**: The compiler statically validates that only data structures free of raw pointers may traverse channels, eradicating data races and catching bugs at compile time.
- **🛡️ Robust Resource Semantics (Go + Zig)**: Unconditional LIFO `defer` statements and **`errdefer`** statements that execute cleanup or transactional rollbacks **exclusively if an error or panic occurs**.
- **🧩 Implicit Interfaces (Structural Duck Typing)**: Strict separation of data and behavior. Structs define data layout, and methods are bound via value receivers `fn (s: Type) ...` or pointer receivers `fn (s: *Type) ...`. Interfaces are satisfied automatically without keywords like `implements`.
- **⚡ Tail-Call Optimization (TCO)**: Tail-recursive functions are automatically transformed into iterative `while (true)` loops with $O(1)$ stack consumption.
- **🗄️ "Batteries-Included" Standard Library**: Typed native drivers for **PostgreSQL**, **MySQL**, **MongoDB**, and **Redis**, alongside an ultra-high-performance **HTTP ServeMux** web server (**126k+ req/s**).
- **📊 Native Observability**: HTTP server with `/metrics` endpoint in Prometheus format and distributed context propagation via W3C `traceparent` (OpenTelemetry).
- **📦 Decentralized Package Manager (`auramod`)**: Dependency resolution via Git URLs (`github.com/user/pkg`), `aura.mod` manifest, cryptographic `aura.lock` lockfile with SHA-256 hash trees (`h1:...`), offline vendoring, and integrity checks.
- **🔌 C/Rust FFI Binding Generator (`aurabindgen`)**: Parses `.h` headers and automatically generates `extern "C"` declarations, packed structs (`packed struct`), and safe wrapper functions.
- **🐛 Interactive Terminal Debugger and Source Maps V3**: Native debugger with step-by-step commands (`aurac step`), plus DAP / V8 Inspector integration (`aurac debug`) for modern editors.
- **🔄 Self-Hosted Compiler**: Complete compiler implementation written in the Aura language itself (`src/aura_compiler/`).

---

## ⚡ Benchmarks and Empirical Performance

Aura includes an automated, reproducible benchmark suite located in [`benchmarks/`](benchmarks), rigorously evaluated on Apple Silicon hardware (Darwin arm64) comparing **Aura Lang (`aurac`)** against **Golang (`go 1.27+`)**.

### 1. Execution Times and Memory Consumption (Peak RSS)

| Evaluated Scenario                      | Aura Lang (`aurac`) | Golang (`go`) |     Ratio (Aura vs Go)      | Aura Memory (RSS) | Go Memory (RSS) |        Status / Aura Advantage        |
| :-------------------------------------- | :-----------------: | :-----------: | :-------------------------: | :---------------: | :-------------: | :-----------------------------------: |
| **Recursive Fibonacci ($N=38$)**        |    **149.18 ms**    |   116.49 ms   |    1.28x *(from 1.32x)*     |    **2.83 MB**    |     4.28 MB     |         🏆 **-34.0% less RAM**        |
| **Sieve of Eratosthenes (2M primes)**   |    **14.61 ms**     |    9.41 ms    |   **1.55x (from 1.76x)**    |    **4.83 MB**    |     6.33 MB     | 🏆 **-23.7% less RAM than Go** (1-byte compact array & `strb`/`ldrb`)|
| **Functional Pipeline (1M items)**      |      13.01 ms       |    8.39 ms    |            1.55x            |     20.28 MB      |    20.42 MB     |     Parity with Go RAM consumption    |
| **CSP Ping-Pong Channels (200k msgs)**  |    **29.57 ms**     |   25.41 ms    |   **1.16x (Near Parity)**   |      9.17 MB      |     4.36 MB     |    Optimized fiber stack cache pool   |
| **Concurrent Spawn (50k tasks)**        |      15.73 ms       |   11.76 ms    |            1.34x            |     11.11 MB      |     6.39 MB     |    Scheduler O(1) + 8192 Stack Pool   |
| **Cold Start (CLI process)**            |     **6.11 ms**     |    5.88 ms    |   **1.04x (Full Parity)**   |    **2.80 MB**    |     4.25 MB     | 🏆 **-34.1% RAM (2.80 MB vs 4.25 MB)**|
| **Mutex Synchronization (50k ops)**     |      19.98 ms       |   13.97 ms    |            1.43x            |     13.30 MB      |     7.72 MB     |           Atomic fast-path            |
| **Concurrent KV Cache (50k ops)**       |    **18.39 ms**     |   18.14 ms    |  **1.01x (Absolute Parity)**|     15.42 MB      |    10.27 MB     | 🚀 **Absolute execution parity**      |

### 2. Standalone Binary Sizes (Zero Dependencies)

| Scenario                                | Go Binary (MB) | Go Stripped (`-s -w`) | Aura Standalone Binary |               Ratio Aura vs Go                |
| :-------------------------------------- | :------------: | :-------------------: | :--------------------: | :-------------------------------------------: |
| **Numerical Calculation (Cranelift)**   |    2.32 MB     |        1.51 MB        |      **0.50 MB**       |  🏆 **0.22x (Aura 67% smaller than stripped)**|
| **Data Pipeline (Cranelift)**           |    2.32 MB     |        1.51 MB        |      **0.50 MB**       |  🏆 **0.22x (Aura 67% smaller than stripped)**|
| **HTTP REST Server / Microservice**     |    8.85 MB     |        5.96 MB        |      **0.57 MB**       |  🏆 **0.06x (Aura 15x more compact / -93%)**  |
| **CSP Concurrency / Channels**          |    2.33 MB     |        1.53 MB        |      **3.40 MB**       |       1.46x *(improved from 5.07 MB)*         |

### 3. Compilation and Static Typecheck Times

| Scenario                 | `aurac check` (HM Typecheck) | `aurac build` (Standalone) | `go build` |
| :----------------------- | :--------------------------: | :------------------------: | :--------: |
| **Recursive Fibonacci**  |         **12.8 ms**          |        **52.4 ms**         |  95.6 ms   |
| **Sieve of Eratosthenes**|          **9.2 ms**          |        **43.7 ms**         |  86.4 ms   |
| **Data Pipeline**        |          **7.4 ms**          |        **45.3 ms**         |  87.5 ms   |
| **CSP Channels**         |          **7.8 ms**          |        **206.7 ms**        |  86.3 ms   |
| **Concurrent Spawn**     |          **7.5 ms**          |        **165.1 ms**        |  86.1 ms   |
| **Cold Start (CLI)**     |          **9.7 ms**          |        **43.1 ms**         |  86.6 ms   |
| **Mutex Synchronization**|          **7.5 ms**          |        **163.9 ms**        |  86.6 ms   |
| **Concurrent KV Cache**  |          **8.8 ms**          |        **167.6 ms**        |  86.4 ms   |

### 4. HTTP Server / REST Microservice (10,000 requests, Concurrency = 50)

Throughput and latency percentile evaluation under concurrent requests against JSON endpoints:

| Performance Metric                 | Aura Standalone Server | Go Native Server (`net/http`) |           Aura Advantage               |
| :--------------------------------- | :--------------------: | :---------------------------: | :------------------------------------: |
| **Throughput (req/s)**             |    **96,092 req/s**    |         88,744 req/s          |  🏆 **Aura higher throughput (+8.3%)** |
| **Mean Latency**                   |      **0.48 ms**       |            0.51 ms            |  🏆 **Aura lower mean latency**        |
| **Median Latency (p50)**           |      **0.38 ms**       |            0.42 ms            |  🏆 **Aura faster median (0.38 ms)**   |
| **99th Percentile Latency (p99)**  |      **1.50 ms**       |            1.72 ms            |  🏆 **Aura lower tail latency**        |
| **Distributable Binary Size**      |      **0.57 MB**       |  8.85 MB (5.96 MB stripped)   |  🏆 **15.5x more compact (-93.6% disk)**|

### 5. Compilation Throughput vs TypeScript (`tsc`)

| Key Metric                       |      Aura Lang (`aurac`)       |   TypeScript (`tsc`)   |           Aura Advantage           |
| :------------------------------- | :----------------------------: | :--------------------: | :--------------------------------: |
| **Compiler Cold Start**          |     **~1.50 ms – 1.78 ms**     |    ~130 ms – 150 ms    |    🚀 **~80x – 100x faster**       |
| **Internal Engine Latency**      |    **0.016 ms – 0.089 ms**     |     ~15 ms – 45 ms     | ⚡ **~200x – 500x lower latency**   |
| **Compilation Throughput**       | **~850,000 – 1,680,000 LOC/s** | ~25,000 – 60,000 LOC/s | 📈 **~20x – 40x higher throughput**|
| **RAM Consumption (Peak RSS)**   |          **2.27 MB**           |  103.9 MB – 143.0 MB   |      📉 **45x lower memory**       |

### 6. How to Run the Benchmark Suite

To deterministically reproduce all measurements on your local machine:

```bash
# Compile and run the automated suite:
go run benchmarks/benchmark_suite.go

# Or run the precompiled binary:
./benchmarks/benchmark_suite
```

Complete results are stored in structured JSON format in [`benchmarks/results.json`](benchmarks/results.json).

---

## 📁 Compiler and Runtime Architecture

### Pipeline Flow Diagram

```mermaid
flowchart TD
    Src["Aura Source Code (*.aura)"] --> Lex["Deterministic Lexer (lexer.rs)"]
    Lex --> Parse["Recursive Pratt Parser (parser.rs)"]
    Parse --> AST["Abstract Syntax Tree (ast.rs)"]

    DTS["TypeScript Definitions (*.d.ts)"] --> DTSP["DTS Parser (dts_parser.rs)"]
    DTSP --> TC

    HDR["C Headers (*.h)"] --> BGEN["Aura Bindgen (bindgen.rs)"]
    BGEN --> AST

    AST --> TC["Hindley-Milner TypeChecker (typechecker.rs)<br/>• Bidirectional Inference & Generics<br/>• Structural Duck Typing<br/>• Sendable Concurrency Safety [E0401]"]

    TC --> BackendMux{"Backend Selector<br/>(aurac build / run)"}

    BackendMux -->|"--target native (default)"| CL["Cranelift Native Backend (codegen_cranelift.rs)<br/>• Cranelift IR Generation<br/>• SIMD Autovectorization (-O3)<br/>• LTO Dead-Code Stripping"]
    CL --> Runtime["Native Runtime (crates/aura-runtime)<br/>• M:N Fiber Scheduler & Stack Switching<br/>• Mark-and-Sweep Garbage Collector (GC)<br/>• CSP Channels & Deadlock Sentinel<br/>• Zero-Alloc HTTP Server & Prometheus Metrics"]
    Runtime --> BinNative["Standalone Native Binary<br/>(Mach-O on macOS / ELF on Linux)"]

    BackendMux -->|"--target go / emit-go"| GoCG["Idiomatic Go Transpiler (codegen_go.rs)"]
    GoCG --> GoTool["Go Toolchain (go build)"]
    GoTool --> BinGo["Standalone Native Go Binary"]

    BackendMux -->|"aurac run / debug / watch"| NodeCG["ES6 Generator + Source Maps V3 (codegen.rs)"]
    NodeCG --> NodeRuntime["Instant Execution / V8 DAP Inspector"]
```

### Dual-Target Strategy: Native Binaries & JavaScript Interoperability

Aura is built on a hybrid **Dual-Target compilation model** that unifies the performance, memory safety, and autonomy of standalone systems programming with the rapid developer feedback, library ecosystem breadth, and web versatility of the modern JavaScript/TypeScript ecosystem.

```mermaid
graph TD
    A["Aura Source Code (*.aura)"] --> B["Aura Compiler (aurac / Rust)"]
    B --> C["TypeChecker & AST Analysis"]
    
    C -->|Development, Web, NPM, Edge| D["JavaScript Backend (codegen.rs)"]
    D --> E["Node.js / Bun / Browser / Cloudflare / Serverless"]
    
    C -->|Production, Microservices, CLI| F["Native Backend (codegen_go.rs / Cranelift)"]
    F --> G["Standalone Binary Executable (Mach-O / ELF)"]
```

#### Why Does Aura Generate and Support JavaScript?

1. **Immediate Ingestion of the NPM and TypeScript Ecosystem (`.d.ts`)**:
   Instead of waiting years for an entire third-party library ecosystem to be rewritten from scratch, Aura includes a built-in TypeScript definition parser ([`src/dts_parser.rs`](src/dts_parser.rs)). Aura programs can directly import, type-check, and invoke any existing NPM library with complete static type safety, structural duck typing, and IDE autocomplete.
2. **Sub-20ms Developer Feedback Loop (`aurac watch` / `aurac run`)**:
   During active local development, executing via the V8 engine (Node.js or Bun) starts in under **20 ms**, bypassing native binary linkers (like C or Go toolchain linking) for near-instant hot reloads.
3. **Universal Portability (Web Browsers, Serverless & Edge)**:
   Generating ECMAScript standard modules allows Aura code to run everywhere native machine code is restricted:
   - **Interactive Web Playground**: Run the full compiler and code directly in client-side web browsers without a remote backend server.
   - **Edge & Serverless Platforms**: Native deployment to Cloudflare Workers, AWS Lambda (Node.js runtime), Vercel Edge Functions, Deno, and Bun with zero cross-compilation friction for different CPU architectures.
4. **Interactive Step Debugging with High-Precision Source Maps V3**:
   Aura emits standard Source Maps V3 with Base64 VLQ encoding ([`src/sourcemap.rs`](src/sourcemap.rs)), enabling line-by-line step debugging of original `.aura` source files directly inside Chrome DevTools and VS Code / Antigravity IDE.

#### When to Use Each Backend

| Use Case | Compilation Target | Core Advantage |
| :--- | :--- | :--- |
| **Production Microservices & APIs** | `aurac build --standalone` (Go/Cranelift) | Single native binary (Mach-O / ELF), zero dependencies, high CSP fiber throughput. |
| **Daily Local Development** | `aurac watch --run` (Node.js/Bun) | Millisecond cold start, instant live-reload. |
| **NPM Package Integration** | Target JS (`.mjs` + `.d.ts`) | Direct reuse of mature JavaScript/TypeScript libraries. |
| **Web & Edge Computing** | Target JS / Web | Seamless execution on Cloudflare Workers, Vercel, and browser Playgrounds. |

### Why Rust for the Implementation: Advantages and Trade-offs

The Aura language compiler (`aurac`), tooling ecosystem (`auralsp`, `aurafmt`, `auratest`, `auramod`), and native runtime (`crates/aura-runtime`) are implemented in **Rust (2024 Edition)**. This design choice brings profound architectural advantages for performance and safety, alongside distinct engineering trade-offs.

#### 🚀 Key Advantages of Rust

1. **Extreme Compilation Throughput & Zero-GC Compiler Latency**:
   - Compiling directly to optimized machine code enables `aurac` to parse, type-check, and generate code at over **850,000 lines of code per second** with cold-start times under **2 ms**.
   - Unlike compilers written in garbage-collected languages (such as TypeScript `tsc` or Go), the Aura compiler operates with deterministic memory management and zero GC pauses during large codebase builds.
   - Dedicated high-performance data structures such as `FxHashMap` ([`rustc-hash`](https://crates.io/crates/rustc-hash)) power $O(1)$ symbol table lookups and record field indexing with 2x–4x faster hashing than standard algorithms.
2. **Compile-Time Memory Safety & Data-Race Freedom**:
   - The ownership and borrowing model guarantees that the Abstract Syntax Tree (AST), multi-pass TypeChecker, and parallel module loaders are strictly free from *use-after-free*, *null pointer dereferences*, and *data races*.
   - In [`crates/aura-runtime`](crates/aura-runtime), Rust enables building the cooperative M:N fiber scheduler, compact garbage collector, and CSP channels with zero-overhead static references (`&'static Scheduler`) and bare atomic primitives.
3. **Algebraic Data Types (ADTs) & Exhaustive Pattern Matching**:
   - Compiler pipeline passes (Lexer $\to$ Parser $\to$ TypeChecker $\to$ CodeGen) naturally map to Rust's tagged `enum` variants ([`src/ast.rs`](src/ast.rs)).
   - Adding new AST nodes, type variants, or compiler optimizations triggers exhaustive compile-time checks across all visitors, ensuring unhandled cases cannot slip into production.
4. **Direct In-Memory Machine Code Generation (Cranelift)**:
   - Native integration with [**Cranelift**](https://cranelift.dev/) ([`src/codegen_cranelift.rs`](src/codegen_cranelift.rs)) allows emitting standalone native binaries (Mach-O on macOS, ELF on Linux) directly from memory without relying on external C/C++ toolchains.
5. **Standalone Self-Contained Binary Distribution**:
   - The entire toolchain compiles to a single, compact (~6–10 MB) static binary with zero external dependencies (no Node.js, Python, or JVM installation required).

#### ⚠️ Engineering Trade-offs & Challenges

1. **Steep Learning Curve for Community Contributors**:
   - Rust's strict type system, explicit lifetimes, and Borrow Checker increase the barrier to entry for casual open-source contributors compared to TypeScript, Go, or Python.
2. **Toolchain Compilation Times (`rustc` Build Times)**:
   - Compiling the full compiler workspace from scratch with ThinLTO and release optimizations takes 20–60 seconds on clean builds (affecting compiler engineers and CI pipelines, though not Aura end users).
3. **Graph & Cyclic Structure Design Rigor**:
   - Complex bidirectional symbol environments and control-flow graphs require disciplined arena-index or numeric-ID architectures rather than arbitrary raw pointers.
4. **Dual-Maintenance During Self-Hosting Bootstrapping**:
   - Maintaining both the reference Rust compiler implementation and the self-hosted Aura implementation ([`src/aura_compiler/`](src/aura_compiler)) during early bootstrapping phases requires synchronized language specification updates.

#### ⚖️ Architectural Balance Summary

| Dimension | Rust Implementation (`aurac`) | Alternative (e.g. TypeScript / Go / C++) |
| :--- | :--- | :--- |
| **Compiler Throughput** | 🟢 **Extreme (>850k LOC/s)** | 🟡 Moderate in Go (~400k LOC/s), 🔴 Slow in TS (~80k LOC/s) |
| **Memory Safety** | 🟢 **100% guaranteed at compile-time** | 🔴 Unsafe in C/C++, 🟡 GC-dependent in Go/TS |
| **Memory Footprint** | 🟢 **Minimal (~5–15 MB RAM)** | 🔴 Heavy (V8 runtime or Go runtime heap) |
| **Native Cranelift JIT/AOT** | 🟢 **Zero-cost in-memory integration** | 🔴 Requires complex FFI bindings |
| **Contributor Accessibility** | 🟡 **Requires advanced Rust proficiency** | 🟢 Highly accessible in TypeScript / Go |
| **Toolchain Build Speed** | 🟡 **Moderate (`cargo build --release`)** | 🟢 Fast in Go |

### Source Code Organization

- [`src/ast.rs`](src/ast.rs) — Formal AST definitions (expressions, statements, algebraic types, CSP concurrency, receivers, and interfaces).
- [`src/lexer.rs`](src/lexer.rs) — Deterministic lexer with support for pipeline operators (`|>`), channels (`<-`, `chan<-`), pointers, and systems keywords.
- [`src/parser.rs`](src/parser.rs) — Recursive descent and Pratt operator precedence parser.
- [`src/typechecker.rs`](src/typechecker.rs) — Bidirectional Hindley-Milner inference, pattern exhaustiveness, structural duck typing, concurrency safety rules (`Sendable`), and structured `TypeError` diagnostics conforming to `core::error::Error` (Rust 2024).
- [`src/codegen_cranelift.rs`](src/codegen_cranelift.rs) — Native backend compiling the AST directly to machine code using Cranelift.
- [`src/codegen_go.rs`](src/codegen_go.rs) — Idiomatic Golang code generator (`net/http`, goroutines, channels, select, struct receivers).
- [`src/codegen.rs`](src/codegen.rs) — Code generator for rapid development, TCO, debugging support, and Source Maps V3.
- [`src/bindgen.rs`](src/bindgen.rs) — C header parser (`.h`) and safe FFI bindings/wrappers generator (`aurac bindgen`).
- [`src/package.rs`](src/package.rs) — Decentralized dependency and module manager (`auramod` / `aurapkg`), SHA-256 cryptographic hashing (`h1:...`), and offline vendoring.
- [`src/backend.rs`](src/backend.rs) — Backend validation and binary compilation orchestrator.
- [`src/testing.rs`](src/testing.rs) — Unit testing engine, benchmarks (`ns/op`), and HTML coverage reports (`auratest`).
- [`src/formatter.rs`](src/formatter.rs) — Fast, opinionated, and idempotent source formatter (`aurafmt`, `gofmt` style).
- [`src/lsp.rs`](src/lsp.rs) — JSON-RPC 2.0 Language Server Protocol implementation (diagnostics, hover, go to definition, autocomplete, and formatting).
- [`src/sourcemap.rs`](src/sourcemap.rs) — Source Maps V3 generator with Base64 VLQ encoder for high-precision debugging.
- [`src/aura_compiler/`](src/aura_compiler) — Self-hosted compiler written entirely in the Aura language.

### The Standalone Native Runtime (`crates/aura-runtime`)

Located in [`crates/aura-runtime/`](crates/aura-runtime), this is a static Rust library (`libaura_runtime.a`) linked without external runtime dependencies:

- **M:N Fiber Scheduler (`scheduler.rs`, `fiber.rs`)**: Executes thousands of cooperative fibers over a pool of operating system threads using lightweight stack switching. Uses zero-overhead static scheduler references (`&'static Scheduler`) without atomic reference counted wrappers (`Arc`), eliminating heap allocations on startup and context switch contention across worker threads.
- **CSP Channels & Deadlock Sentinel (`channel.rs`)**: Thread-safe concurrent circular queues for message passing, operational channel metrics, and automated deadlock detection.
- **Zero-Alloc HTTP Server (`http.rs`)**: HTTP/1.1 server with persistent Keep-Alive, `TCP_NODELAY`, reusable buffers, coalesced header/body writes, zero-intermediate-allocation `/metrics` Prometheus endpoint via `writeln!`, and W3C `traceparent` propagation.
- **Concurrent Compact Garbage Collector (`gc.rs`)**: Fast Mark-and-Sweep GC with 64 pointer shards protected by `RwLock` and `rustc-hash` (`FxHashMap`) for lock-free parallel reads during root and pointer validation phases.
- **Synchronization Primitives (`sync.rs`)**: Native Mutex, RWMutex, Once, Pool, and WaitGroup with cooperative fiber yielding and spin loops.
- **High-Speed JSON & Record Serializers (`json.rs`, `record.rs`)**: High-speed binary and text encoding/decoding, with `FxHashMap` powering 2x-4x faster record field lookups.

---

## 📘 Complete Aura Language Guide

### 1. Comprehensive Type System

Aura features an expressive static type system with full bidirectional type inference.

#### A. Primitive Types and Fixed-Size Integers

```aura
// Standard and explicit-width signed integers
let i: Int = 42;             // Native 64-bit signed integer
let i8: Int8 = 127;          // Signed: -128 to 127
let i16: Int16 = 32767;
let i32: Int32 = 2147483647;
let i64: Int64 = 9223372036854775807;

// Unsigned integers
let u8: Uint8 = 255;         // Unsigned: 0 to 255 (alias Byte)
let u16: Uint16 = 65535;
let u32: Uint32 = 4294967295;
let u64: Uint64 = 18446744073709551615;
let b: Byte = 255;           // Binary byte
let r: Rune = 65;            // Unicode UTF-32 Rune (equivalent to 'A')
let up: Uintptr = 0;         // Integer pointer for systems programming

// Floating-point numbers
let f: Float = 3.14159265;   // 64-bit float (default)
let f32: Float32 = 3.14;
let f64: Float64 = 2.718281828459;

// Strings, Booleans, and Unit Type
let s: String = "Aura Language";
let flag: Bool = true;
let empty: Unit = ();        // Represents the absence of value (void)
```

#### B. Pointers and Memory Addressing

Aura supports explicit pointers for low-level systems interoperability and efficient mutation:

```aura
let mut counter: Int = 10;
let ptrCounter: *Int = &counter; // Address-of operator (&)
*ptrCounter = 20;                // Dereference operator (*)
```

#### C. Directional CSP Channels

```aura
let ch: Channel<String> = Channel.new(10);
let sendOnly: SendChannel<String> = ch;  // Send-only: ch <- val
let recvOnly: RecvChannel<String> = ch;  // Receive-only: <-ch
```

#### D. Collections: Slices, Maps, Sets, and Tuples

```aura
let numbers: []Int = [1, 2, 3, 4, 5];         // Slices (Go model)
let users = #{ "alice" => 100, "bob" => 200 }; // Literal hash map
let tags = #[ "backend", "native", "csp" ];    // Literal hash set
let tuple: (Int, String, Bool) = (1, "ok", true); // Heterogeneous tuple
```

---

### 2. Variables, Immutability, and Destructuring

In Aura, variables declared with `let` are **immutable by default**. If a variable needs to be reassigned, it must be declared with `let mut`:

```aura
let version = "0.1.0"; // Immutable
// version = "0.2.0";  // ✕ Compile error: Cannot assign to immutable variable

let mut active = false;
active = true;         // ✓ Valid

// Tuple Destructuring
let (id, name, valid) = (101, "Carlos", true);

// Record / Struct Destructuring
let user = { uid: 42, role: "admin", email: "admin@aura.dev" };
let { uid, role } = user;
```

#### Arithmetic Operators and Compound Assignment (`+=`, `-=`, `*=`, `/=`, `%=`)

Aura supports both binary arithmetic operators (`+`, `-`, `*`, `/`, `%`) and compound assignment operators to update mutable variables (`let mut`), record fields, and indexed elements in slices or collections:

| Operator | Operation | Example | Internal Desugaring | Supported Types |
|---|---|---|---|---|
| `+=` | Addition & assignment / Concatenation | `total += x;` | `total = total + x;` | `Int`, `Float`, fixed-size numeric types, `String` |
| `-=` | Subtraction & assignment | `balance -= withdrawal;` | `balance = balance - withdrawal;` | `Int`, `Float`, fixed-size numeric types |
| `*=` | Multiplication & assignment | `factor *= 2;` | `factor = factor * 2;` | `Int`, `Float`, fixed-size numeric types |
| `/=` | Division & assignment | `val /= divisor;` | `val = val / divisor;` | `Int`, `Float`, fixed-size numeric types |
| `%=` | Modulo & assignment | `idx %= limit;` | `idx = idx % limit;` | `Int`, fixed-size integer types |

##### Code Example:

```aura
export fn demoArithmetic(): Unit => {
    // 1. Accumulative arithmetic on mutable variables
    let mut counter: Int = 10;
    counter += 15; // 25
    counter -= 5;  // 20
    counter *= 3;  // 60
    counter /= 2;  // 30
    counter %= 7;  // 2
    println(`Counter: ${counter}`);

    // 2. Floating-point arithmetic
    let mut average: Float = 10.5;
    average += 4.5; // 15.0
    average *= 2.0; // 30.0
    average /= 3.0; // 10.0
    println(`Average: ${average}`);

    // 3. String concatenation
    let mut greeting = "Hello";
    greeting += ", Aura!";
    println(greeting); // "Hello, Aura!"

    // 4. In-place accumulator inside loops
    let mut total = 0;
    let numbers: []Int = [1, 2, 3, 4, 5];
    for x in numbers {
        total += x;
    }
    println(`Total sum: ${total}`); // 15

    // 5. In-place modification on slice elements
    let mut items = [100, 200, 300];
    items[0] += 50;
    println(`items[0]: ${items[0]}`); // 150
}
```

---

### 3. Control Flow, Expressions, and Labeled Loops

In Aura, code blocks `{ ... }` and conditional constructs are **expressions** that evaluate to the value of their final statement without a semicolon:

```aura
// Block as expression
let totalRate = {
    let base = 100.0;
    let surcharge = 0.15;
    base * (1.0 + surcharge) // Evaluates to 115.0
};

// If-Else as a safe ternary expression
let status = if grade >= 60 { "Passed" } else { "Failed" };

// Traditional While loop
let mut k = 0;
while k < 5 {
    k += 1; // Compound assignment
};

// For-In loop over slices or collections
for item in [10, 20, 30] {
    println(`Item: ${item}`);
}

// For-In loop with index and value
for i, name in ["Aura", "Go", "Rust"] {
    println(`Index ${i}: ${name}`);
}

// Labeled Loops with targeted break and continue
'matrixSearch: for r, row in matrix {
    for c, val in row {
        if val == target {
            println(`Found at (${r}, ${c})`);
            break 'matrixSearch; // Breaks the outer labeled loop directly
        }
    }
}
```

---

### 4. Slices and Collection Segmentation (Golang Model)

Aura implements Golang's slice model as an abstraction over contiguous memory segments:

#### A. Type Syntax and Declaration

```aura
let mut items: []Int = [10, 20, 30, 40, 50, 60];
```

#### B. Half-Open Slicing Expressions `[low:high]`

```aura
let s = [10, 20, 30, 40, 50, 60];

let sub1 = s[1:4];     // [20, 30, 40] (from index 1 through 3)
let sub2 = s[:3];      // [10, 20, 30] (from start through index 2)
let sub3 = s[3:];      // [40, 50, 60] (from index 3 to the end)
let sub4 = s[:];       // [10, 20, 30, 40, 50, 60] (full slice)
let sub5 = s[1:3:5];   // [20, 30] with max capacity bound to 5 (Go 3-index slice)
```

#### C. Go-Style Built-in Functions (`len`, `cap`, `append`, `make`)

```aura
println(`Length: ${len(items)}`);    // 6
println(`Capacity: ${cap(items)}`);  // 6

// Dynamically grow slice with append
items = append(items, 70);

// Preallocate sized slices
let buffer = make([]Int, 10, 20); // length 10, capacity 20
```

#### D. String Slicing

```aura
let text = "Aura Engine";
let name = text[0:4];   // "Aura"
let engine = text[5:];  // "Engine"
```

---

### 5. Sum Types (ADTs) and Exhaustive Pattern Matching

Sum types or tagged unions allow precise domain modeling. The compiler exhaustively verifies that all variants are handled in `match` expressions:

```aura
export type OrderStatus =
    | Pending
    | Processing { workerId: Int }
    | Shipped(String, String) // (carrier, trackingNumber)
    | Delivered
    | Cancelled { reason: String, at: Int };

fn describeStatus(status: OrderStatus): String => {
    match status {
        Pending => "Order is pending.",
        Processing { workerId } => `Processing by operator #${workerId}.`,
        Shipped(carrier, tracking) => `In transit with ${carrier}, tracking: ${tracking}.`,
        Delivered => "Order delivered to customer.",
        Cancelled { reason, at } if at > 0 => `Cancelled: ${reason} (Timestamp: ${at}).`,
        Cancelled { reason, .. } => `Cancelled: ${reason}.`,
    }
}
```

If a variant or branch is omitted, the compiler reports a static error, preventing code generation.

---

### 6. Error Handling with Result, Option, and `?` Operator

Aura **completely lacks `null` and `undefined`**. Fallible operations and optional values are formally modeled:

```aura
// Standard prelude types:
// type Option<T> = Some(T) | None;
// type Result<T, E> = Ok(T) | Err(E);

fn divide(a: Float, b: Float): Result<Float, String> => {
    if b == 0.0 {
        return Err("Division by zero is not permitted.");
    }
    Ok(a / b)
}

// Automatic unwrapping and error propagation with '?'
fn calculateRatio(x: Float, y: Float, z: Float): Result<Float, String> => {
    let r1 = divide(x, y)?; // Returns early if Err
    let r2 = divide(r1, z)?;
    Ok(r2)
}
```

---

### 7. Pipelines (`|>`) and Tail-Call Optimization (TCO)

The pipeline operator (`|>`) chains data transformations cleanly from left to right. Recursive functions in tail position are compiled into zero-overhead iterative `while (true)` loops with $O(1)$ memory consumption:

```aura
// 1. Functional data pipeline
let totalEvenTransformed = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    |> filter(fn(x: Int): Bool => x % 2 == 0)
    |> map(fn(x: Int): Int => x * 3)
    |> toList();

// 2. Tail-recursive function with TCO
fn factorialTCO(n: Int, acc: Int = 1): Int => {
    if n <= 1 {
        acc
    } else {
        factorialTCO(n - 1, acc * n) // Optimized into an iterative loop with no stack overhead
    }
}
```

---

### 8. CSP Concurrency (Go-Style Model)

Aura implements the CSP (*Communicating Sequential Processes*) concurrency model with lightweight fibers running cooperatively on the M:N scheduler:

```aura
// 1. Spawning concurrent fibers
let ch = Channel<String>::new(5); // Buffered channel with capacity 5

spawn {
    ch <- "Message 1";
    ch <- "Message 2";
    Channel.close(ch);
};

// 2. Iterating over channels until closed
spawn {
    for msg in ch {
        println(`Received: ${msg}`);
    }
};

// 3. Non-blocking multiplexing with select
let chA = Channel<Int>::new(1);
let chB = Channel<String>::new(1);

select {
    case val = <-chA => {
        println(`Numeric data received: ${val}`);
    },
    case chB <- "hello" => {
        println("Message successfully sent to chB");
    },
    case timeout(500) => {
        println("Timeout reached (500ms)");
    },
    default => {
        println("No channel ready to process");
    }
}

// 4. Synchronization with Mutex and WaitGroup
let wg = WaitGroup.new();
let mu = Mutex.new();
let mut balance = 1000;

wg.add(2);

spawn {
    defer wg.done();
    mu.lock();
    balance = balance + 200;
    mu.unlock();
};

spawn {
    defer wg.done();
    mu.lock();
    balance = balance - 150;
    mu.unlock();
};

wg.wait();
println(`Final synchronized balance: ${balance}`);
```

---

### 9. Concurrency Safety (`Sendable`) and Deadlock Sentinel

To ensure fearless concurrency, Aura's typechecker enforces strict compile-time checks:

- **`Sendable` Rule (Error [E0401])**: Raw pointers (`*T`) cannot be sent across channels. Transmitted data must be primitive types, immutable records, sum types, or values with exclusive ownership.
- **Runtime Deadlock Sentinel**: The M:N scheduler in `crates/aura-runtime` actively tracks parked fibers and channel locks. If all fibers are parked waiting for messages that can never arrive, the runtime terminates gracefully with detailed deadlock diagnostics.

---

### 10. Guaranteed Cleanup: LIFO Defer and Conditional ErrDefer (Zig)

Aura provides deterministic resource cleanup executing in LIFO (*Last-In, First-Out*) order:

```aura
fn processTransaction(accountId: Int, amount: Float, forceFailure: Bool): Result<String, String> => {
    println("1. Opening database connection...");
    // defer executes ALWAYS when exiting the function scope (success or failure)
    defer println("5. Connection returned to pool.");

    let tx = startTransaction();
    // errdefer executes ONLY if the function exits via Err(...) or panic
    errdefer println("⚠️ [Rollback] Reverting changes due to transaction failure.");

    println("2. Verifying balance...");
    if forceFailure {
        return Err("Insufficient funds to complete debit.");
    }

    println("3. Applying charge...");
    println("4. Transaction confirmed.");
    Ok("Operation successful")
}
```

- **On Success (`Ok`)**: Only `defer` runs.
- **On Failure (`Err` or `panic`)**: `errdefer` executes first (performing rollback), followed by `defer` (closing resources), ensuring strict consistency.

---

### 11. Receiver Methods, Visibility, and Implicit Interfaces

Aura follows **Golang's capitalization visibility model**, meaning the `export` keyword is **optional** when declaring functions, methods, structs, and interfaces:

1. **Uppercase Initial (`PascalCase`) — Automatically Public / Exported**:
   Any function, receiver method, struct, or interface whose name starts with an uppercase letter (e.g., `fn Multiply(a: Int, b: Int): Int`, `fn TestMultiply(t: *testing.T)`, `fn BenchmarkMultiply(b: *testing.B)`, `fn ExampleMultiply()`) is **exported automatically** without writing `export`.
2. **Lowercase Initial (`camelCase`) — Module / Package Private**:
   Functions declared in lowercase without `export` (e.g., `fn helper(x: Int): Int` or `fn verifyPin(...)`) remain private to the current module/package.
   - `fn main()` is automatically recognized and executed as the program entrypoint without requiring `export`.
   - Test functions in lowercase (`fn test_addition(t: TestingT)`) are also automatically discovered and executed by `aura test` / `auratest` without requiring `export`.
3. **Explicit `export` Keyword (Optional)**:
   Only needed when you want to explicitly export a symbol whose name starts with a **lowercase** letter (e.g., `export fn multiply(a: Int, b: Int): Int`).

Following idiomatic **Golang** design, structs do not nest methods inside their declarations. Methods are declared externally by binding a receiver:

```aura
// 1. Pure data structure
export struct BankAccount {
    accountNumber: String,
    owner: String,
    balance: Float,
    pinHash: String,
};

// 2. Public method with pointer receiver (Capitalized -> Exported)
fn (b: *BankAccount) Deposit(amount: Float): Result<Float, String> => {
    if amount <= 0.0 {
        return Err("Amount must be positive.");
    }
    b.balance = b.balance + amount;
    Ok(b.balance)
}

// 3. Private / Package-internal method (lowercase -> Not exported)
fn (b: BankAccount) verifyPin(pin: String): Bool => {
    b.pinHash == pin
}

// 4. Structural Interface
export interface AccountViewer {
    GetBalance(): Float;
}

fn (b: BankAccount) GetBalance(): Float => b.balance;

// Polymorphic consumption: BankAccount implicitly satisfies AccountViewer
fn displayBalance(viewer: AccountViewer): Unit => {
    println(`Balance: $${viewer.GetBalance()}`);
}
```

---

### 12. Explicit Pointers, Packed Structs, and C FFI (`extern "C"`)

For low-level systems programming, hardware drivers, and binary protocols:

```aura
// Packed struct with zero padding bytes for network protocols
export packed struct ArpHeader {
    hardware_type: Uint16,
    protocol_type: Uint16,
    hardware_size: Uint8,
    protocol_size: Uint8,
    opcode: Uint16,
};

// Foreign C standard library function declarations
extern "C" {
    fn getpid(): Int;
    fn abs(n: Int): Int;
}

export fn getSystemPid(): Int => {
    getpid()
}
```

---

### 13. Automatic FFI Header Generator (`aurac bindgen`)

Aura includes a native tool to generate typed wrappers from C header files (`.h`):

```bash
aurac bindgen sqlite3.h -o sqlite3.aura --name SQLite3 --strip-prefix "sqlite3_"
```

It parses `#define` constants, `enum`, `struct` with binary alignment, and function prototypes, emitting type-safe Aura bindings with automatic type conversions.

---

### 14. Context and Propagated Cancellation (`Context`)

Inspired by Go's `context.Context`, it enables propagating deadlines, cancellation signals, and request-scoped values across fibers and network calls:

```aura
// Create context with a 200 ms timeout
let (ctx, cancel) = Context.withTimeout(Context.background(), 200);
defer cancel();

spawn {
    select {
        case <-ctx.done() => {
            println("Operation cancelled by context timeout.");
        },
        case res = <-performRemoteRequest() => {
            println(`Request succeeded: ${res}`);
        }
    }
};
```

---

### 15. Generics and Parametric Polymorphism

Aura supports generic functions and types parameterized over arbitrary types:

```aura
export type Box<T> = BoxVal(T);

export fn wrap<T>(item: T): Box<T> => {
    BoxVal(item)
}

export fn unwrapOr<T>(opt: Option<T>, fallback: T): T => {
    match opt {
        Some(v) => v,
        None => fallback,
    }
}
```

---

### 16. Compilation Directives, Build Tags, and Struct Tags

Aura provides compile-time directives to control compilation targets, platform-specific code inclusion, and structural data serialization.

#### A. Conditional Compilation Directives (`//aura:build`)

Aura evaluates build directives placed at the very top of `.aura` source files (before any code or declarations). If the directive evaluates to `false` for the active build target or tags, the compiler safely skips the file during both compilation and test discovery.

- **Modern Boolean Syntax (`//aura:build`)**: Supports `&&` (AND), `||` (OR), `!` (NOT), and balanced parentheses:
  ```aura
  //aura:build (linux && amd64) || (darwin && arm64)
  //aura:build !windows && !wasm
  ```
- **Go Toolchain Compatibility (`//go:build`)**:
  ```aura
  //go:build linux || darwin
  ```
- **Legacy Go Syntax (`// +build`)**: Spaces act as OR; commas act as AND:
  ```aura
  // +build darwin,arm64 linux,amd64
  ```

#### B. Active Built-in and Custom Build Tags

The compiler automatically activates tags based on the compilation environment:
- **Operating Systems**: `darwin`, `macos`, `unix`, `linux`, `windows`
- **Architectures**: `amd64`, `x86_64`, `arm64`, `aarch64`
- **Environments**: `aura`, `es6`, `node`

You can pass custom tags using the `--tags` CLI flag:
```bash
# Try the included example with and without matching tags:
# 1. Without tag: file is skipped
aurac build examples/build_tags_demo.aura -o dist/build_demo
# ➜ ℹ Skipping 'examples/build_tags_demo.aura': build tags do not match.

# 2. With matching tag: compiles successfully
aurac build examples/build_tags_demo.aura -o dist/build_demo --tags "pro,metrics"
./dist/build_demo
# ➜ Output: ¡Compilación exitosa con build tag 'pro' o 'metrics'!

# Typecheck with staging build tags
aurac check examples/build_tags_demo.aura --tags "pro"
```

#### C. Struct Tags for Serialization and ORMs

Struct tags allow annotating fields with metadata for JSON serialization, validation rules, or database mappings:

```aura
export type UserProfile = {
    id: Int,
    fullName: String,
    secretHash: String,
    createdAt: Int,
} `json:"user_profile" db:"users" validate:"required"`;
```

---

### 17. TypeScript Ingestion (`.d.ts`) and Static Asset Embedding (`embed`)

```aura
// Embed static files directly into executable binary memory at compile time
let htmlContent: String = embed("templates/index.html");
let logoBinary: []Byte = embedBytes("assets/logo.png");
```

Type-check against external NPM declarations without writing manual wrappers:

```bash
aurac build app.aura -o app --dts ./node_modules/@types/node/index.d.ts
```

---

### 18. Testing Directives, Assertions, and Benchmarking

Aura provides a built-in testing, benchmarking, and code-coverage framework with **full Golang `testing` (`go test`) parity**, powered by `auratest` or directly via the compiler using `aura test` / `aurac --test`.

#### A. Test Discovery and Categorization Directives

Files matching the following naming conventions are automatically discovered:
- **Test files**: `*_test.aura`, `.test.aura`, `_spec.aura`, `test_*.aura`, or any `.aura` file located in a `tests/` directory.
- **Automatic Categorization**:
  - **Unit Tests (`UNIT`)**: Default for test files, or matching `*_unit_test.aura` (`fn TestXxx(t: *testing.T)` or `fn test_xxx(t: TestingT)`).
  - **Integration Tests (`INTEGRATION`)**: Files containing `_integration_test.aura` or located under an `integration/` directory.
  - **End-to-End Tests (`E2E`)**: Files containing `_e2e_test.aura` or located under an `e2e/` directory.
  - **Examples (`EXAMPLE`)**: Functions beginning with `Example` (e.g., `fn ExampleMultiply()`) that verify standard output against trailing `// Output:` or `// Unordered output:` comments.
  - **Benchmarks (`BENCHMARK`)**: Functions beginning with `Benchmark`, `benchmark_`, or `bench_` (`fn BenchmarkXxx(b: *testing.B)` or `fn benchmark_xxx(b: BenchmarkB)`).
  - **Suite Lifecycle (`TestMain`)**: Optional `fn TestMain(m: *testing.M)` entrypoint calling `m.Run()`.

#### B. Golang `testing` Format (`*testing.T`, `*testing.B`, Table-Driven Tests & Examples)

Aura supports writing tests using the exact same conventions as Go (`package`, `import "testing"`, `*testing.T`, `*testing.B`, `t.Run`, `t.Errorf`, `t.Fatalf`, `b.N`, and `// Output:` examples) as well as Aura's built-in `TestingT` assertion helpers. Notice that `export` is completely optional—functions starting with an uppercase letter (`Test...`, `Benchmark...`, `Example...`, `Multiply`) are automatically exported following Go's visibility rules:

```aura
package math_test

import "testing"

fn Multiply(a: Int, b: Int): Int => a * b

// 1. Standard Go-style test with *testing.T, t.Errorf, and t.Logf (no 'export' needed)
fn TestMultiply(t: *testing.T) => {
    t.Helper();
    let got = Multiply(6, 7);
    let want = 42;
    if (got != want) {
        t.Errorf("Multiply(6, 7) = %d; want %d", got, want);
    }
    t.Logf("Multiply(6, 7) verified: %d", got);
}

// 2. Go-style Table-Driven Tests with t.Run(name, fn(t: *testing.T) => { ... })
fn TestMultiplyTableDriven(t: *testing.T) => {
    let tests = [
        { name: "positive", a: 3, b: 4, want: 12 },
        { name: "by_zero", a: 99, b: 0, want: 0 },
        { name: "negative", a: -5, b: 6, want: -30 },
    ];

    for tc in tests {
        t.Run(tc.name, fn(t: *testing.T) => {
            t.Parallel();
            let got = Multiply(tc.a, tc.b);
            if (got != tc.want) {
                t.Fatalf("Multiply(%d, %d) = %d; want %d", tc.a, tc.b, got, tc.want);
            }
        });
    }
}

// 3. Lifecycle & environment helpers: t.TempDir, t.Setenv, t.Cleanup
fn TestLifecycleAndEnv(t: *testing.T) => {
    let dir = t.TempDir();
    t.Setenv("AURA_GO_TEST_MODE", "enabled");
    t.Cleanup(fn() => {
        t.Log("cleanup executed in LIFO order");
    });
}

// 4. Go-style Benchmark with *testing.B and explicit `b.N` loop
fn BenchmarkMultiplyGoStyle(b: *testing.B) => {
    b.ReportAllocs();
    b.ResetTimer();
    for (let mut i = 0; i < b.N; i = i + 1) {
        Multiply(123, 456);
    }
}

// 5. Go-style Verifiable Example with // Output: comment
fn ExampleMultiply() => {
    println(Multiply(8, 9));
    // Output:
    // 72
}

// 6. Aura assertion helpers on TestingT & async tests
async fn test_async_service(t: TestingT) => {
    let result = await fetchUserData(101);
    t.assertTrue(result.isOk(), "fetching user must succeed");
}
```

#### C. Testing Context Directives (`*testing.T` / `TestingT`)

The `t: *testing.T` (or `t: TestingT`) context supports both Go's `testing.T` / `testing.TB` methods and Aura's assertion helpers:

| Method / Directive | Description |
| :----------------- | :---------- |
| `t.Error(...args)` / `t.Errorf(fmt, ...args)` | Logs an error message and marks the test as failed while continuing execution (Go semantics). |
| `t.Fatal(...args)` / `t.Fatalf(fmt, ...args)` | Logs an error message, marks the test as failed, and immediately stops the current test/subtest. |
| `t.Fail()` / `t.FailNow()` | Marks the test as failed (`Fail` continues; `FailNow` aborts the current test/subtest immediately). |
| `t.Log(...args)` / `t.Logf(fmt, ...args)` | Records formatted diagnostic messages (`%d`, `%s`, `%q`, `%v`, `%.2f`, `%t`, `%x`, `%T`). |
| `t.Run(subName, fn(t: *testing.T))` | Runs an isolated subtest (`TestParent/SubName`), returning `true` if the subtest passed. |
| `t.Skip(...args)` / `t.Skipf(fmt, ...args)` / `t.SkipNow()` | Skips the current test or subtest and records the skip reason. |
| `t.Cleanup(fn)` | Registers a cleanup function executed in LIFO order when the test/subtest completes. |
| `t.TempDir()` | Creates a unique temporary directory that is automatically removed via `Cleanup` when the test finishes. |
| `t.Setenv(key, value)` | Sets an environment variable for the duration of the test and restores its original value on cleanup. |
| `t.Parallel()` / `t.Helper()` | Marks a test as safe for parallel execution or marks the calling function as a test helper. |
| `t.Name()` / `t.Failed()` / `t.Skipped()` | Returns the current test/subtest name, failure state (`Bool`), or skip state (`Bool`). |
| `t.assertEqual(actual, expected, msg?)` | Asserts deep structural equality between two values. |
| `t.assertNotEqual(actual, expected, msg?)` | Asserts that two values are not deeply equal. |
| `t.assertTrue(condition, msg?)` / `t.assertFalse(condition, msg?)` | Asserts that `condition` is `true` or `false`. |
| `t.assertThrows(fn, expectedErr?)` | Asserts that calling `fn()` throws or panics with an optional error substring. |

Global assertions are also available directly without `t`: `assert(cond, msg)`, `assertTrue(cond, msg)`, `assertFalse(cond, msg)`, `assertEqual(a, b, msg)`, and `assertNotEqual(a, b, msg)`.

#### D. Benchmark Context Directives (`*testing.B` / `BenchmarkB`)

The `b: *testing.B` (or `b: BenchmarkB`) context inherits all `testing.TB` methods and manages adaptive iteration counts and throughput metrics:

| Method / Directive | Description |
| :----------------- | :---------- |
| `b.N` / `b.n` | Target iteration count adaptively calibrated by the benchmark runner (`for (let mut i = 0; i < b.N; i = i + 1)` or `for i in b.N`). |
| `b.ResetTimer()` / `b.resetTimer()` | Resets the elapsed time and start timestamp (use after expensive setup). |
| `b.StartTimer()` / `b.startTimer()` | Resumes timer measurement. |
| `b.StopTimer()` / `b.stopTimer()` | Pauses timer measurement during non-benchmark teardown/allocations. |
| `b.ReportAllocs()` / `b.reportAllocs()` | Enables allocation statistics reporting for the benchmark. |
| `b.SetBytes(n)` / `b.setBytes(n)` | Informs the runner of bytes processed per op to compute MB/s throughput. |
| `b.ReportMetric(n, unit)` | Reports a custom user-defined benchmark metric with the given unit. |
| `b.Run(subName, fn(b: *testing.B))` | Runs an isolated sub-benchmark (`BenchmarkParent/SubName`). |
| `b.RunParallel(fn(pb: *testing.PB))` | Executes a benchmark body in parallel using `while (pb.Next()) { ... }`. |

#### E. Conditional Testing with Build Tags

Apply conditional compilation directives to isolate slow or environment-specific test suites:

```aura
//aura:build integration && !ci_fast
// File: tests/postgres_integration_test.aura

export fn test_postgres_pool(t: TestingT) => {
    let pool = postgres.createPool(testConfig);
    t.assertTrue(pool.ping());
}
```

Run test suites matching specific tags:
```bash
auratest --tags "integration" ./...
# or via compiler:
aurac --test --tags "integration" ./...
```

#### F. Statement & Line Code Coverage Analysis

Aura includes a native statement- and line-level code coverage engine accessible through both the compiler (`aurac --test`) and the dedicated `auratest` runner:

| Coverage Flag | Description |
| :------------ | :---------- |
| `-cover`, `--coverage` | Enables coverage analysis and prints statement coverage table and summary in the terminal. |
| `-coverprofile <file>` | Writes detailed line-by-line coverage profile data to the specified file for CI/CD pipelines. |
| `--coverage-html <file>`, `--html <file>` | Generates a standalone interactive HTML coverage report with syntax-highlighted source code. |

Code coverage examples:
```bash
# Terminal summary for the entire project
aurac --test --coverage

# Terminal summary for a single file or directory
aurac examples/tests/math_unit_test.aura --test --coverage

# Generate coverage profile and interactive HTML report
aurac --test -coverprofile coverage.out --coverage-html coverage.html ./...
```

---

### 19. Rust-Inspired Features: Derives, Must-Use, Auto-Fix, and Doctests

Aura incorporates static ergonomics and robustness patterns directly inspired by **Rust**:

#### A. Automatic Type Derivation (`#[derive(...)]`)

Automatically synthesizes standard helper and receiver methods on `type` (record aliases) and `type` (sum types / ADTs) to eliminate boilerplate:

```aura
#[derive(Debug, Clone, Eq, Json, Hash)]
export type User = {
    id: Int,
    username: String,
    active: Bool,
};
```

* **`Debug`**: Synthesizes `User.debug(u)`, `.debug()`, and `.toString()`.
* **`Clone`**: Synthesizes deep structured cloning with `User.clone(u)` and `.clone()`.
* **`Eq`**: Synthesizes structural value equality comparison `User.equals(a, b)` and `.equals(other)`.
* **`Json`**: Synthesizes `User.toJson(u)`, `.toJson()`, and typed parsing `User.fromJson(str)`.
* **`Hash`**: Synthesizes deterministic 32-bit FNV-1a integer hashing `User.hash(u)` and `.hash()`.

#### B. `#[must_use]` Directive for Discarded Value Protection

Emits static linter warnings if the return value of a critical function or handle is discarded without inspection:

```aura
#[must_use = "Database transaction handle must be committed or rolled back"]
fn beginTx(): Int => {
    return 1001;
};

fn main(): Unit => {
    beginTx(); // Triggers LINT003 warning in `aura lint`
};
```

#### C. Linter with Automated Code Repair (`aura fix` / `aurac lint --fix`)

Automatically repairs static analysis findings, such as variables declared with `let mut` that are never reassigned:

```bash
# Automatically repair issues in file
aura fix src/main.aura

# Or via linter flag:
aura lint src/main.aura --fix
```

#### D. Executable Documentation Tests (*Doctests*) with `auratest --doc`

Documentation code snippets embedded in triple-slash comments (`/// ```aura ... ``` `) are automatically discovered, compiled, and executed:

```aura
/// Adds two integers.
/// ```aura
/// let res = add(10, 20);
/// assert(res == 30, "10 + 20 must equal 30");
/// ```
pub fn add(a: Int, b: Int): Int => a + b;
```

Run only documentation tests via:
```bash
auratest --doc
# Or with the primary compiler CLI:
aura test --doc
```

#### E. Modular Feature Flags in `aura.mod`

Enable conditional submodules and optional feature drivers within decentralized dependency declarations:

```mod
module my-service/api

aura 0.8.3

require (
    github.com/aura/db v1.2.0 [postgres, sqlite]
    github.com/aura/metrics v0.4.0
)
```

---

## 🗄️ Standard Library, Web Server, and Databases

### 🌐 HTTP Web Server and Routing Mux (`net/http`)

The native HTTP server provides production-grade performance (**126k+ req/s**), middleware chaining, parameterized routing, JSON encoding, and integrated OpenAPI documentation:

```aura
import { http, os } from "net/http";

let mux = http.newServeMux();

// 1. Global middleware
mux.use(fn(req: Any, res: Any, next: Any) => {
    println(`[HTTP] ${req.method} ${req.url}`);
    res.setHeader("X-Engine", "Aura-Native");
    next();
});

// 2. Route with dynamic parameter (:id) and Swagger documentation
mux.get("/api/books/:id", fn(req: Any, res: Any) => {
    let bookId = http.pathValue(req, "id");
    http.json(res, http.StatusOK, {
        id: bookId,
        title: "The Rust Programming Language",
        inStock: true
    });
});

// 3. POST route with asynchronous JSON parsing
mux.post("/api/books", fn(req: Any, res: Any) => async {
    let parsed = await http.parseJson(req);
    match parsed {
        Ok(data) => http.json(res, http.StatusCreated, { created: true, book: data }),
        Err(err) => http.error(res, `Invalid payload: ${err}`, http.StatusBadRequest)
    }
});

// 4. Enable interactive Swagger UI and OpenAPI 3.0 documentation
mux.enableSwagger("/swagger");

// 5. Start listening
let port = os.env("PORT") != "" ? os.env("PORT") : "8080";
println(`🚀 HTTP Server listening on :${port}`);
mux.listenAndServe(`:${port}`);
```

### 📊 Prometheus Metrics and Distributed Tracing

The HTTP server features out-of-the-box observability:

- **`/metrics` Endpoint**: Exports native metrics (`aura_http_requests_total`, `aura_scheduler_fibers`, `aura_csp_channel_operations`).
- **W3C `traceparent`**: Extracts and propagates distributed tracing headers across microservices, adhering to OpenTelemetry standards.

---

### 🗄️ Unified Native Database Subsystem & Connection Pools

Aura features a high-performance, strongly-typed native database subsystem designed for standalone binary executables compiled directly via **Cranelift**. It requires **zero external runtime dependencies** (no external `.so` or `.dylib` libraries) and follows the standard Go `database/sql` connection pooling and resource management model.

#### Architectural Highlights
- **Embedded SQLite Engine**: Statically bundled SQLite 3 C-engine with in-memory (`:memory:`) and file-based database support.
- **Pure-Rust PostgreSQL Wire Protocol (3.0)**: Native TCP client implementation supporting Cleartext, MD5, and modern `SCRAM-SHA-256` authentication (RFC 7677 / RFC 5802 for PostgreSQL 14+, Supabase, and Neon), parameterized extended queries (`$1..$N`), and streaming row decoding without `libpq`.
- **Pure-Rust MySQL Wire Protocol (10)**: Native TCP client implementation supporting SHA-1 authentication handshakes, prepared statement parameter bindings, and result set parsing.
- **Thread-Safe Connection Pooling (`DB`)**: Automatic connection pooling with strict active connection bounding via `Condvar` synchronization, health checks (`db.ping()`), and deterministic cleanup (`db.close()`).
- **ACID Transactions (`Tx`)**: Full transaction lifecycle management (`db.begin()`, `tx.commit()`, `tx.rollback()`) with polymorphic connection borrowing.
- **Strongly-Typed Field Extractors (`Rows`)**: Fast cursor-based iteration (`rows.next()`), typed value retrieval (`getInt`, `getString`, `getFloat`, `getBool`), and direct map/record decoding (`rows.toRecord()`, `rows.toList()`).

---

### 🪶 Native Embedded SQLite Driver (`sqlite`)

Embedded SQLite is built directly into the Aura runtime binary:

```aura
import { open } from "std/database/sqlite";

// Open in-memory or file database with connection pool of 5 workers
let db: DB = sqlite.open(":memory:", 5);
defer db.close();

// DDL schema execution
let ddl = "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, balance INT);";
let schemaRes: ExecResult = db.exec(ddl, []);
println(`Schema initialized. Affected rows: ${schemaRes.rowsAffected}`);

// Parameterized INSERT statements (SQL-injection safe)
let insertRes = db.exec("INSERT INTO users (name, balance) VALUES (?, ?);", ["Alice", 2500]);
println(`Inserted user with ID: ${insertRes.lastInsertId}`);

// Parameterized Query and Cursor Iteration
let rows: Rows = db.query("SELECT id, name, balance FROM users WHERE balance >= ?;", [1000]);
defer rows.close();

while (rows.next()) {
    let id = rows.getInt(0);
    let name = rows.getString(1);
    let balance = rows.getInt(2);
    println(`[User #${id}] ${name} -> Balance: $${balance}`);
}
```

---

### 🐘 Native PostgreSQL Driver (`postgres` / `pg`)

Pure-Rust TCP client implementing the PostgreSQL 3.0 wire protocol with native **SCRAM-SHA-256**, MD5, and Cleartext authentication:

```aura
import { open } from "std/database/postgres";

// Connect to remote or local PostgreSQL server
let db: DB = postgres.open("postgres://postgres:secret@localhost:5432/orders_db", 10);
defer db.close();

// Parameterized query with $1, $2 placeholders
let rows = db.query("SELECT id, name, email FROM users WHERE active = $1;", [true]);
defer rows.close();

while (rows.next()) {
    let id = rows.getInt(0);
    let name = rows.getString(1);
    let email = rows.getString(2);
    println(`- User ${id}: ${name} <${email}>`);
}

// Convert current row or full dataset to dynamic Aura records
let userRecord = rows.toRecord();
println(`User Name: ${userRecord.name}`);
```

---

### 🐬 Native MySQL Driver (`mysql`)

Pure-Rust TCP client implementing the MySQL Protocol 10 wire protocol:

```aura
import { open } from "std/database/mysql";

let db: DB = mysql.open("mysql://root:secret@127.0.0.1:3306/shop_db", 10);
defer db.close();

let res: ExecResult = db.exec("INSERT INTO products (name, price) VALUES (?, ?);", ["Mechanical Keyboard", 129]);
println(`Inserted product ID: ${res.lastInsertId}, Affected: ${res.rowsAffected}`);
```

---

### 🔄 Generic SQL Interface & ACID Transactions (`database/sql` / `sql`)

Aura provides a unified generic interface modeled after Go's `database/sql` enabling polymorphic database swapping and atomic transaction guarantees:

```aura
import { open, DB, Tx, Rows, ExecResult } from "std/database/sql";

// Polymorphic opening via driver string: "sqlite", "postgres", "mysql"
let db: DB = sql.open("sqlite", ":memory:", 5);
defer db.close();

// Initialize table
db.exec("CREATE TABLE accounts (id INT PRIMARY KEY, balance INT);", []);
db.exec("INSERT INTO accounts (id, balance) VALUES (1, 1000), (2, 500);", []);

// Begin an atomic ACID Transaction
let tx: Tx = db.begin();

// Transfer $200 from Account 1 to Account 2
tx.exec("UPDATE accounts SET balance = balance - 200 WHERE id = 1;", []);
tx.exec("UPDATE accounts SET balance = balance + 200 WHERE id = 2;", []);

// Commit atomically (returns 0 on success, -1 on error)
let status = tx.commit();
if (status == 0) {
    println("✓ Transfer successfully committed!");
} else {
    tx.rollback();
    println("✕ Transfer failed, rolled back!");
}
```

---

### 📚 Native Database Driver Example Suite

Aura includes a comprehensive set of real-world, standalone database examples in the [`examples/`](examples) directory illustrating every aspect of the native database subsystem:

| Example File | Backend / Paradigm | Key Features Demonstrated | Command to Run |
| :--- | :--- | :--- | :--- |
| [`examples/db_sqlite_crud.aura`](examples/db_sqlite_crud.aura) | **SQLite** (Embedded) | DDL table creation, parameterized `INSERT`, cursor-based `SELECT` scanning, `UPDATE`, `DELETE`, and pool metrics | `aurac build examples/db_sqlite_crud.aura -o dist/sqlite_crud --standalone && ./dist/sqlite_crud` |
| [`examples/db_transactions_bank.aura`](examples/db_transactions_bank.aura) | **ACID Transactions** (`Tx`) | Multi-statement financial transfers, atomic `commit()`, automatic `rollback()` on rule failure, and balance integrity | `aurac build examples/db_transactions_bank.aura -o dist/bank_tx --standalone && ./dist/bank_tx` |
| [`examples/db_generic_sql_pool.aura`](examples/db_generic_sql_pool.aura) | **Generic SQL** (`sql.open`) | Database-agnostic driver dispatch, health checks via `ping()`, aggregate queries (`COUNT`, `AVG`, `GROUP BY`), and connection pool lifecycle | `aurac build examples/db_generic_sql_pool.aura -o dist/sql_pool --standalone && ./dist/sql_pool` |
| [`examples/db_postgres_microservice.aura`](examples/db_postgres_microservice.aura) | **PostgreSQL** (Wire 3.0) | Pure-Rust TCP client, `$1, $2` parameterized extended query syntax, and order processing workflows | `aurac check examples/db_postgres_microservice.aura` |
| [`examples/db_mysql_inventory.aura`](examples/db_mysql_inventory.aura) | **MySQL** (Protocol 10) | Pure-Rust TCP client, `?` placeholder bindings, `lastInsertId` & `rowsAffected` metrics, and batch inventory tracking | `aurac check examples/db_mysql_inventory.aura` |
| [`examples/native_database_demo.aura`](examples/native_database_demo.aura) | **All-in-One Demo** | Complete standalone showcase: in-memory SQLite, schema setup, parameterized inserts, cursor reads, and ACID transactions | `aurac build examples/native_database_demo.aura -o dist/native_db_demo --standalone && ./dist/native_db_demo` |
| [`examples/db_stored_procedures_functions.aura`](examples/db_stored_procedures_functions.aura) | **Functions & Stored Procedures** (`CALL` / `SELECT`) | Stored procedures (`CALL`), scalar & table-valued functions (`RETURNS TABLE`), MySQL `OUT` parameters, aggregate queries, and transactions | `aurac build examples/db_stored_procedures_functions.aura -o dist/sp_demo --standalone && ./dist/sp_demo` |
| [`examples/native_nosql_demo.aura`](examples/native_nosql_demo.aura) | **NoSQL Suite** (Redis & Mongo) | Standalone native NoSQL engine: Redis KV, Hashes, FIFO Queues, Counters + MongoDB Document CRUD, filtering, and Aggregation pipelines | `aurac build examples/native_nosql_demo.aura -o dist/nosql_demo --standalone && ./dist/nosql_demo` |

#### Compiling and Running the Native Examples
All examples compile directly into self-contained native machine code binaries (Mach-O on macOS, ELF on Linux) with zero external dynamic dependencies:

```bash
# 1. Compile and run SQLite CRUD
aurac build examples/db_sqlite_crud.aura -o dist/db_sqlite_crud --standalone
./dist/db_sqlite_crud

# 2. Compile and run ACID Bank Transactions
aurac build examples/db_transactions_bank.aura -o dist/db_transactions_bank --standalone
./dist/db_transactions_bank

# 3. Compile and run Generic SQL Connection Pool
aurac build examples/db_generic_sql_pool.aura -o dist/db_generic_sql_pool --standalone
./dist/db_generic_sql_pool

# 4. Compile and run Native NoSQL (Redis & MongoDB) Showcase
aurac build examples/native_nosql_demo.aura -o dist/nosql_demo --standalone
./dist/nosql_demo
```

---

### ⚡ Native Redis Driver & In-Memory Cache (`redis` / `std/redis`)

Aura provides a native, zero-dependency Redis client implementing the **RESP wire protocol** over TCP with automatic embedded in-memory fallback (`:memory:`) for standalone binaries:

- **Key-Value Operations**: `set(key, val, ttlSec?)`, `get(key)`, `del(key)`, `exists(key)`.
- **Atomic Counters**: `incr(key)`, `decr(key)`.
- **Structured Hash Maps**: `hset(hash, field, val)`, `hget(hash, field)`, `hdel(hash, field)`, `hgetall(hash)`.
- **FIFO Queues & Lists**: `lpush(list, val)`, `rpop(list)`, `lrange(list, start, stop)`.
- **TTL & Expirations**: `expire(key, seconds)`, `ttl(key)`.
- **Health Checks & Lifecycle**: `ping()`, `close()`.

```aura
import { open, RedisClient } from "std/database/redis";

// Connect to remote Redis or use standalone embedded in-memory cache
let r: RedisClient = redis.open(":memory:");
defer r.close();

// Ping
let pong = r.ping(); // "PONG"
println(`Redis Status: ${pong}`);

// Key-Value with optional TTL (seconds)
r.set("user:101:session", "token_abc123", 3600);
let token = r.get("user:101:session");
println(`Active Session: ${token}`);

// Atomic counters
let visitors = r.incr("page:home:hits");
println(`Page hits: ${visitors}`);

// Hash map storage
r.hset("user:101:profile", "username", "alice");
r.hset("user:101:profile", "role", "admin");
let role = r.hget("user:101:profile", "role");
println(`User role: ${role}`);

// FIFO Queue / List
r.lpush("jobs:queue", "email_welcome_user_101");
r.lpush("jobs:queue", "generate_pdf_invoice_992");
let nextJob = r.rpop("jobs:queue");
println(`Processed Job: ${nextJob}`);
```

---

### 🍃 Native MongoDB Driver & Document Store (`mongodb` / `mongo` / `std/mongodb`)

Aura includes a pure-Rust native MongoDB document engine supporting structured JSON/BSON document collections, rich filter operators, atomic field updates, and aggregation pipelines:

- **Document Insertions**: `insertOne(doc)`, `insertMany(docs)`.
- **Queries & Comparison Operators**: `findOne(filter)`, `find(filter)`, `$gt`, `$gte`, `$lt`, `$lte`, `$ne`, `$in`.
- **Updates & Modifiers**: `updateOne(filter, update)`, `updateMany(filter, update)`, `$set`, `$inc`, `$unset`.
- **Deletions & Counting**: `deleteOne(filter)`, `deleteMany(filter)`, `count(filter)`.
- **Aggregation Pipelines**: `aggregate(pipeline)` supporting `$match`, `$group`, `$sum`, and `$avg`.
- **Connection Lifecycle**: `open(uri)`, `db(name?)`, `collection(name)`, `ping()`, `close()`.

```aura
import { open, MongoClient, MongoDatabase, MongoCollection } from "std/database/mongodb";

// Connect to MongoDB or run embedded standalone store
let client: MongoClient = mongodb.open(":memory:");
defer client.close();

let db: MongoDatabase = client.db("catalog");
let products: MongoCollection = db.collection("products");

// Insert Documents
let insertResult = products.insertOne({
    sku: "LAPTOP-M3",
    name: "MacBook Pro 16",
    category: "Computers",
    price: 2499,
    stock: 15,
    status: "in_stock"
});
println(`Inserted ID: ${insertResult.insertedId}`);

// Find Document with Filter
let found = products.findOne({ sku: "LAPTOP-M3" });
println(`Found product: ${found}`);

// Atomic Field Update ($set and $inc)
products.updateOne(
    { sku: "LAPTOP-M3" },
    { $set: { status: "discounted" }, $inc: { stock: -1 } }
);

// Query with Range Filters ($gt, $lt, $in)
let expensive = products.find({ price: { $gt: 1000 } });
println(`Expensive products: ${expensive}`);

// Document Count
let totalCount = products.count({});
println(`Total items in catalog: ${totalCount}`);

// Aggregation Pipeline ($match + $group)
let pipeline = [
    { $match: { category: "Computers" } },
    { $group: { _id: "$category", totalValue: { $sum: "$price" }, avgPrice: { $avg: "$price" } } }
];
let stats = products.aggregate(pipeline);
println(`Category Stats: ${stats}`);
```

---

### 🧩 System Modules: `os`, `time`, `crypto`, `jwt`, and `json`

- **`os`**: `os.args`, `os.env("KEY")`, `os.getEnv("KEY")`, `os.setEnv("K", "V")`, `os.readFile(path)`, `os.writeFile(path, data)`, `os.exit(0)`.
- **`time`**: `time.now()`, `time.isoString()`, `time.sleep(ms)`.
- **`crypto`**: `crypto.sha256(text)`, `crypto.hmacSha256(key, message)`, `crypto.base64UrlEncode(data)`, `crypto.base64UrlDecode(data)`.
- **`jwt`**: `jwt.sign(payload, secret)`, `jwt.verify(token, secret)`.
- **`JSON`**: `JSON.stringify(val)`, `JSON.parse(jsonText)`.

---

### 📚 Production REST Microservice: `bookstore_api`

The repository includes a complete enterprise REST microservice in [`examples/bookstore_service.aura`](examples/bookstore_service.aura) validated by automated E2E tests:

- Structured modeling with Structs, Tags (`json:"..."`), and ADTs for order states.
- Complete catalog of **Books, Authors, and Publishers**.
- Stock validation and tax calculations (19% VAT).
- Query endpoints with pagination and search filters.
- Interactive OpenAPI 3.0 / Swagger documentation.

---

## 🐛 Interactive Debugging and Source Maps V3

### Terminal Step-by-Step Debugger (`aurac step`)

Aura includes an interactive terminal debugger allowing developers to step through code execution line by line:

```bash
aurac step examples/defer_demo.aura
```

```
⚡ Aura Interactive Step Debugger
Target file: defer_demo.aura

📍 [defer_demo.aura:28] in main()
      27 | export fn main(): Unit => {
  ➜   28 |     println("--- Defer Demonstration ---");
      29 |     let res = executeQuery();

(aura-dbg) next
(aura-dbg) into
(aura-dbg) vars
(aura-dbg) break 35
(aura-dbg) continue
(aura-dbg) backtrace
```

| Command         | Shortcuts      | Action                                                                  |
| :-------------- | :------------- | :---------------------------------------------------------------------- |
| `next`          | `n`, `<ENTER>` | **Step Over**: Advances to the next source line.                        |
| `into`          | `s`, `step`    | **Step Into**: Enters into the function being called.                   |
| `out`           | `o`, `finish`  | **Step Out**: Executes until returning from the current function.       |
| `continue`      | `c`            | **Continue**: Resumes execution until the next breakpoint.              |
| `break <line>`  | `b <line>`     | **Breakpoint**: Toggles a breakpoint on the specified line.             |
| `vars`          | `locals`       | **Variables**: Displays all currently active local variables.           |
| `backtrace`     | `bt`, `stack`  | **Stack**: Prints the call stack mapped back to `.aura` files.          |
| `print <expr>`  | `p <expr>`     | **Evaluate**: Dynamically evaluates an expression in local scope.       |
| `quit`          | `q`, `exit`    | **Quit**: Ends the debugging session.                                   |

### V8 / DAP Debugger Server (`aurac debug`)

Starts a debug server enabling connections from modern IDEs or Chrome DevTools using Source Maps V3:

```bash
# Start and break at entrypoint (port 9229)
aurac debug src/main.aura

# Connect on custom port without breaking at start
aurac debug src/main.aura --port 9300 --no-brk
```

Compatible with:

- **Google Antigravity IDE / VS Code**: Immediate connection via `F5` (*Attach to Aura Process*).
- **Zed / Neovim**: Standard DAP protocol on `127.0.0.1:9229`.
- **Google Chrome**: Via `chrome://inspect` in the browser.

---

## 🛠️ Full Tooling and CLI Reference

### 1. Unified Toolchain & Core Compiler (`aura` / `aurac`)

The primary command is `aura` (with `aurac` fully supported as an identical alias):

```bash
# 1. Compile to standalone native binary with Cranelift (default)
aura build main.aura -o dist/my-app
# or via the aurac alias:
aurac build main.aura -o dist/my-app

# 2. Compile with release optimizations (-O3, SIMD, dead-code strip)
aurac build main.aura -o dist/my-app --release -O3

# 3. Hermetic cross-compilation (Zig model)
aurac build main.aura -o dist/my-app-linux --target linux/amd64
aurac build main.aura -o dist/my-app-arm64 --target linux/arm64
aurac build main.aura -o dist/my-app.exe   --target windows/amd64
aurac build main.aura -o dist/my-app-musl  --target x86_64-unknown-linux-musl

# 4. Compile via Go toolchain
aurac build main.aura -o dist/my-app --target go

# 5. Emit pure Go source code (.go)
aurac emit-go main.aura -o dist/main.go

# 6. Direct instant execution
aurac run main.aura

# 7. Ultra-fast static type checking without code generation
aurac check main.aura

# 8. Interactive watch mode (recompiles and runs on save)
aurac watch main.aura --run

# 9. Conditional compilation via build tags
aurac build main.aura --tags "premium,darwin"

# 10. Ingest external TypeScript type definitions (.d.ts)
aurac build main.aura -o dist/my-app --dts node_modules/@types/node/index.d.ts

# 11. Run tests directly from the compiler
aurac --test
aurac --test ./...
aurac main_test.aura --test
```

### 2. Test, Benchmark, and Coverage Runner (`aurac --test` / `auratest`)

Tests can be executed directly via the compiler (`aurac --test` / `aurac <file> --test`) or using the dedicated test runner (`auratest`):

```bash
# Run all tests across the project
aurac --test
auratest ./...
aurac test ./...

# Run a specific test file directly via compiler
aurac examples/tests/math_unit_test.aura --test
aurac --test examples/tests/math_unit_test.aura

# Verbose mode with test names, logs, and individual timings (-v / --verbose)
aurac --test -v ./...
aurac examples/tests/math_unit_test.aura --test -v

# Filter tests or subtests by pattern (-run <pattern> or -run <Parent>/<Subtest>)
aurac --test -run TestUserAuthentication ./...
aurac --test -v -run TestMultiplyTableDriven/positive examples/tests/golang_style_test.aura
aurac examples/tests/math_unit_test.aura --test -run test_addition

# Filter tests by category (--unit, --integration, --e2e)
aurac --test --unit ./...
aurac --test --integration ./...
aurac --test --e2e ./...

# Conditional testing via build tags (--tags <tags>)
aurac --test --tags "integration,db" ./...

# Run benchmarks with operations per second report (ns/op)
aurac --test -bench . ./...

# Statement and line code coverage report in terminal
aurac --test --coverage
aurac examples/tests/math_unit_test.aura --test --coverage

# Generate coverage profile for CI/CD and interactive HTML report
aurac --test --coverage -coverprofile=coverage.out --coverage-html=coverage.html ./...

# Stop execution on first test failure (-failfast)
aurac --test -failfast ./...

# Run with timeout limit (-timeout <duration>) and repetitions (-count <n>)
aurac --test -timeout 10s -count 2 ./...

# Run test files in parallel (-parallel <n>)
aurac --test -parallel 4 ./...

# Machine-readable JSON output for CI/CD pipelines
aurac --test -json ./...

# List discovered tests without executing them (-l / --list)
aurac --test -l ./...

# Watch mode to automatically re-run tests on file changes (-w / --watch)
aurac --test -w ./...

# Display complete testing help and flag reference
aurac --test --help
```

### 3. Opinionated Code Formatter (`aurafmt`, `gofmt` Style)

```bash
# Format all .aura files recursively in-place
aurafmt -w .
# or via aurac:
aurac fmt -w .

# Show unified diff without modifying files
aurafmt -d src/main.aura

# List files with inconsistent formatting
aurafmt -l .

# Strict verification mode for CI pipelines (exit code != 0 on mismatch)
aurafmt -c .
```

### 4. Language Server (`auralsp` / `aurac lsp`)

```bash
aurac lsp
# or directly:
auralsp
```

Provides real-time diagnostics, hover with type signatures and documentation, Go-to-Definition, symbol renaming, autocomplete, and format on save.

### 5. Decentralized Module Manager (`auramod` / `aurac mod`)

```bash
# Initialize a new module with an aura.mod manifest
aurac mod init github.com/myuser/my-app

# Download and install a remote dependency
aurac mod get github.com/aura-lang/crypto@v1.0.0

# Automatically tidy and sync dependencies from source files (.aura)
aurac mod tidy

# Vendor all dependencies into ./vendor for 100% offline builds
aurac mod vendor

# Verify SHA-256 cryptographic integrity against aura.lock
aurac mod verify

# Visualize the module dependency graph
aurac mod graph
```

### 6. C/Rust FFI Binding Generator (`aurabindgen` / `aurac bindgen`)

```bash
# Generate typed Aura bindings from C header files
aurac bindgen /usr/include/sqlite3.h -o src/sqlite3.aura --name SQLite3 --strip-prefix "sqlite3_"
```

### 7. Interactive Web Playground

```bash
aurac playground --port 3000
```

Open in your browser: [http://localhost:3000](http://localhost:3000).

---

## 🧩 Editor Configuration

### Google Antigravity IDE

First-class support with intelligent agents located in [`editors/antigravity/`](editors/antigravity):

- **Integrated LSP**: Bidirectional static inference, autocomplete, and automatic formatting with `aurafmt`.
- **AI Pair Programming**: Intelligent completion (`⌘+I`) configured with Aura guidelines (`.agents/plugins/aura-lang/rules/AGENTS.md`).
- **Extension Installation**:
  ```bash
  antigravity --install-extension editors/antigravity/aura-antigravity-0.1.0.vsix
  ```

### Visual Studio Code

Official extension located in [`editors/vscode/`](editors/vscode):

```bash
cd editors/vscode && npm install && npm run package
```

### Neovim (`nvim-lspconfig`)

Add to your `init.lua`:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.aura then
  configs.aura = {
    default_config = {
      cmd = { 'aurac', 'lsp' },
      filetypes = { 'aura' },
      root_dir = lspconfig.util.root_pattern('aura.mod', '.git', 'Cargo.toml'),
      settings = {},
    },
  }
end
lspconfig.aura.setup{}
```

### Zed Editor

Use the extension bundle located in [`editors/zed/`](editors/zed).

---

## 🔄 Self-Hosted Compiler and Bootstrapping

Aura includes a **fully self-hosted compiler** written entirely in pure Aura, located in [`src/aura_compiler/`](src/aura_compiler). It embodies all 4 major compiler engineering phases:

- [`ast.aura`](src/aura_compiler/ast.aura) — Formal AST definitions, `SymbolTable` string interning, source spans, and diagnostic records.
- [`diagnostics.aura`](src/aura_compiler/diagnostics.aura) — Terminal diagnostic formatter (Rust/Ariadne style) and Levenshtein similarity distance for auto-suggestions (*"Did you mean '...'?"*).
- [`lexer.aura`](src/aura_compiler/lexer.aura) — Deterministic lexical analyzer and token stream generator.
- [`parser.aura`](src/aura_compiler/parser.aura) — Recursive descent Pratt parser with operator precedence.
- [`typechecker.aura`](src/aura_compiler/typechecker.aura) — Full static semantic analyzer and Hindley-Milner type inferer in pure Aura.
- [`codegen.aura`](src/aura_compiler/codegen.aura) — High-performance JavaScript / Node.js ES6 code generator.
- [`codegen_go.aura`](src/aura_compiler/codegen_go.aura) — Direct-to-Go transpiler emitting idiomatic, compilable Go 1.21+ source code.
- [`main.aura`](src/aura_compiler/main.aura) — Compiler pipeline entrypoint with multi-target CLI options (`--target js|go`, `--check`).

---

### 🏛️ Architectural Improvement Phases

The self-hosted compiler implements four modular enhancement phases:

```mermaid
graph TD
    A["Aura Source Code (.aura)"] --> B["Lexer (lexer.aura)"]
    B --> C["Pratt Parser (parser.aura)"]
    C --> D["AST + SymbolTable (ast.aura)"]
    D --> E["Semantic Typechecker (typechecker.aura)"]
    E -->|--check| F["Diagnostics & Fuzzy Suggestions (diagnostics.aura)"]
    E -->|--target js| G["JavaScript CodeGen (codegen.aura)"]
    E -->|--target go| H["Golang CodeGen (codegen_go.aura)"]
    G --> I["dist/*.mjs (Node.js / Web)"]
    H --> J["Standalone Native Binary (go build)"]
```

#### Phase 1: Semantic Typechecker in Pure Aura ([`typechecker.aura`](src/aura_compiler/typechecker.aura))
- **Semantic Type Representation (`SemanticType`)**: Formal type hierarchy supporting `STyInt`, `STyFloat`, `STyString`, `STyBool`, `STyUnit`, `STyAny`, `STyOption`, `STyResult`, `STyList`, `STyRecord`, `STyTuple`, `STyFunction`, `STySum`, `STyChannel`, and `STyCustom`.
- **Lexical Type Environment (`TypeEnv`)**: Tracks variable bindings (`bindVar`), function signatures (`bindFunction`), sum type variants (`SemanticSumType`), and type aliases (`SemanticTypeAlias`).
- **Expression & Statement Verification**: Typechecks arithmetic/logical binary operators, branch unification in `if/else`, while-loop boolean conditions, for-in collection iteration types, and exhaustive pattern match arm consistency.
- **Strict Mutability & Signature Enforcement**: Validates immutable vs mutable bindings (`let` vs `let mut`), reassignments (`checkAssign`), let annotations (`checkLetAnnotation`), and function return types (`checkReturnType`).

#### Phase 2: Native Golang Code Generator ([`codegen_go.aura`](src/aura_compiler/codegen_go.aura))
- **AST to Go 1.21+ Transpilation**: Converts Aura AST into clean, idiomatic Go source ready for `go build`.
- **Type Mapping**: Direct mapping of Aura types (`Int` $\rightarrow$ `int64`, `Float` $\rightarrow$ `float64`, `String` $\rightarrow$ `string`, `Bool` $\rightarrow$ `bool`, `Option<T>` $\rightarrow$ `*T`, `List<T>` $\rightarrow$ `[]T`, `Channel<T>` $\rightarrow$ `chan T`).
- **Visibility & Control Flow**: Automatic uppercase identifier capitalization for exported symbols, pattern matching compilation to Go `switch` statements, and module-level top statement aggregation into `func main()` or `func init()`.

#### Phase 3: String Interning, Rich Diagnostics & Levenshtein Search ([`ast.aura`](src/aura_compiler/ast.aura), [`diagnostics.aura`](src/aura_compiler/diagnostics.aura))
- **Symbol Table (`SymbolTable`)**: String interning (`internSymbol`, `lookupSymbol`) for memory-efficient identifier reuse.
- **Rich Diagnostic Formatting**: Ariadne-style error output with severity levels (`DiagError`, `DiagWarning`, `DiagInfo`, `DiagHint`), error codes (`E0308`, `E0425`, `E0384`, `E0004`), help notes, and line context.
- **Fuzzy Identifier Resolution**: Zero-allocation Levenshtein distance algorithm (`findClosestMatch`) that computes edit distance $\le 3$ to provide auto-corrections (*"Did you mean 'msg'?"*) on unresolved symbols.

#### Phase 4: Multi-Target Pipeline & 5-Stage Bootstrap ([`main.aura`](src/aura_compiler/main.aura), [`tests/bootstrap_tests.rs`](tests/bootstrap_tests.rs))
- **Multi-Target CLI Pipeline**: Full CLI options supporting `--target js` (default), `--target go`, fast typecheck-only mode `--check`, and custom output `-o <path>`.
- **Triple-Stage Bootstrapping**: Validates that Rust compiles Aura compiler $\rightarrow$ Aura compiler compiles Aura programs $\rightarrow$ Aura compiler compiles itself into a **6.43 MB** standalone native binary (Mach-O / ELF).

---

### 🚀 Bootstrap Workflow & CLI Usage

#### 1. Stage 1: Compile All 8 Self-Hosted Modules
Compile the self-hosted compiler modules using the reference Rust compiler (`aurac`):

```bash
mkdir -p dist
aurac compile src/aura_compiler/ast.aura -o dist/ast.mjs
aurac compile src/aura_compiler/diagnostics.aura -o dist/diagnostics.mjs
aurac compile src/aura_compiler/lexer.aura -o dist/lexer.mjs
aurac compile src/aura_compiler/parser.aura -o dist/parser.mjs
aurac compile src/aura_compiler/typechecker.aura -o dist/typechecker.mjs
aurac compile src/aura_compiler/codegen.aura -o dist/codegen.mjs
aurac compile src/aura_compiler/codegen_go.aura -o dist/codegen_go.mjs
aurac compile src/aura_compiler/main.aura -o dist/aurac.mjs
```

#### 2. Stage 2: Compile Programs with the Self-Hosted JS Compiler (`dist/aurac.mjs`)
```bash
# Compile to JavaScript
node dist/aurac.mjs examples/bootstrap_demo/hello.aura -o dist/hello.js --target js
node dist/hello.js
# Output:
# Hello Developer from Self-Hosted Aura Compiler!
# Sum of 1..100: 5050

# Fast static typecheck mode
node dist/aurac.mjs examples/bootstrap_demo/hello.aura --check
# Output: ✓ Checked 'examples/bootstrap_demo/hello.aura': 0 errors

# Compile to Golang source
node dist/aurac.mjs examples/bootstrap_demo/hello.aura -o dist/hello.go --target go
```

#### 3. Stage 3: Build the Self-Hosted Compiler into a Standalone Native Binary
Compile the entire self-hosted compiler into a single native binary (zero external dependencies):

```bash
aurac build src/aura_compiler/main.aura -o dist/aura-self-hosted
```

#### 4. Stage 4: Compile Aura Programs Directly with the Native Binary
```bash
./dist/aura-self-hosted examples/hello.aura -o dist/hello_from_native.js
node dist/hello_from_native.js
# Output: Hello Developer from Self-Hosted Aura Compiler!
```

#### 5. Stage 5: Automated 5-Stage Bootstrap Verification Test Suite
Run the automated end-to-end bootstrap integration tests:

```bash
cargo test --test bootstrap_tests -- --test-threads=1
```

---

## 🛡️ Security and Runtime Hardening (v0.8.1)

Aura v0.8.1 introduces extensive defense-in-depth security hardening across the compiler, typechecker, garbage collector, and native standard runtime:

### Evaluated Risk Vectors and Results / Vectores de Riesgo Evaluados y Resultados

| Risk Vector / Vector de Riesgo | Potential Risk / Riesgo Potencial | Aura Defense Mechanism / Mecanismo de Defensa en Aura | Status / Estado |
| :--- | :--- | :--- | :--- |
| **Data Races / Raw Pointer Captures** | Memory corruption and thread inconsistency | **Error `[E0401]`**: Typechecker rejects capturing raw pointers (`*T`) inside `spawn` closures or sending them across CSP channels at compile time. | 🟢 **Protected** |
| **Incomplete Pattern Matching** | Runtime panics or unhandled execution branches | **Match Exhaustiveness**: Enforces complete branch coverage on `Option<T>`, `Result<T, E>`, and Sum Types, or explicit wildcard `_`. | 🟢 **Protected** |
| **Recursion Stack Overflow** | Call stack exhaustion in recursive algorithms | **TCO (Tail-Call Optimization)**: Automatically converts recursive calls into zero-overhead iterative `while (true)` loops with $O(1)$ stack cost. | 🟢 **Protected** |
| **Cryptographic Timing Attacks** | Token / key leakage through side-channel latency | Codegen protected with `crypto.timingSafeEqual` (JS/Node) and `hmac.Equal` (Go) for constant-time comparisons. | 🟢 **Protected** |
| **Resource Leaks / Failed Transactions** | Descriptor starvation and corrupted application state | **LIFO `defer` & `errdefer`**: Guaranteed deterministic cleanup and automatic transactional rollback upon returning `Err`. | 🟢 **Protected** |
| **JSON Depth DoS (JSON Bomb)** | Recursive call stack overflow via deeply nested payloads | Runtime enforcement of `MAX_JSON_DEPTH = 64` across parser entrypoints (`crates/aura-runtime/src/json.rs`). | 🟢 **Protected** |
| **Fiber Buffer Overflow** | Fiber heap corruption under deep execution frames | **Stack Canaries**: `verify_canary()` in runtime detects stack buffer tampering before recycling fiber memory. | 🟢 **Protected** |
| **Channel Saturation & Leaks** | Fiber deadlocks from orphaned or full CSP channels | CSP channels safely drain buffered items and emit clean `None` EOF values upon `Channel.close()`. | 🟢 **Protected** |

### Detailed Subsystem Hardening Matrix

| Subsystem / Layer | Vulnerability Addressed | Hardening & Mitigation Implemented |
|---|---|---|
| **Type Checker** | Incomplete pattern match exhaustiveness | Strict compile-time validation for `Option<T>`, `Result<T, E>`, and custom ADT sum types (record & tuple variants). Eliminates unhandled branch crashes. |
| **Native Code Generator** | Variant tag collision in Cranelift (`aura_variant_new`) | Deterministic hashing with `FxHasher` (`compute_variant_tag`) replacing sequential index collisions across distinct types. |
| **Memory / GC Engine** | Lock contention & GC data races | Thread-Local Allocation Buffers (TLAB) with amortized batch flushing; Stop-The-World (STW) safepoints (`GC_WORLD_LOCK`); $O(\log N)$ binary search pointer marking. |
| **Fiber Stack Integrity** | Stack buffer overflow in fiber execution | Hardware guard pages (`mprotect`) plus `STACK_CANARY_MAGIC` validation on stack allocation, reuse, and recycling. |
| **Database Subsystem** | SQL Injection & Parameter Misalignment | Strictly typed parameter formatting and dialect-specific placeholder binding (`bind_postgres_sql` with `$1..$N`, `bind_mysql_sql` with `?`), plus handle type validation preventing pointer confusion. |
| **Database Lifecycle** | Double-free & dangling pointers in `Tx`/`Pool` | Idempotent transaction and connection pool closing with atomic de-registration from runtime pool registry. |
| **HTTP Web Server** | HTTP Response Splitting, Slowloris & DoS | Response header sanitization (CRLF rejection); dual read/write socket timeouts (5s/10s); method validation (405); auto-injection of `X-Content-Type-Options: nosniff` and `X-Frame-Options: SAMEORIGIN`. |
| **TLS / HTTPS Subsystem** | Insecure silent fallback / false TLS indicator | Server startup aborts with explicit error code (`-1`) and log warnings if TLS configuration is supplied without linked native TLS provider. |
| **JSON Parser** | Stack Overflow DoS via nested payloads | Recursive depth validation enforcing `MAX_JSON_DEPTH = 64` across all native parser entrypoints. |
| **Crypto & Auth** | Side-Channel Timing Attacks on JWT | Constant-time signature verification using `timingSafeEqual` (Node.js) and `hmac.Equal` (Go). |
| **Redis & Mongo Clients** | Remote hostname resolution & credential leaks | Native DNS resolution via `ToSocketAddrs`, full RESP multi-packet response framing, and URI credential stripping in logging. |
| **OS Subsystem** | Data races & memory unsafety in environment vars | Global `ENV_LOCK` Mutex for `aura_os_setenv` and `aura_os_env`, with strict validation rejecting null bytes (`\0`) and `=` in variable names. |
| **Supply Chain / Modules** | Argument and protocol injection in `auramod` / `aurapkg` | Path and version string sanitization against path traversal, command injection, and illegal git protocol characters. |
| **Static Immutability** | Accidental mutation & state drift | Default immutability strictly enforced at typecheck time (`E0400`). Variables declared with `let` cannot be reassigned; `let mut` required explicitly. |
| **Arithmetic Safety** | Hardware `SIGFPE` crash on modulo / division by zero | Cranelift codegen emits conditional zero-divisor trap-safe checks for `BinOp::Mod` matching `BinOp::Div`. |
| **Formal Type Unwrapping** | Undefined unwrapping of non-monadic types with `?` | Strict typechecker inference (`E0302`) unwrapping `Result<T, E> -> T` and `Option<T> -> T`, rejecting arbitrary types. |
| **Fiber Concurrency Boundary** | Data races and raw pointer escapes across fibers | Static verification on `spawn` expressions rejecting raw pointers (`*T`, `E0401`) captured inside concurrent closures. |
| **Garbage Collector** | Recursion stack overflow on deep object graphs | Iterative mark worklist (`Vec<usize>`) replacing recursive call frames in GC tracing. |
| **Adaptive Memory GC** | Memory leak & continuous Stop-The-World stalls | Atomic subtraction of reclaimed bytes in sweep phase with adaptive threshold doubling (`(live * 2).max(8MB)`), preventing infinite GC trigger loops. |
| **PostgreSQL Authentication** | Incompatible modern PG auth & credential interception | Pure-Rust `SCRAM-SHA-256` authentication (RFC 7677 / RFC 5802) via standalone SHA-256, HMAC-SHA-256, and PBKDF2, supporting PostgreSQL 14+, Neon, Supabase, and AWS RDS without `libpq`. |
| **Database Pool Limiting** | Connection starvation and socket exhaustion under spikes | Strict `active_conns` counting in `AuraDbPool` with `Condvar` synchronization (15s timeout), preventing database collapse under concurrent load. |
| **Typechecker Scopes** | $O(N^2)$ memory bloat from recursive scope cloning | $O(1)$ scope entry with `Arc<TypeEnv>` and thread-safe singleton initialization (`OnceLock`) of the standard built-in type catalog. |
| **CLI Runner Isolation** | Race conditions & residual files in `aura run` | Unique process/timestamp-tagged temporary scripts with RAII `CleanupGuard` guaranteeing immediate cleanup upon termination. |
| **HTTP Concurrency Scaling** | Thread contention & OS overhead with static worker pools | Dynamic worker pool sizing based on `std::thread::available_parallelism()`, optimizing socket acceptance latency. |
| **Playground API Hardening** | JSON injection & control character corruption | Fully typed JSON parsing and serialization backed by `serde_json` replacing manual substring extractors. |
| **Security & Quality Linter** | SQL injection and exposed credentials | Integrated static analysis tool (`aura lint`) detecting unparameterized SQL templates (`SEC001`), hardcoded credentials (`SEC002`), and unused mutability (`LINT001`). |
| **CSPRNG Cryptographic Primitives** | Predictable pseudo-randomness in tokens/keys (CWE-338) | Built-in `crypto` standard module functions: `crypto.randomBytes(size)`, `crypto.randomUUID()`, and `crypto.timingSafeEqual()` backed by `crypto/rand` and `crypto/hmac.Equal`. Linter rule `SEC003` flagging `Math.random()` on sensitive variables. |
| **SQL Query Safety (OWASP ASVS)** | SQL Injection via string interpolation (CWE-89) | Strongly typed `SqlQuery` interface and `sqlQuery(text, params)` constructor in `std/database/sql` enforcing safe parameterized database operations. |
| **HTTP Defense-in-Depth Headers** | Clickjacking, MIME Confusion & lack of CSP (CWE-693) | Helper `applySecurityHeaders(w)` in `std/net/http` automatically setting `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`, and `Content-Security-Policy`. |
| **Web Playground Hardening** | Cross-Site Scripting (XSS / CWE-79 / CWE-1021) | Removed `'unsafe-eval'` directives from the Playground Content-Security-Policy header and enforced strict referrer policies. |

---

## 🗺️ Current Status, Roadmap, and Milestones

- [x] **Language Core**: Hindley-Milner type inference, algebraic data types (ADTs), exhaustive pattern matching, TCO optimization, and default immutability.
- [x] **Complete CSP Concurrency**: Lightweight fibers on M:N scheduler (`spawn`), typed channels with buffers, directional channels, channel iteration, `select` multiplexing, synchronization (`Mutex`, `RWMutex`, `WaitGroup`, `Once`, `Pool`), and `Context`.
- [x] **Static Concurrency Safety**: Compile-time `Sendable` validation and runtime Deadlock Sentinel.
- [x] **Golang Model Slices**: `[]T` syntax, `s[low:high:max]` slicing, built-in functions `len`, `cap`, `append`, `make`, and string slicing.
- [x] **Cleanup Semantics**: LIFO `defer` and Zig-inspired transactional `errdefer`.
- [x] **Method-Oriented Programming**: Pure data structs, receiver methods (`fn (r: Recv) Method()`), case-based Go visibility, and structural duck typing without `implements`.
- [x] **Native Cranelift Backend**: Direct native machine code emission for macOS (Mach-O) and Linux (ELF) with static runtime (`libaura_runtime.a`).
- [x] **Golang Backend & Transpilation**: Idiomatic Go code generation (`aurac emit-go`) and `--target go` cross-compilation.
- [x] **Standard Library & Databases**: Ultra-high-performance HTTP ServeMux server (**126k+ req/s**), dynamic worker scaling, `/metrics` Prometheus endpoint, OpenTelemetry `traceparent`, Swagger UI, and native drivers for PostgreSQL (with `SCRAM-SHA-256`, MD5, and Cleartext), MySQL, MongoDB, and Redis with bounded `Condvar` connection pool synchronization.
- [x] **Decentralized Package Manager**: `aura.mod`, cryptographic `aura.lock`, offline vendoring, and integrity checks (`auramod` / `aurac mod`).
- [x] **C/Rust FFI Generator (`aurabindgen`)**: Automatic conversion of `.h` headers into safe Aura interfaces.
- [x] **Production-Grade Tooling**: Official LSP server (`auralsp`), terminal step debugger (`aurac step`), V8 DAP server (`aurac debug`), test runner (`auratest`), opinionated formatter (`aurafmt`), and type-safe `serde_json` Web Playground.
- [x] **Rust 2024 Modernization & Toolchain Optimization**: Centralized Cargo workspace dependencies and lints, deduplicated crate tree (`object 0.39`), Thin LTO release profile (`[profile.release]`), `RwLock` + `FxHashMap` concurrent runtime GC, zero-overhead static scheduler, zero-alloc HTTP metric formatting, $O(1)$ scope entry with `Arc<TypeEnv>`, and structured `TypeError` diagnostics.
- [x] **Security & Runtime Hardening (v0.8.1)**: TLAB lock-free GC allocations, adaptive memory GC with dynamic thresholding, binary search GC marking, fiber stack canaries, constant-time JWT verification (`timingSafeEqual` / `hmac.Equal`), native CSPRNG primitives (`crypto.randomBytes`, `crypto.randomUUID`, `crypto.timingSafeEqual`), linter rule `SEC003` against insecure `Math.random`, typed `SqlQuery` objects (OWASP ASVS), HTTP defense headers (`applySecurityHeaders`), hardened playground CSP without `unsafe-eval`, JSON DoS depth limiting (depth <= 64), Slowloris socket timeouts, HTTP security headers (`nosniff`, `SAMEORIGIN`), Stop-The-World GC synchronization, SQL injection prevention ($1..$N, ?), strict DB pool bounding, pure-Rust PostgreSQL `SCRAM-SHA-256` auth, compile-time match exhaustiveness, supply-chain package sanitization, isolated process-unique `aura run` temp execution with RAII cleanup, static immutability enforcement (`let mut`), Cranelift modulo zero safety (`SIGFPE` guard), formal static type unwrapping for `?`, fiber boundary raw pointer safety (`spawn`), iterative GC worklist, raw string literals (`r"..."`), and integrated static security linter (`aura lint`).
- [x] **Quality Suite & Tests**: **270+ automated tests passing successfully** across the repository.
- [ ] **WebAssembly Backend (WASM/WASI)**: WebAssembly binary emission for serverless edge microservices.
- [ ] **Advanced LLVM Optimizations**: Optional LTO optimization pipeline and SIMD vectorization for compute-intensive workloads.

---

## 📄 License

Aura Language is free and open-source software released under the [MIT License](LICENSE).

### ⚖️ Trademarks

> Go is a trademark of Google LLC. Rust is a trademark of the Rust Foundation. TypeScript is a trademark of Microsoft Corp. Zig is a trademark of the Zig Software Foundation. Aura Lang is an independent open-source project and is not affiliated with or endorsed by these entities.
