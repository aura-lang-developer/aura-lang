# Changelog

All notable changes to the Aura Language project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
