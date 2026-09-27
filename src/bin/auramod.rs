//! Aura Package & Module Manager CLI (auramod)

use aura_lang::package::run_cli;
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    run_cli(&args);
}
