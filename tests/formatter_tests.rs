use aura_lang::formatter::{
    FormatConfig, IndentStyle, Language, format_aura, format_js, format_source, format_ts,
    generate_unified_diff,
};

#[test]
fn test_aura_function_and_pipeline_formatting() {
    let unformatted = r#"
module Math.Utils

export fn calculate(x:Int,y:Int):Int=>
x+y*2|>double|>println

fn double(n:Int):Int=>{
let res=n*2;
return res;
}
"#;

    let formatted = format_aura(unformatted).expect("Formatting failed");

    assert!(formatted.contains("export fn calculate(x: Int, y: Int): Int =>"));
    assert!(formatted.contains("x + y * 2 |> double |> println"));
    assert!(formatted.contains("fn double(n: Int): Int => {"));
    assert!(formatted.contains("  let res = n * 2;"));
    assert!(formatted.contains("  return res;"));

    // Idempotency
    let formatted_twice = format_aura(&formatted).expect("Second formatting failed");
    assert_eq!(formatted, formatted_twice);
}

#[test]
fn test_aura_csp_concurrency_formatting() {
    let unformatted = r#"
module Concurrency.Test

export fn worker(ch:Channel<Int>,out:Channel<Int>):Unit=>{
let val=<-ch;
spawn{
out<-val*2;
};
}
"#;

    let formatted = format_aura(unformatted).expect("Formatting failed");
    assert!(formatted.contains("let val = <-ch;"));
    assert!(formatted.contains("spawn {"));
    assert!(formatted.contains("out <- val * 2;"));

    // Idempotency
    let formatted_twice = format_aura(&formatted).expect("Second formatting failed");
    assert_eq!(formatted, formatted_twice);
}

#[test]
fn test_aura_backend_service_formatting() {
    let unformatted = r#"
module Server.Api

export fn handleRequest(path:String,code:Int):Int=>{
let status=code+200;
return status;
}
"#;

    let formatted = format_aura(unformatted).expect("Formatting failed");
    assert!(formatted.contains("export fn handleRequest(path: String, code: Int): Int => {"));
    assert!(formatted.contains("  let status = code + 200;"));
    assert!(formatted.contains("  return status;"));

    let formatted_twice = format_aura(&formatted).expect("Second formatting failed");
    assert_eq!(formatted, formatted_twice);
}

#[test]
fn test_aura_comments_preservation() {
    let unformatted = r#"
// Top-level comment
module App.Main

// Function doc comment
export fn run(): Unit => {
  // Inside block comment
  let x = 10;
  /* Multi-line
     block comment */
  let y = 20;
  x + y
}
"#;

    let formatted = format_aura(unformatted).expect("Formatting failed");
    assert!(formatted.contains("// Top-level comment"));
    assert!(formatted.contains("// Function doc comment"));
    assert!(formatted.contains("// Inside block comment"));
    assert!(formatted.contains("/* Multi-line"));

    let formatted_twice = format_aura(&formatted).expect("Second formatting failed");
    assert_eq!(formatted, formatted_twice);
}

#[test]
fn test_typescript_formatting() {
    let unformatted = r#"
export interface UserProps<T>{
id:number;
name:string;
data:T;
}

export function renderUser<T>(props:UserProps<T>):string{
const formatted=`User: ${props.name} (#${props.id})`;
return formatted;
}
"#;

    let formatted = format_ts(unformatted).expect("TS Formatting failed");
    assert!(formatted.contains("export interface UserProps<T> {"));
    assert!(formatted.contains("  id: number;"));
    assert!(formatted.contains("  name: string;"));
    assert!(formatted.contains("export function renderUser<T>(props: UserProps<T>): string {"));

    let formatted_twice = format_ts(&formatted).expect("Second formatting failed");
    assert_eq!(formatted, formatted_twice);
}

