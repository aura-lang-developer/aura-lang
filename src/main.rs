//! Aura Language CLI Toolchain (`aura`).
//! Primary CLI entrypoint for the Aura Language unified toolchain.

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    aura_lang::cli::run_cli(&args);
}
