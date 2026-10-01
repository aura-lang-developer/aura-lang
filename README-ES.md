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
   - [Construcción de la Cadena de Herramientas](#1-construir-la-cadena-de-herramientas-de-aura)
   - [Primer Programa en Aura](#2-tu-primer-programa-en-aura)
   - [Modos de Compilación y Ejecución](#3-modos-de-compilación-y-ejecución)
2. [✨ Características Principales](#-características-principales)
3. [⚡ Benchmarks y Rendimiento Empírico](#-benchmarks-y-rendimiento-empírico)
   - [Tiempos de Ejecución y Consumo de Memoria (Peak RSS)](#1-tiempos-de-ejecución-y-consumo-de-memoria-peak-rss)
   - [Tamaño de Binarios Autónomos](#2-tamaño-de-binarios-autónomos-cero-dependencias)
   - [Tiempos de Compilación y Verificación Estática](#3-tiempos-de-compilación-y-verificación-estática)
   - [Benchmark del Servidor HTTP y Microservicio REST](#4-servidor-http--microservicio-rest-10000-requests-concurrencia--50)
   - [Throughput de Compilación frente a TypeScript (`tsc`)](#5-rendimiento-de-compilación-frente-a-typescript-tsc)
   - [Cómo Reproducir la Suite de Pruebas](#6-cómo-ejecutar-la-suite-de-benchmarks)
4. [📁 Arquitectura del Compilador y Runtime](#-arquitectura-del-compilador-y-runtime)
   - [Diagrama de Flujo del Pipeline](#diagrama-de-flujo-del-pipeline)
   - [Organización del Código Fuente](#organización-del-código-fuente)
   - [El Runtime Nativo Autónomo (`aura-runtime`)](#el-runtime-nativo-autónomo-cratesaura-runtime)
5. [📘 Guía Completa del Lenguaje Aura](#-guía-completa-del-lenguaje-aura)
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
   - [16. Struct Tags y Build Tags Condicionales](#16-struct-tags-y-build-tags-condicionales)
   - [17. Ingestión de TypeScript (`.d.ts`) e Inclusión Estática (`embed`)](#17-ingestión-de-typescript-dts-e-inclusión-estática-embed)
6. [🗄️ Librería Estándar, Servidor Web y Bases de Datos](#️-librería-estándar-servidor-web-y-bases-de-datos)
   - [Servidor Web HTTP ServeMux (`net/http`)](#-servidor-web-http-y-mux-de-enrutamiento-nethttp)
   - [Métricas Prometheus y Trazabilidad W3C Traceparent](#-métricas-prometheus-y-trazabilidad-distribuida)
   - [Controlador Nativo de PostgreSQL (`pg` / `postgres`)](#-controlador-nativo-de-postgresql-pg--postgres)
   - [Controlador Nativo de MySQL (`mysql`)](#-controlador-nativo-de-mysql-mysql)
   - [Controlador Nativo de MongoDB (`mongodb` / `mongo`)](#-controlador-nativo-de-mongodb-mongodb--mongo)
   - [Cliente Nativo de Redis (`redis`)](#-cliente-nativo-de-redis-redis)
   - [Módulos OS, Time, Crypto, JWT y JSON](#-módulos-del-sistema-os-time-crypto-jwt-y-json)
   - [Microservicio REST de Producción: `bookstore_api`](#-microservicio-rest-de-producción-bookstore_api)
7. [🐛 Depuración Interactiva y Source Maps V3](#-depuración-interactiva-y-source-maps-v3)
   - [Depurador Paso a Paso en Terminal (`aurac step`)](#depurador-paso-a-paso-en-terminal-aurac-step)
   - [Servidor de Depuración V8 / DAP (`aurac debug`)](#servidor-de-depuración-v8--dap-aurac-debug)
8. [🛠️ Referencia Completa de Herramientas y CLI](#️-referencia-completa-de-herramientas-y-cli)
   - [Compilador Central (`aurac`)](#1-compilador-central-aurac)
   - [Runner de Pruebas, Benchmarks y Cobertura (`auratest`)](#2-runner-de-pruebas-benchmarks-y-cobertura-auratest)
   - [Formateador de Código Opinado (`aurafmt`)](#3-formateador-de-código-aurafmt-estilo-gofmt)
   - [Servidor LSP de Lenguaje (`auralsp`)](#4-servidor-de-lenguaje-auralsp--aurac-lsp)
   - [Gestor Descentralizado de Módulos (`auramod` / `aurac mod`)](#5-gestor-descentralizado-de-módulos-auramod--aurac-mod)
   - [Generador de Bindings FFI C/Rust (`aurabindgen`)](#6-generador-de-bindings-ffi-aurabindgen--aurac-bindgen)
   - [Playground Web Interactivo (`aurac playground`)](#7-playground-web-interactivo)
9. [🧩 Configuración en Editores](#-configuración-en-editores)
10. [🔄 Compilador Self-Hosted y Bootstrapping](#-compilador-self-hosted-y-bootstrapping)
11. [🗺️ Estado Actual, Roadmap e Hitos](#️-estado-actual-roadmap-e-hitos)
12. [📄 Licencia y Marcas Registradas](#-licencia)

---

## 🚀 Inicio Rápido

### 1. Construir la Cadena de Herramientas de Aura

Asegúrate de contar con un entorno Rust moderno (edición 2024):

```bash
git clone https://github.com/mrojasb2000/aura-lang.git
cd aura-lang
cargo build --release
```

Los ejecutables compilados se ubicarán en `target/release/`:

- `aurac`: Compilador central, constructor nativo, ejecutor en caliente, watch, depurador y playground.
- `auratest`: Suite completa de pruebas unitarias, benchmarks (`ns/op`) y cobertura HTML.
- `aurafmt`: Formateador de código rápido, idempotente y opinado (estilo `gofmt`).
- `auralsp`: Demonio del Language Server Protocol (LSP) compatible con cualquier editor moderno.
- `auramod` / `aurapkg`: Gestor de paquetes descentralizado (`aura.mod` y `aura.lock`).
- `aurabindgen`: Generador automático de bindings FFI a partir de archivos de cabecera C (`.h`).

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

### Organización del Código Fuente

- [`src/ast.rs`](/aura-lang/src/ast.rs) — Definición formal del AST (expresiones, sentencias, tipos algebraicos, concurrencia CSP, receptores e interfaces).
- [`src/lexer.rs`](/aura-lang/src/lexer.rs) — Lexer determinista con soporte para operadores de tubería (`|>`), canales (`<-`, `chan<-`), punteros y palabras clave de sistemas.
- [`src/parser.rs`](/aura-lang/src/parser.rs) — Parser descendente recursivo y de precedencia de operadores (Pratt Parser).
- [`src/typechecker.rs`](/aura-lang/src/typechecker.rs) — Inferencia bidireccional Hindley-Milner, exhaustividad de patrones, duck typing estructural y reglas de seguridad de concurrencia (`Sendable`).
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

- **Fiber Scheduler M:N (`scheduler.rs`, `fiber.rs`)**: Ejecuta miles de fibers cooperativos sobre un conjunto de hilos del sistema operativo utilizando cambio de pila liviano (_stack switching_).
- **Canales CSP & Deadlock Sentinel (`channel.rs`)**: Colas circulares concurrentes seguras para paso de mensajes, métricas operacionales de canales y detección automática de interbloqueos (_deadlock detection_).
- **Servidor HTTP Zero-Alloc (`http.rs`)**: Servidor HTTP/1.1 con Keep-Alive persistente, soporte `TCP_NODELAY`, buffers reutilizables, escrituras coalescidas de cabeceras y payload, endpoint `/metrics` en formato Prometheus y propagación W3C `traceparent`.
- **Recolector de Basura Compacto (`gc.rs`)**: GC rápido de tipo Mark-and-Sweep adaptado a objetos de dominio y colecciones dinámicas.
- **Primitivas de Sincronización (`sync.rs`)**: Mutex, RWMutex, Once, Pool y WaitGroup nativos.
- **Serializador JSON y Registros (`json.rs`, `record.rs`)**: Codificación y decodificación binaria y de texto de alta velocidad.

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

### 16. Struct Tags y Build Tags Condicionales

```aura
// Tags de compilación condicional para compilación específica de plataforma
// +build darwin,arm64

// Struct tags para serialización de bases de datos y JSON
export type UserProfile = {
    id: Int,
    fullName: String,
    secretHash: String,
} `json:"user_profile" db:"users"`;
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

### 🐘 Controlador Nativo de PostgreSQL (`pg` / `postgres`)

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
        Err(err) => println(`Error SQL: ${err}`)
    }
}

// Transacciones ACID automáticas
async fn transferBalance(txFn: PgTransaction): Task<(), String> {
    await db.transaction(fn(tx: PgTransaction) => {
        tx.execute("UPDATE accounts SET balance = balance - 100 WHERE id = $1", [1]);
        tx.execute("UPDATE accounts SET balance = balance + 100 WHERE id = $2", [2]);
    });
}
```

---

### 🐬 Controlador Nativo de MySQL (`mysql`)

```aura
let pool: MysqlPool = mysql.open("mysql://root:secret@127.0.0.1:3306/tienda_db");

async fn insertarProducto(nombre: String, precio: Float): Task<Int, String> {
    let res = await pool.execute("INSERT INTO productos (nombre, precio) VALUES (?, ?)", [nombre, precio]);
    match res {
        Ok(info) => {
            println(`Insertado ID: ${info.insertId}, Afectados: ${info.affectedRows}`);
            info.insertId
        },
        Err(e) => -1
    }
}
```

---

### 🍃 Controlador Nativo de MongoDB (`mongodb` / `mongo`)

```aura
let client: MongoClient = mongodb.open("mongodb://127.0.0.1:27017/catalogo");
let articulos: MongoCollection = client.db().collection("articulos");

async fn consultarCatalogo(): Task<(), String> {
    let doc = await articulos.findOne({ sku: "AURA-CORE" });
    match doc {
        Ok(Some(item)) => println(`Encontrado: ${item.nombre}, Stock: ${item.stock}`),
        Ok(None) => println("Artículo no existente."),
        Err(err) => println(`Error Mongo: ${err}`)
    }
}
```

---

### ⚡ Cliente Nativo de Redis (`redis`)

```aura
let cache: RedisClient = redis.open("redis://127.0.0.1:6379");

async fn gestionarSesion(usuarioId: String, token: String): Task<Bool, String> {
    // Guardar token con expiración TTL de 3600 segundos
    await cache.set(`sesion:${usuarioId}`, token, 3600);

    // Almacenamiento en hashes estructurados
    await cache.hset(`perfil:${usuarioId}`, "rol", "administrador");

    let rol = await cache.hget(`perfil:${usuarioId}`, "rol");
    println(`Rol en cache: ${rol}`);
    true
}
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

# Ejecutar benchmarks con reporte de tiempo por operación (ns/op)
auratest -bench . ./...

# Generar reporte de cobertura con perfil y exportación a HTML interactivo
auratest --coverage --coverprofile=coverage.out --coverage-html=coverage.html ./...

# Salida estructurada JSON para integración en CI/CD
auratest -json ./...

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

Aura cuenta con un **compilador completamente auto-alojado** (_self-hosted_) escrito en el propio lenguaje Aura, ubicado en [`src/aura_compiler/`](/aura-lang/src/aura_compiler):

- [`ast.aura`](/aura-lang/src/aura_compiler/ast.aura) — Definición formal del AST escrita en Aura.
- [`lexer.aura`](/aura-lang/src/aura_compiler/lexer.aura) — Analizador léxico determinista.
- [`parser.aura`](/aura-lang/src/aura_compiler/parser.aura) — Parser Pratt descendente recursivo.
- [`codegen.aura`](/aura-lang/src/aura_compiler/codegen.aura) — Generador de código.
- [`main.aura`](/aura-lang/src/aura_compiler/main.aura) — Punto de entrada del compilador.

### 1. Compilación Etapa 1 mediante el compilador en Rust:

```bash
cargo run --bin aurac -- compile src/aura_compiler/ast.aura -o dist/ast.mjs
cargo run --bin aurac -- compile src/aura_compiler/lexer.aura -o dist/lexer.mjs
cargo run --bin aurac -- compile src/aura_compiler/parser.aura -o dist/parser.mjs
cargo run --bin aurac -- compile src/aura_compiler/codegen.aura -o dist/codegen.mjs
cargo run --bin aurac -- compile src/aura_compiler/main.aura -o dist/aurac.mjs
```

### 2. Compilar programas Aura utilizando el compilador Self-Hosted:

```bash
node dist/aurac.mjs examples/bootstrap_demo/hello.aura -o dist/hello.js
node dist/hello.js
# Salida: Hello from Aura self-hosted compiler!
```

### 3. Verificación Automatizada del Bootstrap:

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
- [x] **Suite de Calidad y Tests**: **250 pruebas automatizadas pasando con éxito** en el repositorio.
- [ ] **Backend WebAssembly (WASM/WASI)**: Emisión de binarios WebAssembly para microservicios serverless en el edge.
- [ ] **Optimizaciones Avanzadas LLVM**: Pipeline opcional de optimización LTO y vectorización SIMD para cargas de cómputo intensivo.

---

## 📄 Licencia

Aura Language es software libre y de código abierto distribuido bajo la [Licencia MIT](LICENSE).

### ⚖️ Marcas Registradas (Trademarks)

> Go is a trademark of Google LLC. Rust is a trademark of the Rust Foundation. TypeScript is a trademark of Microsoft Corp. Zig is a trademark of the Zig Software Foundation. Aura Lang is an independent open-source project and is not affiliated with or endorsed by these entities.
