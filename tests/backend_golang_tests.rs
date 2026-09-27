//! Integration and unit tests for Aura Backend & Standalone Executable Generation (Golang Model).

use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use aura_lang::{build_backend_executable, compile_backend, compile_to_go};
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_backend_mode_rejects_react_components() {
    let source_component = r#"
        export component Button(label: String) => {
            return <button>{label}</button>;
        }
    "#;

    let mut lexer = Lexer::new(source_component);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    let parse_res = parser.parse_module();
    assert!(
        parse_res.is_err(),
        "Frontend 'component' keyword must be rejected as invalid syntax in backend-only Aura"
    );
}

#[test]
fn test_backend_mode_rejects_jsx_in_functions() {
    let source_jsx = r#"
        export fn renderCard(): Any => {
            <div className="card">Hello</div>
        }
    "#;

    let res = compile_backend(source_jsx, None, &[]);
    assert!(
        res.is_err(),
        "Frontend JSX syntax must be rejected as invalid syntax in backend-only Aura"
    );
}

#[test]
fn test_backend_os_and_time_typecheck_and_codegen() {
    let source = r#"
        export fn main(): Unit => {
            let args = os.args;
            let host = os.hostname();
            let port = os.env("PORT");
            os.setEnv("AURA_ENV", "production");
            let nowMs = time.now();
            println(`Host: ${host}, Port: ${port}, Time: ${nowMs}`);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Backend compilation should succeed");
    assert!(res.js_code.contains("export const os = Object.freeze({"));
    assert!(res.js_code.contains("export const time = Object.freeze({"));
    assert!(res.dts_code.contains("export declare const os:"));
    assert!(res.dts_code.contains("export declare const time:"));
}

#[test]
fn test_backend_os_e2e_execution_with_node() {
    let source = r#"
        export fn main(): Unit => {
            os.setEnv("AURA_TEST_VAR", "SUCCESS_42");
            let v = os.getEnv("AURA_TEST_VAR");
            assert(v == "SUCCESS_42", "Environment variable match");

            let tmpPath = "aura_test_os_file.tmp.txt";
            os.writeFile(tmpPath, "Aura Backend Content");
            let content = os.readFile(tmpPath);
            assert(content == "Aura Backend Content", "File content match");

            println("AURA_OS_E2E_OK");
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nmain();\n");

    let tmp_file = "test_os_exec.tmp.mjs";
    fs::write(tmp_file, &js_runner).expect("Failed to write runner");

    let output = Command::new("node")
        .arg(tmp_file)
        .output()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    let _ = fs::remove_file("aura_test_os_file.tmp.txt");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("AURA_OS_E2E_OK"),
        "Execution failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_compile_to_go_code_generation() {
    let source = r#"
        fn add(a: Int, b: Int): Int => a + b;

        export fn main(): Unit => {
            let x = add(10, 20);
            println(`Sum is: ${x}`);
        }
    "#;

    let go_code = compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("package main"));
    assert!(go_code.contains("func add(a int64, b int64) int64"));
    assert!(go_code.contains("func main()"));
    assert!(go_code.contains("fmt.Sprintf"));
}

#[test]
fn test_build_backend_standalone_executable_binary() {
    let source = r#"
        export fn main(): Unit => {
            println("AURA_STANDALONE_BINARY_SUCCESS");
        }
    "#;

    let binary_name = "test_aura_standalone_binary";
    let binary_path = Path::new(binary_name);

    let build_res =
        build_backend_executable(source, binary_path).expect("Building standalone binary failed");

    assert!(binary_path.exists(), "Binary file must exist on disk");
    assert!(build_res.size_bytes > 0, "Binary size must be > 0");

    // Execute generated binary
    let exec_output = Command::new(format!("./{}", binary_name))
        .output()
        .expect("Failed to run binary");

    let _ = fs::remove_file(binary_path);

    let stdout = String::from_utf8_lossy(&exec_output.stdout);
    assert!(
        stdout.contains("AURA_STANDALONE_BINARY_SUCCESS"),
        "Binary execution failed: {}",
        stdout
    );
}

#[test]
fn test_struct_declaration_and_json_tags() {
    let source = r#"
        struct Product {
            id: String `json:"id"`,
            name: String `json:"name" validate:"required"`,
            price: Float `json:"price"`
        };

        type InventoryItem struct {
            sku: String `json:"sku"`,
            stock: Int `json:"stock"`
        };

        export fn main(): Unit => {
            let p: Product = { id: "p1", name: "Keyboard", price: 99.99 };
            let inv: InventoryItem = { sku: "SKU-99", stock: 50 };
            println(`Product: ${p.name}, Stock: ${inv.stock}`);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Struct compilation failed");
    assert!(res.js_code.contains("const p = ({"));
    assert!(res.js_code.contains("const inv = ({"));

    let go_code = compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("type Product struct {"));
    assert!(go_code.contains("Name string `json:\"name\"`"));
    assert!(go_code.contains("type InventoryItem struct {"));
}

#[test]
fn test_microservices_examples_compile_successfully() {
    let microservices = [
        "examples/microservices/orders_service.aura",
        "examples/microservices/auth_service.aura",
        "examples/microservices/telemetry_service.aura",
    ];

    for path_str in &microservices {
        let content = fs::read_to_string(path_str)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path_str, e));
        let res = compile_backend(&content, Some(Path::new(path_str)), &[])
            .unwrap_or_else(|e| panic!("Failed to compile microservice {}: {}", path_str, e));
        assert!(
            !res.js_code.is_empty(),
            "Generated JS should not be empty for {}",
            path_str
        );
    }
}
