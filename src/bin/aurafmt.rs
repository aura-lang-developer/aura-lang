//! Aura Code Formatter CLI (aurafmt)

use aura_lang::formatter::run_cli;
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    run_cli(&args);
}
