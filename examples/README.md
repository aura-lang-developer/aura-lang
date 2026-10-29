> 🌍 **Languages / Idiomas:** **English** | [Español](README-ES.md)

# 📚 Aura Language Examples (`examples/`)

This directory contains the comprehensive collection of official **Aura Language** (`aurac`) examples, designed to demonstrate, benchmark, and test all syntactic features, CSP concurrency, low-level memory constructs, and runtime security guarantees.

---

## 📑 Table of Contents

1. [Advanced Architecture & Real-World Showcases](#1-advanced-architecture--real-world-showcases)
2. [CSP Concurrency & Multi-Threading (Go-Style)](#2-csp-concurrency--multi-threading-go-style)
3. [Generic Programming (`generics/`)](#3-generic-programming-generics)
4. [Databases, Native SQL & NoSQL](#4-databases-native-sql--nosql)
5. [Backend Services, HTTP & Microservices](#5-backend-services-http--microservices)
6. [Low-Level Programming, Pointers, Packed Structs & FFI](#6-low-level-programming-pointers-packed-structs--ffi)
7. [Resource Cleanup, Control Flow & Modern Paradigms](#7-resource-cleanup-control-flow--modern-paradigms)
8. [Unit Testing & Stress Suites (`tests/`)](#8-unit-testing--stress-suites-tests)
9. [Subprojects & Tooling Demos](#9-subprojects--tooling-demos)
10. [How to Run and Check the Examples](#-how-to-run-and-check-the-examples)

---

## 1. Advanced Architecture & Real-World Showcases

| File | Core Concepts & Key Features |
| :--- | :--- |
| [`advanced_worker_pool.aura`](advanced_worker_pool.aura) | **CSP Worker Pool with Priority Queue & Structural Duck Typing**: Async fibers via `spawn`, typed channels (`Channel<T>`, `SendChannel<T>`, `RecvChannel<T>`), implicit structural interface `Executable`, non-blocking multiplexing with `select`, `WaitGroup` synchronization, `defer`, `errdefer`, and pipeline operator `\|>`. |
| [`financial_ledger.aura`](financial_ledger.aura) | **High-Assurance Financial Ledger & Transaction Engine**: Automatic transaction rollback via `errdefer`, pointer receiver methods (`*LedgerAccount`), tail-call optimization (TCO) for constant $O(1)$ stack audit recursion, and algebraic data types (ADTs) with multi-argument payloads. |
| [`network_packet_router.aura`](network_packet_router.aura) | **Network Packet Router & Bit-Level Diagnostics**: Binary packed structs without memory padding (`packed struct`), fixed-width integers (`Uint8`, `Uint16`, `Uint32`), C foreign function interface (`extern "C"`), sub-slicing expressions, and functional protocol routing (`TCP`, `UDP`, `ICMP`). |
| [`modern_features_showcase.aura`](modern_features_showcase.aura) | **Unified Modern Paradigm Showcase**: Complete demonstration of sum types, Go-style slices (`[]T`), receiver methods with visibility rules, LIFO cleanup, Result propagation with `?`, and CSP concurrency. |

---

## 2. CSP Concurrency & Multi-Threading (Go-Style)

| File | Core Concepts & Key Features |
| :--- | :--- |
| [`concurrency_csp_pipeline.aura`](concurrency_csp_pipeline.aura) | **Complete CSP Concurrency Pipeline**: Multi-fiber producer-consumer architecture, buffered channels, `WaitGroup` coordination, and multiplexed `select` with `timeout` and `default` fallback branches. |
| [`channel_range_demo.aura`](channel_range_demo.aura) | **Directional Channels & `for-in` Range Iteration**: Strict send-only (`SendChannel<T>`) and receive-only (`RecvChannel<T>`) channel constraints, safe channel closing, and continuous iteration with `for item in inCh`. |
| [`context_demo.aura`](context_demo.aura) | **Context Propagation, Cancellation & Timeouts**: Structured cancellation trees via `Context`, deadline expirations, and distributed trace metadata forwarding (`ctx.value(...)`). |
| [`go_func_concurrency.aura`](go_func_concurrency.aura) | **Anonymous Concurrent Fibers**: Inline fiber spawning with closures, channel handoffs by value, and termination tracking with `WaitGroup`. |
| [`goroutines_demo.aura`](goroutines_demo.aura) | **Asynchronous Lightweight Fibers**: Basic dispatching of parallel background fibers with synchronized result aggregation. |
| [`aura_routines_demo.aura`](aura_routines_demo.aura) | **Fiber Orchestration & Channel Passing**: Channel lifecycle management and parallel worker scheduling across multiple fibers. |
| [`native_concurrency.aura`](native_concurrency.aura) | **Native M:N Scheduler Mapping**: Demonstrating low-overhead fiber task switching on the standalone native runtime. |

---

## 3. Generic Programming (`generics/`)

Located in the [`generics/`](generics/) directory:

| File | Core Concepts & Key Features |
| :--- | :--- |
| [`01_generic_functions.aura`](generics/01_generic_functions.aura) | **Generic Functions & Identity Invariant**: Declaring type parameters `<ValType>`, static type inference, and polymorphic invocation across primitive types (`Int`, `String`, `Bool`). |
| [`02_generic_container.aura`](generics/02_generic_container.aura) | **Generic Box & Wrapper Containers**: Single-parameter generic ADT with `Empty` and `Full(ItemType)` variants, polymorphic constructors, and pattern-matched unwrapping. |
| [`03_generic_pair.aura`](generics/03_generic_pair.aura) | **Multi-Parameter Key-Value Pairs**: Algebraic data types parameterized by `<KeyType, ValType>` and a generic swap function `swapPair<TypeA, TypeB>()`. |
| [`04_generic_result_adt.aura`](generics/04_generic_result_adt.aura) | **Custom Generic Outcome Pattern**: Implementation of `Outcome<SuccType, ErrType>` with generic helper utilities (`unwrapOr`, `isSuccess`) and error-handled computations (`divideSafe`). |
| [`05_generic_stack.aura`](generics/05_generic_stack.aura) | **Generic LIFO Stack Data Structure**: Recursive generic sum type `Stack<ElemType>`, polymorphic `push` operations, and recursive `depth` counting. |
| [`06_generic_optional.aura`](generics/06_generic_optional.aura) | **Generic Optional State**: Encapsulation of present vs. absent state with `OptionalState<ItemType>` and fallback default evaluation via `getOrElse`. |
| [`07_generic_tree.aura`](generics/07_generic_tree.aura) | **Generic Binary Tree Structure**: Recursive hierarchical nodes `Tree<ValType>` (`Leaf` and `Node(...)`) with tree size calculation via recursive traversal. |
| [`08_generic_pipeline.aura`](generics/08_generic_pipeline.aura) | **Generic Data Transformation Pipelines**: Polymorphic transformation wrappers seamlessly chained using the pipeline operator `\|>`. |
| [`09_generic_csp_channels.aura`](generics/09_generic_csp_channels.aura) | **Generic CSP Channels & Fiber Messaging**: Generic envelope type `Message<DataType>`, typed message channels (`Channel<Message<String>>`), and producer-consumer fiber synchronization. |
| [`10_generic_cache_repo.aura`](generics/10_generic_cache_repo.aura) | **Generic Cache & Repository**: Multi-state cache entry `CacheEntry<ValType>` (`Fresh`, `Expired`, `CacheMiss`) and generic cache resolution with fallback strategies. |
| [`example_10_generics.aura`](example_10_generics.aura) | **Introductory Generics Reference**: Standard demonstration of generic types and arrow syntax (`->`). |

---

## 4. Databases, Native SQL & NoSQL

| File | Core Concepts & Key Features |
| :--- | :--- |
| [`db_sqlite_crud.aura`](db_sqlite_crud.aura) | **Embedded SQLite CRUD Lifecycle**: In-memory and file database handles, schema initialization, parameterized queries, and row-to-record mapping. |
| [`db_postgres_microservice.aura`](db_postgres_microservice.aura) | **PostgreSQL Microservice with SCRAM-SHA-256**: Pure-Rust native protocol connection supporting modern PG databases (Neon, Supabase, AWS RDS) with positional placeholders `$1..$N`. |
| [`db_mysql_inventory.aura`](db_mysql_inventory.aura) | **MySQL / MariaDB Inventory Engine**: Native TCP connection, query binding with `?`, transaction blocks, and typed result set extraction. |
| [`db_generic_sql_pool.aura`](db_generic_sql_pool.aura) | **Thread-Safe Generic Database Connection Pool**: `AuraDbPool` connection acquisition and deterministic release with timeout safeguards against socket starvation. |
| [`db_transactions_bank.aura`](db_transactions_bank.aura) | **ACID Banking Transactions**: Atomicity with `BeginTx`, `Commit`, and automated rollback on insufficient balance via `errdefer`. |
| [`db_stored_procedures_functions.aura`](db_stored_procedures_functions.aura) | **Stored Procedures & SQL Functions**: Calling stored procedures and retrieving multiple structured result sets. |
| [`example_postgres.aura`](example_postgres.aura) | **Basic PostgreSQL Client**: Minimal client connection executing parameterized SELECT and INSERT statements. |
| [`example_mysql.aura`](example_mysql.aura) | **Basic MySQL Client**: Concise connection setup and row iteration. |
| [`example_nosql.aura`](example_nosql.aura) | **Embedded Key-Value / NoSQL Store**: Key-value data persistence and querying. |
| [`native_database_demo.aura`](native_database_demo.aura) | **Unified `database/sql` Subsystem**: Multi-dialect SQL operations through Aura's unified database abstractions. |
| [`native_nosql_demo.aura`](native_nosql_demo.aura) | **Native Redis & MongoDB Clients**: Native DNS resolution, Redis RESP command processing, and MongoDB BSON document operations. |

---

## 5. Backend Services, HTTP & Microservices

| File | Core Concepts & Key Features |
| :--- | :--- |
| [`golang_http_server.aura`](golang_http_server.aura) | **Go-Style HTTP Server**: `http.newServeMux()` routing, request logging middleware, custom response headers, and REST endpoint dispatch. |
| [`backend_service.aura`](backend_service.aura) | **Production Backend Service**: REST API server, automated JSON serialization via struct tags, background telemetry fibers, and thread-safe global state with `Mutex`. |
| [`bookstore_service.aura`](bookstore_service.aura) | **Complete Bookstore REST Microservice**: Full CRUD endpoints, model validation, pagination, and HTTP status codes. |
| [`api_service.aura`](api_service.aura) | **Lightweight API Service**: Fast server setup and JSON response generation. |
| [`example_http.aura`](example_http.aura) | **Minimal HTTP Client & Server**: Compact demonstration of the `net/http` standard library package. |
| [`ecommerce.aura`](ecommerce.aura) | **E-Commerce Business Engine**: Order status matching (`OrderStatus`), tail-call recursive subtotal computation, and VIP discount pipelines. |
| [`microservices/`](microservices/) | **Distributed Microservices Suite**: End-to-end multi-service architectures (Orders, Fulfillment, Telemetry, Inventory). |

---

## 6. Low-Level Programming, Pointers, Packed Structs & FFI

| File | Core Concepts & Key Features |
| :--- | :--- |
| [`pointers_demo.aura`](pointers_demo.aura) | **Pointers & Memory References**: Pointer types (`*T`), address-of operator (`&x`), explicit dereferencing (`*p`), in-place value exchange (`swapValues`), pointer receiver methods (`*Receiver`), and compile-time thread-isolation safety (`[E0401]`). |
| [`rust_features_showcase.aura`](rust_features_showcase.aura) | **Rust-Inspired Features Showcase**: `#[derive(Debug, Clone, Eq, Json, Hash)]` attributes on structs and sum types, `#[must_use]` validation, type invariants, and auto-generated boilerplate methods. |
| [`must_use_and_fix_demo.aura`](must_use_and_fix_demo.aura) | **Must-Use and Auto-Fix Demo**: Static linter diagnostics for discarded critical return values (`#[must_use]`) and automatic in-place correction of unmutated `let mut` declarations via `aura fix`. |
| [`zig_inspired_features.aura`](zig_inspired_features.aura) | **Zig-Inspired Low-Level Capabilities**: Transactional cleanup with `errdefer`, zero-padding binary layouts (`packed struct`), and C ABI foreign function declarations (`extern "C"`). |

---

## 7. Resource Cleanup, Control Flow & Modern Paradigms

| File | Core Concepts & Key Features |
| :--- | :--- |
| [`defer_demo.aura`](defer_demo.aura) | **Deterministic LIFO Cleanup (`defer`)**: Reliable file descriptor closing, connection disposal, and mutex unlocking on scope exit. |
| [`runtime_profile_core_demo.aura`](runtime_profile_core_demo.aura) | **Minimal Core Runtime Profile**: Lightweight standalone CLI binary compiling with zero embedded database overhead (`core` profile). |
| [`runtime_profile_full_demo.aura`](runtime_profile_full_demo.aura) | **Full Runtime Profile with SQLite**: Automatic AST detection of database imports/calls linking the embedded SQLite engine (`full` profile). |
| [`slices_golang_model.aura`](slices_golang_model.aura) | **Go-Style Slices**: Dynamic arrays `[]T`, sub-slicing `s[low:high:max]`, built-in functions `len()`, `cap()`, `append()`, and memory-efficient referencing. |
| [`golang_methods_visibility.aura`](golang_methods_visibility.aura) | **Receiver Methods & Visibility Conventions**: Public methods (`PascalCase`) vs. private internal methods (`camelCase`), pointer receivers, and struct encapsulation. |
| [`labeled_loops_demo.aura`](labeled_loops_demo.aura) | **Labeled Loops**: Escaping or continuing nested loops using explicit labels (`break 'outer;`, `continue 'outer;`). |
| [`build_tags_demo.aura`](build_tags_demo.aura) | **Conditional Compilation Directives**: Target filtering with `//aura:build` and `//go:build` supporting boolean operators (`darwin \|\| linux`, `!windows`). |
| [`embed_demo.aura`](embed_demo.aura) | **Static Asset Embedding (`embed`)**: Inlining text or binary files directly into compiled binaries at compile time. |
| [`self_hosted_compiler_showcase.aura`](self_hosted_compiler_showcase.aura) | **Self-Hosted Compiler Engine**: Lexing and tokenization pipelines implemented natively in Aura. |
| [`hello.aura`](hello.aura) | **Minimal Hello World**: Entry point structure using `export fn main()`. |

---

## 8. Unit Testing & Stress Suites (`tests/`)

Located in the [`tests/`](tests/) directory:

| File | Core Concepts & Key Features |
| :--- | :--- |
| [`security_and_performance_stress_test.aura`](tests/security_and_performance_stress_test.aura) | **Security & Performance Stress Suite**: Call stack exhaustion protection via TCO, transactional rollback testing with `errdefer`, closed CSP channel draining, slice boundary integrity, and null-coalescing (`??`) short-circuiting. |
| [`math_unit_test.aura`](tests/math_unit_test.aura) | **Arithmetic Unit Tests**: Basic testing assertions and arithmetic invariant verification. |
| [`golang_style_test.aura`](tests/golang_style_test.aura) | **Go-Style Unit Testing**: Native test runner execution (`aurac --test`) with testing contexts and assertions. |
| [`order_integration_test.aura`](tests/order_integration_test.aura) | **Order Flow Integration Test**: Checkout calculation, TCO subtotal validation, and order lifecycle transitions. |
| [`collections_iterator_test.aura`](tests/collections_iterator_test.aura) | **Collection & Iterator Tests**: Transformation pipelines, filtering, and list traversals. |
| [`api_e2e_test.aura`](tests/api_e2e_test.aura) | **API End-to-End Test**: HTTP request-response roundtrip testing and concurrency verification. |

---

## 9. Subprojects & Tooling Demos

* **[`algorithms_and_functions/`](algorithms_and_functions/)**: Suite of **100 verified algorithms and functions** ([see full catalog](algorithms_and_functions/README.md)) covering arithmetic, primes/sieve, sorting, searching, string manipulation, functional programming, ADTs, CSP, and microbenchmarks.
* **[`aurabun/`](aurabun/)**: High-performance package manager and bundler for the Aura ecosystem inspired by Bun.
* **[`bootstrap_demo/`](bootstrap_demo/)**: Demonstration of the stage-1 compiler bootstrapping process.
* **[`compiler_extension_guide/`](compiler_extension_guide/)**: Comprehensive documentation and source examples for extending Aura's AST, typechecker, and backends.

---

## 🚀 How to Run and Check the Examples

You can validate, test, or execute any example directly using the official Aura CLI:

```bash
# Ultra-fast static type checking
aura check examples/financial_ledger.aura
aura check examples/pointers_demo.aura
aura check examples/generics/01_generic_functions.aura

# Direct execution with the Aura engine
aura run examples/advanced_worker_pool.aura
aura run examples/pointers_demo.aura
aura run examples/generics/05_generic_stack.aura

# Format code with the opinionated official formatter
aurafmt -w examples/pointers_demo.aura

# Run unit tests and security stress suites
aurac --test examples/tests/math_unit_test.aura
aura run examples/tests/security_and_performance_stress_test.aura

# Build a standalone native binary executable
aura build examples/financial_ledger.aura -o ledger-bin
./ledger-bin
```
