use aura_lang::lsp::{DiagnosticSeverity, DocumentState, LspPosition, LspServer};

#[test]
fn test_lsp_hover_function_and_docs() {
    let source = r#"
/// Calculates the factorial of an integer
export fn factorial(n: Int): Int {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

let ans = factorial(5);
"#;

    let doc = DocumentState::new("file:///math.aura".to_string(), source.to_string(), 1);
    assert_eq!(doc.diagnostics.len(), 0);

    // Hover over 'factorial' definition (char 14)
    let hover_fn = doc.hover(&LspPosition {
        line: 2,
        character: 14,
    });
    assert!(hover_fn.is_some());
    let content = hover_fn.unwrap().contents;
    assert!(content.contains("fn factorial(n: Int) -> Int"));
    assert!(content.contains("Calculates the factorial of an integer"));

    // Hover over keyword 'fn' (char 8)
    let hover_kw = doc.hover(&LspPosition {
        line: 2,
        character: 8,
    });
    assert!(hover_kw.is_some());
    assert!(hover_kw.unwrap().contents.contains("`fn` keyword"));

    // Hover over primitive type 'Int' (char 25)
    let hover_ty = doc.hover(&LspPosition {
        line: 2,
        character: 25,
    });
    assert!(hover_ty.is_some());
    assert!(hover_ty.unwrap().contents.contains("64-bit signed integer"));
}

#[test]
fn test_lsp_go_to_definition_local_and_functions() {
    let source = r#"
fn compute_total(price: Float, tax_rate: Float): Float {
    let subtotal = price * (1.0 + tax_rate);
    return subtotal;
}

let my_price = 100.0;
let final_total = compute_total(my_price, 0.15);
"#;

    let doc = DocumentState::new("file:///billing.aura".to_string(), source.to_string(), 1);
    assert_eq!(doc.diagnostics.len(), 0);

    // Jump to definition of compute_total from call site (line 7, col 20)
    let def_fn = doc.definition(&LspPosition {
        line: 7,
        character: 20,
    });
    assert!(def_fn.is_some());
    let loc = def_fn.unwrap();
    assert_eq!(loc.uri, "file:///billing.aura");
    assert_eq!(loc.range.start.line, 1);

    // Jump to definition of my_price from call site (line 7, col 35)
    let def_var = doc.definition(&LspPosition {
        line: 7,
        character: 35,
    });
    assert!(def_var.is_some());
    let loc_var = def_var.unwrap();
    assert_eq!(loc_var.range.start.line, 6);
}

#[test]
fn test_lsp_sum_types_and_functions_hover_and_symbols() {
    let source = r#"
type Status = 
    | Active
    | Pending(Int)
    | Suspended(String);

fn checkStatus(status: Status): Bool => {
    match status {
        Active => true,
        _ => false,
    }
}
"#;

    let doc = DocumentState::new("file:///status.aura".to_string(), source.to_string(), 1);
    assert_eq!(doc.diagnostics.len(), 0);

    // Document symbols
    assert!(doc.symbols.iter().any(|s| s.name == "Status"));
    assert!(doc.symbols.iter().any(|s| s.name == "checkStatus"));

    // Hover over function
    let hover_fn = doc.hover(&LspPosition {
        line: 6,
        character: 7,
    });
    assert!(hover_fn.is_some());
    assert!(hover_fn.unwrap().contents.contains("fn checkStatus"));
}

#[test]
fn test_lsp_diagnostics_reporting() {
    // Syntax error test
    let bad_syntax = "fn broken_fn( { let x = 1; }";
    let doc_syntax = DocumentState::new("file:///bad.aura".to_string(), bad_syntax.to_string(), 1);
    assert!(!doc_syntax.diagnostics.is_empty());
    assert_eq!(
        doc_syntax.diagnostics[0].severity,
        DiagnosticSeverity::Error
    );

    // Type error test
    let bad_type = "fn get_num(): Int => \"not a number\";";
    let doc_type = DocumentState::new("file:///bad_type.aura".to_string(), bad_type.to_string(), 1);
    assert!(!doc_type.diagnostics.is_empty());
    assert_eq!(doc_type.diagnostics[0].severity, DiagnosticSeverity::Error);
}

#[test]
fn test_lsp_completions_and_formatting() {
    let source = r#"
fn multiply(a: Int, b: Int): Int => a * b;
let factor = 10;
"#;

    let doc = DocumentState::new("file:///test.aura".to_string(), source.to_string(), 1);
    let comps = doc.completions(&LspPosition {
        line: 2,
        character: 0,
    });

    assert!(comps.iter().any(|c| c.label == "multiply"));
    assert!(comps.iter().any(|c| c.label == "factor"));
    assert!(comps.iter().any(|c| c.label == "match"));
    assert!(comps.iter().any(|c| c.label == "spawn"));

    // Test formatting
    let edits = doc.format(None).expect("Formatting should succeed");
    // Format shouldn't fail
    assert!(edits.len() <= 1);
}

#[test]
fn test_lsp_server_json_rpc_pipeline() {
    let mut server = LspServer::new();

    // 1. Initialize
    let init_req = r#"{"jsonrpc":"2.0","id":100,"method":"initialize","params":{"processId":1234,"capabilities":{}}}"#;
    let init_res = server
        .handle_message(init_req)
        .expect("Expected initialize response");
    assert!(init_res.contains(r#""id":100"#));
    assert!(init_res.contains("hoverProvider"));
    assert!(init_res.contains("definitionProvider"));

    // 2. didOpen with valid file
    let file_uri = "file:///app/src/index.aura";
    let did_open_req = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","version":1,"text":"export fn add(x: Int, y: Int): Int => x + y;\nlet total = add(1, 2);"}} }}"#,
        file_uri
    );
    let diag_res = server
        .handle_message(&did_open_req)
        .expect("Expected diagnostics notification");
    assert!(diag_res.contains("publishDiagnostics"));

    // 3. Hover query
    let hover_req = format!(
        r#"{{"jsonrpc":"2.0","id":101,"method":"textDocument/hover","params":{{"uri":"{}","line":0,"character":11}}}}"#,
        file_uri
    );
    let hover_res = server
        .handle_message(&hover_req)
        .expect("Expected hover response");
    assert!(hover_res.contains("fn add(x: Int, y: Int) -> Int"));

    // 4. Definition query
    let def_req = format!(
        r#"{{"jsonrpc":"2.0","id":102,"method":"textDocument/definition","params":{{"uri":"{}","line":1,"character":13}}}}"#,
        file_uri
    );
    let def_res = server
        .handle_message(&def_req)
        .expect("Expected definition response");
    assert!(def_res.contains(file_uri));

    // 5. Document symbols query
    let sym_req = format!(
        r#"{{"jsonrpc":"2.0","id":103,"method":"textDocument/documentSymbol","params":{{"uri":"{}"}}}}"#,
        file_uri
    );
    let sym_res = server
        .handle_message(&sym_req)
        .expect("Expected documentSymbol response");
    assert!(sym_res.contains("add"));

    // 6. Formatting query
    let fmt_req = format!(
        r#"{{"jsonrpc":"2.0","id":104,"method":"textDocument/formatting","params":{{"uri":"{}"}}}}"#,
        file_uri
    );
    let fmt_res = server
        .handle_message(&fmt_req)
        .expect("Expected formatting response");
    assert!(fmt_res.contains("newText"));

    // 7. Shutdown
    let shutdown_req = r#"{"jsonrpc":"2.0","id":105,"method":"shutdown","params":{}}"#;
    let shutdown_res = server
        .handle_message(shutdown_req)
        .expect("Expected shutdown response");
    assert!(shutdown_res.contains("null"));
    assert!(server.is_shutdown);
}

#[test]
fn test_lsp_hover_import_and_keywords() {
    let source = r#"
import { Server, Request } from "net/http";

let s = Server.new(":8080");
"#;

    let doc = DocumentState::new("file:///app.aura".to_string(), source.to_string(), 1);
    for d in &doc.diagnostics {
        println!("Diagnostic: {:?}", d);
    }

    // Hover over 'import' (line 1, char 2)
    let hover_imp = doc.hover(&LspPosition {
        line: 1,
        character: 2,
    });
    assert!(hover_imp.is_some());
    assert!(hover_imp.unwrap().contents.contains("`import`"));

    // Hover over 'Server' in import (line 1, char 11)
    let hover_server = doc.hover(&LspPosition {
        line: 1,
        character: 11,
    });
    assert!(hover_server.is_some());
    assert!(hover_server.unwrap().contents.contains("Server"));

    // Hover over 'Server' usage (line 3, char 10)
    let hover_server_use = doc.hover(&LspPosition {
        line: 3,
        character: 10,
    });
    assert!(hover_server_use.is_some());
    assert!(hover_server_use.unwrap().contents.contains("Server"));
}

#[test]
fn test_lsp_hover_operators_and_builtins() {
    let source = r#"
let nums = [1, 2, 3];
let res = nums |> map(fn(x) => x * 2);
let count = len(nums);
println("done");
"#;

    let doc = DocumentState::new("file:///pipeline.aura".to_string(), source.to_string(), 1);

    // Hover over pipeline operator '|>' (line 2, col 16)
    let hover_pipe = doc.hover(&LspPosition {
        line: 2,
        character: 16,
    });
    assert!(hover_pipe.is_some());
    let pipe_contents = hover_pipe.unwrap().contents;
    assert!(pipe_contents.contains("Pipeline Operator"));
    assert!(pipe_contents.contains("Chains transformations cleanly"));

    // Hover over 'map' (line 2, col 19)
    let hover_map = doc.hover(&LspPosition {
        line: 2,
        character: 19,
    });
    assert!(hover_map.is_some());
    let map_contents = hover_map.unwrap().contents;
    assert!(map_contents.contains("Collection & Iterator Transform"));
    assert!(map_contents.contains("Transforms each element"));

    // Hover over 'len' (line 3, col 13)
    let hover_len = doc.hover(&LspPosition {
        line: 3,
        character: 13,
    });
    assert!(hover_len.is_some());
    let len_contents = hover_len.unwrap().contents;
    assert!(len_contents.contains("Built-in Function"));
    assert!(len_contents.contains("len(container: Any): Int"));

    // Hover over 'println' (line 4, col 2)
    let hover_println = doc.hover(&LspPosition {
        line: 4,
        character: 2,
    });
    assert!(hover_println.is_some());
    let println_contents = hover_println.unwrap().contents;
    assert!(println_contents.contains("println(value: Any): Unit"));
}

fn escape_test_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[test]
fn test_lsp_find_references() {
    let source = r#"
fn add(a: Int, b: Int): Int => a + b;

let x = add(1, 2);
let y = add(3, 4);
"#;
    let mut server = LspServer::new();
    let uri = "file:///math.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    // Find references for 'add' with includeDeclaration = true (line 1, col 3)
    let ref_req = format!(
        r#"{{"jsonrpc":"2.0","id":10,"method":"textDocument/references","params":{{"uri":"{}","line":1,"character":3,"context":{{"includeDeclaration":true}}}}}}"#,
        uri
    );
    let resp = server
        .handle_message(&ref_req)
        .expect("Expected references response");
    assert!(resp.contains("file:///math.aura"));
    let count = resp.matches("math.aura").count();
    assert_eq!(count, 3);
}

#[test]
fn test_lsp_prepare_rename_and_rename() {
    let source = r#"
fn add(a: Int, b: Int): Int => a + b;

let x = add(1, 2);
"#;
    let mut server = LspServer::new();
    let uri = "file:///calc.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    // 1. Prepare rename on 'add' (line 1, col 3)
    let prep_req = format!(
        r#"{{"jsonrpc":"2.0","id":20,"method":"textDocument/prepareRename","params":{{"uri":"{}","line":1,"character":3}}}}"#,
        uri
    );
    let prep_resp = server
        .handle_message(&prep_req)
        .expect("Expected prepareRename response");
    assert!(prep_resp.contains("placeholder"));
    assert!(prep_resp.contains("add"));

    // 2. Rename 'add' to 'sum'
    let rename_req = format!(
        r#"{{"jsonrpc":"2.0","id":21,"method":"textDocument/rename","params":{{"uri":"{}","line":1,"character":3,"newName":"sum"}}}}"#,
        uri
    );
    let rename_resp = server
        .handle_message(&rename_req)
        .expect("Expected rename response");
    assert!(rename_resp.contains("changes"));
    assert!(rename_resp.contains("sum"));
}

#[test]
fn test_lsp_signature_help() {
    let source = r#"
fn multiply(x: Int, y: Int): Int => x * y;

let r = multiply(10, 20);
"#;
    let mut server = LspServer::new();
    let uri = "file:///sig.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    // Request signature help after comma (line 3, char 21)
    let sig_req = format!(
        r#"{{"jsonrpc":"2.0","id":30,"method":"textDocument/signatureHelp","params":{{"uri":"{}","line":3,"character":21}}}}"#,
        uri
    );
    let sig_resp = server
        .handle_message(&sig_req)
        .expect("Expected signatureHelp response");
    assert!(sig_resp.contains("signatures"));
    assert!(sig_resp.contains("multiply"));
    assert!(sig_resp.contains("activeParameter"));
}

#[test]
fn test_lsp_document_highlight() {
    let source = r#"
let mut count = 0;
count = count + 1;
"#;
    let mut server = LspServer::new();
    let uri = "file:///hl.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    let hl_req = format!(
        r#"{{"jsonrpc":"2.0","id":40,"method":"textDocument/documentHighlight","params":{{"uri":"{}","line":1,"character":8}}}}"#,
        uri
    );
    let hl_resp = server
        .handle_message(&hl_req)
        .expect("Expected documentHighlight response");
    assert!(hl_resp.contains("kind"));
    assert!(hl_resp.contains("\"kind\":3") || hl_resp.contains("\"kind\":2"));
}

#[test]
fn test_lsp_inlay_hints() {
    let source = r#"
let score = 100;
let message = "hello";
"#;
    let mut server = LspServer::new();
    let uri = "file:///inlay.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    let hint_req = format!(
        r#"{{"jsonrpc":"2.0","id":50,"method":"textDocument/inlayHint","params":{{"uri":"{}"}}}}"#,
        uri
    );
    let hint_resp = server
        .handle_message(&hint_req)
        .expect("Expected inlayHint response");
    assert!(hint_resp.contains("label"));
}

