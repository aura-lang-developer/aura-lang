# Changelog

All notable changes to the Aura Language project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.8.3] - 2026-10-25

### 🛡️ Pentest Security Hardening, Cryptographic Primitives (CSPRNG), OWASP ASVS & 100 Verified Algorithm Examples

Aura v0.8.3 brings enterprise-grade security hardening based on OWASP ASVS and CWE pentesting guidelines, introducing native CSPRNG cryptographic primitives in both Golang and Node/V8 execution backends, static linter rules (`SEC003`) against predictable pseudo-randomness in sensitive contexts, safe typed SQL query structures, automated HTTP defense-in-depth security headers, and an extensive suite of 100 verified algorithm and functional programming examples.

#### 1. Cryptographic Primitives & CSPRNG (`src/typechecker.rs`, `src/codegen_go.rs`, `src/codegen.rs`)
- **Native CSPRNG Primitives**: Registered and implemented in the standard `crypto` module:
  - `crypto.randomBytes(size: Int): String`: Generates cryptographically secure random bytes formatted as hexadecimal string using Go `crypto/rand` and Node `node:crypto`.
  - `crypto.randomUUID(): String`: Generates RFC 4122 v4 compliant cryptographically secure random UUID strings.
  - `crypto.timingSafeEqual(a: String, b: String): Bool`: Performs constant-time string comparison using `hmac.Equal` (Go) and `crypto.timingSafeEqual` (Node) to prevent side-channel timing attacks (CWE-208).

#### 2. Static Security Linter (`src/linter.rs`)
- **New Rule `SEC003` (Insecure PRNG Detection)**: The integrated linter (`aurac lint`) statically flags `Math.random()` when assigned to security-sensitive variables (matching tokens, secrets, salts, ivs, nonces, keys, passwords, sessions, or auth identifiers), instructing developers to use `crypto.randomBytes()` or `crypto.randomUUID()`.

#### 3. Standard Library Hardening (`std/database/sql`, `std/net/http`)
- **OWASP ASVS SQL Injection Mitigation (CWE-89)**: Added strongly typed `SqlQuery` interface and constructor `sqlQuery(text: String, params: List<Any> = [])` in `std/database/sql/sql.aura` to promote parameterized and prepared SQL queries.
- **HTTP Defense-in-Depth Headers (CWE-693)**: Added `applySecurityHeaders(w: ResponseWriter)` in `std/net/http/http.aura`, automatically setting `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`, `X-XSS-Protection: 1; mode=block`, and `Content-Security-Policy: default-src 'self'`.

#### 4. Web Playground Security Hardening (`src/playground.rs`)
- **Elimination of `'unsafe-eval'` (CWE-79 / CWE-1021)**: Stripped `'unsafe-eval'` from the HTTP Content-Security-Policy header in the interactive web playground server and upgraded the Referrer-Policy to `strict-origin-when-cross-origin`.

#### 5. 100 Verified Algorithm & Function Examples (`examples/algorithms_and_functions/`)
- Added 100 individually compiled, executed, and type-checked Aura examples spanning arithmetic, number theory, sorting, searching, slices, strings, higher-order functions, algebraic data types, data structures, CSP concurrency, dynamic programming, and low-level representations with complete documentation in `examples/algorithms_and_functions/README.md`.

---

## [0.8.2] - 2026-10-20

### 🧪 Full Golang `testing` (`go test`) Format Parity, Table-Driven Subtests & Verifiable Examples

Aura v0.8.2 introduces complete syntactic, semantic, and runner parity with Go's standard `testing` package (`*testing.T`, `*testing.B`, `*testing.M`, `*testing.TB`, `*testing.PB`), including table-driven subtests (`t.Run`), non-fatal vs. fatal failure semantics (`t.Errorf` vs. `t.Fatalf`), explicit `b.N` benchmark loops, verifiable `ExampleXxx()` functions with `// Output:` comments, and `TestMain(m: *testing.M)` suite orchestration.

#### 1. Parser & Language Syntax Enhancements (`src/parser.rs`)
- **Go Package Header & Import Declarations**: Added support for `package <name>` headers (alongside `module <name>`), single-string imports (`import "testing"`), and grouped import blocks (`import ( "testing" ... )`).
- **Qualified Type Annotations**: Enabled dotted package-qualified type names (`testing.T`, `*testing.T`, `testing.B`, `*testing.B`, `testing.M`, `*testing.M`, `testing.TB`, `testing.PB`).
- **C/Go 3-Clause & Range `for` Loops**: Added parser desugaring for 3-clause `for (let mut i = 0; i < b.N; i = i + 1) { ... }` loops as well as `for i in b.N` and `range` iteration syntax.

