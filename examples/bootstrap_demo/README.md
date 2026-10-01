# 🌟 Aura Self-Hosted Compiler Demo

This example demonstrates the **Bootstrapping / Self-Hosting** capability of the Aura language.

## Files
- `hello.aura`: Sample Aura program using pure functions, string interpolation, `while` loops, and native types.
- `hello.js`: ES6 JavaScript code generated directly by the Aura compiler written in Aura (`dist/aurac.mjs`).

## How to Run

1. **Compile with the Self-Hosted Compiler:**
   ```bash
   node ../../dist/aurac.mjs hello.aura -o hello.js
   ```

2. **Run the resulting JavaScript program:**
   ```bash
   node hello.js
   ```

3. **Expected Output:**
   ```text
   Hello Developer from Self-Hosted Aura Compiler!
   Sum of 1..100: 5050
   ```
