# Changelog

All notable changes to the Aura Language project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.7.1] - 2026-10-15

### 🛡️ Security Hardening & Runtime Integrity

Aura v0.7.1 is a dedicated security hardening and stability release that resolves vulnerabilities across the compiler type system, native Cranelift backend, multithreaded runtime, and standard network/database services.

#### 1. Concurrency & Memory Safety
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
- **HTTP Response Splitting / CRLF Prevention**: Added strict sanitization on HTTP response header keys and values in `crates/aura-runtime/src/http.rs`, rejecting `\r` and `\n` to prevent response splitting and header injection.
- **Explicit TLS Link Validation**: Prevented silent plain-text fallback and false TLS log output by returning `-1` with error diagnostics if TLS certificates are configured without an active TLS provider.
- **Remote Hostname Resolution in Redis & MongoDB**: Integrated `ToSocketAddrs` for robust DNS host resolution in `crates/aura-runtime/src/redis_client.rs` and `mongo_client.rs`.
- **Complete RESP Framing**: Implemented multi-packet RESP parsing (`is_resp_complete`) to prevent partial payload truncation under heavy network loads.
- **Credential Stripping in Logging**: Sanitized MongoDB connection strings prior to diagnostic logging to protect credentials.

#### 6. Supply Chain & Module Security
- **Package Path & Version Sanitization**: Enforced validation in `src/package.rs` against command injection, path traversal (`..`), and unsafe git protocols, ensuring secure module resolution.

#### 7. Test Suite & Verification
- Added comprehensive security regression suite in `tests/security_fixes_tests.rs` (6 test scenarios covering STW GC locks, env sanitization, SQL binding, CRLF protection, TLS validation, and RESP framing).
- Workspace test suite expanded to **256 passing tests** (0 failures).

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