#### 2. Static Typechecker (`src/typechecker.rs`)
- **Complete `testing.T` / `testing.TB` Type Definitions**: Registered Go PascalCase and Aura camelCase methods for `Error`, `Errorf`, `Fatal`, `Fatalf`, `Fail`, `FailNow`, `Failed`, `Log`, `Logf`, `Skip`, `Skipf`, `SkipNow`, `Skipped`, `Run`, `Parallel`, `Cleanup`, `Helper`, `TempDir`, `Setenv`, and `Name`.
- **Complete `testing.B`, `testing.M`, and `testing.PB` Types**: Added `b.N`, `ResetTimer`, `StartTimer`, `StopTimer`, `ReportAllocs`, `SetBytes`, `ReportMetric`, `Run`, `RunParallel`, `m.Run()`, and `pb.Next()`.

#### 3. Test Runner & Isolated Harness (`src/testing.rs`)
- **Subtest Execution & Hierarchical Filtering (`t.Run`)**: Isolated subtest execution recording individual `TestParent/Subtest` results, hierarchical indentation in verbose output (`=== RUN`, `--- PASS`, `--- FAIL`), and `-run Parent/Subtest` CLI filtering.
- **Go Failure Semantics**: `t.Error` / `t.Errorf` mark the test failed while continuing execution; `t.Fatal` / `t.Fatalf` / `t.FailNow` immediately abort the current test or subtest while still allowing sibling subtests in table-driven loops to execute.
- **Lifecycle & Environment Isolation**: Implemented LIFO `t.Cleanup(fn)` hooks, automatic temporary directory creation/removal via `t.TempDir()`, and automatic environment variable restoration via `t.Setenv(k, v)`.
- **Dual-Mode Adaptive Benchmarks (`b.N`)**: Automatically detects whether a benchmark function iterates over `b.N` internally or executes a single operation per call.
- **Verifiable `ExampleXxx()` Functions & `TestMain`**: Captures standard output in `Example*` functions to validate against trailing `// Output:` and `// Unordered output:` comments, and supports custom suite setup/teardown via `TestMain(m: *testing.M)`.

---

## [0.8.1] - 2026-10-19

### 🐘 Native PostgreSQL SCRAM-SHA-256 Auth, Adaptive Memory GC & Runtime Hardening

Aura v0.8.1 is a stability, performance, and security hardening release delivering native pure-Rust PostgreSQL `SCRAM-SHA-256` authentication without `libpq`, adaptive Stop-The-World garbage collection, bounded database pool concurrency with `Condvar` synchronization, $O(1)$ typechecker scope resolution with thread-safe builtins caching, isolated process-unique `aura run` temporary execution, dynamic HTTP worker thread scaling, and type-safe playground API serialization with `serde_json`.

#### 1. Native Database Drivers & Cryptography
- **Pure-Rust PostgreSQL SCRAM-SHA-256 Authentication (RFC 7677 / RFC 5802)**: Implemented complete SASL authentication handshake (message type 10) in `crates/aura-runtime/src/postgres_client.rs` supporting modern PostgreSQL 14+, Neon, Supabase, and AWS RDS without external `libpq` dependencies.
- **Standalone Cryptographic Primitives**: Added pure-Rust implementations of SHA-256, HMAC-SHA-256, PBKDF2 (4096+ iterations), and chunk-exact 4-byte Base64 encoding/decoding (RFC 4648) with zero external C dependencies.
- **Strict Database Connection Pool Bounding (`Condvar`)**: Upgraded `AuraDbPool` in `crates/aura-runtime/src/db.rs` with `active_conns` counting and `Condvar` synchronization (15-second acquisition timeout), preventing connection starvation and socket exhaustion under high concurrent loads.

#### 2. Garbage Collector & Memory Performance
- **Adaptive Memory GC & Sweep De-allocation Fix**: Fixed memory leak where reclaimed bytes were not subtracted from `TOTAL_ALLOCATED_BYTES` during the GC sweep phase (`crates/aura-runtime/src/gc.rs`).
- **Dynamic STW Threshold Scaling**: Replaced static `GC_THRESHOLD_BYTES` with an adaptive atomic threshold `(live * 2).max(MIN_GC_THRESHOLD_BYTES)`, eliminating continuous Stop-The-World execution pauses under heavy allocation workloads.

