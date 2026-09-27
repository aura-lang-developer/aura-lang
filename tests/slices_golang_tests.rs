//! Tests for Golang Slice semantics, slicing syntax, []T types, and builtins in Aura.

use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use aura_lang::{build_backend_executable, compile_backend, compile_to_go};
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_slice_syntax_parsing_and_ast() {
    let source = r#"
        export fn main(): Unit => {
            let nums = [10, 20, 30, 40, 50];
            let s1 = nums[1:4];
            let s2 = nums[:3];
            let s3 = nums[2:];
            let s4 = nums[:];
            let s5 = nums[1:3:5];
            let s6 = nums[1..4];
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().expect("Parsing failed");

    assert_eq!(module.items.len(), 1);
}

#[test]
fn test_slice_type_annotation_go_style() {
    let source = r#"
        export fn sumSlice(items: []Int): Int => {
            let mut acc = 0;
            for x in items {
                acc = acc + x;
            }
            return acc;
        }

        export fn main(): Unit => {
            let nums: []Int = [1, 2, 3, 4, 5];
            let total = sumSlice(nums);
            println(`Total: ${total}`);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Compilation failed");
    assert!(res.js_code.contains("sumSlice"));
    assert!(
        res.dts_code
            .contains("sumSlice(items: Array<number>): number")
    );
}

#[test]
fn test_slice_go_codegen_native_expressions() {
    let source = r#"
        export fn processSlice(items: []Int): []Int => {
            return items[1:4];
        }

        export fn testSlices(): Unit => {
            let nums: []Int = [10, 20, 30, 40, 50];
            let sub = nums[1:4];
            let head = nums[:2];
            let tail = nums[3:];
            let all = nums[:];
            let threeIdx = nums[1:3:5];

            let l1 = len(nums);
            let c1 = cap(nums);
            let l2 = nums.len;
            let c2 = nums.cap;

            let appended = append(nums, 60);
            let subMethod = nums.slice(1, 3);
        }
    "#;

    let go_code = compile_to_go(source).expect("Go compilation failed");

    // Verify native Golang slice expressions were generated
    assert!(go_code.contains("nums[1:4]"), "Must contain nums[1:4]");
    assert!(go_code.contains("nums[:2]"), "Must contain nums[:2]");
    assert!(go_code.contains("nums[3:]"), "Must contain nums[3:]");
    assert!(go_code.contains("nums[:]"), "Must contain nums[:]");
    assert!(go_code.contains("nums[1:3:5]"), "Must contain nums[1:3:5]");
    assert!(go_code.contains("len(nums)"), "Must contain len(nums)");
    assert!(go_code.contains("cap(nums)"), "Must contain cap(nums)");
    assert!(
        go_code.contains("append(nums, 60)"),
        "Must contain append(nums, 60)"
    );
    assert!(
        go_code.contains("nums[1:3]"),
        "nums.slice(1,3) must compile to nums[1:3]"
    );
    assert!(
        go_code.contains("items []int64"),
        "[]Int must map to []int64"
    );
    assert!(
        go_code.contains("[]int64{10, 20, 30, 40, 50}"),
        "Slice literal must be typed"
    );
}

#[test]
fn test_slice_e2e_node_execution() {
    let source = r#"
        export fn main(): Unit => {
            let nums = [10, 20, 30, 40, 50];

            let sub = nums[1:4];
            assert(len(sub) == 3, "sub length must be 3");
            assert(sub[0] == 20, "sub[0] must be 20");
            assert(sub[1] == 30, "sub[1] must be 30");
            assert(sub[2] == 40, "sub[2] must be 40");

            let head = nums[:2];
            assert(len(head) == 2, "head length must be 2");
            assert(head[0] == 10 && head[1] == 20, "head elements match");

            let tail = nums[3:];
            assert(len(tail) == 2, "tail length must be 2");
            assert(tail[0] == 40 && tail[1] == 50, "tail elements match");

            let all = nums[:];
            assert(len(all) == 5, "all length must be 5");

            let s = "Hello, Aura Backend!";
            let greeting = s[0:5];
            assert(greeting == "Hello", "string slicing works");

            let appended = append(nums, 60);
            assert(len(appended) == 6, "appended length must be 6");
            assert(appended[5] == 60, "last element is 60");

            println("SLICE_E2E_OK");
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nmain();\n");

    let tmp_file = "test_slice_exec.tmp.mjs";
    fs::write(tmp_file, &js_runner).expect("Failed to write runner");

    let output = Command::new("node")
        .arg(tmp_file)
        .output()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("SLICE_E2E_OK"),
        "Execution failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_slice_e2e_go_native_executable() {
    let source = r#"
        export fn main(): Unit => {
            let nums: []Int = [100, 200, 300, 400, 500];
            let sub = nums[1:4];
            let total = sub[0] + sub[1] + sub[2];
            println(`NATIVE_GO_SLICE_SUM:${total}`);
        }
    "#;

    let binary_name = "test_aura_slice_native_bin";
    let binary_path = Path::new(binary_name);

    let build_res =
        build_backend_executable(source, binary_path).expect("Building slice binary failed");

    assert!(binary_path.exists(), "Binary file must exist on disk");
    assert!(build_res.size_bytes > 0, "Binary size must be > 0");

    let exec_output = Command::new(format!("./{}", binary_name))
        .output()
        .expect("Failed to run binary");

    let _ = fs::remove_file(binary_path);

    let stdout = String::from_utf8_lossy(&exec_output.stdout);
    assert!(
        stdout.contains("NATIVE_GO_SLICE_SUM:900"),
        "Go binary execution failed. Output: {}",
        stdout
    );
}
