> 🌍 **Languages / Idiomas:** [English](README.md) | **Español**

# 📚 Ejemplos de Aura Language (`examples/`)

Este directorio contiene la colección exhaustiva de ejemplos oficiales de **Aura Language** (`aurac`), diseñados para aprender, probar y poner a prueba todas las características sintácticas, de concurrencia CSP, de rendimiento y de seguridad del lenguaje.

---

## 📑 Tabla de Contenidos

1. [Ejemplos Avanzados y de Arquitectura](#1-ejemplos-avanzados-y-de-arquitectura)
2. [Concurrencia CSP y Programación Multihilo (Go-Style)](#2-concurrencia-csp-y-programación-multihilo-go-style)
3. [Programación Genérica (`generics/`)](#3-programación-genérica-generics)
4. [Bases de Datos, SQL Nativo y NoSQL](#4-bases-de-datos-sql-nativo-y-nosql)
5. [Servicios Backend, HTTP y Microservicios](#5-servicios-backend-http-y-microservicios)
6. [Bajo Nivel, Punteros, Packed Structs y FFI](#6-bajo-nivel-punteros-packed-structs-y-ffi)
7. [Manejo de Recursos, Flujo de Control y Paradigmas Modernos](#7-manejo-de-recursos-flujo-de-control-y-paradigmas-modernos)
8. [Suites de Pruebas Unitarias y de Estrés (`tests/`)](#8-suites-de-pruebas-unitarias-y-de-estrés-tests)
9. [Subproyectos y Demos Adicionales](#9-subproyectos-y-demos-adicionales)
10. [Cómo Ejecutar los Ejemplos](#-cómo-ejecutar-los-ejemplos)

---

## 1. Ejemplos Avanzados y de Arquitectura

| Archivo | Concepto y Características Clave |
| :--- | :--- |
| [`advanced_worker_pool.aura`](advanced_worker_pool.aura) | **Pool Concurrente CSP con Priorización y Duck Typing**: Fibras `spawn`, canales tipados (`Channel<T>`, `SendChannel<T>`, `RecvChannel<T>`), interfaz implícita `Executable`, multiplexado no bloqueante con `select`, `WaitGroup`, `defer`, `errdefer` y operador pipeline `\|>`. |
| [`financial_ledger.aura`](financial_ledger.aura) | **Motor Transaccional Contable de Alta Seguridad**: Rollbacks automáticos con `errdefer`, receivers con mutación sobre punteros (`*LedgerAccount`), optimización Tail-Call (TCO) para auditorías recursivas en $O(1)$ stack, y Sum Types con carga enriquecida. |
| [`network_packet_router.aura`](network_packet_router.aura) | **Router de Paquetes de Red y Diagnóstico Binario**: Estructuras de empaquetado binario a nivel de bits (`packed struct`), enteros de ancho fijo (`Uint8`, `Uint16`, `Uint32`), interfaces FFI (`extern "C"`), sub-rebanadas y descomposición funcional de protocolos (`TCP`, `UDP`, `ICMP`). |
| [`modern_features_showcase.aura`](modern_features_showcase.aura) | **Showcase Global de Paradigmas Modernos**: Demostración unificada de ADTs, rebanadas estilo Go (`[]T`), receiver methods, `defer`/`errdefer`, propagación `?`, pipelines y concurrencia CSP básica. |

---

## 2. Concurrencia CSP y Programación Multihilo (Go-Style)

| Archivo | Concepto y Características Clave |
| :--- | :--- |
| [`concurrency_csp_pipeline.aura`](concurrency_csp_pipeline.aura) | **Pipeline CSP Completo**: Patrón productor-consumidor con múltiples fibras concurrentes, canales con búfer, sincronización mediante `WaitGroup` y multiplexado `select` con ramas `timeout` y `default`. |
| [`channel_range_demo.aura`](channel_range_demo.aura) | **Canales Direccionales e Iteración `for-in`**: Uso de canales restringidos a solo envío (`SendChannel<T>`) y solo recepción (`RecvChannel<T>`), cierre de canal y consumo continuo con `for item in inCh`. |
| [`context_demo.aura`](context_demo.aura) | **Cancelación y Tiempos Límite con Context**: Propagación jerárquica de señales de cancelación (`ctx.done()`), timeouts y almacenamiento de metadatos de traza distribuida. |
| [`go_func_concurrency.aura`](go_func_concurrency.aura) | **Funciones Anónimas Concurrentes**: Lanzamiento de fibras anónimas con paso de canales por valor y monitoreo de terminación con `WaitGroup`. |
| [`goroutines_demo.aura`](goroutines_demo.aura) | **Fibras Asíncronas Básicas**: Despacho de tareas paralelas concurrentes y coordinación de resultados agregados. |
| [`aura_routines_demo.aura`](aura_routines_demo.aura) | **Orquestación de Fibras Aura**: Creación de canales y balanceo de carga entre múltiples workers en segundo plano. |
| [`native_concurrency.aura`](native_concurrency.aura) | **Mapeo al Scheduler Nativo M:N**: Demostración de bajo overhead de ejecución de fibras sobre el runtime nativo de Aura. |

---

## 3. Programación Genérica (`generics/`)

Ubicados en la carpeta [`generics/`](generics/):

| Archivo | Concepto y Características Clave |
| :--- | :--- |
| [`01_generic_functions.aura`](generics/01_generic_functions.aura) | **Funciones Genéricas e Invariante de Identidad**: Parámetros de tipo `<ValType>`, inferencia estática y llamadas polimórficas con tipos primitivos (`Int`, `String`, `Bool`). |
| [`02_generic_container.aura`](generics/02_generic_container.aura) | **Contenedores de Tipo Caja (Box/Wrapper)**: Sum Types genéricos con `Empty` y `Full(ItemType)`, constructores polimórficos y extracción segura con `match`. |
| [`03_generic_pair.aura`](generics/03_generic_pair.aura) | **Estructuras Multi-Parámetro (Pares Clave-Valor)**: ADTs con múltiples tipos genéricos `<KeyType, ValType>` y función genérica de inversión `swapPair<TypeA, TypeB>()`. |
| [`04_generic_result_adt.aura`](generics/04_generic_result_adt.aura) | **Patrón de Resultado Personalizado**: Implementación de `Outcome<SuccType, ErrType>` con utilidades genéricas `unwrapOr`, validaciones seguras y cálculos con control de errores (`divideSafe`). |
| [`05_generic_stack.aura`](generics/05_generic_stack.aura) | **Estructura de Datos Pila (Stack LIFO)**: Tipos recursivos genéricos `Stack<ElemType>`, operaciones `push` y cálculo recursivo de profundidad `depth`. |
| [`06_generic_optional.aura`](generics/06_generic_optional.aura) | **Estado Opcional Genérico**: `OptionalState<ItemType>` con variantes `Present(v)` y `Absent`, resolución de valores por defecto con `getOrElse`. |
| [`07_generic_tree.aura`](generics/07_generic_tree.aura) | **Árbol Binario Genérico (Tree ADT)**: Nodos jerárquicos recursivos `Tree<ValType>` con `Leaf` y `Node(...)`, conteo de nodos con recorrido recursivo de árbol. |
| [`08_generic_pipeline.aura`](generics/08_generic_pipeline.aura) | **Pipelines Genéricos con `\|>`**: Funciones de transformación de datos polimórficas encadenadas fluidamente con el operador pipeline. |
| [`09_generic_csp_channels.aura`](generics/09_generic_csp_channels.aura) | **Concurrencia CSP con Canales Genéricos**: Envoltura de mensajes `Message<DataType>`, canales tipados `Channel<Message<String>>` y sincronización productor/consumidor mediante `spawn`. |
| [`10_generic_cache_repo.aura`](generics/10_generic_cache_repo.aura) | **Repositorio y Caché Genérica**: Estados `CacheEntry<ValType>` (`Fresh`, `Expired`, `CacheMiss`), resolución de claves y políticas de fallback polimórficas. |
| [`example_10_generics.aura`](example_10_generics.aura) | **Introducción Clásica a Genéricos**: Demostración de funciones genéricas con sintaxis arrow (`->`) y estructuras sum types. |

---

## 4. Bases de Datos, SQL Nativo y NoSQL

| Archivo | Concepto y Características Clave |
| :--- | :--- |
| [`db_sqlite_crud.aura`](db_sqlite_crud.aura) | **CRUD Completo con SQLite Embebido**: Apertura de conexión en memoria o archivo, creación de esquemas, inserciones parametrizadas y consultas mapeadas a registros. |
| [`db_postgres_microservice.aura`](db_postgres_microservice.aura) | **Microservicio PostgreSQL con Auth SCRAM-SHA-256**: Conexión a bases de datos PostgreSQL modernas (Neon, Supabase, RDS) mediante protocolo nativo pure-Rust y placeholders `$1..$N`. |
| [`db_mysql_inventory.aura`](db_mysql_inventory.aura) | **Gestor de Inventario con MySQL**: Conexión TCP nativa a MySQL/MariaDB, consultas seguras con `?`, transacciones y mapeo de filas tipadas. |
| [`db_generic_sql_pool.aura`](db_generic_sql_pool.aura) | **Pool de Conexiones Genérico Thread-Safe**: Demostración del `AuraDbPool` con adquisición y liberación determinista de conexiones y timeout contra agotamiento de sockets. |
| [`db_transactions_bank.aura`](db_transactions_bank.aura) | **Transacciones Bancarias ACID**: Bloques transaccionales con `BeginTx`, `Commit` y `Rollback` garantizado mediante `errdefer` ante fallos de saldo. |
| [`db_stored_procedures_functions.aura`](db_stored_procedures_functions.aura) | **Procedimientos Almacenados y Funciones SQL**: Ejecución de rutinas SQL complejas y recuperación de múltiples conjuntos de resultados tipados. |
| [`example_postgres.aura`](example_postgres.aura) | **Cliente PostgreSQL Básico**: Conexión directa y ejecución de queries SELECT e INSERT parametrizadas. |
| [`example_mysql.aura`](example_mysql.aura) | **Cliente MySQL Básico**: Ejemplo conciso de conexión y lectura de filas. |
| [`example_nosql.aura`](example_nosql.aura) | **Almacenamiento NoSQL / Clave-Valor**: Operaciones sobre motores embebidos de clave-valor. |
| [`native_database_demo.aura`](native_database_demo.aura) | **Subflujo Unificado de BD**: Demostración de la API estándar `database/sql` de Aura con soporte multivariante. |
| [`native_nosql_demo.aura`](native_nosql_demo.aura) | **Clientes Nativos Redis y MongoDB**: Operaciones sobre Redis (GET/SET/RESP) y MongoDB (BSON/colecciones) con DNS nativo. |

---

## 5. Servicios Backend, HTTP y Microservicios

| Archivo | Concepto y Características Clave |
| :--- | :--- |
| [`golang_http_server.aura`](golang_http_server.aura) | **Servidor HTTP Idiomático Estilo Go**: Uso de `http.newServeMux()`, registro de middlewares, cabeceras personalizadas y enrutamiento REST. |
| [`backend_service.aura`](backend_service.aura) | **Servicio Backend de Producción**: Integración de enrutador HTTP, serialización JSON automática vía struct tags, worker fibers en background y mutexes para estado global. |
| [`bookstore_service.aura`](bookstore_service.aura) | **Microservicio RESTful Integral (Librería)**: Endpoints CRUD de catálogo, validación de modelos, paginación y manejo idiomático de códigos de respuesta HTTP. |
| [`api_service.aura`](api_service.aura) | **API Service Ligero**: Creación de servidor HTTP básico y respuestas en formato JSON. |
| [`example_http.aura`](example_http.aura) | **Cliente y Servidor HTTP Mínimo**: Demostración compacta del paquete estándar `net/http`. |
| [`ecommerce.aura`](ecommerce.aura) | **Lógica de Negocio de Comercio Electrónico**: Sumas con TCO, match sobre estados de pedidos (`OrderStatus`), reglas de descuento y cálculo de carritos. |
| [`microservices/`](microservices/) | **Suite de Microservicios Distribuidos**: Directorio con microservicios completos interactuando entre sí (Orders, Fulfillment, Telemetry, etc.). |

---

## 6. Bajo Nivel, Punteros, Packed Structs y FFI

| Archivo | Concepto y Características Clave |
| :--- | :--- |
| [`pointers_demo.aura`](pointers_demo.aura) | **Punteros y Referencias de Memoria**: Tipos de puntero (`*T`), operador address-of (`&x`), desreferenciación (`*p`), intercambio de memoria `swapValues`, métodos con receptor de puntero (`*Receiver`) y reglas de seguridad de aislamiento en hilos `[E0401]`. |
| [`zig_inspired_features.aura`](zig_inspired_features.aura) | **Características Inspiradas en Zig**: Limpieza transaccional condicional con `errdefer`, estructuras de padding cero `packed struct`, enlaces FFI con `extern "C"` y cross-compilación. |

---

## 7. Manejo de Recursos, Flujo de Control y Paradigmas Modernos

| Archivo | Concepto y Características Clave |
| :--- | :--- |
| [`defer_demo.aura`](defer_demo.aura) | **Limpieza LIFO Garantizada (`defer`)**: Cierre determinístico de descriptores, conexiones y desbloqueo de mutexes al salir del ámbito léxico. |
| [`slices_golang_model.aura`](slices_golang_model.aura) | **Modelo de Slices de Go**: Sintaxis `[]T`, segmentación `s[low:high:max]`, funciones integradas `len()`, `cap()`, `append()` y paso eficiente por referencia. |
| [`golang_methods_visibility.aura`](golang_methods_visibility.aura) | **Receiver Methods y Reglas de Visibilidad**: Métodos de struct públicos (`PascalCase`) vs privados (`camelCase`), mutabilidad in-place con `*Receiver` y aislamiento de datos bancarios. |
| [`labeled_loops_demo.aura`](labeled_loops_demo.aura) | **Bucles Etiquetados**: Romper o continuar bucles anidados con etiquetas explícitas (`break 'outer;`, `continue 'outer;`). |
| [`build_tags_demo.aura`](build_tags_demo.aura) | **Directivas de Compilación Condicional**: Uso de `//aura:build` y `//go:build` con operadores booleanos (`darwin \|\| linux`, `!windows`). |
| [`embed_demo.aura`](embed_demo.aura) | **Incrustación Estática de Recursos (`embed`)**: Empaquetado de archivos de texto o binarios dentro del ejecutable en tiempo de compilación. |
| [`self_hosted_compiler_showcase.aura`](self_hosted_compiler_showcase.aura) | **Showcase del Compilador Self-Hosted**: Demostración del pipeline de análisis y tokenización de Aura escrito en Aura mismo. |
| [`hello.aura`](hello.aura) | **Hola Mundo Básico**: Estructura mínima de un programa ejecutable con `export fn main()`. |

---

## 8. Suites de Pruebas Unitarias y de Estrés (`tests/`)

Ubicados en la carpeta [`tests/`](tests/):

| Archivo | Concepto y Características Clave |
| :--- | :--- |
| [`security_and_performance_stress_test.aura`](tests/security_and_performance_stress_test.aura) | **Suite de Estrés de Seguridad y Rendimiento**: Pruebas de call stack contra desbordamientos mediante TCO, reversión de estado con `errdefer`, saturación y drenaje de canales CSP cerrados, validación de límites de rebanadas y cortocircuito con `??`. |
| [`math_unit_test.aura`](tests/math_unit_test.aura) | **Pruebas Unitarias de Matemáticas**: Aserciones básicas y verificación de funciones aritméticas y lógicas. |
| [`golang_style_test.aura`](tests/golang_style_test.aura) | **Pruebas de Idiomática Go**: Tests con el runner nativo (`aurac --test`) usando receivers y convenciones de testing. |
| [`order_integration_test.aura`](tests/order_integration_test.aura) | **Prueba de Integración de Órdenes**: Flujo completo de checkout, cálculo de subtotales con TCO y emisión de eventos. |
| [`collections_iterator_test.aura`](tests/collections_iterator_test.aura) | **Pruebas de Iteradores y Colecciones**: Operaciones de transformación, filtros y recorridos sobre listas. |
| [`api_e2e_test.aura`](tests/api_e2e_test.aura) | **Prueba End-to-End de APIs**: Verificación de ciclo completo de request/response y concurrencia. |

---

## 9. Subproyectos y Demos Adicionales

* **[`aurabun/`](aurabun/)**: Gestor de paquetes y herramienta de empaquetado de alto rendimiento inspirada en Bun para el ecosistema de Aura.
* **[`bootstrap_demo/`](bootstrap_demo/)**: Demostración del proceso de auto-hospedaje (*bootstrapping*) del compilador.
* **[`compiler_extension_guide/`](compiler_extension_guide/)**: Guía y código base para desarrolladores que desean extender el AST, el typechecker o los generadores de código de Aura.

---

## 🚀 Cómo Ejecutar los Ejemplos

Puedes validar el análisis estático o ejecutar directamente cualquier ejemplo mediante la CLI oficial de Aura:

```bash
# Comprobación de tipos estática ultrarrápida
aura check examples/financial_ledger.aura
aura check examples/pointers_demo.aura
aura check examples/generics/01_generic_functions.aura

# Ejecución directa con el motor de desarrollo
aura run examples/advanced_worker_pool.aura
aura run examples/pointers_demo.aura
aura run examples/generics/05_generic_stack.aura

# Formatear archivos con el formateador oficial
aurafmt -w examples/pointers_demo.aura

# Ejecutar la suite de pruebas unitarias y de estrés
aurac --test examples/tests/math_unit_test.aura
aura run examples/tests/security_and_performance_stress_test.aura

# Compilar a binario nativo autónomo ejecutable
aura build examples/financial_ledger.aura -o ledger-bin
./ledger-bin
```
