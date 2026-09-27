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
        Ok(res) => {
            format!(
                r#"{{"success":true,"js":"{}","dts":"{}"}}"#,
                escape_json_str(&res.js_code),
                escape_json_str(&res.dts_code)
            )
        }
        Err(e) => {
            format!(r#"{{"success":false,"error":"{}"}}"#, escape_json_str(&e))
        }
    }
}

pub fn handle_format_api(source: &str) -> String {
    match format_aura(source) {
        Ok(formatted) => {
            format!(
                r#"{{"success":true,"formatted":"{}"}}"#,
                escape_json_str(&formatted)
            )
        }

        Err(e) => {
            format!(
                r#"{{"success":false,"error":"{}"}}"#,
                escape_json_str(&e.to_string())
            )
        }
    }
}

pub fn handle_ast_api(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => return format!(r#"{{"success":false,"error":"{}"}}"#, escape_json_str(&e)),
    };

    let mut parser = Parser::new(tokens);
    match parser.parse_module() {
        Ok(module) => {
            let ast_debug = format!("{:#?}", module);
            format!(
                r#"{{"success":true,"ast":"{}"}}"#,
                escape_json_str(&ast_debug)
            )
        }
        Err(e) => {
            format!(r#"{{"success":false,"error":"{}"}}"#, escape_json_str(&e))
        }
    }
}

pub fn handle_hover_api(source: &str, line: u32, column: u32) -> String {
    let doc = DocumentState::new("playground.aura".to_string(), source.to_string(), 1);
    let pos = LspPosition {
        line,
        character: column,
    };

    if let Some(hover) = doc.hover(&pos) {
        format!(
            r#"{{"success":true,"hover":"{}"}}"#,
            escape_json_str(&hover.contents)
        )
    } else {
        r#"{"success":true,"hover":null}"#.to_string()
    }
}

fn send_html_response(stream: &mut TcpStream, html: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-src 'self' blob:;\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: SAMEORIGIN\r\nReferrer-Policy: no-referrer\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.as_bytes().len(),
        html
    );
    let _ = stream.write_all(response.as_bytes());
}

fn send_json_response(stream: &mut TcpStream, json: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nX-Content-Type-Options: nosniff\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        json.as_bytes().len(),
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
    if let Some(code) = extract_json_str(body, "code") {
        return code;
    }
    body.to_string()
}

fn extract_int_param(body: &str, field: &str) -> Option<i64> {
    let pattern = format!("\"{}\"", field);
    let field_pos = body.find(&pattern)?;
    let after_field = &body[field_pos + pattern.len()..];
    let colon_pos = after_field.find(':')?;
    let after_colon = &after_field[colon_pos + 1..].trim_start();
    let num_str: String = after_colon
        .chars()
        .take_while(|c| c.is_digit(10) || *c == '-')
        .collect();
    num_str.parse::<i64>().ok()
}

fn extract_json_str(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\"", field);
    let field_pos = json.find(&pattern)?;
    let after_field = &json[field_pos + pattern.len()..];
    let colon_pos = after_field.find(':')?;
    let after_colon = &after_field[colon_pos + 1..].trim_start();

    if after_colon.starts_with('"') {
        let mut s = String::new();
        let mut escaped = false;
        for c in after_colon[1..].chars() {
            if escaped {
                match c {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    _ => s.push(c),
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                return Some(s);
            } else {
                s.push(c);
            }
        }
    }
    None
}

fn escape_json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
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
