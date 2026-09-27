# 🌟 Aura Self-Hosted Compiler Demo

Este ejemplo demuestra la capacidad de **Bootstrapping / Self-Hosting** del lenguaje Aura.

## Archivos
- `hello.aura`: Programa de ejemplo en Aura que utiliza funciones puras, interpolación de cadenas, bucles `while` y tipos nativos.
- `hello.js`: Código JavaScript ES6 generado directamente por el compilador de Aura escrito en Aura (`dist/aurac.mjs`).

## Cómo Ejecutar

1. **Compilar con el Compilador Auto-Alojado:**
   ```bash
   node ../../dist/aurac.mjs hello.aura -o hello.js
   ```

2. **Ejecutar el programa JavaScript resultante:**
   ```bash
   node hello.js
   ```

3. **Salida esperada:**
   ```text
   Hello Developer from Self-Hosted Aura Compiler!
   Sum of 1..100: 5050
   ```