#[test]
fn test_lsp_code_actions() {
    let source = r#"
let x = 10;
"#;
    let mut server = LspServer::new();
    let uri = "file:///actions.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    let action_req = format!(
        r#"{{"jsonrpc":"2.0","id":60,"method":"textDocument/codeAction","params":{{"uri":"{}"}}}}"#,
        uri
    );
    let action_resp = server
        .handle_message(&action_req)
        .expect("Expected codeAction response");
    assert!(action_resp.contains("title") || action_resp.contains("[]"));
}

#[test]
fn test_lsp_semantic_tokens_full() {
    let source = r#"
fn greet(name: String): String => `Hello, ${name}`;
let msg = greet("Aura");
"#;
    let mut server = LspServer::new();
    let uri = "file:///tokens.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    let token_req = format!(
        r#"{{"jsonrpc":"2.0","id":70,"method":"textDocument/semanticTokens/full","params":{{"uri":"{}"}}}}"#,
        uri
    );
    let token_resp = server
        .handle_message(&token_req)
        .expect("Expected semanticTokens response");
    assert!(token_resp.contains("data"));
}

#[test]
fn test_lsp_workspace_symbols() {
    let source1 = r#"
export fn computeHash(data: String): String => data;
"#;
    let source2 = r#"
type UserProfile = | Guest | Member(Int);
"#;
    let mut server = LspServer::new();
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"file:///crypto.aura","text":"{}","version":1}}}}"#, escape_test_string(source1))
    );
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"file:///user.aura","text":"{}","version":1}}}}"#, escape_test_string(source2))
    );

    let ws_req =
        r#"{"jsonrpc":"2.0","id":80,"method":"workspace/symbol","params":{"query":"UserProfile"}}"#;
    let ws_resp = server
        .handle_message(ws_req)
        .expect("Expected workspace/symbol response");
    assert!(ws_resp.contains("UserProfile"));
    assert!(ws_resp.contains("file:///user.aura"));
}

