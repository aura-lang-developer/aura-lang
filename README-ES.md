> 🌍 **Languages / Idiomas:** [English](README.md) | **Español**

# 🌟 Aura Language (`aurac`)

[![Rust CI](https://img.shields.io/badge/Rust_CI-250_tests_passing-brightgreen.svg)](tests)
[![Compiler Speed](https://img.shields.io/badge/Speed-850k%2B_LOC%2Fs-blue.svg)#-benchmarks-y-rendimiento-empírico)
[![Native Backend](<https://img.shields.io/badge/Backend-Cranelift_Native_(Mach--O_/_ELF)-orange.svg>)#-arquitectura-del-compilador-y-runtime)
[![Go Backend](https://img.shields.io/badge/Backend-Golang_Toolchain_Transpiler-blue.svg)#-backend-nativo-y-modelo-golang)
[![Concurrency](https://img.shields.io/badge/Concurrency-Go--style_CSP_Fibers_%26_Channels-purple.svg)#8-concurrencia-csp-modelo-estilo-go)
[![Self-Hosted](https://img.shields.io/badge/Self--Hosted-Stage_1_Bootstrapped-success.svg)#-compilador-self-hosted-y-bootstrapping)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Aura** es un lenguaje de programación de sistemas y servicios backend de alto rendimiento, fuertemente tipado, con inmutabilidad por defecto y compilación a código máquina nativo. Fusiona armónicamente la sintaxis moderna y ergonómica de **TypeScript y Go** con la seguridad estática formal, los tipos algebraicos (ADTs), la verificación exhaustiva de patrones y el manejo seguro de errores de **Rust y ML**.

Aura se compila directamente a **binarios nativos ejecutables autónomos** (formato Mach-O en macOS, ELF en Linux) con cero dependencias externas de runtime gracias a su backend nativo [**Cranelift**](https://cranelift.dev/) y a su runtime M:N ultraligero (`aura-runtime`), o alternativamente mediante transpilación directa a la cadena de herramientas de **Golang**. Su motor de compilación escrito en Rust procesa más de **850,000 líneas de código por segundo** con un tiempo de arranque en frío (_cold-start_) inferior a **2 ms**.

---

## 📑 Tabla de Contenidos

1. [🚀 Inicio Rápido](#-inicio-rápido)
   - [Construcción de la Cadena de Herramientas con Cargo](#1-construir-la-cadena-de-herramientas-de-aura-con-cargo)
   - [Primer Programa en Aura](#2-tu-primer-programa-en-aura)
   - [Modos de Compilación y Ejecución](#3-modos-de-compilación-y-ejecución)
2. [🚀 Novedades en v0.2.0: Subsistema Unificado de Bases de Datos Nativas y Motores NoSQL](#-novedades-en-v020-subsistema-unificado-de-bases-de-datos-nativas-y-motores-nosql)
3. [✨ Características Principales](#-características-principales)
4. [⚡ Benchmarks y Rendimiento Empírico](#-benchmarks-y-rendimiento-empírico)
   - [Tiempos de Ejecución y Consumo de Memoria (Peak RSS)](#1-tiempos-de-ejecución-y-consumo-de-memoria-peak-rss)
   - [Tamaño de Binarios Autónomos](#2-tamaño-de-binarios-autónomos-cero-dependencias)
   - [Tiempos de Compilación y Verificación Estática](#3-tiempos-de-compilación-y-verificación-estática)
   - [Benchmark del Servidor HTTP y Microservicio REST](#4-servidor-http--microservicio-rest-10000-requests-concurrencia--50)
   - [Throughput de Compilación frente a TypeScript (`tsc`)](#5-rendimiento-de-compilación-frente-a-typescript-tsc)
   - [Cómo Reproducir la Suite de Pruebas](#6-cómo-ejecutar-la-suite-de-benchmarks)
5. [📁 Arquitectura del Compilador y Runtime](#-arquitectura-del-compilador-y-runtime)
   - [Diagrama de Flujo del Pipeline](#diagrama-de-flujo-del-pipeline)
   - [Estrategia Dual-Target: Binarios Nativos e Interoperabilidad con JavaScript](#estrategia-dual-target-binarios-nativos-e-interoperabilidad-con-javascript)
   - [Por qué Rust para la Implementación: Ventajas y Compromisos](#por-qué-rust-para-la-implementación-ventajas-y-compromisos)
   - [Organización del Código Fuente](#organización-del-código-fuente)
   - [El Runtime Nativo Autónomo (`aura-runtime`)](#el-runtime-nativo-autónomo-cratesaura-runtime)
6. [📘 Guía Completa del Lenguaje Aura](#-guía-completa-del-lenguaje-aura)
   - [1. Sistema de Tipos de Datos Completo](#1-sistema-de-tipos-de-datos-completo)
   - [2. Variables, Inmutabilidad y Desestructuración](#2-variables-inmutabilidad-y-desestructuración)
   - [3. Control de Flujo, Expresiones y Bucles Etiquetados](#3-control-de-flujo-expresiones-y-bucles-etiquetados)
   - [4. Slices y Segmentación de Colecciones (Modelo Golang)](#4-slices-y-segmentación-de-colecciones-modelo-golang)
   - [5. Tipos Suma (ADTs) y Pattern Matching Exhaustivo](#5-tipos-suma-adts-y-pattern-matching-exhaustivo)
   - [6. Manejo de Errores con Result, Option y Operador `?`](#6-manejo-de-errores-con-result-option-y-operador-)
   - [7. Pipelines (`|>`) y Optimización Tail-Call (TCO)](#7-pipelines--y-optimización-de-llamadas-por-la-cola-tco)
   - [8. Concurrencia CSP (Fibers, Canales, Select y Sync)](#8-concurrencia-csp-modelo-estilo-go)
   - [9. Seguridad en Concurrencia (`Sendable`) y Deadlock Sentinel](#9-seguridad-en-concurrencia-sendable-y-deadlock-sentinel)
   - [10. Limpieza Garantizada: Defer LIFO y ErrDefer Condicional](#10-limpieza-garantizada-defer-lifo-y-errdefer-condicional-zig)
   - [11. Métodos con Receptor, Visibilidad e Interfaces Implícitas](#11-métodos-con-receptor-visibilidad-e-interfaces-implícitas)
   - [12. Punteros Explícitos, Packed Structs y C FFI (`extern "C"`)](#12-punteros-explícitos-packed-structs-y-c-ffi-extern-c)
   - [13. Generador Automático de Cabeceras FFI (`aurac bindgen`)](#13-generador-automático-de-cabeceras-ffi-aurac-bindgen)
   - [14. Contexto y Cancelación Propagada (`Context`)](#14-contexto-y-cancelación-propagada-context)
   - [15. Genéricos y Polimorfismo Paramétrico](#15-genéricos-y-polimorfismo-paramétrico)
   - [16. Directivas de Compilación, Build Tags y Struct Tags](#16-directivas-de-compilación-build-tags-y-struct-tags)
   - [17. Ingestión de TypeScript (`.d.ts`) e Inclusión Estática (`embed`)](#17-ingestión-de-typescript-dts-e-inclusión-estática-embed)
   - [18. Directivas de Prueba, Aserciones y Benchmarks](#18-directivas-de-prueba-aserciones-y-benchmarks)
7. [🗄️ Librería Estándar, Servidor Web y Bases de Datos](#️-librería-estándar-servidor-web-y-bases-de-datos)
   - [Servidor Web HTTP ServeMux (`net/http`)](#-servidor-web-http-y-mux-de-enrutamiento-nethttp)
   - [Métricas Prometheus y Trazabilidad W3C Traceparent](#-métricas-prometheus-y-trazabilidad-distribuida)
   - [Subsistema Unificado de Bases de Datos Nativas y Connection Pools](#-subsistema-unificado-de-bases-de-datos-nativas-y-connection-pools)
   - [Controlador Nativo Embebido de SQLite (`sqlite`)](#-controlador-nativo-embebido-de-sqlite-sqlite)
   - [Controlador Nativo de PostgreSQL (`postgres` / `pg`)](#-controlador-nativo-de-postgresql-postgres--pg)
   - [Controlador Nativo de MySQL (`mysql`)](#-controlador-nativo-de-mysql-mysql)
   - [Interfaz Genérica SQL y Transacciones ACID (`database/sql` / `sql`)](#-interfaz-genérica-sql-y-transacciones-acid-databasesql--sql)
   - [Suite de Ejemplos de Drivers de Bases de Datos Nativas](#-suite-de-ejemplos-de-drivers-de-bases-de-datos-nativas)
   - [Cliente y Cache Nativo de Redis (`redis` / `std/redis`)](#-cliente-y-cache-nativo-de-redis-redis--stdredis)
   - [Controlador Nativo de MongoDB (`mongodb` / `mongo` / `std/mongodb`)](#-controlador-nativo-de-mongodb-mongodb--mongo--stdmongodb)
   - [Módulos OS, Time, Crypto, JWT y JSON](#-módulos-del-sistema-os-time-crypto-jwt-y-json)
   - [Microservicio REST de Producción: `bookstore_api`](#-microservicio-rest-de-producción-bookstore_api)
8. [🐛 Depuración Interactiva y Source Maps V3](#-depuración-interactiva-y-source-maps-v3)
   - [Depurador Paso a Paso en Terminal (`aurac step`)](#depurador-paso-a-paso-en-terminal-aurac-step)
   - [Servidor de Depuración V8 / DAP (`aurac debug`)](#servidor-de-depuración-v8--dap-aurac-debug)
9. [🛠️ Referencia Completa de Herramientas y CLI](#️-referencia-completa-de-herramientas-y-cli)
   - [Compilador Central (`aurac`)](#1-compilador-central-aurac)
   - [Runner de Pruebas, Benchmarks y Cobertura (`auratest`)](#2-runner-de-pruebas-benchmarks-y-cobertura-auratest)
   - [Formateador de Código Opinado (`aurafmt`)](#3-formateador-de-código-aurafmt-estilo-gofmt)
   - [Servidor LSP de Lenguaje (`auralsp`)](#4-servidor-de-lenguaje-auralsp--aurac-lsp)
   - [Gestor Descentralizado de Módulos (`auramod` / `aurac mod`)](#5-gestor-descentralizado-de-módulos-auramod--aurac-mod)
   - [Generador de Bindings FFI C/Rust (`aurabindgen`)](#6-generador-de-bindings-ffi-aurabindgen--aurac-bindgen)
   - [Playground Web Interactivo (`aurac playground`)](#7-playground-web-interactivo)
10. [🧩 Configuración en Editores](#-configuración-en-editores)
11. [🔄 Compilador Self-Hosted y Bootstrapping](#-compilador-self-hosted-y-bootstrapping)
12. [🗺️ Estado Actual, Roadmap e Hitos](#️-estado-actual-roadmap-e-hitos)
13. [📄 Licencia y Marcas Registradas](#-licencia)

---

## 🚀 Inicio Rápido

### 1. Construir la Cadena de Herramientas de Aura con Cargo

Aura está implementado en Rust (edición 2024) y utiliza Cargo como gestor de compilación y dependencias del motor del compilador y su runtime nativo ([`crates/aura-runtime`](crates/aura-runtime)).

#### A. Requisitos Previos

Asegúrate de contar con una cadena de herramientas de Rust moderna:

```bash
rustc --version # Requiere Rust 1.85+ (Edición 2024)
cargo --version
```

#### B. Clonar y Compilar la Toolchain Completa

```bash
git clone https://github.com/aura-lang-developer/aura-lang.git
cd aura-lang

# Compilación optimizada para producción (Release con Thin LTO, 1 codegen-unit y strip de símbolos)
cargo build --release

# O compilación rápida de desarrollo (Debug)
cargo build
```

El workspace está estructurado con `[workspace.dependencies]` centralizadas, `[workspace.lints]` estandarizados y perfiles de optimización en `Cargo.toml`.

Los ejecutables compilados se ubicarán en `target/release/` (o `target/debug/`):

- `aurac`: Compilador central, constructor nativo, ejecutor en caliente, watcher, depurador y playground.
- `auratest`: Suite completa de pruebas unitarias, benchmarks (`ns/op`) y cobertura HTML.
- `aurafmt`: Formateador de código rápido, idempotente y opinado (estilo `gofmt`).
- `auralsp`: Demonio del Language Server Protocol (LSP) compatible con cualquier editor moderno.
- `auramod` / `aurapkg`: Gestor de paquetes descentralizado (`aura.mod` y `aura.lock`).
- `aurabindgen`: Generador automático de bindings FFI a partir de archivos de cabecera C (`.h`).

#### C. Compilación Selectiva por Binario

Puedes compilar únicamente los binarios que necesites mediante la opción `--bin`:

```bash
# Compilar solo el compilador central
cargo build --release --bin aurac

# Compilar solo el runner de pruebas y benchmarks
cargo build --release --bin auratest

# Compilar solo el formateador de código
cargo build --release --bin aurafmt

# Compilar solo el Language Server
cargo build --release --bin auralsp
```

#### D. Ejecución Directa mediante Cargo (sin agregar a PATH)

Puedes invocar las herramientas directamente a través de `cargo run` usando `--` para separar los argumentos del comando de Aura:

```bash
# Compilar un programa Aura a binario nativo independiente:
cargo run --release --bin aurac -- build examples/hello.aura -o dist/hello

# Ejecución rápida en caliente:
cargo run --bin aurac -- run examples/hello.aura

# Verificación de tipos estática (Hindley-Milner):
cargo run --bin aurac -- check examples/hello.aura

# Compilar con build tags condicionales:
cargo run --bin aurac -- build examples/build_tags_demo.aura -o dist/build_demo --tags "pro"

# Ejecutar las pruebas de Aura:
cargo run --bin auratest -- examples/tests/

# Formatear archivos Aura:
cargo run --bin aurafmt -- -w examples/
```

#### E. Instalación Global en el Sistema

Para que todos los comandos (`aurac`, `auratest`, `aurafmt`, etc.) estén disponibles globalmente en tu terminal:

```bash
cargo install --path .
```

Esto instalará todos los binarios en `$HOME/.cargo/bin`. Asegúrate de que esta ruta se encuentre en tu variable de entorno `PATH`.

#### F. Verificación y Pruebas Internas con Cargo

Para verificar la integridad del compilador y todos sus componentes:

```bash
# Ejecutar todas las pruebas unitarias del compilador
cargo test --lib

# Ejecutar las pruebas completas de integración y bootstrapping
cargo test --test bootstrap_tests -- --test-threads=1

# Ejecutar pruebas de concurrencia y validación de build tags
cargo test --test build_tags_and_race_tests
```

### 2. Tu Primer Programa en Aura

Crea un archivo llamado `hello.aura`:

```aura
export fn main(): Unit => {
    println("¡Hola desde Aura Lang!");
}
```

Puedes agregar un alias ergonómico a tu shell (`~/.zshrc` o `~/.bashrc`):

```bash
alias aura="aurac"
aurac --version
# Salida: aurac 0.3.0
```

### 3. Modos de Compilación y Ejecución

#### A. Ejecución Rápida en Desarrollo:

```bash
aurac run hello.aura
# Salida: ¡Hola desde Aura Lang!
```

#### B. Compilación a Binario Nativo Independiente (Cranelift):

Genera un binario nativo autónomo sin dependencias externas:

```bash
aurac build hello.aura -o hello --target native
./hello
# Salida: ¡Hola desde Aura Lang!
```

#### C. Compilación Optimizada con SIMD y Dead-Code Stripping:

```bash
aurac build hello.aura -o hello --release -O3
```

#### D. Compilación mediante la Cadena de Herramientas de Go:

```bash
aurac build hello.aura -o hello --target go
./hello
# O generar el código fuente Go puro inspeccionable:
aurac emit-go hello.aura -o hello.go
```

---

## 🚀 Novedades en v0.2.0: Subsistema Unificado de Bases de Datos Nativas y Motores NoSQL

Aura **v0.2.0** introduce un subsistema unificado de bases de datos nativas de alto rendimiento y fuertemente tipado, diseñado específicamente para binarios ejecutables autónomos compilados directamente mediante **Cranelift**. Funciona con **cero dependencias externas dinámicas en tiempo de ejecución** (sin librerías compartidas `.so` ni `.dylib`), incorpora protocolos de red wire implementados en pure-Rust y proporciona ejecución con almacenamiento embebido en memoria de respaldo:

### 🌟 Aspectos Arquitectónicos Clave

1. **Protocolos Wire y Motores Embebidos en Pure-Rust (Cero Dependencias Externas)**:
   - **SQLite 3 Embebido**: Motor SQLite compilado e integrado estáticamente con soporte para almacenamiento ultrarrápido en memoria (`:memory:`) y bases de datos en archivo único.
   - **Protocolo Wire PostgreSQL (3.0) en Pure-Rust**: Ejecución de consultas parametrizadas extendidas (`$1, $2`), streaming de cursores y autenticación MD5 sin requerir `libpq`.
   - **Protocolo MySQL (10) en Pure-Rust**: Binding de parámetros con sentencias preparadas (`?`), parseo binario de conjuntos de resultados y negociación SHA-1 sin requerir `libmysqlclient`.
   - **Cliente Redis (RESP) en Pure-Rust**: Motor nativo de serialización RESP con memoria caché de respaldo embebida (`:memory:`).
   - **Motor Documental MongoDB en Pure-Rust**: Base de datos documental nativa con soporte para registros BSON/JSON anidados, filtros dinámicos, actualizaciones atómicas y pipelines de agregación multietapa con motor embebido (`:memory:`).

2. **Interfaz Genérica SQL Polimórfica (`database/sql` / `sql`)**:
   - Pool de conexiones estilo Go (`DB`) con límite configurable de conexiones (`maxConns`), comprobaciones de salud (`db.ping()`) y limpieza determinista (`db.close()`).
   - Transacciones ACID Atómicas (`Tx`) con garantías automáticas de `commit()` y `rollback()`, totalmente integradas con `errdefer` transaccional.
   - Iteración de cursores fuertemente tipada (`Rows`) con extractores directos sin copias redundantes (`getInt`, `getString`, `getFloat`, `getBool`, `toRecord`, `toList`).

3. **Motores NoSQL y Almacén Clave-Valor de Alto Rendimiento**:
   - **Redis (`std/database/redis` / `std/redis`)**: Soporte completo para Cadenas Clave-Valor con TTL, Hashes estructurados (`hset`, `hget`, `hdel`, `hgetall`), Contadores atómicos (`incr`, `decr`), Colas FIFO y Listas (`lpush`, `rpop`, `lrange`), Expiración (`expire`, `ttl`) y Health checks (`ping`).
   - **MongoDB (`std/database/mongodb` / `std/mongo`)**: Soporte CRUD exhaustivo (`insertOne`, `insertMany`, `findOne`, `find`, `updateOne`, `updateMany`, `deleteOne`, `deleteMany`, `count`), operadores avanzados de filtrado (`$gt`, `$gte`, `$lt`, `$lte`, `$ne`, `$in`), modificadores atómicos (`$set`, `$inc`, `$unset`) y pipelines de agregación multietapa (`$match`, `$group`, `$sum`, `$avg`).

4. **Emisión Directa de Código Máquina Nativo Cranelift**:
   - Más de 33 puntos de entrada C ABI registrados en `libaura_runtime.a` y mapeados directamente en el código máquina emitido por Cranelift.
   - Acceso a miembros transparente e interoperabilidad total entre la sintaxis síncrona estilo Go (`let res = col.insertOne({...});`) y asíncrona estilo TypeScript (`let res = await col.insertOne({...});`).

5. **7 Ejemplos Autónomos Listos para Producción**:
   - Suite completa de ejemplos ejecutables en el directorio [`examples/`](examples) demostrando CRUD en SQLite, transacciones bancarias ACID, pool de conexiones SQL genérico, procesamiento de pedidos en PostgreSQL, inventario en MySQL, showcase integral SQL y flujos de trabajo NoSQL con Redis y MongoDB.

---

## ✨ Características Principales

- **🏎️ Backend Nativo Cranelift**: Genera código máquina nativo (Mach-O en macOS, ELF en Linux) enlazado estáticamente con el runtime ligero en Rust (`libaura_runtime.a`), sin intérpretes ni dependencias dinámicas.
- **🔒 Inferencia de Tipos Hindley-Milner Bidireccional**: Resolución de tipos robusta, genéricos paramétricos completos (`<T, E>`) y eliminación total de errores de puntero nulo (`null` / `undefined`) mediante los tipos algebraicos estándar `Option<T>` (`Some(v)` / `None`) y `Result<T, E>` (`Ok(v)` / `Err(e)`).
- **🔪 Slices y Colecciones Dinámicas (Modelo Go)**: Sintaxis concisa `[]T` (equivalente a `List<T>`), expresiones de segmentación semiabiertas `s[low:high]`, `s[:high]`, `s[low:]`, `s[:]` y segmentación con 3 índices `s[low:high:max]`, con funciones nativas `len`, `cap`, `append`, `make` y soporte en todos los backends.
- **🔄 Concurrencia CSP (Estilo Go)**: Fibers livianos sobre un scheduler M:N con stack switching (`spawn`), canales fuertemente tipados con buffers configurables (`Channel<T>`), canales direccionales (`SendChannel<T>`, `RecvChannel<T>`), iteración de canales (`for val in ch`), multiplexación selectiva (`select`) con soporte de timeouts y ramas por defecto, además de primitivas de sincronización (`Mutex`, `RWMutex`, `WaitGroup`, `Once`, `Pool`).
- **🛡️ Concurrencia Segura en Compilación (`Sendable`)**: El compilador valida estáticamente que únicamente estructuras libres de punteros crudos puedan cruzar canales, erradicando _data races_ y bloqueando errores en tiempo de compilación.
- **🛡️ Semántica Robusta de Recursos (Go + Zig)**: Sentencias `defer` con garantía LIFO incondicional y sentencias **`errdefer`** que ejecutan limpiezas o rollbacks transaccionales **exclusivamente si ocurre un error o pánico**.
- **🧩 Interfaces Implícitas (Duck Typing Estructural)**: Separación estricta de datos y comportamiento. Las estructuras definen datos y los métodos se asocian mediante receptores por valor `fn (s: Type) ...` o puntero `fn (s: *Type) ...`. Las interfaces se satisfacen automáticamente sin palabras clave como `implements`.
- **⚡ Optimización Tail-Call (TCO)**: Las funciones recursivas en posición de cola se transforman automáticamente en bucles iterativos `while (true)` con consumo de stack $O(1)$.
- **🗄️ Librería Estándar "Batteries-Included"**: Controladores nativos tipados para **PostgreSQL**, **MySQL**, **MongoDB** y **Redis**, junto con un servidor web **HTTP ServeMux** de ultra alto rendimiento (**126k+ req/s**).
- **📊 Observabilidad Nativa**: Servidor HTTP con endpoint `/metrics` en formato Prometheus y propagación distribuida de contexto W3C `traceparent` (OpenTelemetry).
- **📦 Gestor de Paquetes Descentralizado (`auramod`)**: Resolución de dependencias mediante URLs y Git (`github.com/user/pkg`), manifiesto `aura.mod`, lockfile criptográfico `aura.lock` con árbol de hashes SHA-256 (`h1:...`), vendoring offline y verificación de integridad.
- **🔌 Generador de Bindings C/Rust FFI (`aurabindgen`)**: Parsea cabeceras `.h` y genera automáticamente declaraciones `extern "C"`, structs empaquetados (`packed struct`) y funciones wrapper seguras.
- **🐛 Depurador Interactivo de Terminal y Source Maps V3**: Depurador nativo con comandos de paso a paso (`aurac step`), e integración con DAP / V8 Inspector (`aurac debug`) para editores modernos.
- **🔄 Compilador Self-Hosted**: Implementación completa del compilador escrita en el propio lenguaje Aura (`src/aura_compiler/`).

---

## ⚡ Benchmarks y Rendimiento Empírico

Aura incluye una suite de pruebas de rendimiento automatizada y reproducible ubicada en [`benchmarks/`](benchmarks), evaluada rigurosamente en hardware Apple Silicon (Darwin arm64) comparando **Aura Lang (`aurac`)** frente a **Golang (`go 1.27+`)**.

### 1. Tiempos de Ejecución y Consumo de Memoria (Peak RSS)

| Escenario Evaluado                      | Aura Lang (`aurac`) | Golang (`go`) |     Ratio (Aura vs Go)      | Memoria Aura (RSS) | Memoria Go (RSS) |    Ventaja de Aura     |
| :-------------------------------------- | :-----------------: | :-----------: | :-------------------------: | :----------------: | :--------------: | :--------------------: |
| **Fibonacci Recursivo ($N=38$)**        |    **41.43 ms**     |   88.04 ms    | **0.47x (2.1x más rápido)** |    **2.45 MB**     |     4.09 MB      |  **-40.1% menos RAM**  |
| **Criba de Eratóstenes (2M primos)**    |    **14.62 ms**     |    5.50 ms    |            2.66x            |    **3.05 MB**     |     5.94 MB      |  **-48.6% menos RAM**  |
| **Pipeline Funcional (1M items)**       |    **14.80 ms**     |    4.21 ms    |            3.52x            |    **2.86 MB**     |     20.02 MB     |  **-85.7% menos RAM**  |
| **Canales CSP Ping-Pong (200k msgs)**   |    **25.03 ms**     |   21.15 ms    |            1.18x            |      11.05 MB      |     4.02 MB      |    Paridad virtual     |
| **Spawn Concurrente (50k tareas)**      |    **12.27 ms**     |   10.62 ms    |            1.16x            |    **11.84 MB**    |     13.14 MB     |  **-9.9% menos RAM**   |
| **Arranque en Frío / Cold-Start (CLI)** |     **3.59 ms**     |    2.54 ms    |            1.41x            |    **2.25 MB**     |     3.92 MB      |  **-42.6% menos RAM**  |
| **Sincronización Mutex (50k ops)**      |    **16.65 ms**     |   17.34 ms    | **0.96x (Aura más rápido)** |      13.47 MB      |     8.17 MB      | Paridad de rendimiento |
| **Cache KV Concurrente (50k ops)**      |    **16.70 ms**     |   19.23 ms    | **0.87x (13% más rápido)**  |      15.55 MB      |     15.20 MB     |   Paridad de memoria   |

### 2. Tamaño de Binarios Autónomos (Cero Dependencias)

| Escenario                              | Binario Go (MB) | Binario Go Stripped (`-s -w`) | Binario Standalone Aura |                  Ratio Aura vs Go                  |
| :------------------------------------- | :-------------: | :---------------------------: | :---------------------: | :------------------------------------------------: |
| **Cálculo Numérico (Cranelift)**       |     2.32 MB     |            1.51 MB            |   **2.97 – 2.99 MB**    |                       1.28x                        |
| **Pipeline de Datos (Cranelift)**      |     2.32 MB     |            1.51 MB            |       **2.99 MB**       |                       1.29x                        |
| **Servidor HTTP REST / Microservicio** |     8.84 MB     |            5.96 MB            |       **1.78 MB**       | **0.20x (Aura 80% más liviano / 5x más compacto)** |
| **Concurrencia CSP / Canales**         |     2.33 MB     |            1.53 MB            |       **5.06 MB**       |                       2.17x                        |

### 3. Tiempos de Compilación y Verificación Estática

| Escenario                 | `aurac check` (HM Typecheck) | `aurac build` (Standalone) | `go build` |
| :------------------------ | :--------------------------: | :------------------------: | :--------: |
| **Fibonacci Recursivo**   |          **5.7 ms**          |          36.3 ms           |  37.3 ms   |
| **Criba de Eratóstenes**  |          **5.9 ms**          |          39.2 ms           |  36.1 ms   |
| **Pipeline de Datos**     |          **5.1 ms**          |          34.4 ms           |  37.5 ms   |
| **Canales CSP Ping-Pong** |          **5.0 ms**          |          69.5 ms           |  34.6 ms   |
| **Spawn Concurrente**     |          **4.3 ms**          |          69.0 ms           |  34.8 ms   |
| **Cold Start (CLI)**      |          **4.0 ms**          |          35.1 ms           |  33.7 ms   |
| **Sincronización Mutex**  |          **4.3 ms**          |          69.6 ms           |  34.2 ms   |
| **Cache KV Concurrente**  |          **5.2 ms**          |          70.2 ms           |  35.2 ms   |

### 4. Servidor HTTP / Microservicio REST (10,000 requests, Concurrencia = 50)

Evaluación de throughput y percentiles de latencia en peticiones concurrentes contra endpoints JSON:

| Métrica de Rendimiento             | Aura Standalone Server | Go Native Server (`net/http`) |          Ventaja de Aura Lang          |
| :--------------------------------- | :--------------------: | :---------------------------: | :------------------------------------: |
| **Throughput (req/s)**             |   **126,953 req/s**    |         72,122 req/s          |  🏆 **+76% mayor throughput (1.76x)**  |
| **Latencia Media**                 |      **0.36 ms**       |            0.66 ms            |       🏆 **45% menor latencia**        |
| **Latencia Mediana (p50)**         |      **0.32 ms**       |            0.59 ms            |   🏆 **1.8x más rápido en mediana**    |
| **Latencia Percentil 95 (p95)**    |      **0.78 ms**       |            1.35 ms            |   🏆 **42% menor latencia en cola**    |
| **Latencia Percentil 99 (p99)**    |      **1.03 ms**       |            1.97 ms            |     🏆 **48% menor latencia p99**      |
| **Tamaño de Binario Distribuible** |      **1.78 MB**       |  8.84 MB (5.96 MB stripped)   | 🏆 **5x más compacto (-80% de disco)** |

### 5. Rendimiento de Compilación frente a TypeScript (`tsc`)

| Métrica Clave                    |      Aura Lang (`aurac`)       |   TypeScript (`tsc`)   |          Ventaja de Aura           |
| :------------------------------- | :----------------------------: | :--------------------: | :--------------------------------: |
| **Arranque Frío de Compilador**  |     **~1.50 ms – 1.78 ms**     |    ~130 ms – 150 ms    |    🚀 **~80x – 100x más veloz**    |
| **Latencia Interna del Motor**   |    **0.016 ms – 0.089 ms**     |     ~15 ms – 45 ms     | ⚡ **~200x – 500x menor latencia** |
| **Throughput de Compilación**    | **~850,000 – 1,680,000 LOC/s** | ~25,000 – 60,000 LOC/s | 📈 **~20x – 40x mayor throughput** |
| **Consumo de Memoria RAM (RSS)** |          **2.27 MB**           |  103.9 MB – 143.0 MB   |      📉 **45x menor consumo**      |

### 6. Cómo Ejecutar la Suite de Benchmarks

Para reproducir de forma determinista todas las mediciones en tu propia máquina:

```bash
# Compilar y ejecutar la suite automatizada:
go run benchmarks/benchmark_suite.go

# O invocar el binario precompilado:
./benchmarks/benchmark_suite
```

Los resultados completos quedarán guardados en formato JSON estructurado en [`benchmarks/results.json`](benchmarks/results.json).

---

## 📁 Arquitectura del Compilador y Runtime

### Diagrama de Flujo del Pipeline

```mermaid
flowchart TD
    Src["Código Fuente Aura (*.aura)"] --> Lex["Lexer Determinista (lexer.rs)"]
    Lex --> Parse["Parser Pratt Recursivo (parser.rs)"]
    Parse --> AST["Árbol de Sintaxis Abstracta (ast.rs)"]

    DTS["Definiciones TypeScript (*.d.ts)"] --> DTSP["DTS Parser (dts_parser.rs)"]
    DTSP --> TC

    HDR["Cabeceras C (*.h)"] --> BGEN["Aura Bindgen (bindgen.rs)"]
    BGEN --> AST

    AST --> TC["TypeChecker Hindley-Milner (typechecker.rs)<br/>• Inferencia Bidireccional & Genéricos<br/>• Structural Duck Typing<br/>• Sendable Concurrency Safety [E0401]"]

    TC --> BackendMux{"Selector de Backend<br/>(aurac build / run)"}

    BackendMux -->|"--target native (por defecto)"| CL["Cranelift Native Backend (codegen_cranelift.rs)<br/>• Generación de IR Cranelift<br/>• Autovectorización SIMD (-O3)<br/>• Optimización LTO de Código Muerto"]
    CL --> Runtime["Runtime Nativo (crates/aura-runtime)<br/>• M:N Fiber Scheduler & Stack Switching<br/>• Mark-and-Sweep Garbage Collector (GC)<br/>• Canales CSP & Deadlock Sentinel<br/>• Servidor HTTP Zero-Alloc & Métricas Prometheus"]
    Runtime --> BinNative["Binario Nativo Autónomo<br/>(Mach-O en macOS / ELF en Linux)"]

    BackendMux -->|"--target go / emit-go"| GoCG["Transpilador Go Idiomático (codegen_go.rs)"]
    GoCG --> GoTool["Go Toolchain (go build)"]
    GoTool --> BinGo["Binario Go Nativo Autónomo"]

    BackendMux -->|"aurac run / debug / watch"| NodeCG["Generador ES6 + Source Maps V3 (codegen.rs)"]
    NodeCG --> NodeRuntime["Ejecución Instantánea / Inspector V8 DAP"]
```

### Estrategia Dual-Target: Binarios Nativos e Interoperabilidad con JavaScript

Aura adopta un modelo de compilación híbrido **Dual-Target** que combina el rendimiento, la seguridad de memoria y la autonomía de la programación de sistemas con la velocidad de desarrollo, el vasto ecosistema y la versatilidad web del entorno moderno de JavaScript/TypeScript.

```mermaid
graph TD
    A["Código Fuente Aura (*.aura)"] --> B["Compilador Aura (aurac / Rust)"]
    B --> C["TypeChecker & Análisis AST"]
    
    C -->|Desarrollo, Web, NPM, Edge| D["Backend JavaScript (codegen.rs)"]
    D --> E["Node.js / Bun / Browser / Cloudflare / Serverless"]
    
    C -->|Producción, Microservicios, CLI| F["Backend Nativo (codegen_go.rs / Cranelift)"]
    F --> G["Binario Nativo Autónomo (Mach-O / ELF)"]
```

#### ¿Por qué Aura genera y utiliza JavaScript?

1. **Acceso Inmediato al Ecosistema NPM y TypeScript (`.d.ts`)**:
   En lugar de esperar décadas a que madure un ecosistema de librerías desde cero, el compilador de Aura integra un parser de definiciones de TypeScript ([`src/dts_parser.rs`](/aura-lang/src/dts_parser.rs)). Esto permite que los programas en Aura importen, validen tipos y consuman cualquier paquete de NPM con total seguridad estática, duck typing estructural y autocompletado en el IDE.
2. **Bucle de Desarrollo Sub-20ms (`aurac watch` / `aurac run`)**:
   Durante el desarrollo local activo, ejecutar a través del motor V8 (Node.js o Bun) inicia en menos de **20 ms**, evitando el tiempo de enlace (*linking*) de binarios nativos de Go o C para lograr recarga en caliente (*live-reload*) casi instantánea.
3. **Portabilidad Universal (Navegadores Web, Serverless y Edge)**:
   Generar módulos estándar ECMAScript permite ejecutar código Aura en entornos donde los binarios nativos tradicionales no pueden ejecutarse o son difíciles de desplegar:
   - **Playground Web Interactivo**: Ejecución directa de Aura en el navegador del cliente sin necesidad de un servidor backend remoto.
   - **Plataformas Edge & Serverless**: Despliegue nativo en Cloudflare Workers, AWS Lambda (runtime Node.js), Vercel Edge Functions, Deno y Bun sin fricciones de compilación cruzada para múltiples arquitecturas de CPU.
4. **Depuración Paso a Paso con Source Maps V3 de Alta Precisión**:
   Aura emite mapas de fuentes estándar V3 con codificación Base64 VLQ ([`src/sourcemap.rs`](/aura-lang/src/sourcemap.rs)), permitiendo depurar código fuente `.aura` línea a línea directamente en Chrome DevTools y depuradores DAP como VS Code / Antigravity IDE.

#### ¿Cuándo se utiliza cada Backend?

| Caso de Uso | Modo de Compilación / Target | Beneficio Principal |
| :--- | :--- | :--- |
| **Microservicios y APIs en Producción** | `aurac build --standalone` (Go/Cranelift) | Binario ejecutable nativo independiente (Mach-O / ELF), cero dependencias externas y alto rendimiento CSP. |
| **Desarrollo Diario / Live Reload** | `aurac watch --run` (Node.js/Bun) | Arranque en milisegundos y retroalimentación interactiva instantánea. |
| **Integración con Paquetes NPM** | Target JS (`.mjs` + `.d.ts`) | Reutilización directa de librerías consolidadas de JavaScript/TypeScript. |
| **Web & Edge Computing** | Target JS / Web | Despliegue sin fricción en Cloudflare Workers, Vercel Edge y navegadores. |

### Por qué Rust para la Implementación: Ventajas y Compromisos

El compilador del lenguaje Aura (`aurac`), su ecosistema de herramientas (`auralsp`, `aurafmt`, `auratest`, `auramod`) y el runtime nativo (`crates/aura-runtime`) están desarrollados en **Rust (Edición 2024)**. Esta decisión de diseño aporta ventajas arquitectónicas determinantes para el rendimiento y la seguridad del sistema, junto con ciertos compromisos de ingeniería (*trade-offs*).

#### 🚀 Ventajas Principales de Rust

1. **Throughput de Compilación Extremo y Cero Pausas de GC**:
   - Al compilar directamente a código máquina optimizado con LLVM, `aurac` puede analizar sintácticamente, verificar tipos y generar código a más de **850,000 líneas de código por segundo** con un tiempo de arranque en frío (*cold-start*) inferior a **2 ms**.
   - A diferencia de compiladores escritos en lenguajes con recolección de basura (como TypeScript `tsc` o Go), el compilador de Aura opera con gestión de memoria determinista y cero pausas de GC en bases de código masivas.
   - Uso de estructuras de datos especializadas como `FxHashMap` ([`rustc-hash`](https://crates.io/crates/rustc-hash)) para búsquedas $O(1)$ en tablas de símbolos con un hashing 2x–4x más rápido que los algoritmos convencionales.
2. **Seguridad de Memoria en Tiempo de Compilación y Ausencia de Data Races**:
   - El modelo de *Ownership* y *Borrowing* garantiza que el Árbol de Sintaxis Abstracta (AST), el TypeChecker multipaso y los cargadores paralelos de módulos estén formalmente libres de accesos inválidos (*use-after-free*, *null pointer dereference*) y condiciones de carrera (*data races*).
   - En [`crates/aura-runtime`](/aura-lang/crates/aura-runtime), Rust permite construir el scheduler de fibras M:N, el recolector de basura compacto y los canales CSP con referencias estáticas de costo cero (`&'static Scheduler`) y primitivas atómicas directas.
3. **Tipos de Datos Algebraicos (ADTs) y Pattern Matching Exhaustivo**:
   - Las etapas del compilador (Lexer $\to$ Parser $\to$ TypeChecker $\to$ CodeGen) encajan de manera natural con los `enum` etiquetados de Rust ([`src/ast.rs`](/aura-lang/src/ast.rs)).
   - Añadir nuevos nodos de AST o variantes de tipos activa comprobaciones exhaustivas en tiempo de compilación en todos los visitors, impidiendo que queden casos sin manejar en producción.
4. **Generación Directa de Código Máquina en Memoria (Cranelift)**:
   - Integración nativa con [**Cranelift**](https://cranelift.dev/) ([`src/codegen_cranelift.rs`](/aura-lang/src/codegen_cranelift.rs)) para emitir binarios nativos autónomos (Mach-O en macOS, ELF en Linux) directamente en memoria sin depender de compiladores externos de C o C++.
5. **Distribución en un Único Binario Autónomo (Zero Dependencies)**:
   - Toda la suite de herramientas se compila en un ejecutable estático y compacto (~6–10 MB) que **no requiere Node.js, Python ni JVM preinstalados** en el sistema del usuario.

#### ⚠️ Compromisos de Ingeniería y Desafíos (*Trade-offs*)

1. **Curva de Aprendizaje para Contribuidores de la Comunidad**:
   - El estricto sistema de tipos de Rust, los *lifetimes* y el *Borrow Checker* elevan la barrera de entrada para colaboradores casuales en comparación con TypeScript, Go o Python.
2. **Tiempos de Compilación de la Toolchain (`rustc`)**:
   - Compilar el compilador completo desde cero con optimizaciones ThinLTO toma entre 20 y 60 segundos en compilaciones limpias (impacta a los desarrolladores del compilador y pipelines de CI, aunque no a los usuarios finales de Aura).
3. **Rigor en Estructuras de Grafos y Árboles Bidireccionales**:
   - Tablas de símbolos con ámbitos bidireccionales y grafos de control de flujo exigen arquitecturas basadas en índices numéricos o arenas de memoria en lugar de punteros arbitrarios.
4. **Mantenimiento Dual durante la Fase de Autohospedaje (*Self-Hosting*)**:
   - Mantener sincronizadas la implementación de referencia en Rust y la implementación nativa en Aura ([`src/aura_compiler/`](/aura-lang/src/aura_compiler)) durante las fases de bootstrapping requiere esfuerzo de ingeniería adicional.

#### ⚖️ Balance Arquitectónico

| Dimensión | Implementación en Rust (`aurac`) | Alternativa (p. ej. TypeScript / Go / C++) |
| :--- | :--- | :--- |
| **Throughput del Compilador** | 🟢 **Extremo (>850k LOC/s)** | 🟡 Moderado en Go (~400k LOC/s), 🔴 Lento en TS (~80k LOC/s) |
| **Seguridad de Memoria** | 🟢 **100% garantizada en compile-time** | 🔴 Insegura en C/C++, 🟡 Dependiente de GC en Go/TS |
| **Consumo de Memoria** | 🟢 **Mínimo (~5–15 MB RAM)** | 🔴 Alto (V8 runtime o heap de Go) |
| **Generación Nativa Cranelift** | 🟢 **Integración directa en memoria** | 🔴 Requiere FFI / wrappers complejos |
| **Accesibilidad para Contribuir** | 🟡 **Requiere dominio avanzado de Rust** | 🟢 Más accesible en TypeScript o Go |
| **Velocidad de Build de la Toolchain** | 🟡 **Moderada (`cargo build --release`)** | 🟢 Muy rápida en Go |

### Organización del Código Fuente

- [`src/ast.rs`](/aura-lang/src/ast.rs) — Definición formal del AST (expresiones, sentencias, tipos algebraicos, concurrencia CSP, receptores e interfaces).
- [`src/lexer.rs`](/aura-lang/src/lexer.rs) — Lexer determinista con soporte para operadores de tubería (`|>`), canales (`<-`, `chan<-`), punteros y palabras clave de sistemas.
- [`src/parser.rs`](/aura-lang/src/parser.rs) — Parser descendente recursivo y de precedencia de operadores (Pratt Parser).
- [`src/typechecker.rs`](/aura-lang/src/typechecker.rs) — Inferencia bidireccional Hindley-Milner, exhaustividad de patrones, duck typing estructural, reglas de seguridad de concurrencia (`Sendable`) y diagnósticos estructurados con `TypeError` conforme a `core::error::Error` (Rust 2024).
- [`src/codegen_cranelift.rs`](/aura-lang/src/codegen_cranelift.rs) — Backend nativo que compila el AST directamente a código máquina nativo utilizando Cranelift.
- [`src/codegen_go.rs`](/aura-lang/src/codegen_go.rs) — Generador de código idiomático de Golang (`net/http`, goroutines, canales, select, struct receivers).
- [`src/codegen.rs`](/aura-lang/src/codegen.rs) — Generador de código para desarrollo ágil, TCO, soporte de debug y Source Maps V3.
- [`src/bindgen.rs`](/aura-lang/src/bindgen.rs) — Analizador de cabeceras C (`.h`) y generador de bindings FFI y wrappers seguros (`aurac bindgen`).
- [`src/package.rs`](/aura-lang/src/package.rs) — Gestor descentralizado de dependencias y módulos (`auramod` / `aurapkg`), cálculo de hashes criptográficos SHA-256 (`h1:...`) y vendoring offline.
- [`src/backend.rs`](/aura-lang/src/backend.rs) — Orquestador de validación de backend y compilación de binarios.
- [`src/testing.rs`](/aura-lang/src/testing.rs) — Motor de pruebas unitarias, benchmarks (`ns/op`) y reportes de cobertura HTML (`auratest`).
- [`src/formatter.rs`](/aura-lang/src/formatter.rs) — Formateador de código fuente opinado e idempotente estilo `gofmt` (`aurafmt`).
- [`src/lsp.rs`](/aura-lang/src/lsp.rs) — Servidor LSP JSON-RPC 2.0 (diagnósticos, hover, go to definition, autocompletado y formateo).
- [`src/sourcemap.rs`](/aura-lang/src/sourcemap.rs) — Generador de Source Maps V3 con codificador VLQ en Base64 para depuración de alta precisión.
- [`src/aura_compiler/`](/aura-lang/src/aura_compiler) — Compilador _self-hosted_ escrito enteramente en Aura.

### El Runtime Nativo Autónomo (`crates/aura-runtime`)

Ubicado en [`crates/aura-runtime/`](/aura-lang/crates/aura-runtime), es una biblioteca estática en Rust (`libaura_runtime.a`) que se enlaza sin dependencias externas:

- **Fiber Scheduler M:N (`scheduler.rs`, `fiber.rs`)**: Ejecuta miles de fibers cooperativos sobre un conjunto de hilos del sistema operativo utilizando cambio de pila liviano (_stack switching_). Opera con referencias estáticas de costo cero (`&'static Scheduler`) sin envoltorios atómicos `Arc`, eliminando asignaciones en el heap al iniciar y sobrecarga de contención entre worker threads.
- **Canales CSP & Deadlock Sentinel (`channel.rs`)**: Colas circulares concurrentes seguras para paso de mensajes, métricas operacionales de canales y detección automática de interbloqueos (_deadlock detection_).
- **Servidor HTTP Zero-Alloc (`http.rs`)**: Servidor HTTP/1.1 con Keep-Alive persistente, soporte `TCP_NODELAY`, buffers reutilizables, escrituras coalescidas de cabeceras y payload, endpoint `/metrics` en formato Prometheus con formateo directo vía `writeln!` (cero asignaciones intermedias) y propagación W3C `traceparent`.
- **Recolector de Basura Compacto y Concurrente (`gc.rs`)**: GC rápido de tipo Mark-and-Sweep con 64 shards de punteros protegidos mediante `RwLock` y `rustc-hash` (`FxHashMap`) para lecturas paralelas sin bloqueos durante el escaneo de raíces.
- **Primitivas de Sincronización (`sync.rs`)**: Mutex, RWMutex, Once, Pool y WaitGroup nativos con yielding cooperativo y spin loops.
- **Serializador JSON y Registros de Alta Velocidad (`json.rs`, `record.rs`)**: Codificación y decodificación binaria y de texto, con `FxHashMap` acelerando el acceso a campos de registros de 2x a 4x frente a SipHash.

---

## 📘 Guía Completa del Lenguaje Aura

### 1. Sistema de Tipos de Datos Completo

Aura cuenta con un sistema de tipos estático y expresivo con inferencia bidireccional completa.

#### A. Tipos Primitivos y Enteros de Tamaño Fijo

```aura
// Enteros estándar y con tamaño explícito
let i: Int = 42;             // Entero nativo de 64 bits
let i8: Int8 = 127;          // Con signo: -128 a 127
let i16: Int16 = 32767;
let i32: Int32 = 2147483647;
let i64: Int64 = 9223372036854775807;

// Enteros sin signo
let u8: Uint8 = 255;         // Sin signo: 0 a 255 (alias Byte)
let u16: Uint16 = 65535;
let u32: Uint32 = 4294967295;
let u64: Uint64 = 18446744073709551615;
let b: Byte = 255;           // Byte binario
let r: Rune = 65;            // Rune Unicode UTF-32 (equivalente a 'A')
let up: Uintptr = 0;         // Puntero entero para sistemas

// Números de coma flotante
let f: Float = 3.14159265;   // Float de 64 bits (por defecto)
let f32: Float32 = 3.14;
let f64: Float64 = 2.718281828459;

// Cadenas, Booleanos y Tipo Unidad
let s: String = "Aura Language";
let flag: Bool = true;
let empty: Unit = ();        // Representa la ausencia de valor (void)
```

#### B. Punteros y Direccionamiento de Memoria

Aura permite el uso de punteros para interoperabilidad con sistemas y mutación eficiente por referencia:

```aura
let mut counter: Int = 10;
let ptrCounter: *Int = &counter; // Operador de dirección (&)
*ptrCounter = 20;                // Operador de desreferencia (*)
```

#### C. Canales CSP Direccionales

```aura
let ch: Channel<String> = Channel.new(10);
let sendOnly: SendChannel<String> = ch;  // Solo permite enviar: ch <- valor
let recvOnly: RecvChannel<String> = ch;  // Solo permite recibir: <-ch
```

#### D. Colecciones: Slices, Mapas, Conjuntos y Tuplas

```aura
let numeros: []Int = [1, 2, 3, 4, 5];         // Slices (modelo Go)
let usuarios = #{ "alice" => 100, "bob" => 200 }; // Mapa asociativo literal
let etiquetas = #[ "backend", "native", "csp" ];  // Conjunto literal
let tupla: (Int, String, Bool) = (1, "ok", true); // Tupla heterogénea
```

---

### 2. Variables, Inmutabilidad y Desestructuración

En Aura, las variables declaradas con `let` son **inmutables por defecto**. Si una variable debe cambiar su valor, es obligatorio declararla con `let mut`:

```aura
let version = "0.1.0"; // Inmutable
// version = "0.2.0";  // ✕ Error de compilación: Cannot assign to immutable variable

let mut activo = false;
activo = true;         // ✓ Válido

// Desestructuración de Tuplas
let (id, nombre, vigente) = (101, "Carlos", true);

// Desestructuración de Registros / Estructuras
let usuario = { uid: 42, role: "admin", email: "admin@aura.dev" };
let { uid, role } = usuario;
```

---

### 3. Control de Flujo, Expresiones y Bucles Etiquetados

En Aura, los bloques `{ ... }` y las estructuras condicionales son **expresiones** que retornan el valor de su última sentencia sin punto y coma:

```aura
// Bloque como expresión
let tasaTotal = {
    let base = 100.0;
    let recargo = 0.15;
    base * (1.0 + recargo) // Retorna 115.0
};

// If-Else como expresión ternaria segura
let estado = if calificacion >= 60 { "Aprobado" } else { "Reprobado" };

// Bucle While tradicional
let mut k = 0;
while k < 5 {
    k = k + 1;
};

// Bucle For-In sobre slices o colecciones
for item in [10, 20, 30] {
    println(`Item: ${item}`);
}

// Bucle For-In con índice posicional
for i, nombre in ["Aura", "Go", "Rust"] {
    println(`Índice ${i}: ${nombre}`);
}

// Bucles Etiquetados (Labeled Loops) con break y continue dirigidos
'busquedaMatriz: for f, fila in matriz {
    for c, valor in fila {
        if valor == objetivo {
            println(`Encontrado en (${f}, ${c})`);
            break 'busquedaMatriz; // Rompe el bucle exterior directamente
        }
    }
}
```

---

### 4. Slices y Segmentación de Colecciones (Modelo Golang)

Aura implementa el modelo de slices de Golang como abstracción de secuencias continuas de memoria:

#### A. Sintaxis de Tipos y Declaración

```aura
let mut items: []Int = [10, 20, 30, 40, 50, 60];
```

#### B. Expresiones de Slicing Semiabiertas `[low:high]`

```aura
let s = [10, 20, 30, 40, 50, 60];

let sub1 = s[1:4];     // [20, 30, 40] (desde índice 1 hasta 3)
let sub2 = s[:3];      // [10, 20, 30] (desde el inicio hasta índice 2)
let sub3 = s[3:];      // [40, 50, 60] (desde índice 3 hasta el final)
let sub4 = s[:];       // [10, 20, 30, 40, 50, 60] (vista completa)
let sub5 = s[1:3:5];   // [20, 30] con capacidad limitada a 5 (Go 3-index slice)
```

#### C. Funciones Nativas Go-Style (`len`, `cap`, `append`, `make`)

```aura
println(`Longitud: ${len(items)}`);    // 6
println(`Capacidad: ${cap(items)}`);   // 6

// Añadir elementos dinámicamente con append
items = append(items, 70);

// Crear buffers pre-dimensionados
let buffer = make([]Int, 10, 20); // longitud 10, capacidad 20
```

#### D. Segmentación de Cadenas de Texto

```aura
let texto = "Aura Engine";
let nombre = texto[0:4]; // "Aura"
let motor = texto[5:];   // "Engine"
```

---

### 5. Tipos Suma (ADTs) y Pattern Matching Exhaustivo

Los tipos suma o uniones discriminadas permiten modelar de forma precisa el dominio de negocio. El compilador valida exhaustivamente que todos los casos sean cubiertos en las expresiones `match`:

```aura
export type OrderStatus =
    | Pending
    | Processing { workerId: Int }
    | Shipped(String, String) // (carrier, trackingNumber)
    | Delivered
    | Cancelled { reason: String, at: Int };

fn describeStatus(status: OrderStatus): String => {
    match status {
        Pending => "El pedido está en espera.",
        Processing { workerId } => `Procesando por el operador #${workerId}.`,
        Shipped(carrier, tracking) => `En camino con ${carrier}, guía: ${tracking}.`,
        Delivered => "Pedido entregado al cliente.",
        Cancelled { reason, at } if at > 0 => `Cancelado: ${reason} (Timestamp: ${at}).`,
        Cancelled { reason, .. } => `Cancelado: ${reason}.`,
    }
}
```

Si se omite alguna variante o una rama del `match`, el compilador genera un error estático impidiendo la generación de código.

---

### 6. Manejo de Errores con Result, Option y Operador `?`

Aura **carece totalmente de `null` y `undefined`**. Las operaciones falibles o con valores ausentes se modelan formalmente:

```aura
// Tipos estándar del preludio:
// type Option<T> = Some(T) | None;
// type Result<T, E> = Ok(T) | Err(E);

fn divide(a: Float, b: Float): Result<Float, String> => {
    if b == 0.0 {
        return Err("División por cero no permitida.");
    }
    Ok(a / b)
}

// Desempaquetado y propagación automática con el operador '?'
fn calculateRatio(x: Float, y: Float, z: Float): Result<Float, String> => {
    let r1 = divide(x, y)?; // Si es Err, retorna inmediatamente
    let r2 = divide(r1, z)?;
    Ok(r2)
}
```

---

### 7. Pipelines (`|>`) y Optimización de Llamadas por la Cola (TCO)

El operador de tubería o pipeline (`|>`) permite encadenar transformaciones de datos legibles de izquierda a derecha. Las funciones recursivas que finalizan invocándose a sí mismas son transformadas por el compilador en bucles iterativos `while (true)` con memoria $O(1)$:

```aura
// 1. Pipeline de datos funcional
let totalParesTransformados = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    |> filter(fn(x: Int): Bool => x % 2 == 0)
    |> map(fn(x: Int): Int => x * 3)
    |> toList();

// 2. Función recursiva optimizada con TCO
fn factorialTCO(n: Int, acc: Int = 1): Int => {
    if n <= 1 {
        acc
    } else {
        factorialTCO(n - 1, acc * n) // Optimizado a bucle iterativo sin sobrecoste de pila
    }
}
```

---

### 8. Concurrencia CSP (Modelo Estilo Go)

Aura implementa el modelo de concurrencia CSP (_Communicating Sequential Processes_) con fibers livianos que se ejecutan cooperativamente sobre el scheduler M:N:

```aura
// 1. Spawning de fibers concurrentes
let ch = Channel<String>::new(5); // Canal con buffer de capacidad 5

spawn {
    ch <- "Mensaje 1";
    ch <- "Mensaje 2";
    Channel.close(ch);
};

// 2. Iteración sobre canales hasta su cierre
spawn {
    for msg in ch {
        println(`Recibido: ${msg}`);
    }
};

// 3. Multiplexación no bloqueante con select
let chA = Channel<Int>::new(1);
let chB = Channel<String>::new(1);

select {
    case val = <-chA => {
        println(`Dato numérico recibido: ${val}`);
    },
    case chB <- "hola" => {
        println("Mensaje enviado con éxito a chB");
    },
    case timeout(500) => {
        println("Tiempo límite de espera alcanzado (500ms)");
    },
    default => {
        println("Ningún canal disponible para procesar");
    }
}

// 4. Sincronización con Mutex y WaitGroup
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
println(`Balance sincronizado final: ${balance}`);
```

---

### 9. Seguridad en Concurrencia (`Sendable`) y Deadlock Sentinel

Para garantizar una concurrencia libre de errores de sincronización (_Fearless Concurrency_), el analizador de tipos de Aura impone reglas estrictas en tiempo de compilación:

- **Regla `Sendable` (Error [E0401])**: Está terminantemente prohibido transmitir punteros de memoria crudos (`*T`) a través de canales. Los datos transmitidos deben ser tipos primitivos, registros inmutables, sum types o estructuras que posean propiedad exclusiva (_ownership_).
- **Deadlock Sentinel en Runtime**: El planificador M:N en `crates/aura-runtime` monitoriza activamente el estado de los fibers estacionados y los bloqueos en canales; si todas las tareas quedan bloqueadas esperando mensajes que nunca llegarán, el runtime aborta de forma controlada indicando la causa exacta del interbloqueo.

---

### 10. Limpieza Garantizada: Defer LIFO y ErrDefer Condicional (Zig)

Aura proporciona dos mecanismos deterministas de limpieza de recursos que se ejecutan en orden LIFO (_Last-In, First-Out_):

```aura
fn procesarTransaccion(cuentaId: Int, monto: Float, forzarFallo: Bool): Result<String, String> => {
    println("1. Abriendo conexión a base de datos...");
    // defer se ejecuta SIEMPRE al salir del ámbito de la función (éxito o error)
    defer println("5. Conexión liberada al pool.");

    let tx = iniciarTransaccion();
    // errdefer se ejecuta ÚNICAMENTE si la función sale con Err(...) o panic
    errdefer println("⚠️ [Rollback] Revertiendo cambios por falla en la transacción.");

    println("2. Verificando saldo...");
    if forzarFallo {
        return Err("Saldo insuficiente para completar el débito.");
    }

    println("3. Aplicando cargo...");
    println("4. Transacción confirmada.");
    Ok("Operación exitosa")
}
```

- **En caso de éxito (`Ok`)**: Solo se ejecuta `defer`.
- **En caso de fallo (`Err` o `panic`)**: Se ejecuta primero `errdefer` (haciendo rollback) y luego `defer` (cerrando la conexión), garantizando consistencia absoluta en sistemas distribuidos.

---

### 11. Métodos con Receptor, Visibilidad e Interfaces Implícitas

Siguiendo el diseño idiomático de **Golang**, las estructuras de datos no anidan métodos en su definición. Los métodos se declaran externamente asociando un receptor (_receiver_):

```aura
// 1. Estructura de datos pura
export struct BankAccount {
    accountNumber: String,
    owner: String,
    balance: Float,
    pinHash: String,
};

// 2. Método Público con receptor por puntero (comienza con Mayúscula -> Exportado)
fn (b: *BankAccount) Deposit(amount: Float): Result<Float, String> => {
    if amount <= 0.0 {
        return Err("El monto debe ser positivo.");
    }
    b.balance = b.balance + amount;
    Ok(b.balance)
}

// 3. Método Privado / Interno de paquete (comienza con minúscula -> No exportado)
fn (b: BankAccount) verifyPin(pin: String): Bool => {
    b.pinHash == pin
}

// 4. Interfaz Estructural
export interface AccountViewer {
    GetBalance(): Float;
}

fn (b: BankAccount) GetBalance(): Float => b.balance;

// Consumo polimórfico: BankAccount satisface implícitamente AccountViewer
fn displayBalance(viewer: AccountViewer): Unit => {
    println(`Saldo: $${viewer.GetBalance()}`);
}
```

---

### 12. Punteros Explícitos, Packed Structs y C FFI (`extern "C"`)

Para desarrollo de bajo nivel, emuladores y protocolos de red binarios:

```aura
// Estructura sin relleno (cero padding bytes) para protocolos de red
export packed struct ArpHeader {
    hardware_type: Uint16,
    protocol_type: Uint16,
    hardware_size: Uint8,
    protocol_size: Uint8,
    opcode: Uint16,
};

// Declaración de funciones foráneas de la biblioteca estándar de C
extern "C" {
    fn getpid(): Int;
    fn abs(n: Int): Int;
}

export fn getSystemPid(): Int => {
    getpid()
}
```

---

### 13. Generador Automático de Cabeceras FFI (`aurac bindgen`)

Aura incluye una herramienta nativa para generar wrappers e interfaces tipadas a partir de archivos de cabecera en C (`.h`):

```bash
aurac bindgen sqlite3.h -o sqlite3.aura --name SQLite3 --strip-prefix "sqlite3_"
```

Parsea automáticamente `#define`, constantes, `enum`, `struct` con alineación binaria y prototipos de funciones foráneas, produciendo código Aura nativo seguro con wrappers de conversión de tipos.

---

### 14. Contexto y Cancelación Propagada (`Context`)

Inspirado en `context.Context` de Go, facilita la propagación de plazos máximos (_deadlines_), señales de cancelación y metadatos entre fibras concurrentes y llamadas HTTP:

```aura
// Crear contexto con límite de tiempo de 200 ms
let (ctx, cancel) = Context.withTimeout(Context.background(), 200);
defer cancel();

spawn {
    select {
        case <-ctx.done() => {
            println("Operación cancelada por timeout de contexto.");
        },
        case res = <-realizarPeticionRemota() => {
            println(`Petición exitosa: ${res}`);
        }
    }
};
```

---

### 15. Genéricos y Polimorfismo Paramétrico

Aura soporta funciones y estructuras genéricas parametrizadas sobre tipos arbitrarios:

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

### 16. Directivas de Compilación, Build Tags y Struct Tags

Aura proporciona potentes directivas en tiempo de compilación para controlar los objetivos de compilación, la inclusión condicional de código según plataforma o características, y la disposición de serialización de datos.

#### A. Directivas de Compilación Condicional (`//aura:build`)

Aura evalúa las directivas de compilación situadas en la cabecera absoluta de los archivos `.aura` (antes de cualquier declaración o código ejecutable). Si la directiva se evalúa como `false` para las etiquetas activas o el objetivo de compilación actual, el compilador descarta el archivo de forma transparente tanto en compilación como en descubrimiento de pruebas.

- **Sintaxis Booleana Moderna (`//aura:build`)**: Soporta operadores lógicos `&&` (AND), `||` (OR), `!` (NOT) y paréntesis balanceados:
  ```aura
  //aura:build (linux && amd64) || (darwin && arm64)
  //aura:build !windows && !wasm
  ```
- **Compatibilidad con Toolchain de Go (`//go:build`)**:
  ```aura
  //go:build linux || darwin
  ```
- **Sintaxis Clásica de Go (`// +build`)**: Los espacios actúan como OR; las comas actúan como AND:
  ```aura
  // +build darwin,arm64 linux,amd64
  ```

#### B. Tags Activos del Sistema y Tags Personalizados

El compilador activa automáticamente un conjunto de etiquetas según la plataforma y arquitectura del host:
- **Sistemas Operativos**: `darwin`, `macos`, `unix`, `linux`, `windows`
- **Arquitecturas de CPU**: `amd64`, `x86_64`, `arm64`, `aarch64`
- **Entornos de Runtime**: `aura`, `es6`, `node`

Es posible declarar e inyectar etiquetas personalizadas mediante el flag `--tags` en el CLI:
```bash
# Prueba el ejemplo incluido con y sin etiquetas coincidentes:
# 1. Sin el tag requerido: el archivo se descarta de forma segura
aurac build examples/build_tags_demo.aura -o dist/build_demo
# ➜ ℹ Skipping 'examples/build_tags_demo.aura': build tags do not match.

# 2. Con la etiqueta coincidente: compila exitosamente
aurac build examples/build_tags_demo.aura -o dist/build_demo --tags "pro,metrics"
./dist/build_demo
# ➜ Salida: ¡Compilación exitosa con build tag 'pro' o 'metrics'!

# Verificación de tipos estática con tags de entorno
aurac check examples/build_tags_demo.aura --tags "pro"
```

#### C. Struct Tags para Serialización y ORMs

Los *struct tags* permiten anotar los campos de una estructura con directivas de metadatos para serialización JSON, mapeo relacional a bases de datos o validación:

```aura
export type UserProfile = {
    id: Int,
    fullName: String,
    secretHash: String,
    createdAt: Int,
} `json:"user_profile" db:"users" validate:"required"`;
```

---

### 17. Ingestión de TypeScript (`.d.ts`) e Inclusión Estática (`embed`)

```aura
// Cargar archivos estáticos directamente en memoria en tiempo de compilación
let htmlContent: String = embed("templates/index.html");
let logoBinary: []Byte = embedBytes("assets/logo.png");
```

Para validar tipos frente a definiciones externas de NPM sin escribir envoltorios manuales:

```bash
aurac build app.aura -o app --dts ./node_modules/@types/node/index.d.ts
```

---

### 18. Directivas de Prueba, Aserciones y Benchmarks

Aura incluye un motor nativo e integrado de pruebas unitarias, benchmarks de rendimiento y cobertura de código inspirado en el modelo de Go, impulsado por `auratest` (o `aurac test`).

#### A. Directivas y Convenciones de Descubrimiento de Pruebas

Los archivos que cumplen con las siguientes convenciones de nomenclatura son descubiertos y analizados automáticamente:
- **Archivos de prueba reconocidos**: `*_test.aura`, `.test.aura`, `_spec.aura`, `test_*.aura`, o cualquier archivo `.aura` dentro de directorios `tests/`.
- **Categorización automática**:
  - **Pruebas Unitarias (`UNIT`)**: Categoría predeterminada, o archivos que coinciden con `*_unit_test.aura`.
  - **Pruebas de Integración (`INTEGRATION`)**: Archivos que contienen `_integration_test.aura` o ubicados en directorios `integration/`.
  - **Pruebas End-to-End (`E2E`)**: Archivos que contienen `_e2e_test.aura` o ubicados en directorios `e2e/`.
  - **Benchmarks (`BENCHMARK`)**: Funciones que inician con `benchmark_`, `Benchmark` o `bench_`.

#### B. Firmas de Funciones de Prueba y Benchmarks

Las funciones de prueba deben ser exportadas (`export`) y pueden recibir el contexto `TestingT` o ejecutarse sin parámetros:

```aura
// 1. Prueba unitaria estándar con TestingT
export fn test_addition(t: TestingT) => {
    let sum = 10 + 25;
    t.assertEqual(sum, 35, "10 + 25 debe ser 35");
    t.assertTrue(sum > 30, "la suma debe ser mayor a 30");
}

// 2. Prueba asíncrona para fibers, canales y operaciones de red
export async fn test_async_service(t: TestingT) => {
    let result = await fetchUserData(101);
    t.assertTrue(result.isOk(), "la consulta de usuario debe ser exitosa");
}

// 3. Subpruebas (t.run) y pasos lógicos (t.step)
export fn test_order_lifecycle(t: TestingT) => {
    t.step("Verificar inventario", fn() => {
        t.assertTrue(checkStock("SKU-100"));
    });

    t.run("Descuento cliente VIP", fn(subT: TestingT) => {
        let total = calculateTotal({ isVip: true }, 100.0);
        subT.assertEqual(total, 80.0, "Los VIP obtienen 20% de descuento");
    });

    t.run("Precio cliente estándar", fn(subT: TestingT) => {
        let total = calculateTotal({ isVip: false }, 100.0);
        subT.assertEqual(total, 100.0, "Clientes regulares pagan tarifa completa");
    });
}

// 4. Función de Benchmark con medición adaptativa
export fn benchmark_factorial(b: BenchmarkB) => {
    b.resetTimer();
    factorial(15, 1);
}
```

#### C. Directivas del Contexto de Pruebas (`TestingT`)

El contexto `t: TestingT` proporciona el catálogo de aserciones, logging y control de ciclo de vida:

| Método / Directiva | Descripción |
| :----------------- | :---------- |
| `t.assertEqual(actual, expected, msg?)` | Comprueba igualdad estructural profunda entre dos valores. |
| `t.assertNotEqual(actual, expected, msg?)` | Comprueba que dos valores sean distintos. |
| `t.assertTrue(condicion, msg?)` | Valida que la condición sea estrictamente `true`. |
| `t.assertFalse(condicion, msg?)` | Valida que la condición sea estrictamente `false`. |
| `t.assertDeepEqual(actual, expected, msg?)` | Alias para comparación estructural recursiva profunda. |
| `t.assertThrows(fn, errorEsperado?)` | Valida que invocar `fn()` arroje una excepción o pánico con mensaje coincidente. |
| `t.step(nombre, fn)` | Delimita y cronometra un paso lógico dentro de una prueba compleja. |
| `t.run(subNombre, fn)` | Ejecuta una subprueba aislada con su propio contexto hijo `TestingT`. |
| `t.log(...args)` | Registra mensajes de diagnóstico en la salida de las pruebas. |
| `t.skip(razon)` | Omite la prueba en tiempo de ejecución indicando el motivo. |
| `t.fail(razon)` | Marca inmediatamente la prueba en estado de fallo. |

Aserciones globales disponibles directamente sin el prefijo `t`: `assert(cond, msg)`, `assertTrue(cond, msg)`, `assertFalse(cond, msg)`, `assertEqual(a, b, msg)` y `assertNotEqual(a, b, msg)`.

#### D. Directivas del Contexto de Benchmarks (`BenchmarkB`)

El contexto `b: BenchmarkB` administra el número adaptativo de iteraciones y métricas de rendimiento:

| Método / Directiva | Descripción |
| :----------------- | :---------- |
| `b.n` | Número objetivo de iteraciones calculadas adaptativamente por el runner. |
| `b.resetTimer()` | Reinicia el cronómetro y nanosegundos transcurridos (útil tras inicializaciones costosas). |
| `b.startTimer()` | Reanuda la medición del temporizador. |
| `b.stopTimer()` | Pausa el cronómetro durante limpiezas o asignaciones fuera del benchmark. |
| `b.setBytes(n)` | Indica la cantidad de bytes procesados por iteración para calcular throughput en MB/s. |

#### E. Pruebas Condicionales mediante Build Tags

Aplica directivas de compilación condicional para aislar pruebas lentas o dependientes de infraestructura:

```aura
//aura:build integration && !ci_fast
// Archivo: tests/postgres_integration_test.aura

export fn test_postgres_pool(t: TestingT) => {
    let pool = postgres.createPool(testConfig);
    t.assertTrue(pool.ping());
}
```

Ejecutar pruebas filtradas por tags:
```bash
auratest --tags "integration" ./...
```

---

## 🗄️ Librería Estándar, Servidor Web y Bases de Datos

### 🌐 Servidor Web HTTP y Mux de Enrutamiento (`net/http`)

El servidor HTTP de Aura ofrece rendimiento de grado de producción (**126k+ req/s**), middlewares en cadena, soporte para parámetros dinámicos de ruta, codificación JSON y documentación OpenAPI integrada:

```aura
import { http, os } from "net/http";

let mux = http.newServeMux();

// 1. Middleware global
mux.use(fn(req: Any, res: Any, next: Any) => {
    println(`[HTTP] ${req.method} ${req.url}`);
    res.setHeader("X-Engine", "Aura-Native");
    next();
});

// 2. Ruta con parámetro dinámico (:id) y documentación Swagger
mux.get("/api/books/:id", fn(req: Any, res: Any) => {
    let bookId = http.pathValue(req, "id");
    http.json(res, http.StatusOK, {
        id: bookId,
        title: "The Rust Programming Language",
        inStock: true
    });
});

// 3. Ruta POST con parseo asíncrono de JSON
mux.post("/api/books", fn(req: Any, res: Any) => async {
    let parsed = await http.parseJson(req);
    match parsed {
        Ok(data) => http.json(res, http.StatusCreated, { created: true, book: data }),
        Err(err) => http.error(res, `Cuerpo inválido: ${err}`, http.StatusBadRequest)
    }
});

// 4. Habilitar documentación interactiva Swagger UI y OpenAPI 3.0
mux.enableSwagger("/swagger");

// 5. Iniciar escucha
let port = os.env("PORT") != "" ? os.env("PORT") : "8080";
println(`🚀 Servidor HTTP iniciado en :${port}`);
mux.listenAndServe(`:${port}`);
```

### 📊 Métricas Prometheus y Trazabilidad Distribuida

El servidor HTTP incorpora soporte nativo para monitoreo en producción:

- **Endpoint `/metrics`**: Exporta contadores y gauges nativos (`aura_http_requests_total`, `aura_scheduler_fibers`, `aura_csp_channel_operations`).
- **W3C `traceparent`**: Extrae y propaga identificadores de traza distributed tracing (`traceparent` header) entre llamadas de microservicios, cumpliendo los estándares de OpenTelemetry.

---

### 🗄️ Subsistema Unificado de Bases de Datos Nativas y Connection Pools

Aura incorpora un subsistema de bases de datos nativas de alto rendimiento fuertemente tipado, diseñado para binarios ejecutables autónomos compilados directamente mediante **Cranelift**. No requiere **ninguna dependencia dinámica externa** (cero librerías `.so` o `.dylib`) y sigue el estándar de gestión de pools de conexiones y recursos de Go `database/sql`.

#### Puntos Destacados de la Arquitectura
- **Motor SQLite Embebido**: Motor C de SQLite 3 integrado estáticamente, con soporte de bases de datos en memoria (`:memory:`) y en ficheros de disco.
- **Protocolo de Red PostgreSQL 3.0 (Pure-Rust)**: Cliente TCP nativo con soporte de autenticación Cleartext y MD5, consultas parametrizadas (extended query) y decodificación de filas en streaming.
- **Protocolo de Red MySQL 10 (Pure-Rust)**: Cliente TCP nativo con soporte de handshake y autenticación SHA-1 scramble, binding de parámetros en sentencias preparadas y parseo de conjuntos de resultados.
- **Connection Pooling Concurrente y Seguro (`DB`)**: Pool de conexiones automático con límites máximos configurables (`maxConns`), comprobación de salud (`db.ping()`) y limpieza determinista (`db.close()`).
- **Transacciones ACID (`Tx`)**: Control completo del ciclo de vida transaccional (`db.begin()`, `tx.commit()`, `tx.rollback()`) reutilizando de forma polimórfica la conexión activa.
- **Extractores de Columnas Fuertemente Tipados (`Rows`)**: Iteración rápida con cursor (`rows.next()`), lectura con tipo (`getInt`, `getString`, `getFloat`, `getBool`) y conversión directa a registros/mapas (`rows.toRecord()`, `rows.toList()`).

---

### 🪶 Controlador Nativo Embebido de SQLite (`sqlite`)

SQLite está embebido directamente en el binario del runtime de Aura:

```aura
import { open } from "std/database/sqlite";

// Abrir base de datos en memoria o archivo con pool de 5 conexiones
let db: DB = sqlite.open(":memory:", 5);
defer db.close();

// Ejecución de esquema DDL
let ddl = "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, balance INT);";
let schemaRes: ExecResult = db.exec(ddl, []);
println(`Esquema creado. Filas afectadas: ${schemaRes.rowsAffected}`);

// Sentencias INSERT parametrizadas (protegidas contra inyección SQL)
let insertRes = db.exec("INSERT INTO users (name, balance) VALUES (?, ?);", ["Alice", 2500]);
println(`Usuario insertado con ID: ${insertRes.lastInsertId}`);

// Consultas parametrizadas e iteración de cursor
let rows: Rows = db.query("SELECT id, name, balance FROM users WHERE balance >= ?;", [1000]);
defer rows.close();

while (rows.next()) {
    let id = rows.getInt(0);
    let name = rows.getString(1);
    let balance = rows.getInt(2);
    println(`[Usuario #${id}] ${name} -> Saldo: $${balance}`);
}
```

---

### 🐘 Controlador Nativo de PostgreSQL (`postgres` / `pg`)

Cliente TCP pure-Rust con el protocolo de red PostgreSQL 3.0:

```aura
import { open } from "std/database/postgres";

// Conectar a servidor PostgreSQL local o remoto
let db: DB = postgres.open("postgres://postgres:secret@localhost:5432/orders_db", 10);
defer db.close();

// Consultas parametrizadas con marcadores de posición $1, $2
let rows = db.query("SELECT id, name, email FROM users WHERE active = $1;", [true]);
defer rows.close();

while (rows.next()) {
    let id = rows.getInt(0);
    let name = rows.getString(1);
    let email = rows.getString(2);
    println(`- Usuario ${id}: ${name} <${email}>`);
}

// Conversión de la fila actual o dataset completo a registros dinámicos de Aura
let userRecord = rows.toRecord();
println(`Nombre del usuario: ${userRecord.name}`);
```

---

### 🐬 Controlador Nativo de MySQL (`mysql`)

Cliente TCP pure-Rust con el protocolo de red MySQL 10:

```aura
import { open } from "std/database/mysql";

let db: DB = mysql.open("mysql://root:secret@127.0.0.1:3306/tienda_db", 10);
defer db.close();

let res: ExecResult = db.exec("INSERT INTO productos (nombre, precio) VALUES (?, ?);", ["Teclado Mecánico", 129]);
println(`ID del producto insertado: ${res.lastInsertId}, Afectados: ${res.rowsAffected}`);
```

---

### 🔄 Interfaz Genérica SQL y Transacciones ACID (`database/sql` / `sql`)

Aura ofrece una interfaz genérica unificada inspirada en `database/sql` de Go que permite intercambiar motores y garantizar transacciones atómicas ACID:

```aura
import { open, DB, Tx, Rows, ExecResult } from "std/database/sql";

// Apertura polimórfica mediante el identificador de driver: "sqlite", "postgres", "mysql"
let db: DB = sql.open("sqlite", ":memory:", 5);
defer db.close();

// Inicializar tabla de cuentas bancarias
db.exec("CREATE TABLE accounts (id INT PRIMARY KEY, balance INT);", []);
db.exec("INSERT INTO accounts (id, balance) VALUES (1, 1000), (2, 500);", []);

// Iniciar una Transacción Atómica ACID
let tx: Tx = db.begin();

// Transferir $200 de la Cuenta 1 a la Cuenta 2
tx.exec("UPDATE accounts SET balance = balance - 200 WHERE id = 1;", []);
tx.exec("UPDATE accounts SET balance = balance + 200 WHERE id = 2;", []);

// Confirmación atómica (retorna 0 en éxito, -1 en caso de error)
let status = tx.commit();
if (status == 0) {
    println("✓ Transferencia confirmada exitosamente!");
} else {
    tx.rollback();
    println("✕ Falló la transferencia, rollback ejecutado!");
}
```

---

### 📚 Suite de Ejemplos de Drivers de Bases de Datos Nativas

Aura incluye un catálogo completo de ejemplos de bases de datos autónomos en el directorio [`examples/`](examples) que demuestran todos los aspectos del subsistema nativo:

| Archivo de Ejemplo | Backend / Paradigma | Aspectos Clave Demostrados | Comando de Ejecución |
| :--- | :--- | :--- | :--- |
| [`examples/db_sqlite_crud.aura`](examples/db_sqlite_crud.aura) | **SQLite** (Embebido) | Creación de esquemas DDL, `INSERT` parametrizado, escaneo `SELECT` con cursor, `UPDATE`, `DELETE` y métricas del pool | `aurac build examples/db_sqlite_crud.aura -o dist/sqlite_crud --standalone && ./dist/sqlite_crud` |
| [`examples/db_transactions_bank.aura`](examples/db_transactions_bank.aura) | **Transacciones ACID** (`Tx`) | Transferencias financieras multisentencia, `commit()` atómico, `rollback()` automático ante fallos e integridad de saldos | `aurac build examples/db_transactions_bank.aura -o dist/bank_tx --standalone && ./dist/bank_tx` |
| [`examples/db_generic_sql_pool.aura`](examples/db_generic_sql_pool.aura) | **SQL Genérico** (`sql.open`) | Despacho de drivers agnóstico a la base de datos, health check con `ping()`, consultas agregadas (`COUNT`, `AVG`, `GROUP BY`) y gestión del ciclo de vida del pool | `aurac build examples/db_generic_sql_pool.aura -o dist/sql_pool --standalone && ./dist/sql_pool` |
| [`examples/db_postgres_microservice.aura`](examples/db_postgres_microservice.aura) | **PostgreSQL** (Wire 3.0) | Cliente TCP pure-Rust, sintaxis de parámetros extendidos `$1, $2`, y flujo de procesamiento de pedidos | `aurac check examples/db_postgres_microservice.aura` |
| [`examples/db_mysql_inventory.aura`](examples/db_mysql_inventory.aura) | **MySQL** (Protocol 10) | Cliente TCP pure-Rust, binding de marcadores `?`, métricas `lastInsertId` y `rowsAffected`, y control de inventario | `aurac check examples/db_mysql_inventory.aura` |
| [`examples/native_database_demo.aura`](examples/native_database_demo.aura) | **Demo Integral** | Showcase integral autónomo: SQLite en memoria, inicialización de esquemas, inserciones parametrizadas, lectura con cursor y transacciones ACID | `aurac build examples/native_database_demo.aura -o dist/native_db_demo --standalone && ./dist/native_db_demo` |
| [`examples/db_stored_procedures_functions.aura`](examples/db_stored_procedures_functions.aura) | **Funciones & Procedimientos** (`CALL` / `SELECT`) | Procedimientos almacenados (`CALL`), funciones escalares y tipo tabla (`RETURNS TABLE`), variables `OUT` en MySQL, agregados y transacciones | `aurac build examples/db_stored_procedures_functions.aura -o dist/sp_demo --standalone && ./dist/sp_demo` |
| [`examples/native_nosql_demo.aura`](examples/native_nosql_demo.aura) | **Suite NoSQL** (Redis y Mongo) | Motor NoSQL autónomo nativo: Clave-Valor en Redis, Hashes, Colas FIFO, Contadores + CRUD Documental en MongoDB, filtros y pipelines de Agregación | `aurac build examples/native_nosql_demo.aura -o dist/nosql_demo --standalone && ./dist/nosql_demo` |

#### Compilación y Ejecución de los Ejemplos Nativos
Todos los ejemplos se compilan directamente a binarios ejecutables nativos en código máquina (Mach-O en macOS, ELF en Linux) con cero dependencias dinámicas externas:

```bash
# 1. Compilar y ejecutar SQLite CRUD
aurac build examples/db_sqlite_crud.aura -o dist/db_sqlite_crud --standalone
./dist/db_sqlite_crud

# 2. Compilar y ejecutar Transacciones Bancarias ACID
aurac build examples/db_transactions_bank.aura -o dist/db_transactions_bank --standalone
./dist/db_transactions_bank

# 3. Compilar y ejecutar Pool de Conexiones SQL Genérico
aurac build examples/db_generic_sql_pool.aura -o dist/db_generic_sql_pool --standalone
./dist/db_generic_sql_pool

# 4. Compilar y ejecutar Showcase NoSQL Nativo (Redis y MongoDB)
aurac build examples/native_nosql_demo.aura -o dist/nosql_demo --standalone
./dist/nosql_demo
```

---

### ⚡ Cliente y Cache Nativo de Redis (`redis` / `std/redis`)

Aura provee un cliente nativo de Redis sin dependencias externas implementando el **protocolo wire RESP** sobre TCP con motor de fallback embebido en memoria (`:memory:`) para binarios 100% autónomos:

- **Operaciones Clave-Valor**: `set(key, val, ttlSeg?)`, `get(key)`, `del(key)`, `exists(key)`.
- **Contadores Atómicos**: `incr(key)`, `decr(key)`.
- **Hashes Estructurados**: `hset(hash, campo, val)`, `hget(hash, campo)`, `hdel(hash, campo)`, `hgetall(hash)`.
- **Colas FIFO y Listas**: `lpush(lista, val)`, `rpop(lista)`, `lrange(lista, inicio, fin)`.
- **TTL y Expiraciones**: `expire(key, segundos)`, `ttl(key)`.
- **Health Checks y Ciclo de Vida**: `ping()`, `close()`.

```aura
import { open, RedisClient } from "std/database/redis";

// Conectar a Redis remoto o usar almacenamiento embebido en memoria autónomo
let r: RedisClient = redis.open(":memory:");
defer r.close();

// Ping
let pong = r.ping(); // "PONG"
println(`Estado Redis: ${pong}`);

// Clave-Valor con TTL opcional (segundos)
r.set("user:101:sesion", "token_abc123", 3600);
let token = r.get("user:101:sesion");
println(`Sesión Activa: ${token}`);

// Contadores atómicos
let visitas = r.incr("pagina:inicio:visitas");
println(`Visitas de la página: ${visitas}`);

// Almacenamiento en tabla hash
r.hset("user:101:perfil", "username", "alice");
r.hset("user:101:perfil", "rol", "admin");
let rol = r.hget("user:101:perfil", "rol");
println(`Rol de usuario: ${rol}`);

// Cola FIFO / Lista
r.lpush("tareas:cola", "email_bienvenida_user_101");
r.lpush("tareas:cola", "generar_factura_pdf_992");
let siguienteTarea = r.rpop("tareas:cola");
println(`Tarea Procesada: ${siguienteTarea}`);
```

---

### 🍃 Controlador Nativo de MongoDB (`mongodb` / `mongo` / `std/mongodb`)

Aura incluye un motor documental NoSQL nativo en pure-Rust compatible con colecciones de documentos JSON/BSON, operadores avanzados de filtrado, modificaciones atómicas y pipelines de agregación:

- **Inserción de Documentos**: `insertOne(doc)`, `insertMany(docs)`.
- **Consultas y Operadores de Comparación**: `findOne(filtro)`, `find(filtro)`, `$gt`, `$gte`, `$lt`, `$lte`, `$ne`, `$in`.
- **Actualizaciones y Modificadores**: `updateOne(filtro, actualizacion)`, `updateMany(filtro, actualizacion)`, `$set`, `$inc`, `$unset`.
- **Eliminación y Conteo**: `deleteOne(filtro)`, `deleteMany(filtro)`, `count(filtro)`.
- **Pipelines de Agregación**: `aggregate(pipeline)` con soporte para `$match`, `$group`, `$sum`, y `$avg`.
- **Ciclo de Vida de Conexión**: `open(uri)`, `db(nombre?)`, `collection(nombre)`, `ping()`, `close()`.

```aura
import { open, MongoClient, MongoDatabase, MongoCollection } from "std/database/mongodb";

// Conectar a MongoDB o ejecutar almacén embebido en memoria
let client: MongoClient = mongodb.open(":memory:");
defer client.close();

let db: MongoDatabase = client.db("catalogo");
let productos: MongoCollection = db.collection("productos");

// Inserción de Documentos
let resultadoInsert = productos.insertOne({
    sku: "LAPTOP-M3",
    nombre: "MacBook Pro 16",
    categoria: "Computadoras",
    precio: 2499,
    stock: 15,
    estado: "disponible"
});
println(`ID Insertado: ${resultadoInsert.insertedId}`);

// Buscar Documento con Filtro
let encontrado = productos.findOne({ sku: "LAPTOP-M3" });
println(`Producto encontrado: ${encontrado}`);

// Actualización Atómica de Campos ($set e $inc)
productos.updateOne(
    { sku: "LAPTOP-M3" },
    { $set: { estado: "descuento" }, $inc: { stock: -1 } }
);

// Consulta con Filtros de Rango ($gt, $lt, $in)
let caros = productos.find({ precio: { $gt: 1000 } });
println(`Productos premium: ${caros}`);

// Conteo de Documentos
let totalArticulos = productos.count({});
println(`Total de artículos en catálogo: ${totalArticulos}`);

// Pipeline de Agregación ($match + $group)
let pipeline = [
    { $match: { categoria: "Computadoras" } },
    { $group: { _id: "$categoria", valorTotal: { $sum: "$precio" }, precioPromedio: { $avg: "$precio" } } }
];
let estadisticas = productos.aggregate(pipeline);
println(`Estadísticas de Categoría: ${estadisticas}`);
```

---

### 🧩 Módulos del Sistema: `os`, `time`, `crypto`, `jwt` y `json`

- **`os`**: `os.args`, `os.env("CLAVE")`, `os.getEnv("CLAVE")`, `os.setEnv("K", "V")`, `os.readFile(path)`, `os.writeFile(path, data)`, `os.exit(0)`.
- **`time`**: `time.now()`, `time.isoString()`, `time.sleep(ms)`.
- **`crypto`**: `crypto.sha256(text)`, `crypto.hmacSha256(key, message)`, `crypto.base64UrlEncode(data)`, `crypto.base64UrlDecode(data)`.
- **`jwt`**: `jwt.sign(payload, secret)`, `jwt.verify(token, secret)`.
- **`JSON`**: `JSON.stringify(valor)`, `JSON.parse(textoJson)`.

---

### 📚 Microservicio REST de Producción: `bookstore_api`

El repositorio incluye la implementación de un microservicio REST enterprise en [`examples/bookstore_service.aura`](examples/bookstore_service.aura) y [`AuraProjects/bookstore_api/`](/AuraProjects/bookstore_api) evaluado con tests E2E automatizados:

- Modelado con Structs, Tags (`json:"..."`) y ADTs para estados de orden.
- Catálogo completo de **Libros, Autores y Editoriales**.
- Validación de stock y cálculo de impuestos (IVA 19%).
- Endpoints de consulta con paginación y filtros de búsqueda.
- Documentación OpenAPI 3.0 interactiva generada automáticamente.

---

## 🐛 Depuración Interactiva y Source Maps V3

### Depurador Paso a Paso en Terminal (`aurac step`)

Aura incluye un depurador interactivo de consola que permite inspeccionar la ejecución línea por línea sin dependencias gráficas:

```bash
aurac step examples/defer_demo.aura
```

```
⚡ Aura Interactive Step Debugger
Target file: defer_demo.aura

📍 [defer_demo.aura:28] in main()
      27 | export fn main(): Unit => {
  ➜   28 |     println("--- Demostración de Defer ---");
      29 |     let res = executeQuery();

(aura-dbg) next
(aura-dbg) into
(aura-dbg) vars
(aura-dbg) break 35
(aura-dbg) continue
(aura-dbg) backtrace
```

| Comando         | Atajos         | Acción Realizada                                                        |
| :-------------- | :------------- | :---------------------------------------------------------------------- |
| `next`          | `n`, `<ENTER>` | **Step Over**: Avanza a la siguiente línea del código fuente.           |
| `into`          | `s`, `step`    | **Step Into**: Entra en la función que se ejecuta.                      |
| `out`           | `o`, `finish`  | **Step Out**: Ejecuta hasta retornar de la función actual.              |
| `continue`      | `c`            | **Continuar**: Reanuda la ejecución hasta el próximo breakpoint.        |
| `break <linea>` | `b <linea>`    | **Breakpoint**: Añade o retira un punto de interrupción en esa línea.   |
| `vars`          | `locals`       | **Variables**: Muestra el valor de todas las variables locales activas. |
| `backtrace`     | `bt`, `stack`  | **Stack**: Imprime la traza de pila mapeada a los archivos `.aura`.     |
| `print <expr>`  | `p <expr>`     | **Evaluar**: Evalúa dinámicamente una expresión en el ámbito local.     |
| `quit`          | `q`, `exit`    | **Salir**: Finaliza la sesión de depuración.                            |

### Servidor de Depuración V8 / DAP (`aurac debug`)

Inicia un servidor de depuración para conectar IDEs modernos o Chrome DevTools mediante Source Maps V3:

```bash
# Iniciar y detenerse en el punto de entrada (puerto 9229)
aurac debug src/main.aura

# Conectar en un puerto específico sin detenerse en la primera línea
aurac debug src/main.aura --port 9300 --no-brk
```

Compatible con:

- **Google Antigravity IDE / VS Code**: Conexión inmediata vía `F5` (_Attach to Aura Process_).
- **Zed / Neovim**: Vía protocolos DAP estándar en `127.0.0.1:9229`.
- **Google Chrome**: Abriendo `chrome://inspect` en el navegador.

---

## 🛠️ Referencia Completa de Herramientas y CLI

### 1. Compilador Central (`aurac`)

```bash
# 1. Compilar a binario nativo independiente con Cranelift (por defecto)
aurac build main.aura -o dist/mi-app

# 2. Compilar con optimizaciones de release (-O3, SIMD, strip de código muerto)
aurac build main.aura -o dist/mi-app --release -O3

# 3. Compilación cruzada hermética (modelo Zig)
aurac build main.aura -o dist/mi-app-linux --target linux/amd64
aurac build main.aura -o dist/mi-app-arm64 --target linux/arm64
aurac build main.aura -o dist/mi-app.exe   --target windows/amd64
aurac build main.aura -o dist/mi-app-musl  --target x86_64-unknown-linux-musl

# 4. Compilar mediante la cadena de herramientas de Go
aurac build main.aura -o dist/mi-app --target go

# 5. Emitir código fuente puro Go (.go)
aurac emit-go main.aura -o dist/main.go

# 6. Ejecución directa e inmediata
aurac run main.aura

# 7. Verificación estática ultra rápida de tipos (sin emitir archivos)
aurac check main.aura

# 8. Modo Watch interactivo (recompilación y ejecución ante cambios)
aurac watch main.aura --run

# 9. Compilación condicional mediante build tags
aurac build main.aura --tags "premium,darwin"

# 10. Ingestión de definiciones TypeScript externas (.d.ts)
aurac build main.aura -o dist/mi-app --dts node_modules/@types/node/index.d.ts
```

### 2. Runner de Pruebas, Benchmarks y Cobertura (`auratest`)

```bash
# Ejecutar todas las pruebas del proyecto
auratest ./...
# o alternativamente:
aurac test ./...

# Modo detallado (verbose) con tiempos individuales
auratest -v ./...

# Filtrar pruebas por expresión regular
auratest -run TestUserAuthentication ./...

# Filtrar pruebas por categoría
auratest --unit ./...
auratest --integration ./...
auratest --e2e ./...

# Pruebas condicionales mediante build tags
auratest --tags "integration,db" ./...

# Ejecutar benchmarks con reporte de tiempo por operación (ns/op)
auratest -bench . ./...

# Generar reporte de cobertura con perfil y exportación a HTML interactivo
auratest --coverage --coverprofile=coverage.out --coverage-html=coverage.html ./...

# Salida estructurada JSON para integración en CI/CD
auratest -json ./...

# Detener ejecución en el primer fallo (fail-fast)
auratest --fail-fast ./...

# Ejecutar pruebas con repeticiones y límite de tiempo
auratest -count=2 -timeout=10000 ./...

# Modo Watch para ejecutar pruebas automáticamente al guardar
auratest -w ./...
```

### 3. Formateador de Código (`aurafmt`, Estilo `gofmt`)

```bash
# Formatear todos los archivos .aura recursivamente in-place
aurafmt -w .
# o mediante aurac:
aurac fmt -w .

# Mostrar diferencias unificadas (diff) sin modificar archivos
aurafmt -d src/main.aura

# Listar archivos con formato inconsistente
aurafmt -l .

# Modo verificación estricta para pipelines CI (código de salida != 0 si hay desalineación)
aurafmt -c .
```

### 4. Servidor de Lenguaje (`auralsp` / `aurac lsp`)

```bash
aurac lsp
# o directamente:
auralsp
```

Proporciona diagnósticos en tiempo real, hover con firmas y documentación, Go-to-Definition, renombrado de símbolos, autocompletado y formateo automático on-save.

### 5. Gestor Descentralizado de Módulos (`auramod` / `aurac mod`)

```bash
# Inicializar un nuevo módulo con archivo aura.mod
aurac mod init github.com/miusuario/mi-app

# Descargar e instalar una dependencia remota
aurac mod get github.com/aura-lang/crypto@v1.0.0

# Sincronizar dependencias automáticamente desde el código (.aura)
aurac mod tidy

# Empaquetar dependencias localmente en ./vendor para builds 100% offline
aurac mod vendor

# Verificar integridad criptográfica SHA-256 contra aura.lock
aurac mod verify

# Visualizar el árbol de dependencias
aurac mod graph
```

### 6. Generador de Bindings FFI (`aurabindgen` / `aurac bindgen`)

```bash
# Generar bindings tipados Aura a partir de cabeceras C
aurac bindgen /usr/include/sqlite3.h -o src/sqlite3.aura --name SQLite3 --strip-prefix "sqlite3_"
```

### 7. Playground Web Interactivo

```bash
aurac playground --port 3000
```

Abre en tu navegador: [http://localhost:3000](http://localhost:3000).

---

## 🧩 Configuración en Editores

### Google Antigravity IDE

Soporte de primera clase con agentes inteligentes ubicado en [`editors/antigravity/`](editors/antigravity):

- **LSP Integrado**: Inferencia estática bidireccional, autocompletado y formateo automático con `aurafmt`.
- **Programación en Pareja con IA**: Autocompletado inteligente (`⌘+I`) con directrices nativas de Aura (`.agents/plugins/aura-lang/rules/AGENTS.md`).
- **Instalación de Extensión**:
  ```bash
  antigravity --install-extension editors/antigravity/aura-antigravity-0.1.0.vsix
  ```

### Visual Studio Code

Extensión oficial ubicada en [`editors/vscode/`](editors/vscode):

```bash
cd editors/vscode && npm install && npm run package
```

### Neovim (`nvim-lspconfig`)

Agrega a tu configuración `init.lua`:

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

### Editor Zed

Utiliza el plugin empaquetado en [`editors/zed/`](editors/zed).

---

## 🔄 Compilador Self-Hosted y Bootstrapping

Aura cuenta con un **compilador completamente auto-alojado** (*self-hosted*) escrito en el propio lenguaje Aura, ubicado en [`src/aura_compiler/`](src/aura_compiler). Incorpora las 4 fases fundamentales de la ingeniería de compiladores:

- [`ast.aura`](src/aura_compiler/ast.aura) — Definiciones formales del AST, tabla de símbolos (`SymbolTable`) para internado de cadenas (*string interning*), rangos fuente (`Span`) y registros de diagnóstico.
- [`diagnostics.aura`](src/aura_compiler/diagnostics.aura) — Formateador de diagnósticos para terminal (estilo Rust/Ariadne) y distancia de Levenshtein para sugerencias automáticas (*"¿Quisiste decir '...'?"*).
- [`lexer.aura`](src/aura_compiler/lexer.aura) — Analizador léxico determinista y generador de flujo de tokens.
- [`parser.aura`](src/aura_compiler/parser.aura) — Parser Pratt descendente recursivo con precedencia de operadores.
- [`typechecker.aura`](src/aura_compiler/typechecker.aura) — Analizador semántico estático e inferencia de tipos Hindley-Milner completo en Aura puro.
- [`codegen.aura`](src/aura_compiler/codegen.aura) — Generador de código JavaScript / Node.js ES6 de alto rendimiento.
- [`codegen_go.aura`](src/aura_compiler/codegen_go.aura) — Transpilador directo a Go que emite código fuente Go 1.21+ idiomático y listo para compilar.
- [`main.aura`](src/aura_compiler/main.aura) — Punto de entrada del pipeline del compilador con opciones CLI multiobjetivo (`--target js|go`, `--check`).

---

### 🏛️ Fases de Mejoras Arquitectónicas

El compilador auto-alojado implementa cuatro fases de mejora modulares:

```mermaid
graph TD
    A["Código Fuente Aura (.aura)"] --> B["Lexer (lexer.aura)"]
    B --> C["Pratt Parser (parser.aura)"]
    C --> D["AST + SymbolTable (ast.aura)"]
    D --> E["Typechecker Semántico (typechecker.aura)"]
    E -->|--check| F["Diagnósticos y Sugerencias Fuzzy (diagnostics.aura)"]
    E -->|--target js| G["CodeGen JavaScript (codegen.aura)"]
    E -->|--target go| H["CodeGen Golang (codegen_go.aura)"]
    G --> I["dist/*.mjs (Node.js / Web)"]
    H --> J["Binario Nativo Autónomo (go build)"]
```

#### Fase 1: Typechecker Semántico en Aura Puro ([`typechecker.aura`](src/aura_compiler/typechecker.aura))
- **Representación de Tipos Semánticos (`SemanticType`)**: Jerarquía formal de tipos con soporte para `STyInt`, `STyFloat`, `STyString`, `STyBool`, `STyUnit`, `STyAny`, `STyOption`, `STyResult`, `STyList`, `STyRecord`, `STyTuple`, `STyFunction`, `STySum`, `STyChannel` y `STyCustom`.
- **Entorno de Tipos Léxico (`TypeEnv`)**: Rastreo de asignaciones de variables (`bindVar`), firmas de funciones (`bindFunction`), variantes de tipos suma (`SemanticSumType`) y alias de tipos (`SemanticTypeAlias`).
- **Verificación de Expresiones y Sentencias**: Comprobación de operadores aritméticos/lógicos binarios, unificación de ramas en `if/else`, condiciones booleanas en bucles `while`, tipos de iteración en colecciones `for-in` y consistencia exhaustiva de brazos en `match`.
- **Inmutabilidad Estricta y Validación de Firmas**: Validación de variables inmutables vs mutables (`let` vs `let mut`), reasignaciones (`checkAssign`), anotaciones de tipo (`checkLetAnnotation`) y tipos de retorno en funciones (`checkReturnType`).

#### Fase 2: Generador de Código Nativo Golang ([`codegen_go.aura`](src/aura_compiler/codegen_go.aura))
- **Transpilación AST a Go 1.21+**: Convierte el AST de Aura en código fuente Go limpio e idiomático, listo para `go build`.
- **Mapeo Directo de Tipos**: Mapeo de tipos Aura a Go (`Int` $\rightarrow$ `int64`, `Float` $\rightarrow$ `float64`, `String` $\rightarrow$ `string`, `Bool` $\rightarrow$ `bool`, `Option<T>` $\rightarrow$ `*T`, `List<T>` $\rightarrow$ `[]T`, `Channel<T>` $\rightarrow$ `chan T`).
- **Visibilidad y Control de Flujo**: Capitalización automática a mayúscula para símbolos exportados, compilación de coincidencia de patrones a sentencias `switch` de Go y agregación de sentencias a nivel de módulo dentro de `func main()` o `func init()`.

#### Fase 3: Internado de Cadenas, Diagnósticos Ricos y Búsqueda Levenshtein ([`ast.aura`](src/aura_compiler/ast.aura), [`diagnostics.aura`](src/aura_compiler/diagnostics.aura))
- **Tabla de Símbolos (`SymbolTable`)**: Internado de cadenas (*string interning* con `internSymbol`, `lookupSymbol`) para reutilización eficiente de memoria en identificadores.
- **Formateo de Diagnósticos Ricos**: Salida de errores estilo Ariadne con niveles de severidad (`DiagError`, `DiagWarning`, `DiagInfo`, `DiagHint`), códigos de error (`E0308`, `E0425`, `E0384`, `E0004`), notas de ayuda y contexto de línea de código.
- **Resolución Difusa de Identificadores**: Algoritmo de distancia Levenshtein de cero asignaciones (`findClosestMatch`) que calcula distancia de edición $\le 3$ para brindar sugerencias automáticas (*"¿Quisiste decir 'msg'?"*) en identificadores no resueltos.

#### Fase 4: Pipeline Multiobjetivo y Bootstrap de 5 Etapas ([`main.aura`](src/aura_compiler/main.aura), [`tests/bootstrap_tests.rs`](tests/bootstrap_tests.rs))
- **Pipeline CLI Multiobjetivo**: Opciones completas de CLI con soporte para `--target js` (por defecto), `--target go`, modo rápido de solo verificación de tipos `--check`, y ruta de salida personalizada `-o <path>`.
- **Bootstrapping en Triple Etapa**: Valida que el compilador Rust compila el compilador Aura $\rightarrow$ el compilador Aura compila programas Aura $\rightarrow$ el compilador Aura se compila a sí mismo en un ejecutable nativo autónomo de **6.43 MB** (Mach-O / ELF).

---

### 🚀 Flujo de Bootstrapping y Uso de la CLI

#### 1. Etapa 1: Compilar los 8 Módulos del Compilador Self-Hosted
Compila los módulos del compilador escrito en Aura utilizando el compilador de referencia en Rust (`aurac`):

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

#### 2. Etapa 2: Compilar Programas con el Compilador JS Self-Hosted (`dist/aurac.mjs`)
```bash
# Compilar a JavaScript
node dist/aurac.mjs examples/bootstrap_demo/hello.aura -o dist/hello.js --target js
node dist/hello.js
# Salida:
# Hello Developer from Self-Hosted Aura Compiler!
# Sum of 1..100: 5050

# Modo rápido de solo verificación estática de tipos
node dist/aurac.mjs examples/bootstrap_demo/hello.aura --check
# Salida: ✓ Checked 'examples/bootstrap_demo/hello.aura': 0 errors

# Compilar a código fuente Golang
node dist/aurac.mjs examples/bootstrap_demo/hello.aura -o dist/hello.go --target go
```

#### 3. Etapa 3: Compilar el Compilador Self-Hosted a Binario Nativo Autónomo
Construye un binario nativo independiente con cero dependencias externas:

```bash
aurac build src/aura_compiler/main.aura -o dist/aura-self-hosted
```

#### 4. Etapa 4: Compilar Código Aura Directamente con el Binario Nativo
```bash
./dist/aura-self-hosted examples/hello.aura -o dist/hello_from_native.js
node dist/hello_from_native.js
# Salida: Hello Developer from Self-Hosted Aura Compiler!
```

#### 5. Etapa 5: Suite de Pruebas Automatizadas de Bootstrapping (5 Etapas)
Ejecuta la suite integral de pruebas de bootstrapping que valida de extremo a extremo las etapas:

```bash
cargo test --test bootstrap_tests -- --test-threads=1
```

---

## 🗺️ Estado Actual, Roadmap e Hitos

- [x] **Núcleo del Lenguaje**: Inferencia de tipos Hindley-Milner, tipos algebraicos (ADTs), coincidencia de patrones exhaustiva, optimización TCO e inmutabilidad por defecto.
- [x] **Concurrencia CSP Completa**: Fibers livianos sobre scheduler M:N (`spawn`), canales tipados con buffers, canales direccionales, iteración de canales, multiplexación `select`, sincronización (`Mutex`, `RWMutex`, `WaitGroup`, `Once`, `Pool`) y `Context`.
- [x] **Seguridad Estática de Concurrencia**: Validación `Sendable` en tiempo de compilación y runtime Deadlock Sentinel.
- [x] **Slices Modelo Golang**: Sintaxis `[]T`, segmentación `s[low:high:max]`, built-ins `len`, `cap`, `append`, `make` y segmentación de cadenas.
- [x] **Semántica de Limpieza**: Sentencias `defer` LIFO y `errdefer` transaccional inspirado en Zig.
- [x] **Programación Orientada a Métodos**: Structs puros de datos, métodos con receptor (`fn (r: Recv) Method()`), visibilidad Go por mayúsculas/minúsculas y duck typing estructural sin `implements`.
- [x] **Backend Nativo Cranelift**: Emisión directa de código máquina nativo para macOS (Mach-O) y Linux (ELF) con runtime estático (`libaura_runtime.a`).
- [x] **Backend y Transpilación Golang**: Generación de código idiomático Go (`aurac emit-go`) y compilación cruzada `--target go`.
- [x] **Librería Estándar y Bases de Datos**: Servidor HTTP ServeMux de ultra alto rendimiento (**126k+ req/s**), métricas Prometheus `/metrics`, OpenTelemetry `traceparent`, Swagger UI y controladores nativos para PostgreSQL, MySQL, MongoDB y Redis.
- [x] **Gestor de Paquetes Descentralizado**: `aura.mod`, lockfile criptográfico `aura.lock`, vendoring offline y verificación de integridad (`auramod` / `aurac mod`).
- [x] **Generador FFI C/Rust (`aurabindgen`)**: Conversión automática de cabeceras `.h` a interfaces Aura seguras.
- [x] **Herramientas de Grado de Producción**: Servidor LSP oficial (`auralsp`), depurador interactivo de terminal (`aurac step`), servidor DAP V8 (`aurac debug`), suite de pruebas (`auratest`) y formateador opinado (`aurafmt`).
- [x] **Modernización y Optimización Rust 2024**: Dependencias y lints de Cargo centralizados en el workspace, árbol de crates deduplicado (`object 0.39`), perfil de release con Thin LTO (`[profile.release]`), GC concurrente del runtime con `RwLock` + `FxHashMap`, scheduler estático de costo cero, formateo de métricas HTTP zero-alloc y diagnósticos estructurados con `TypeError`.
- [x] **Suite de Calidad y Tests**: **250 pruebas automatizadas pasando con éxito** en el repositorio.
- [ ] **Backend WebAssembly (WASM/WASI)**: Emisión de binarios WebAssembly para microservicios serverless en el edge.
- [ ] **Optimizaciones Avanzadas LLVM**: Pipeline opcional de optimización LTO y vectorización SIMD para cargas de cómputo intensivo.

---

## 📄 Licencia

Aura Language es software libre y de código abierto distribuido bajo la [Licencia MIT](LICENSE).

### ⚖️ Marcas Registradas (Trademarks)

> Go is a trademark of Google LLC. Rust is a trademark of the Rust Foundation. TypeScript is a trademark of Microsoft Corp. Zig is a trademark of the Zig Software Foundation. Aura Lang is an independent open-source project and is not affiliated with or endorsed by these entities.
