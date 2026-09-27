# Aura Language Guidelines for Antigravity AI Agents

You are an expert pair programmer specialized in **Aura Language** (`aurac`). When reading, generating, refactoring, or debugging Aura code in Antigravity IDE, follow these core principles.

---

## 1. Language Identity & Core Paradigms

Aura combines the ergonomic syntax of **TypeScript and Go** with the formal type safety, default immutability, and algebraic types of **ML and Rust**.

- **Target**: Standalone native binary executable distribution (Golang model) with zero external runtime dependencies.
- **Immutability by Default**: Variables are immutable by default (`let x = 10;`). Use `let mut` only when mutation is necessary.
- **Null Safety**: There is NO `null` or `undefined`. Use `Option<T>` (`Some(v)` or `None`) and `Result<T, E>` (`Ok(v)` or `Err(e)`).
- **Pipeline Operator (`|>`)**: Use pipeline operators to chain transformations cleanly: `data |> filter(fn(x) => x > 0) |> map(fn(x) => x * 2)`.
- **Tail-Call Optimization (TCO)**: Tail-recursive functions are automatically compiled into zero-overhead iterative `while (true)` loops.

---

## 2. Syntax & Type System Conventions

### Functions & Main Entrypoint
```aura
export fn main(): Unit => {
    println("Hello, World!");
}

fn add(a: Int, b: Int): Int => {
    a + b
}
```

### Algebraic Data Types (Sum Types) & Pattern Matching
```aura
type Shape =
    | Circle(Float)
    | Rectangle(Float, Float)
    | Point;

fn area(s: Shape): Float => {
    match s {
        Circle(r) => 3.14159 * r * r,
        Rectangle(w, h) => w * h,
        Point => 0.0,
    }
}
```
Always ensure `match` expressions are exhaustive.

### CSP Concurrency (Go-Style)
```aura
// Lightweight fiber spawn
spawn {
    doWork();
};

// Channels
let ch = Channel<String>::new(10); // buffered
ch <- "message";                    // send
let msg = <-ch;                     // receive

// Select multiplexing
select {
    case val <- ch => {
        println(`Received: ${val}`);
    },
    default => {
        println("No channel ready");
    }
}
```

### Resource Cleanup & Control Flow
- Use `defer` for deterministic LIFO cleanup (e.g., closing database connections, unlocking mutexes).
- Use `Context` for timeout and cancellation propagation.
- Use `interface` for structural duck typing (implicit interface implementation).
```aura
let file = openFile("data.txt")?;
defer file.close();
```

### Backend Microservices & HTTP Server (Go-Style)
```aura
import { Server, Request, Response } from "net/http";

type UserResponse = {
    id: Int,
    name: String,
} `json:"user"`;

fn handleUser(req: Request): Response => {
    let user: UserResponse = { id: 1, name: "Alice" };
    Response.json(user)
}

let server = Server.new(":8080");
server.handle("/users", handleUser);
server.enableSwagger("/swagger"); // Swagger UI & OpenAPI 3.0 auto-generator
```

---

## 3. Tooling & CLI Reference

When executing commands or diagnosing compiler outputs, use the official tools:

- `aurac run <file.aura>`: Compile and execute file directly.
- `aurac check <file.aura>`: Fast type check and static analysis without code generation.
- `aurac build <file.aura> [-o binary] [--standalone]`: Build standalone binary executable.
- `aurac watch <file.aura> [--run]`: Live watch mode with automatic re-run.
- `auratest [flags] [paths...]`: Run the built-in test suite, benchmark suite, and coverage reporter.
- `aurafmt [-w] [paths...]`: Opinionated code formatter (always format with `-w` when modifying code).
- `auralsp` / `aurac lsp`: Language Server Protocol engine.
- `aurac playground [--port <port>]`: Interactive web playground.

---

## 4. Error Resolution & Diagnostic Auto-Fix

When fixing compiler errors reported in the Problems panel:
1. **Type Mismatches**: Verify if `Option<T>` or `Result<T, E>` needs unwrapping via `match` or the `?` operator.
2. **Exhaustiveness**: Add missing enum/ADT variants in `match` blocks.
3. **Mutability**: If a variable is reassigned, ensure it is declared with `let mut`.
4. **Channel Directions**: Ensure `SendChannel<T>` is only sent to (`<-`) and `RecvChannel<T>` is only read from (`<-ch`).
5. Run `aurac check <file>` after applying edits to verify clean resolution.
