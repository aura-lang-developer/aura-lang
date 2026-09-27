---
name: aura-workflow
description: Build, type-check, test, format, and run Aura Language projects using aurac, auratest, aurafmt, and auralsp.
---

# Aura Language Developer Workflow

Use this skill when interacting with, compiling, testing, or formatting Aura Language projects (`.aura`).

## Common Tasks

### 1. Type Checking Code
Run fast Hindley-Milner type inference on an Aura file:
```bash
aurac check <path/to/file.aura>
```
If using custom build tags:
```bash
aurac check <path/to/file.aura> --tags "pro,linux"
```

### 2. Running Code
Execute Aura code directly (Golang `go run` style):
```bash
aurac run <path/to/file.aura>
```

### 3. Compiling to Standalone Native Binary
Compile directly to a standalone native binary without external runtime dependencies:
```bash
aurac build <path/to/file.aura> -o <binary_name>
# Or simply:
aurac <path/to/file.aura> -o <binary_name>
```

### 4. Running Unit Tests and Benchmarks
Execute the full test suite with coverage:
```bash
auratest
# Or test specific directory/file:
auratest tests/
```

### 5. Formatting Code
Format `.aura` files idempotently:
```bash
# Overwrite files with formatted output
aurafmt -w <path/to/file.aura>
# Or check if files are formatted:
aurafmt -c src/
```

### 6. Starting Watch Mode
Start interactive watch mode with re-execution on save:
```bash
aurac watch <path/to/file.aura> --run
```