#[test]
fn test_lsp_code_lens() {
    let source = r#"
export fn main(): Unit => {
    println("Hello");
}

fn helper(): Int => 42;
"#;
    let mut server = LspServer::new();
    let uri = "file:///main_lens.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    let lens_req = format!(
        r#"{{"jsonrpc":"2.0","id":90,"method":"textDocument/codeLens","params":{{"uri":"{}"}}}}"#,
        uri
    );
    let lens_resp = server
        .handle_message(&lens_req)
        .expect("Expected codeLens response");
    assert!(lens_resp.contains("Run main") || lens_resp.contains("reference"));
}

#[test]
fn test_lsp_folding_ranges() {
    let source = r#"
fn run() {
    let x = 1;
    let y = 2;
}
"#;
    let mut server = LspServer::new();
    let uri = "file:///fold.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    let fold_req = format!(
        r#"{{"jsonrpc":"2.0","id":100,"method":"textDocument/foldingRange","params":{{"uri":"{}"}}}}"#,
        uri
    );
    let fold_resp = server
        .handle_message(&fold_req)
        .expect("Expected foldingRange response");
    assert!(fold_resp.contains("startLine"));
    assert!(fold_resp.contains("endLine"));
}

#[test]
fn test_lsp_call_hierarchy() {
    let source = r#"
fn callee(): Int => 42;

fn caller(): Int {
    return callee();
}
"#;
    let mut server = LspServer::new();
    let uri = "file:///calls.aura";
    server.handle_message(
        &format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"uri":"{}","text":"{}","version":1}}}}"#, uri, escape_test_string(source))
    );

    // 1. Prepare call hierarchy on 'callee' (line 1, col 3)
    let prep_req = format!(
        r#"{{"jsonrpc":"2.0","id":110,"method":"textDocument/prepareCallHierarchy","params":{{"uri":"{}","line":1,"character":3}}}}"#,
        uri
    );
    let prep_resp = server
        .handle_message(&prep_req)
        .expect("Expected prepareCallHierarchy response");
    assert!(prep_resp.contains("callee"));

    // 2. Incoming calls to 'callee'
    let inc_req = format!(
        r#"{{"jsonrpc":"2.0","id":111,"method":"callHierarchy/incomingCalls","params":{{"uri":"{}","name":"callee"}}}}"#,
        uri
    );
    let inc_resp = server
        .handle_message(&inc_req)
        .expect("Expected incomingCalls response");
    assert!(inc_resp.contains("caller"));

    // 3. Outgoing calls from 'caller'
    let out_req = format!(
        r#"{{"jsonrpc":"2.0","id":112,"method":"callHierarchy/outgoingCalls","params":{{"uri":"{}","name":"caller"}}}}"#,
        uri
    );
    let out_resp = server
        .handle_message(&out_req)
        .expect("Expected outgoingCalls response");
    assert!(out_resp.contains("callee"));
}
