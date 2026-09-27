use aura_lang::compile;
use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use aura_lang::typechecker::TypeChecker;
use std::process::Command;

#[test]
fn test_parse_and_typecheck_defer_statement() {
    let source = r#"
fn testDefer(): Int => {
    let mut x = 10;
    defer x = x + 5;
    defer println("Executing defer 1");
    defer {
        println("Executing block defer");
    };
    return x;
};
"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Tokenize failed");
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().expect("Parse module failed");

    let mut checker = TypeChecker::new();
    checker.check_module(&module).expect("Type check failed");
}

#[test]
fn test_e2e_defer_lifo_execution_with_node() {
    let source = r#"
export fn runDeferTest(): List<String> => {
    let mut logs: List<String> = [];
    defer logs.push("first_defer");
    defer logs.push("second_defer");
    defer logs.push("third_defer");
    
    logs.push("body_execution");
    return logs;
};
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    assert!(result.js_code.contains("__aura_defers"));

    // Execute with Node.js to verify LIFO ordering
    let temp_js_path = "dist/test_defer_e2e.mjs";
    std::fs::create_dir_all("dist").unwrap();
    std::fs::write(temp_js_path, &result.js_code).unwrap();

    let runner_script = format!(
        r#"
import {{ runDeferTest }} from './test_defer_e2e.mjs';
const result = runDeferTest();
console.log("Defer execution order:", JSON.stringify(result));
if (!result || result.length !== 4) {{
    console.error("Expected 4 items, got:", result);
    process.exit(1);
}}
if (result[0] !== 'body_execution' || result[1] !== 'third_defer' || result[2] !== 'second_defer' || result[3] !== 'first_defer') {{
    console.error("LIFO order mismatch:", result);
    process.exit(2);
}}
"#
    );

    let script_path = "dist/run_defer_check.mjs";
    std::fs::write(script_path, runner_script).unwrap();

    let output = Command::new("node")
        .arg(script_path)
        .output()
        .expect("Failed to run node");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!(
            "Node execution failed:\nSTDOUT: {}\nSTDERR: {}",
            stdout, stderr
        );
    }

    assert!(output.status.success());
}