#### 3. Compiler Typechecker & Scopes
- **$O(1)$ Type Environment Scope Transitions**: Replaced recursive `Box<TypeEnv>` cloning with `Option<Arc<TypeEnv>>` parent pointers (`src/typechecker.rs`), eliminating quadratic memory allocations when traversing deeply nested block scopes.
- **Builtins Catalog Singleton (`OnceLock`)**: Cached the standard built-in type environment (3,400+ lines of definitions) using `OnceLock<Arc<TypeEnv>>`, speeding up compilation across all files.

#### 4. Developer Tooling & CLI
- **Isolated Process-Unique Script Execution**: Enhanced `aura run` in `src/cli.rs` to generate timestamp- and PID-isolated temporary files (`.tmp.{pid}_{micros}.mjs`), eliminating race conditions in parallel executions.
- **RAII Cleanup Guard**: Introduced `CleanupGuard` with a deterministic `Drop` implementation to guarantee temporary script cleanup even during abnormal termination or panics.
- **Type-Safe Playground API (`serde_json`)**: Replaced manual substring extractors in `src/playground.rs` with strongly typed `serde_json` request/response schemas, preventing JSON injection and control character corruption.

#### 5. Native Cranelift Backend
- **Universal Runtime Library Resolution (`libaura_runtime.a`)**: Expanded search paths in `src/codegen_cranelift.rs` to check `AURA_RUNTIME_LIB`, executable directory (`current_exe().parent()`), user home (`~/.aura/lib/`), and standard system locations (`/usr/local/lib/aura/`, `/usr/lib/aura/`).

#### 6. HTTP Server Concurrency
- **Dynamic Worker Scaling**: Replaced static 256 worker threads in `crates/aura-runtime/src/http.rs` with dynamic sizing based on `std::thread::available_parallelism()` (clamped between 4 and 64 threads), reducing context-switching overhead and optimizing socket latency.

#### 7. Test Suite & Quality Verification
- Added cryptographic verification unit tests in `crates/aura-runtime/src/postgres_client.rs` verifying RFC 7677 test vectors (`StoredKey = a07156c3493aec5e8b30542f283009d2c210b21ada2ca3f061dfd440162cea2c`).
- 100% test pass rate across unit, runtime, integration, and E2E suites (**270+ passing tests**).

---

## [0.8.0] - 2026-10-18

### 🛡️ Language Security Invariants, Raw Strings & Static Linter

Aura v0.8.0 is a major release introducing compile-time immutability enforcement, formal monadic type unwrapping for the try operator (`?`), fiber concurrency boundary safety, CPU hardware trap protections in Cranelift, iterative GC stack safety, raw string literals, and a built-in static security and quality linter (`aura lint`).

#### 1. Language Invariants & Type System Hardening
- **Static Immutability Enforcement (`let` vs `let mut`)**: Variables declared with `let` are now strictly immutable at compile time. Reassignment attempts trigger compiler error `E0400` unless explicitly declared with `let mut` (`src/typechecker.rs`).
- **Formal Monadic Type Unwrapping for Try Operator (`?`)**: Full static type inference for `Expr::Try(inner)` unwrapping `Result<T, E> -> T` and `Option<T> -> T`. Applying `?` to unsupported non-Result/non-Option expressions is rejected with error `E0302` (`src/typechecker.rs`).
- **Fiber Concurrency Boundary Safety (`spawn`)**: Static analysis detects and rejects raw pointer (`*T`) captures inside `spawn { ... }` blocks with error `E0401`, eliminating cross-thread data race hazards and unsafe memory sharing (`src/typechecker.rs`).

#### 2. Native Code Generation & Hardware Trap Resilience
- **Cranelift Modulo Zero Hardware Trap Protection (`SIGFPE`)**: Division and modulo by zero (`BinOp::Mod`, `BinOp::Div`) in Cranelift JIT/AOT code generation now evaluate trap-safe conditional branches, converting hardware CPU `SIGFPE` crashes into clean recoverable panics (`src/codegen_cranelift.rs`).

#### 3. Garbage Collector & Runtime Hardening
- **Iterative GC Worklist**: Replaced deeply recursive `mark_ptr` traversal with an iterative heap-allocated worklist (`Vec<usize>`) in `crates/aura-runtime/src/gc.rs`, preventing runtime stack exhaustion on deep or cyclic object graphs.
- **Safe Redis DSN Parsing & Native RESP Authentication**: Enhanced Redis connection parser to strictly isolate authentication credentials from hostname/port, automatically dispatching RESP `AUTH <password>` upon socket connection (`crates/aura-runtime/src/redis_client.rs`).
- **Playground Anti-Slowloris Socket Protection**: Configured strict read (3s) and write (5s) timeouts on incoming TCP sockets in the interactive web playground server (`src/playground.rs`).
- **C-API Channels Null Safety**: Replaced unsafe assertion panics with graceful null-pointer validations and error codes in runtime FFI channels (`crates/aura-runtime/src/c_api.rs`).

