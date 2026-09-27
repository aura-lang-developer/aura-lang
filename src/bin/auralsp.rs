//! Standalone binary for Aura Language Server Protocol (auralsp)

use aura_lang::lsp::LspServer;

fn main() {
    let mut server = LspServer::new();
    if let Err(e) = server.run_stdio() {
        eprintln!("Aura LSP Server encountered an error: {}", e);
        std::process::exit(1);
    }
}