#[test]
fn test_javascript_formatting() {
    let unformatted = r#"
import http from "node:http";

export const createServer=(port,handler)=>{
const server=http.createServer((req,res)=>{
console.log("request received",req.url);
handler(req,res);
});
return server;
};
"#;

    let formatted = format_js(unformatted).expect("JS Formatting failed");
    assert!(formatted.contains("export const createServer = (port, handler) => {"));
    assert!(formatted.contains("  const server = http.createServer((req, res) => {"));
    assert!(formatted.contains("    console.log(\"request received\", req.url);"));
    assert!(formatted.contains("    handler(req, res);"));
    assert!(formatted.contains("  return server;"));

    let formatted_twice = format_js(&formatted).expect("Second formatting failed");
    assert_eq!(formatted, formatted_twice);
}

#[test]
fn test_tab_and_indent_configuration() {
    let code = "fn test(): Unit => {\nlet a = 1;\nlet b = 2;\n}\n";

    // 4 spaces config
    let config_4_spaces = FormatConfig {
        indent_style: IndentStyle::Spaces(4),
        ..FormatConfig::default()
    };
    let fmt_4_spaces = format_source(code, Language::Aura, &config_4_spaces).unwrap();
    assert!(fmt_4_spaces.contains("    let a = 1;"));

    // Tabs config
    let config_tabs = FormatConfig {
        indent_style: IndentStyle::Tabs,
        ..FormatConfig::default()
    };
    let fmt_tabs = format_source(code, Language::Aura, &config_tabs).unwrap();
    assert!(fmt_tabs.contains("\tlet a = 1;"));
}

#[test]
fn test_unified_diff_generation() {
    let original = "fn test():Unit=>{\nlet x=1;\n}\n";
    let formatted = format_aura(original).unwrap();
    let diff = generate_unified_diff("test.aura", original, &formatted);

    assert!(diff.contains("--- test.aura.orig"));
    assert!(diff.contains("+++ test.aura"));
    assert!(diff.contains("-let x=1;"));
    assert!(diff.contains("+  let x = 1;"));
}

#[test]
fn test_language_detection_from_path() {
    assert_eq!(Language::from_path("src/app.aura"), Some(Language::Aura));
    assert_eq!(
        Language::from_path("types/api.d.ts"),
        Some(Language::TypeScript)
    );
    assert_eq!(
        Language::from_path("services/client.ts"),
        Some(Language::TypeScript)
    );
    assert_eq!(Language::from_path("index.js"), Some(Language::JavaScript));
    assert_eq!(
        Language::from_path("bundle.mjs"),
        Some(Language::JavaScript)
    );
    assert_eq!(Language::from_path("Cargo.toml"), None);
}

#[test]
fn test_complex_aura_example_formatting_idempotency() {
    let example_path = "examples/api_service.aura";
    let config = FormatConfig::default();
    let res = aura_lang::format_file(example_path, &config).expect("Failed to format example file");

    // Formatting the formatted version must be 100% identical (idempotent)
    let res_second = aura_lang::format_source(&res.formatted, Language::Aura, &config)
        .expect("Second pass failed");
    assert_eq!(res.formatted, res_second);
}

#[test]
fn test_aura_match_and_sum_types_formatting() {
    let code = r#"
module App.Types

export type Action =
  | Login { username: String, pass: String }
  | Logout
  | UpdateProfile { id: Int, email: String }

export fn reducer(state: State, action: Action): State => {
  match action {
    Login { username, pass } => handleLogin(state, username, pass),
    Logout => initialState,
    UpdateProfile { id, email } => updateEmail(state, id, email)
  }
}
"#;

    let formatted = format_aura(code).expect("Formatting failed");
    assert!(formatted.contains("export type Action ="));
    assert!(formatted.contains("export fn reducer(state: State, action: Action): State => {"));
    assert!(formatted.contains("match action {"));

    let formatted_twice = format_aura(&formatted).expect("Second formatting failed");
    assert_eq!(formatted, formatted_twice);
}
