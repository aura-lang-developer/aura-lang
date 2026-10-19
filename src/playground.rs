//! Aura Online Playground Web Server & Interactive Runner
//!
//! Provides an embedded web-based IDE and API server for compiling, formatting,
//! inspecting ASTs, simulating hover info, and executing Aura programs live.

use crate::compile;
use crate::formatter::format_aura;
use crate::lexer::Lexer;

use crate::lsp::{DocumentState, LspPosition};
use crate::parser::Parser;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub const DEFAULT_PORT: u16 = 3000;

/// Embedded single-page web app for the Aura Online Playground.
pub const PLAYGROUND_HTML: &str = include_str!("../playground/index.html");

/// Starts the Aura Playground HTTP Server.
pub fn start_server(port: u16) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);
    let listener =
        TcpListener::bind(&addr).map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 ⚡ Aura Online Playground ⚡                 ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  ➜ Local URL: http://{}                           ║",
        addr
    );
    println!("║  ➜ Interactive Editor, Live ES6 & DTS, AST Viewer & Console  ║");
    println!("║  ➜ Press Ctrl+C to stop the server                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));

    let mut buffer = [0u8; 65536];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = request.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    if method == "OPTIONS" {
        send_cors_response(&mut stream);
        return;
    }

    // Extract body if POST
    let body = if method == "POST" {
        if let Some(pos) = request.find("\r\n\r\n") {
            &request[pos + 4..]
        } else {
            ""
        }
    } else {
        ""
    };

    match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => {
            send_html_response(&mut stream, PLAYGROUND_HTML);
        }
        ("POST", "/api/compile") => {
            let source = extract_code_param(body);
            let resp = handle_compile_api(&source);
            send_json_response(&mut stream, &resp);
        }
        ("POST", "/api/format") => {
            let source = extract_code_param(body);
            let resp = handle_format_api(&source);
            send_json_response(&mut stream, &resp);
        }
        ("POST", "/api/ast") => {
            let source = extract_code_param(body);
            let resp = handle_ast_api(&source);
            send_json_response(&mut stream, &resp);
        }
        ("POST", "/api/hover") => {
            let source = extract_code_param(body);
            let line = extract_int_param(body, "line").unwrap_or(0) as u32;
            let col = extract_int_param(body, "column").unwrap_or(0) as u32;
            let resp = handle_hover_api(&source, line, col);
            send_json_response(&mut stream, &resp);
        }
        _ => {
            send_404_response(&mut stream);
        }
    }
}

pub fn handle_compile_api(source: &str) -> String {
    match compile(source, &[]) {
        Ok(res) => serde_json::json!({
            "success": true,
            "js": res.js_code,
            "dts": res.dts_code,
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "success": false,
            "error": e,
        })
        .to_string(),
    }
}

pub fn handle_format_api(source: &str) -> String {
    match format_aura(source) {
        Ok(formatted) => serde_json::json!({
            "success": true,
            "formatted": formatted,
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })
        .to_string(),
    }
}

pub fn handle_ast_api(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return serde_json::json!({
                "success": false,
                "error": e,
            })
            .to_string();
        }
    };

    let mut parser = Parser::new(tokens);
    match parser.parse_module() {
        Ok(module) => {
            let ast_debug = format!("{:#?}", module);
            serde_json::json!({
                "success": true,
                "ast": ast_debug,
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "success": false,
            "error": e,
        })
        .to_string(),
    }
}

pub fn handle_hover_api(source: &str, line: u32, column: u32) -> String {
    let doc = DocumentState::new("playground.aura".to_string(), source.to_string(), 1);
    let pos = LspPosition {
        line,
        character: column,
    };

    if let Some(hover) = doc.hover(&pos) {
        serde_json::json!({
            "success": true,
            "hover": hover.contents,
        })
        .to_string()
    } else {
        serde_json::json!({
            "success": true,
            "hover": serde_json::Value::Null,
        })
        .to_string()
    }
}

fn send_html_response(stream: &mut TcpStream, html: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-src 'self' blob:;\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: SAMEORIGIN\r\nReferrer-Policy: no-referrer\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes());
}

fn send_json_response(stream: &mut TcpStream, json: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nX-Content-Type-Options: nosniff\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        json.len(),
        json
    );
    let _ = stream.write_all(response.as_bytes());
}

fn send_cors_response(stream: &mut TcpStream) {
    let response = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n";
    let _ = stream.write_all(response.as_bytes());
}

fn send_404_response(stream: &mut TcpStream) {
    let body = "404 Not Found";
    let response = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn extract_code_param(body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body)
        && let Some(code) = v.get("code").and_then(|c| c.as_str())
    {
        return code.to_string();
    }
    body.to_string()
}

fn extract_int_param(body: &str, field: &str) -> Option<i64> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body)
        && let Some(num) = v.get(field).and_then(|n| n.as_i64())
    {
        return Some(num);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playground_compile_and_ast_api() {
        let code = r#"
export fn square(n: Int): Int => n * n;
"#;
        let compile_res = handle_compile_api(code);
        assert!(compile_res.contains(r#""success":true"#));
        assert!(compile_res.contains("function square"));

        let ast_res = handle_ast_api(code);
        assert!(ast_res.contains(r#""success":true"#));
        assert!(ast_res.contains("FunctionDecl"));

        let format_res = handle_format_api("fn   foo ( x : Int ) : Int => x ;");
        assert!(format_res.contains(r#""success":true"#));

        let hover_res = handle_hover_api(code, 1, 10);
        assert!(hover_res.contains(r#""success":true"#));
    }
}
