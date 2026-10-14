//! Standalone binary for Aura Language Server Protocol (auralsp)

use aura_lang::lsp::LspServer;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V" || a == "version") {
        println!("auralsp {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Aura Language Server Protocol (auralsp) v{}",
            env!("CARGO_PKG_VERSION")
        );
        println!("Usage: auralsp [--stdio]");
        return;
    }

    let mut server = LspServer::new();
    if let Err(e) = server.run_stdio() {
        eprintln!("Aura LSP Server encountered an error: {}", e);
        std::process::exit(1);
    }
}
