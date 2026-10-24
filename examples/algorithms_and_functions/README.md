# 100 Ejemplos de Algoritmos y Funciones en Aura Language (`aurac`)

Directorio: [`examples/algorithms_and_functions/`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions)

Esta suite contiene **100 ejemplos individuales, autónomos y completamente funcionales**, testeados y verificados con el compilador oficial de Aura (`aurac check` / `aurac run`).

---

## Índice de Ejemplos por Categoría

### 1. Aritmética, Números y Fundamentos (001 - 010)
- [`001_sum_two_numbers.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/001_sum_two_numbers.aura): Suma básica de enteros.
- [`002_difference_and_product.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/002_difference_and_product.aura): Operaciones de resta y multiplicación.
- [`003_even_or_odd.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/003_even_or_odd.aura): Evaluación par/impar mediante operador módulo (`%`).
- [`004_maximum_of_three.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/004_maximum_of_three.aura): Búsqueda del mayor de tres enteros.
- [`005_absolute_value.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/005_absolute_value.aura): Función de valor absoluto sin dependencias.
- [`006_factorial_iterative.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/006_factorial_iterative.aura): Factorial con bucle imperativo (`while`).
- [`007_factorial_recursive.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/007_factorial_recursive.aura): Factorial clásico recursivo.
- [`008_fibonacci_iterative.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/008_fibonacci_iterative.aura): Sucesión de Fibonacci iterativa en tiempo $O(n)$ y espacio $O(1)$.
- [`009_gcd_euclidean.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/009_gcd_euclidean.aura): Máximo Común Divisor (Algoritmo de Euclides).
- [`010_lcm_computation.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/010_lcm_computation.aura): Mínimo Común Múltiplo derivado del MCD.

### 2. Teoría de Números y Algoritmos Numéricos (011 - 020)
- [`011_is_prime.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/011_is_prime.aura): Comprobación de números primos en $O(\sqrt{n})$.
- [`012_integer_power_binary_exponentiation.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/012_integer_power_binary_exponentiation.aura): Exponenciación binaria rápida en $O(\log n)$.
- [`013_sum_of_digits.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/013_sum_of_digits.aura): Suma de dígitos decimales individuales.
- [`014_reverse_integer.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/014_reverse_integer.aura): Inversión aritmética de dígitos numéricos.
- [`015_palindrome_number.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/015_palindrome_number.aura): Validación de números capicúa / palíndromos.
- [`016_armstrong_number.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/016_armstrong_number.aura): Comprobación de número de Armstrong / narcisista.
- [`017_count_primes_sieve.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/017_count_primes_sieve.aura): Conteo de primos menores a un límite $N$.
- [`018_collatz_conjecture_steps.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/018_collatz_conjecture_steps.aura): Contador de iteraciones de la conjetura de Collatz.
- [`019_integer_square_root.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/019_integer_square_root.aura): Raíz cuadrada entera mediante búsqueda binaria.
- [`020_perfect_number_checker.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/020_perfect_number_checker.aura): Detector de números perfectos.

### 3. Slices, Búsqueda y Ordenamiento (021 - 030)
- [`021_slice_sum.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/021_slice_sum.aura): Suma acumulativa de elementos en un slice con `for in`.
- [`022_slice_min_max.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/022_slice_min_max.aura): Extracción de mínimo y máximo en una pasada.
- [`023_linear_search.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/023_linear_search.aura): Búsqueda lineal secuencial.
- [`024_binary_search.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/024_binary_search.aura): Búsqueda binaria sobre slice ordenado.
- [`025_bubble_sort.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/025_bubble_sort.aura): Ordenamiento burbuja in-place.
- [`026_selection_sort.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/026_selection_sort.aura): Algoritmo de ordenamiento por selección.
- [`027_insertion_sort.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/027_insertion_sort.aura): Ordenamiento por inserción directa.
- [`028_reverse_slice.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/028_reverse_slice.aura): Inversión in-place con dos punteros (`left` y `right`).
- [`029_filter_even_numbers.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/029_filter_even_numbers.aura): Filtro de elementos pares hacia un nuevo slice.
- [`030_map_slice_elements.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/030_map_slice_elements.aura): Mapeo de elementos (duplicación de valores).

### 4. Cadenas de Texto y Procesamiento de Caracteres (031 - 040)
- [`031_string_length.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/031_string_length.aura): Inspección de longitud y cadenas.
- [`032_string_concat.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/032_string_concat.aura): Interpolación y concatenación con plantillas `\`${}\``.
- [`033_string_contains.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/033_string_contains.aura): Búsqueda de subcadenas con `.includes()`.
- [`034_string_reverse.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/034_string_reverse.aura): Inversión carácter por carácter de strings.
- [`035_count_character_occurrences.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/035_count_character_occurrences.aura): Frecuencia de ocurrencias de un carácter.
- [`036_string_palindrome.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/036_string_palindrome.aura): Verificación de palíndromos en cadenas de texto.
- [`037_caesar_cipher.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/037_caesar_cipher.aura): Cifrado César con `.charCodeAt()` y `String.fromCharCode()`.
- [`038_trim_whitespace.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/038_trim_whitespace.aura): Limpieza de espacios en blanco mediante `.trim()`.
- [`039_split_and_join_words.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/039_split_and_join_words.aura): División (`split`) y unión (`join`) de palabras.
- [`040_count_vowels.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/040_count_vowels.aura): Conteo de vocales mayúsculas y minúsculas.

