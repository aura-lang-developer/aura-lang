---
name: aura-workflow
description: Build, type-check, test, format, and run Aura Language projects using aura (or aurac), auratest, aurafmt, and auralsp.
---

# Aura Language Developer Workflow

Use this skill when interacting with, compiling, testing, or formatting Aura Language projects (`.aura`). The primary CLI command is `aura` (with `aurac` fully supported as an identical alias).

## Common Tasks

### 1. Type Checking Code
Run fast Hindley-Milner type inference on an Aura file:
```bash
aura check <path/to/file.aura>
```
If using custom build tags:
```bash
aura check <path/to/file.aura> --tags "pro,linux"
```

### 2. Running Code
Execute Aura code directly (Golang `go run` style):
```bash
aura run <path/to/file.aura>
```

### 3. Compiling to Standalone Native Binary
Compile directly to a standalone native binary without external runtime dependencies:
```bash
aura build <path/to/file.aura> -o <binary_name>
# Or simply:
aura <path/to/file.aura> -o <binary_name>
```

### 4. Running Unit Tests and Benchmarks
Execute the full test suite with coverage:
```bash
# Run tests directly with the aura CLI:
aura --test
# Run tests on specific directory/file with compiler:
aura --test tests/
aura tests/math_unit_test.aura --test

# Or using the dedicated runner:
auratest
auratest tests/
```

### 5. Formatting Code
Format `.aura` files idempotently:
```bash
# Overwrite files with formatted output
aura fmt -w <path/to/file.aura>
# Or using the dedicated tool:
aurafmt -w <path/to/file.aura>
# Or check if files are formatted:
aurafmt -c src/
```

### 6. Starting Watch Mode
Start interactive watch mode with re-execution on save:
```bash
aura watch <path/to/file.aura> --run
```