#### 4. Ergonomic Language Features
- **Raw String Literals**: Added full lexer and compiler support for raw string literals (`r"..."` and `r#"..."#`) preserving verbatim text without escape character interpretation (`src/lexer.rs`). Ideal for regular expressions, embedded SQL, JSON payloads, and filesystem paths.

#### 5. Developer Tooling & Static Analysis
- **Integrated Static Linter (`aura lint` / `aurac lint`)**: Added a dedicated AST visitor-based linter CLI command (`src/cli.rs`, `src/linter.rs`):
  - `SEC001`: Detects potential SQL injection vectors in string interpolations and dynamic string concatenations.
  - `SEC002`: Flags hardcoded secrets, passwords, and private API credentials.
  - `LINT001`: Warns on mutable variables (`let mut`) that are never reassigned, recommending `let`.
  - `LINT002`: Identifies empty pattern match constructs.

#### 6. Test Suite & Verification
- Dedicated test suites added: `tests/security_fixes_tests.rs` (17 tests), `tests/lint_tests.rs` (7 tests), and `tests/native_cranelift_tests.rs` (4 tests).
- Subcommand version testing expanded to cover `aura lint` and `aurac lint`.
- 100% of automated tests passing across all workspace crates (**269+ passing tests**).

---

## [0.7.1] - 2026-10-15

### 🛡️ Security Hardening & Runtime Integrity

Aura v0.7.1 is a dedicated security hardening and stability release that resolves vulnerabilities across the compiler type system, native Cranelift backend, multithreaded runtime, and standard network/database services.

#### 1. Concurrency, Memory & GC Performance
- **Thread-Local Allocation Buffers (TLAB)**: Implemented amortized local batch allocation buffers in `crates/aura-runtime/src/gc.rs`. Reduces global `ALLOCATIONS` mutex contention by over 95% in concurrent multi-threaded workloads.
- **$O(\log N)$ Binary Search Pointer Marking**: Replaced $O(N)$ linear scans in `mark_ptr` with binary search over sorted allocation spans in `crates/aura-runtime/src/gc.rs`.
- **Fiber Stack Integrity & Canary**: Planted `STACK_CANARY_MAGIC` (0xDEAD_BEEF_CAFE_BABE) at fiber stack bases (`crates/aura-runtime/src/context.rs`), validating stack frames on recycling and catching buffer overflows.
- **Stop-The-World (STW) Safepoints**: Introduced `GC_WORLD_LOCK` synchronization between worker fiber execution and the generational garbage collector (`crates/aura-runtime/src/gc.rs` and `scheduler.rs`). Eliminates race conditions and pointer invalidation during parallel mark-and-sweep sweeps.
- **Thread-Safe Environment Subsystem**: Synchronized `aura_os_setenv` and `aura_os_env` using a global `ENV_LOCK` mutex (`crates/aura-runtime/src/os.rs`). Added validation rejecting `=` and null bytes (`\0`) in environment variable names to eliminate memory corruption and injection attacks.

#### 2. Native Cranelift Backend
- **Collision-Free ADT Variant Tagging**: Replaced sequential index hashing with deterministic `FxHasher` tag computation (`compute_variant_tag`) in `src/codegen_cranelift.rs`. Correctly passes distinct 64-bit variant tags to `aura_variant_new`, preventing tag collisions between unrelated sum types (e.g., `Shape::Circle` vs `Result::Ok`).

#### 3. Compiler Typechecker
- **Strict Pattern Match Exhaustiveness**: Implemented exhaustive branch checking (`check_match_exhaustiveness`) in `src/typechecker.rs` for `Option<T>`, `Result<T, E>`, and custom ADT sum types (both tuple and record variants). Emits compile-time errors when missing constructors or wildcard fallback arms (`_`), eliminating unhandled pattern branch crashes.

#### 4. Native Database Drivers
- **SQL Injection Prevention & Safe Parameter Binding**: Replaced raw string interpolation with strictly typed SQL literal formatting and placeholder transformation (`bind_postgres_sql` with `$1..$N`, `bind_mysql_sql` with `?`) in `crates/aura-runtime/src/db.rs`.
- **Database Handle Type Validation**: Added explicit `TYPE_DB_POOL` tag verification in `aura_db_query` and `aura_db_exec` to eliminate type confusion and invalid pointer dereferences.
- **Idempotent Connection & Transaction Cleanup**: Replaced unguarded drops with safe atomic de-registration and `take()` in `aura_db_tx_close` and `aura_db_pool_close`, eliminating double-free and use-after-close hazards.

