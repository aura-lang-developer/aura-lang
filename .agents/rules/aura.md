# Aura Workspace Rules for Antigravity AI

When assisting with Aura Language files (`*.aura`) and compiler workflows in this repository:
1. Follow Aura syntax and idiomatic functional/CSP programming (immutability by default, ADTs, pattern matching, channels/fibers).
2. When creating or refactoring Aura code, run `aurac check <file>` or `auratest` to verify sound typing.
3. Automatically format modified `.aura` files using `aurafmt -w <file>`.
4. Respect existing error models: always use `Option<T>` or `Result<T, E>` and handle branches exhaustively with `match`.