### 5. Funciones de Orden Superior y Paradigma Funcional (041 - 050)
- [`041_higher_order_apply.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/041_higher_order_apply.aura): Función de orden superior recibiendo tipo `fn(Int) -> Int`.
- [`042_closure_adder_generator.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/042_closure_adder_generator.aura): Generador de closures que capturan el entorno léxico.
- [`043_pipeline_operator_transform.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/043_pipeline_operator_transform.aura): Transformaciones encadenadas con el operador pipeline (`|>`).
- [`044_custom_reduce_fold.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/044_custom_reduce_fold.aura): Implementación de reducción/plegado (fold/reduce).
- [`045_function_composition.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/045_function_composition.aura): Composición funcional matemática $f(g(x))$.
- [`046_tail_call_optimization_sum.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/046_tail_call_optimization_sum.aura): Recursión de cola (TCO) para suma acumulativa.
- [`047_tail_call_fibonacci.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/047_tail_call_fibonacci.aura): Fibonacci con recursión de cola en $O(n)$ sin sobrecarga de pila.
- [`048_currying_simulation.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/048_currying_simulation.aura): Currificación de funciones en Aura.
- [`049_generic_identity_transform.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/049_generic_identity_transform.aura): Función genérica monomórfica `<ValType>`.
- [`050_generic_pair_maker.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/050_generic_pair_maker.aura): Estructura genérica de pares clave-valor `<KeyType, ValType>`.

### 6. Tipos Algebraicos (ADTs), Pattern Matching y Control de Flujo (051 - 060)
- [`051_option_type_usage.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/051_option_type_usage.aura): Manejo seguro sin `null` usando `Option` (`Some` / `None`).
- [`052_result_type_handling.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/052_result_type_handling.aura): Retorno seguro de operaciones mediante `Result` (`Ok` / `Err`).
- [`053_adt_geometric_shapes.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/053_adt_geometric_shapes.aura): Modelado de variantes con sum types (`Circle`, `Rectangle`, `Point`).
- [`054_traffic_light_state_machine.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/054_traffic_light_state_machine.aura): Máquina de estados finitos modelada con ADT.
- [`055_eval_expression_tree.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/055_eval_expression_tree.aura): Evaluador recursivo de árboles de expresión aritmética.
- [`056_custom_linked_list_adt.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/056_custom_linked_list_adt.aura): Representación funcional de listas enlazadas (`Nil` / `Cons`).
- [`057_command_action_dispatcher.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/057_command_action_dispatcher.aura): Patrón comando y despacho exhaustivo con `match`.
- [`058_maybe_unwrap_or_default.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/058_maybe_unwrap_or_default.aura): Desempaquetado seguro con valor por defecto.
- [`059_http_status_matcher.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/059_http_status_matcher.aura): Clasificación de códigos de estado HTTP.
- [`060_binary_tree_search_adt.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/060_binary_tree_search_adt.aura): Búsqueda de valores en nodos de árbol binario.

### 7. Estructuras de Datos y Modelado de Dominios (061 - 070)
- [`061_stack_data_structure.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/061_stack_data_structure.aura): Estructura de datos Pila (LIFO) basada en slices.
- [`062_queue_data_structure.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/062_queue_data_structure.aura): Estructura de datos Cola (FIFO).
- [`063_point2d_euclidean_distance.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/063_point2d_euclidean_distance.aura): Estructura de punto cartesiano 2D y distancia euclidiana al cuadrado.
- [`064_rectangle_methods.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/064_rectangle_methods.aura): Cálculo de área y perímetro de rectángulos.
- [`065_bank_account_ledger.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/065_bank_account_ledger.aura): Modelo bancario de cuenta con depósitos y retiros inmutables.
- [`066_inventory_product_tracker.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/066_inventory_product_tracker.aura): Registro de productos e inventario.
- [`067_student_grade_calculator.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/067_student_grade_calculator.aura): Cálculo del promedio de calificaciones de alumnos.
- [`068_matrix2x2_determinant.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/068_matrix2x2_determinant.aura): Determinante de matrices $2 \times 2$.
- [`069_timestamp_span.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/069_timestamp_span.aura): Cálculo de lapsos de tiempo y duraciones en milisegundos.
- [`070_priority_task_struct.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/070_priority_task_struct.aura): Tareas con prioridades y marcado de criticidad.

