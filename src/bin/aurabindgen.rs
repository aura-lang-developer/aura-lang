//! Standalone CLI executable for Aura Bindgen (`aurabindgen`).
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    aura_lang::bindgen::run_cli(&args);
}
