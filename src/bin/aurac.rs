//! Aura Language Compiler Alias (`aurac`).
//! Companion/backward-compatible CLI entrypoint for `aurac`.

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    aura_lang::cli::run_cli(&args);
}