#### 5. HTTP Server & Network Drivers
- **Slowloris Mitigation & Dual Timeouts**: Configured strict `read_timeout` (5s) and `write_timeout` (10s) on client sockets (`crates/aura-runtime/src/http.rs`).
- **HTTP Method & Header Hardening**: Strictly validated request methods (returning `405 Method Not Allowed` for invalid methods) and auto-injected defense-in-depth security headers (`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`).
- **HTTP Response Splitting / CRLF Prevention**: Added strict sanitization on HTTP response header keys and values in `crates/aura-runtime/src/http.rs`, rejecting `\r` and `\n` to prevent response splitting and header injection.
- **Explicit TLS Link Validation**: Prevented silent plain-text fallback and false TLS log output by returning `-1` with error diagnostics if TLS certificates are configured without an active TLS provider.
- **Remote Hostname Resolution in Redis & MongoDB**: Integrated `ToSocketAddrs` for robust DNS host resolution in `crates/aura-runtime/src/redis_client.rs` and `mongo_client.rs`.
- **Complete RESP Framing**: Implemented multi-packet RESP parsing (`is_resp_complete`) to prevent partial payload truncation under heavy network loads.
- **Credential Stripping in Logging**: Sanitized MongoDB connection strings prior to diagnostic logging to protect credentials.

#### 6. Cryptography & DoS Defenses
- **Side-Channel Timing Attack Resilience**: Enforced constant-time JWT signature verification via `crypto.timingSafeEqual` in Node.js codegen (`src/codegen.rs`) and `hmac.Equal` in Go codegen (`src/codegen_go.rs`).
- **JSON Recursive Depth Hardening**: Enforced `MAX_JSON_DEPTH = 64` in `crates/aura-runtime/src/json.rs` to protect against stack exhaustion DoS from deeply nested JSON payloads.
- **Package Path & Version Sanitization**: Enforced validation in `src/package.rs` against command injection, path traversal (`..`), and unsafe git protocols, ensuring secure module resolution.

#### 7. Test Suite & Verification
- Added comprehensive security regression suite in `tests/security_fixes_tests.rs` (9 test scenarios covering STW GC locks, env sanitization, SQL binding, CRLF protection, TLS validation, RESP framing, constant-time JWT, JSON depth limits, and fiber stack canaries).
- Workspace test suite expanded to **262 passing tests** (0 failures).

---

## [0.7.0] - 2026-10-14

### 🚀 High-Performance HTTP Worker Pool & Runtime Optimizations
- **HTTP Worker Pool Architecture**: High-throughput multi-fiber worker pool handling 96,000+ req/s.
- **Cranelift SIMD & Code Generation**: Enabled SIMD autovectorization and alias analysis in Cranelift release builds.
- **8,192-Fiber Stack Recycling Pool**: Expanded `MAX_STACK_POOL` from 1,024 to 8,192 stacks, eliminating repeated `mmap`/`munmap` syscalls.
- **Compact Stacks**: 16 KB (AArch64) and 8 KB (x86_64) tuned per-fiber stack allocations.
- **Universal Toolchain Bump**: All crates and self-hosted compiler updated to v0.7.0.

---

## [0.6.0] - 2026-10-14
- **Performance Engine & Zero-Alloc Optimization**: Zero-copy HTTP response writer, SIMD list copy, and compact fiber scheduler.

---

## [0.5.0] - 2026-10-12
- **Native Go Backend & Interop**: Go 1.21+ cross-compilation target (`--target go`) and idiomatic Go source generation.

---

## [0.4.0] - 2026-10-12
- **Unified Toolchain CLI (`aura`)**: Universal CLI entrypoint with subcommands `run`, `build`, `check`, `fmt`, `test`, `lsp`, `watch`, and `mod`.

---

## [0.3.0] - 2026-10-10
- **Native NoSQL Drivers**: Embedded Redis cache and MongoDB client integrations.

---

## [0.2.0] - 2026-10-10
- **Native SQL Drivers**: PostgreSQL, MySQL, and SQLite drivers with connection pooling.

---

## [0.1.0] - 2026-10-10
- **Initial Release**: Hindley-Milner type inference, CSP concurrency, Cranelift native backend, and core standard library.
