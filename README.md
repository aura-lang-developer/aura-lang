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
2. [✨ Key Features](#-key-features)
3. [⚡ Benchmarks and Empirical Performance](#-benchmarks-and-empirical-performance)
   - [Execution Times and Peak RSS Memory](#1-execution-times-and-memory-consumption-peak-rss)
   - [Standalone Binary Sizes](#2-standalone-binary-sizes-zero-dependencies)
   - [Compilation and Static Checking Times](#3-compilation-and-static-typecheck-times)
   - [HTTP Server and REST Microservice Benchmark](#4-http-server--rest-microservice-10000-requests-concurrency--50)
   - [Compilation Throughput vs TypeScript (`tsc`)](#5-compilation-throughput-vs-typescript-tsc)
   - [How to Reproduce the Benchmark Suite](#6-how-to-run-the-benchmark-suite)
4. [📁 Compiler and Runtime Architecture](#-compiler-and-runtime-architecture)
   - [Pipeline Flow Diagram](#pipeline-flow-diagram)
   - [Source Code Organization](#source-code-organization)
   - [The Standalone Native Runtime (`aura-runtime`)](#the-standalone-native-runtime-cratesaura-runtime)
5. [📘 Complete Aura Language Guide](#-complete-aura-language-guide)
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
6. [🗄️ Standard Library, Web Server, and Databases](#️-standard-library-web-server-and-databases)
   - [HTTP Web Server & ServeMux Router (`net/http`)](#-http-web-server-and-routing-mux-nethttp)
   - [Prometheus Metrics and W3C Traceparent Tracing](#-prometheus-metrics-and-distributed-tracing)
   - [Native PostgreSQL Driver (`pg` / `postgres`)](#-native-postgresql-driver-pg--postgres)
   - [Native MySQL Driver (`mysql`)](#-native-mysql-driver-mysql)
   - [Native MongoDB Driver (`mongodb` / `mongo`)](#-native-mongodb-driver-mongodb--mongo)
   - [Native Redis Client (`redis`)](#-native-redis-client-redis)
   - [System Modules: OS, Time, Crypto, JWT, and JSON](#-system-modules-os-time-crypto-jwt-and-json)
   - [Production REST Microservice: `bookstore_api`](#-production-rest-microservice-bookstore_api)
7. [🐛 Interactive Debugging and Source Maps V3](#-interactive-debugging-and-source-maps-v3)
   - [Terminal Step-by-Step Debugger (`aurac step`)](#terminal-step-by-step-debugger-aurac-step)
   - [V8 / DAP Debugger Server (`aurac debug`)](#v8--dap-debugger-server-aurac-debug)
8. [🛠️ Full Tooling and CLI Reference](#️-full-tooling-and-cli-reference)
   - [Core Compiler (`aurac`)](#1-core-compiler-aurac)
   - [Test, Benchmark, and Coverage Runner (`auratest`)](#2-test-benchmark-and-coverage-runner-auratest)
   - [Opinionated Code Formatter (`aurafmt`)](#3-opinionated-code-formatter-aurafmt-gofmt-style)
   - [Language Server (`auralsp`)](#4-language-server-auralsp--aurac-lsp)
   - [Decentralized Module Manager (`auramod` / `aurac mod`)](#5-decentralized-module-manager-auramod--aurac-mod)
   - [C/Rust FFI Binding Generator (`aurabindgen`)](#6-c-rust-ffi-binding-generator-aurabindgen--aurac-bindgen)
   - [Interactive Web Playground (`aurac playground`)](#7-interactive-web-playground)
9. [🧩 Editor Configuration](#-editor-configuration)
10. [🔄 Self-Hosted Compiler and Bootstrapping](#-self-hosted-compiler-and-bootstrapping)
11. [🗺️ Current Status, Roadmap, and Milestones](#️-current-status-roadmap-and-milestones)
12. [📄 License and Trademarks](#-license)

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
git clone https://github.com/mrojasb2000/aura-lang.git
cd aura-lang

# Optimized production compilation (Release)
cargo build --release

# Or fast development compilation (Debug)
cargo build
```

The compiled binaries will be placed in `target/release/` (or `target/debug/`):

- `aurac`: Core compiler, native builder, hot-runner, watcher, debugger, and playground runner.
- `auratest`: Complete unit test, benchmark (`ns/op`), and HTML coverage suite.
- `aurafmt`: Fast, idempotent, and opinionated code formatter (`gofmt` style).
- `auralsp`: Language Server Protocol (LSP) daemon compatible with any modern editor.
- `auramod` / `aurapkg`: Decentralized package manager (`aura.mod` and `aura.lock`).
- `aurabindgen`: Automated FFI binding generator from C header files (`.h`).

#### C. Selective Binary Compilation

You can compile individual binaries using the `--bin` flag:

```bash
# Compile only the core compiler
cargo build --release --bin aurac

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

You can add an ergonomic alias to your shell configuration (`~/.zshrc` or `~/.bashrc`):

```bash
alias aura="aurac"
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

| Evaluated Scenario                      | Aura Lang (`aurac`) | Golang (`go`) |     Ratio (Aura vs Go)      | Aura Memory (RSS) | Go Memory (RSS) |     Aura Advantage     |
| :-------------------------------------- | :-----------------: | :-----------: | :-------------------------: | :---------------: | :-------------: | :--------------------: |
| **Recursive Fibonacci ($N=38$)**        |    **41.43 ms**     |   88.04 ms    | **0.47x (2.1x faster)**     |    **2.45 MB**    |     4.09 MB     |  **-40.1% less RAM**   |
| **Sieve of Eratosthenes (2M primes)**   |    **14.62 ms**     |    5.50 ms    |            2.66x            |    **3.05 MB**    |     5.94 MB     |  **-48.6% less RAM**   |
| **Functional Pipeline (1M items)**      |    **14.80 ms**     |    4.21 ms    |            3.52x            |    **2.86 MB**    |    20.02 MB     |  **-85.7% less RAM**   |
| **CSP Ping-Pong Channels (200k msgs)**  |    **25.03 ms**     |   21.15 ms    |            1.18x            |     11.05 MB      |     4.02 MB     |     Near parity        |
| **Concurrent Spawn (50k tasks)**        |    **12.27 ms**     |   10.62 ms    |            1.16x            |   **11.84 MB**    |    13.14 MB     |   **-9.9% less RAM**   |
| **Cold Start (CLI process)**            |     **3.59 ms**     |    2.54 ms    |            1.41x            |    **2.25 MB**    |     3.92 MB     |  **-42.6% less RAM**   |
| **Mutex Synchronization (50k ops)**     |    **16.65 ms**     |   17.34 ms    | **0.96x (Aura faster)**     |     13.47 MB      |     8.17 MB     |   Performance parity   |
| **Concurrent KV Cache (50k ops)**       |    **16.70 ms**     |   19.23 ms    | **0.87x (13% faster)**      |     15.55 MB      |    15.20 MB     |     Memory parity      |

### 2. Standalone Binary Sizes (Zero Dependencies)

| Scenario                                | Go Binary (MB) | Go Stripped (`-s -w`) | Aura Standalone Binary |               Ratio Aura vs Go                |
| :-------------------------------------- | :------------: | :-------------------: | :--------------------: | :-------------------------------------------: |
| **Numerical Calculation (Cranelift)**   |    2.32 MB     |        1.51 MB        |   **2.97 – 2.99 MB**   |                     1.28x                     |
| **Data Pipeline (Cranelift)**           |    2.32 MB     |        1.51 MB        |      **2.99 MB**       |                     1.29x                     |
| **HTTP REST Server / Microservice**     |    8.84 MB     |        5.96 MB        |      **1.78 MB**       | **0.20x (Aura 80% lighter / 5x more compact)**|
| **CSP Concurrency / Channels**          |    2.33 MB     |        1.53 MB        |      **5.06 MB**       |                     2.17x                     |

### 3. Compilation and Static Typecheck Times

| Scenario                 | `aurac check` (HM Typecheck) | `aurac build` (Standalone) | `go build` |
| :----------------------- | :--------------------------: | :------------------------: | :--------: |
| **Recursive Fibonacci**  |          **5.7 ms**          |          36.3 ms           |  37.3 ms   |
| **Sieve of Eratosthenes**|          **5.9 ms**          |          39.2 ms           |  36.1 ms   |
| **Data Pipeline**        |          **5.1 ms**          |          34.4 ms           |  37.5 ms   |
| **CSP Channels**         |          **5.0 ms**          |          69.5 ms           |  34.6 ms   |
| **Concurrent Spawn**     |          **4.3 ms**          |          69.0 ms           |  34.8 ms   |
| **Cold Start (CLI)**     |          **4.0 ms**          |          35.1 ms           |  33.7 ms   |
| **Mutex Synchronization**|          **4.3 ms**          |          69.6 ms           |  34.2 ms   |
| **Concurrent KV Cache**  |          **5.2 ms**          |          70.2 ms           |  35.2 ms   |

### 4. HTTP Server / REST Microservice (10,000 requests, Concurrency = 50)

Throughput and latency percentile evaluation under concurrent requests against JSON endpoints:

| Performance Metric                 | Aura Standalone Server | Go Native Server (`net/http`) |           Aura Advantage               |
| :--------------------------------- | :--------------------: | :---------------------------: | :------------------------------------: |
| **Throughput (req/s)**             |   **126,953 req/s**    |         72,122 req/s          |  🏆 **+76% higher throughput (1.76x)** |
| **Mean Latency**                   |      **0.36 ms**       |            0.66 ms            |       🏆 **45% lower latency**         |
| **Median Latency (p50)**           |      **0.32 ms**       |            0.59 ms            |   🏆 **1.8x faster median**            |
| **95th Percentile Latency (p95)**  |      **0.78 ms**       |            1.35 ms            |   🏆 **42% lower tail latency**        |
| **99th Percentile Latency (p99)**  |      **1.03 ms**       |            1.97 ms            |     🏆 **48% lower p99 latency**       |
| **Distributable Binary Size**      |      **1.78 MB**       |  8.84 MB (5.96 MB stripped)   | 🏆 **5x more compact (-80% disk)**     |

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

### Source Code Organization

- [`src/ast.rs`](src/ast.rs) — Formal AST definitions (expressions, statements, algebraic types, CSP concurrency, receivers, and interfaces).
- [`src/lexer.rs`](src/lexer.rs) — Deterministic lexer with support for pipeline operators (`|>`), channels (`<-`, `chan<-`), pointers, and systems keywords.
- [`src/parser.rs`](src/parser.rs) — Recursive descent and Pratt operator precedence parser.
- [`src/typechecker.rs`](src/typechecker.rs) — Bidirectional Hindley-Milner inference, pattern exhaustiveness, structural duck typing, and concurrency safety rules (`Sendable`).
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

- **M:N Fiber Scheduler (`scheduler.rs`, `fiber.rs`)**: Executes thousands of cooperative fibers over a pool of operating system threads using lightweight stack switching.
- **CSP Channels & Deadlock Sentinel (`channel.rs`)**: Thread-safe concurrent circular queues for message passing, operational channel metrics, and automated deadlock detection.
- **Zero-Alloc HTTP Server (`http.rs`)**: HTTP/1.1 server with persistent Keep-Alive, `TCP_NODELAY`, reusable buffers, coalesced header/body writes, `/metrics` Prometheus endpoint, and W3C `traceparent` propagation.
- **Compact Garbage Collector (`gc.rs`)**: Fast Mark-and-Sweep GC tailored for domain objects and dynamic collections.
- **Synchronization Primitives (`sync.rs`)**: Native Mutex, RWMutex, Once, Pool, and WaitGroup.
- **JSON & Record Serializers (`json.rs`, `record.rs`)**: High-speed binary and text encoding/decoding.

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
    k = k + 1;
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

Aura provides a built-in testing, benchmarking, and code-coverage framework inspired by the Go testing model, powered by `auratest` (or `aurac test`).

#### A. Test Discovery and Categorization Directives

Files matching the following naming conventions are automatically discovered:
- **Test files**: `*_test.aura`, `.test.aura`, `_spec.aura`, `test_*.aura`, or any `.aura` file located in a `tests/` directory.
- **Automatic Categorization**:
  - **Unit Tests (`UNIT`)**: Default for test files, or matching `*_unit_test.aura`.
  - **Integration Tests (`INTEGRATION`)**: Files containing `_integration_test.aura` or located under an `integration/` directory.
  - **End-to-End Tests (`E2E`)**: Files containing `_e2e_test.aura` or located under an `e2e/` directory.
  - **Benchmarks (`BENCHMARK`)**: Functions beginning with `benchmark_`, `Benchmark`, or `bench_`.

#### B. Test and Benchmark Function Signatures

Test functions must be exported and can either accept a test context (`TestingT`) or run parameter-free:

```aura
// 1. Standard unit test taking TestingT
export fn test_addition(t: TestingT) => {
    let sum = 10 + 25;
    t.assertEqual(sum, 35, "10 + 25 must equal 35");
    t.assertTrue(sum > 30, "sum should be greater than 30");
}

// 2. Async test for fibers, channels, and network calls
export async fn test_async_service(t: TestingT) => {
    let result = await fetchUserData(101);
    t.assertTrue(result.isOk(), "fetching user must succeed");
}

// 3. Subtests (t.run) and logical test steps (t.step)
export fn test_order_lifecycle(t: TestingT) => {
    t.step("Verify inventory", fn() => {
        t.assertTrue(checkStock("SKU-100"));
    });

    t.run("VIP customer discount", fn(subT: TestingT) => {
        let total = calculateTotal({ isVip: true }, 100.0);
        subT.assertEqual(total, 80.0, "VIPs receive a 20% discount");
    });

    t.run("Standard customer price", fn(subT: TestingT) => {
        let total = calculateTotal({ isVip: false }, 100.0);
        subT.assertEqual(total, 100.0, "Regular customers pay full price");
    });
}

// 4. Benchmark function with adaptive timing
export fn benchmark_factorial(b: BenchmarkB) => {
    b.resetTimer();
    factorial(15, 1);
}
```

#### C. Testing Context Directives (`TestingT`)

The `t: TestingT` context provides standard assertion, logging, and lifecycle directives:

| Method / Directive | Description |
| :----------------- | :---------- |
| `t.assertEqual(actual, expected, msg?)` | Asserts deep structural equality between two values. |
| `t.assertNotEqual(actual, expected, msg?)` | Asserts that two values are not deeply equal. |
| `t.assertTrue(condition, msg?)` | Asserts that `condition === true`. |
| `t.assertFalse(condition, msg?)` | Asserts that `condition === false`. |
| `t.assertDeepEqual(actual, expected, msg?)` | Alias for deep structural assertion. |
| `t.assertThrows(fn, expectedErr?)` | Asserts that calling `fn()` throws or panics with an optional error substring. |
| `t.step(name, fn)` | Encapsulates a distinct logical step with timing logs. |
| `t.run(subName, fn)` | Spawns an isolated subtest with its own `TestingT` scope. |
| `t.log(...args)` | Records formatted diagnostic messages in the test output. |
| `t.skip(reason)` | Skips the current test at runtime with an explanation. |
| `t.fail(reason)` | Immediately marks the test as failed. |

Global assertions are also available directly without `t`: `assert(cond, msg)`, `assertTrue(cond, msg)`, `assertFalse(cond, msg)`, `assertEqual(a, b, msg)`, and `assertNotEqual(a, b, msg)`.

#### D. Benchmark Context Directives (`BenchmarkB`)

The `b: BenchmarkB` context manages adaptive iteration counts and throughput metrics:

| Method / Directive | Description |
| :----------------- | :---------- |
| `b.n` | Target iteration count adaptively calibrated by the benchmark runner. |
| `b.resetTimer()` | Resets the elapsed time and start timestamp (use after expensive setup). |
| `b.startTimer()` | Resumes timer measurement. |
| `b.stopTimer()` | Pauses timer measurement during non-benchmark teardown/allocations. |
| `b.setBytes(n)` | Informs the runner of bytes processed per op to compute MB/s throughput. |

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

### 🐘 Native PostgreSQL Driver (`pg` / `postgres`)

```aura
let db: PgPool = postgres.createPool({
    host: "localhost",
    port: 5432,
    user: "postgres",
    password: "secret_password",
    database: "orders_db"
});

async fn getActiveUsers(): Task<(), String> {
    let res = await db.query("SELECT id, name, email FROM users WHERE active = $1", [true]);
    match res {
        Ok(rows) => for u in rows { println(`- ${u.name} (${u.email})`); },
        Err(err) => println(`SQL Error: ${err}`)
    }
}

// Automatic ACID Transactions
async fn transferBalance(txFn: PgTransaction): Task<(), String> {
    await db.transaction(fn(tx: PgTransaction) => {
        tx.execute("UPDATE accounts SET balance = balance - 100 WHERE id = $1", [1]);
        tx.execute("UPDATE accounts SET balance = balance + 100 WHERE id = $2", [2]);
    });
}
```

---

### 🐬 Native MySQL Driver (`mysql`)

```aura
let pool: MysqlPool = mysql.open("mysql://root:secret@127.0.0.1:3306/shop_db");

async fn insertProduct(name: String, price: Float): Task<Int, String> {
    let res = await pool.execute("INSERT INTO products (name, price) VALUES (?, ?)", [name, price]);
    match res {
        Ok(info) => {
            println(`Inserted ID: ${info.insertId}, Affected: ${info.affectedRows}`);
            info.insertId
        },
        Err(e) => -1
    }
}
```

---

### 🍃 Native MongoDB Driver (`mongodb` / `mongo`)

```aura
let client: MongoClient = mongodb.open("mongodb://127.0.0.1:27017/catalog");
let items: MongoCollection = client.db().collection("items");

async fn queryCatalog(): Task<(), String> {
    let doc = await items.findOne({ sku: "AURA-CORE" });
    match doc {
        Ok(Some(item)) => println(`Found: ${item.name}, Stock: ${item.stock}`),
        Ok(None) => println("Item does not exist."),
        Err(err) => println(`Mongo Error: ${err}`)
    }
}
```

---

### ⚡ Native Redis Client (`redis`)

```aura
let cache: RedisClient = redis.open("redis://127.0.0.1:6379");

async fn manageSession(userId: String, token: String): Task<Bool, String> {
    // Save session token with 3600-second TTL
    await cache.set(`session:${userId}`, token, 3600);

    // Hash store
    await cache.hset(`profile:${userId}`, "role", "admin");

    let role = await cache.hget(`profile:${userId}`, "role");
    println(`Cached role: ${role}`);
    true
}
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

### 1. Core Compiler (`aurac`)

```bash
# 1. Compile to standalone native binary with Cranelift (default)
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
```

### 2. Test, Benchmark, and Coverage Runner (`auratest`)

```bash
# Run all tests in the project
auratest ./...
# or via aurac:
aurac test ./...

# Verbose mode with per-test timings
auratest -v ./...

# Filter tests by regular expression
auratest -run TestUserAuthentication ./...

# Filter tests by test category
auratest --unit ./...
auratest --integration ./...
auratest --e2e ./...

# Conditional testing using build tags
auratest --tags "integration,db" ./...

# Run benchmarks with operations per second (ns/op)
auratest -bench . ./...

# Generate test coverage profile and interactive HTML report
auratest --coverage --coverprofile=coverage.out --coverage-html=coverage.html ./...

# Structured JSON output for CI/CD integration
auratest -json ./...

# Stop test execution on first failure
auratest --fail-fast ./...

# Run tests with count and timeout limits
auratest -count=2 -timeout=10000 ./...

# Watch mode to re-run tests on file changes
auratest -w ./...
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

Aura includes a **fully self-hosted compiler** written in the Aura language itself, located in [`src/aura_compiler/`](src/aura_compiler):

- [`ast.aura`](src/aura_compiler/ast.aura) — Formal AST definitions written in Aura.
- [`lexer.aura`](src/aura_compiler/lexer.aura) — Deterministic lexical analyzer.
- [`parser.aura`](src/aura_compiler/parser.aura) — Recursive descent Pratt parser.
- [`codegen.aura`](src/aura_compiler/codegen.aura) — Code generator.
- [`main.aura`](src/aura_compiler/main.aura) — Compiler entrypoint.

### 1. Stage 1: Compile the Self-Hosted Compiler Modules

Compile the `.aura` compiler source modules into ES6 JavaScript modules:

```bash
mkdir -p dist
aurac compile src/aura_compiler/ast.aura -o dist/ast.mjs
aurac compile src/aura_compiler/lexer.aura -o dist/lexer.mjs
aurac compile src/aura_compiler/parser.aura -o dist/parser.mjs
aurac compile src/aura_compiler/codegen.aura -o dist/codegen.mjs
aurac compile src/aura_compiler/main.aura -o dist/aurac.mjs
```

### 2. Stage 2: Compile Aura Programs with the Self-Hosted Compiler (`dist/aurac.mjs`)

Use the compiled self-hosted compiler to compile Aura programs:

```bash
node dist/aurac.mjs examples/bootstrap_demo/hello.aura -o dist/hello.js
node dist/hello.js
# Output:
# Hello Developer from Self-Hosted Aura Compiler!
# Sum of 1..100: 5050
```

### 3. Stage 3: Build the Self-Hosted Compiler into a Standalone Native Binary

Compile the self-hosted compiler into a standalone native executable with zero external runtime dependencies:

```bash
aurac build src/aura_compiler/main.aura -o dist/aura-self-hosted
```

### 4. Stage 4: Compile Aura Programs Directly with the Native Self-Hosted Binary

Run the native self-hosted compiler binary to compile any `.aura` file:

```bash
./dist/aura-self-hosted examples/hello.aura -o dist/hello_from_native.js
node dist/hello_from_native.js
# Output: Hello from Aura Lang!
```

### 5. Automated Bootstrap Verification

Run the end-to-end automated bootstrap test suite validating all stages:

```bash
cargo test --test bootstrap_tests -- --test-threads=1
```

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
- [x] **Standard Library & Databases**: Ultra-high-performance HTTP ServeMux server (**126k+ req/s**), `/metrics` Prometheus endpoint, OpenTelemetry `traceparent`, Swagger UI, and native drivers for PostgreSQL, MySQL, MongoDB, and Redis.
- [x] **Decentralized Package Manager**: `aura.mod`, cryptographic `aura.lock`, offline vendoring, and integrity checks (`auramod` / `aurac mod`).
- [x] **C/Rust FFI Generator (`aurabindgen`)**: Automatic conversion of `.h` headers into safe Aura interfaces.
- [x] **Production-Grade Tooling**: Official LSP server (`auralsp`), terminal step debugger (`aurac step`), V8 DAP server (`aurac debug`), test runner (`auratest`), and opinionated formatter (`aurafmt`).
- [x] **Quality Suite & Tests**: **250 automated tests passing successfully** across the repository.
- [ ] **WebAssembly Backend (WASM/WASI)**: WebAssembly binary emission for serverless edge microservices.
- [ ] **Advanced LLVM Optimizations**: Optional LTO optimization pipeline and SIMD vectorization for compute-intensive workloads.

---

## 📄 License

Aura Language is free and open-source software released under the [MIT License](LICENSE).

### ⚖️ Trademarks

> Go is a trademark of Google LLC. Rust is a trademark of the Rust Foundation. TypeScript is a trademark of Microsoft Corp. Zig is a trademark of the Zig Software Foundation. Aura Lang is an independent open-source project and is not affiliated with or endorsed by these entities.