### 8. Concurrencia CSP, Fibras y Sincronización (071 - 080)
- [`071_spawn_fiber_concurrency.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/071_spawn_fiber_concurrency.aura): Ejecución concurrente ligera con `spawn` sobre el scheduler M:N.
- [`072_csp_unbuffered_channel.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/072_csp_unbuffered_channel.aura): Paso de mensajes entre fibras mediante canales CSP (`ch <- val`, `<-ch`).
- [`073_channel_fan_in.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/073_channel_fan_in.aura): Patrón Fan-In con múltiples productores concurrentes.
- [`074_waitgroup_synchronization.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/074_waitgroup_synchronization.aura): Coordinación de fibras con `WaitGroup` (`add`, `done`, `wait`).
- [`075_mutex_mutual_exclusion.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/075_mutex_mutual_exclusion.aura): Exclusión mutua para secciones críticas con `Mutex`.
- [`076_select_multiplexing.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/076_select_multiplexing.aura): Multiplexación no bloqueante con `select` y cláusula `default`.
- [`077_defer_cleanup_lifo.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/077_defer_cleanup_lifo.aura): Limpieza garantizada de recursos con `defer` (orden LIFO).
- [`078_producer_consumer_pipeline.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/078_producer_consumer_pipeline.aura): Tubería productor-consumidor clásica desacoplada por canal.
- [`079_worker_pool_dispatch.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/079_worker_pool_dispatch.aura): Despacho ordenado de tareas hacia worker fibers.
- [`080_channel_ping_pong.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/080_channel_ping_pong.aura): Patrón ping-pong de sincronización bidireccional entre canales.

### 9. Algoritmos Avanzados y Programación Dinámica (081 - 090)
- [`081_fibonacci_memoization.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/081_fibonacci_memoization.aura): Programación dinámica top-down con tabla de memorización.
- [`082_coin_change_greedy.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/082_coin_change_greedy.aura): Problema del cambio de monedas usando algoritmo codicioso (greedy).
- [`083_kadane_maximum_subarray.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/083_kadane_maximum_subarray.aura): Algoritmo de Kadane para suma máxima de subarreglo contiguo en $O(n)$.
- [`084_dutch_national_flag_sort.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/084_dutch_national_flag_sort.aura): Algoritmo de partición en 3 vías (Dutch National Flag) en $O(n)$.
- [`085_quick_select_kth_smallest.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/085_quick_select_kth_smallest.aura): Selección del mínimo elemento en colecciones numéricas.
- [`086_graph_adjacency_matrix.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/086_graph_adjacency_matrix.aura): Consulta de adyacencia de grafos mediante matriz aplanada.
- [`087_breadth_first_traversal_simulation.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/087_breadth_first_traversal_simulation.aura): Recorrido en anchura (BFS) simulado sobre nodos de red.
- [`088_levenshtein_distance_simple.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/088_levenshtein_distance_simple.aura): Contador de diferencias posicionales entre cadenas.
- [`089_matrix_multiplication_2x2.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/089_matrix_multiplication_2x2.aura): Multiplicación de matrices algebraicas cuadradas $2 \times 2$.
- [`090_pascal_triangle_row.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/090_pascal_triangle_row.aura): Generador combinatorio de filas del Triángulo de Pascal.

### 10. Representación Binaria, Sistemas y Rendimiento (091 - 100)
- [`091_bitwise_and_or_xor.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/091_bitwise_and_or_xor.aura): Conversión y extracción de cadena binaria de un entero.
- [`092_count_set_bits_hamming.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/092_count_set_bits_hamming.aura): Conteo del peso de Hamming (bits activos en representación binaria).
- [`093_is_power_of_two.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/093_is_power_of_two.aura): Verificación algorítmica de potencias exactas de dos.
- [`094_swap_integers_xor.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/094_swap_integers_xor.aura): Intercambio aritmético de valores in-place sin variable temporal.
- [`095_bloom_filter_hash_simulation.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/095_bloom_filter_hash_simulation.aura): Filtro de pertenencia a conjuntos discretos.
- [`096_hex_byte_formatting.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/096_hex_byte_formatting.aura): Formateo de dígitos hexadecimales a partir de enteros.
- [`097_rate_limiter_token_bucket.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/097_rate_limiter_token_bucket.aura): Algoritmo de control de flujo Token Bucket para APIs.
- [`098_lru_cache_access_list.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/098_lru_cache_access_list.aura): Rastreo y actualización del orden de acceso LRU en slices.
- [`099_json_serializer_model.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/099_json_serializer_model.aura): Serialización JSON nativa de DTOs y modelos tipados.
- [`100_standalone_system_benchmark.aura`](file:///Users/mavro/Projects/aura-lang/examples/algorithms_and_functions/100_standalone_system_benchmark.aura): Microbenchmark y medición de tiempos del sistema con `time.now()`.

---

## Cómo ejecutar o compilar cualquier ejemplo

Para correr cualquier ejemplo directamente:
```bash
cargo run --bin aura -- run examples/algorithms_and_functions/024_binary_search.aura
```

Para verificar tipos estáticamente:
```bash
cargo run --bin aura -- check examples/algorithms_and_functions/074_waitgroup_synchronization.aura
```

Para compilarlo a un binario nativo independiente con cero dependencias externas:
```bash
cargo run --bin aura -- build examples/algorithms_and_functions/083_kadane_maximum_subarray.aura -o kadane_bin
./kadane_bin
```
