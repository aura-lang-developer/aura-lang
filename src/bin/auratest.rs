//! Dedicated CLI Entrypoint for `auratest` (Aura Language Test Runner).

use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    aura_lang::testing::run_cli(&args);
}
