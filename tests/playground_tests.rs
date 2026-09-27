use aura_lang::playground::{
    PLAYGROUND_HTML, handle_ast_api, handle_compile_api, handle_format_api, handle_hover_api,
};

#[test]
fn test_playground_html_embedded_and_valid() {
    assert!(!PLAYGROUND_HTML.is_empty());
    assert!(PLAYGROUND_HTML.contains("Aura Language Online Playground"));
    assert!(PLAYGROUND_HTML.contains("code-input"));
    assert!(PLAYGROUND_HTML.contains("console-output"));
    assert!(PLAYGROUND_HTML.contains("Pattern Matching"));
    assert!(PLAYGROUND_HTML.contains("CSP Concurrency"));
}

#[test]
fn test_playground_compile_api_success_and_error() {
    let valid_code = r#"
export fn greet(name: String): String {
    return `Hello, ${name}!`;
}
"#;
    let res = handle_compile_api(valid_code);
    assert!(res.contains(r#""success":true"#));
    assert!(res.contains("export function greet"));
    assert!(res.contains("export declare function greet"));

    let invalid_code = "fn broken() ->";
    let err_res = handle_compile_api(invalid_code);
    assert!(err_res.contains(r#""success":false"#));
    assert!(err_res.contains("error"));
}

#[test]
fn test_playground_format_and_ast_api() {
    let code = "fn   add ( a : Int , b : Int ) : Int => a + b ;";
    let fmt_res = handle_format_api(code);
    assert!(fmt_res.contains(r#""success":true"#));
    assert!(fmt_res.contains("formatted"));

    let ast_res = handle_ast_api("let x = 42;");
    assert!(ast_res.contains(r#""success":true"#));
    assert!(ast_res.contains("ast"));
}

#[test]
fn test_playground_hover_api() {
    let code = r#"
/// Doubles an integer
fn double(x: Int): Int => x * 2;
"#;
    let hover_res = handle_hover_api(code, 2, 5);
    assert!(hover_res.contains(r#""success":true"#));
    assert!(hover_res.contains("double"));
}
