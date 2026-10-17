//! Comprehensive Tests for Language Performance & Ergonomics Improvements
//!
//! Features tested:
//! 1. Safe Null/Result-coalescing operator (`??`) with short-circuiting on Option and Result
//! 2. Guard statement (`guard <cond> else { <diverging_block> };`) with compile-time divergence check
//! 3. `if let <pattern> = <expr> { ... } else { ... }` pattern matching ergonomics
//! 4. Distinct nominal types (`type UserId = distinct Int;`) with zero runtime overhead
//! 5. Or-Patterns in pattern matching (`1 | 2 | 3 => ...`, `Pending | InReview => ...`)

use aura_lang::{compile, compile_to_go};
use std::fs;
use std::process::Command;

fn run_node(js_code: &str) -> String {
    let tmp_dir = std::env::temp_dir();
    let file_path = tmp_dir.join(format!(
        "aura_test_{}.js",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&file_path, js_code).expect("Failed to write temporary test file");

    let output = Command::new("node")
        .arg(&file_path)
        .output()
        .expect("Failed to execute node process");

    let _ = fs::remove_file(&file_path);

    if !output.status.success() {
        panic!(
            "Node execution failed with status {}:\nSTDOUT:\n{}\nSTDERR:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_null_and_result_coalescing_operator() {
    let source = r#"
        export fn testCoalescing(): Unit => {
            let optSome: Option<Int> = Some(42);
            let optNone: Option<Int> = None;

            let val1 = optSome ?? 0;
            assert(val1 == 42, "optSome ?? 0 should be 42");

            let val2 = optNone ?? 100;
            assert(val2 == 100, "optNone ?? 100 should be 100");

            let resOk: Result<String, String> = Ok("success");
            let resErr: Result<String, String> = Err("failure");

            let s1 = resOk ?? "default";
            assert(s1 == "success", "resOk ?? default should be success");

            let s2 = resErr ?? "fallback";
            assert(s2 == "fallback", "resErr ?? fallback should be fallback");

            // Short-circuiting verification: right operand must not be evaluated when left is Some
            let safeVal = optSome ?? panic("should not be called");
            assert(safeVal == 42, "safeVal should be 42 without panicking");
        }

        export fn main(): Unit => {
            testCoalescing();
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of ?? operator should succeed");
    assert!(res.js_code.contains("__aura_coalesce"));

    // Verify Go backend
    let go_code = compile_to_go(source).expect("Go codegen for ?? operator should succeed");
    assert!(go_code.contains("__auraCoalesce"));

    // E2E execution
    let mut js_runner = res.js_code.clone();
    js_runner.push_str("\n\nmain();\n");
    run_node(&js_runner);
}

#[test]
fn test_guard_statement_execution_and_enforcement() {
    let valid_source = r#"
        export fn processAge(age: Int): String => {
            guard age >= 0 else {
                return "invalid age";
            };

            guard age >= 18 else {
                return "minor";
            };

            return "adult";
        }

        export fn main(): Unit => {
            assert(processAge(-5) == "invalid age", "Negative age should be invalid");
            assert(processAge(10) == "minor", "Age 10 should be minor");
            assert(processAge(25) == "adult", "Age 25 should be adult");
        }
    "#;

    let res = compile(valid_source, &[]).expect("Valid guard compilation should succeed");
    assert!(res.js_code.contains("if (!("));

    // Verify Go backend
    let go_code = compile_to_go(valid_source).expect("Go codegen for guard should succeed");
    assert!(go_code.contains("if !("));

    // E2E execution
    let mut js_runner = res.js_code.clone();
    js_runner.push_str("\n\nmain();\n");
    run_node(&js_runner);

    // Verify compile-time enforcement: else body MUST diverge
    let non_diverging_source = r#"
        export fn testInvalidGuard(x: Int): Int => {
            guard x > 0 else {
                let y = 1;
            };
            return x;
        }
    "#;
    let err =
        compile(non_diverging_source, &[]).expect_err("Non-diverging guard must fail typechecking");
    assert!(
        err.contains("Guard 'else' block must diverge"),
        "Error message should mention divergence requirement: {}",
        err
    );

    // Verify condition must be Bool
    let non_bool_cond_source = r#"
        export fn testNonBoolGuard(x: Int): Int => {
            guard x else {
                return 0;
            };
            return x;
        }
    "#;
    let err_cond = compile(non_bool_cond_source, &[])
        .expect_err("Non-bool guard condition must fail typechecking");
    assert!(
        err_cond.contains("Guard statement condition: expected 'Bool'"),
        "Error message should mention boolean requirement: {}",
        err_cond
    );
}

#[test]
fn test_if_let_pattern_matching_ergonomics() {
    let source = r#"
        export fn describeOption(opt: Option<Int>): String => {
            if let Some(val) = opt {
                `Value is ${val}`
            } else {
                "No value"
            }
        }

        export fn main(): Unit => {
            let s1 = describeOption(Some(99));
            assert(s1 == "Value is 99", "if let Some match failed");

            let s2 = describeOption(None);
            assert(s2 == "No value", "if let None match failed");
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of if let should succeed");
    let mut js_runner = res.js_code.clone();
    js_runner.push_str("\n\nmain();\n");
    run_node(&js_runner);

    // Verify Go backend codegen
    let go_code = compile_to_go(source).expect("Go codegen for if let should succeed");
    assert!(go_code.contains("Some"));
}

#[test]
fn test_distinct_nominal_types() {
    let valid_source = r#"
        type UserId = distinct Int;
        type OrderId = distinct Int;

        fn formatUser(id: UserId): String => {
            let raw = Int(id);
            `User #${raw}`
        }

        export fn main(): Unit => {
            let u = UserId(1001);
            let o = OrderId(5005);

            assert(formatUser(u) == "User #1001", "formatUser failed");
            assert(Int(o) == 5005, "Unwrap OrderId failed");
        }
    "#;

    let res = compile(valid_source, &[]).expect("Distinct type compilation should succeed");
    let mut js_runner = res.js_code.clone();
    js_runner.push_str("\n\nmain();\n");
    run_node(&js_runner);

    // Test rejection: cannot assign raw Int to distinct type without explicit constructor
    let raw_assign_err_source = r#"
        type UserId = distinct Int;
        export fn main(): Unit => {
            let u: UserId = 42;
        }
    "#;
    let err1 = compile(raw_assign_err_source, &[])
        .expect_err("Implicit raw int assignment must be rejected");
    assert!(
        err1.contains("Type mismatch"),
        "Error should report type mismatch between Int and UserId: {}",
        err1
    );

    // Test rejection: cannot pass OrderId where UserId is expected
    let type_cross_err_source = r#"
        type UserId = distinct Int;
        type OrderId = distinct Int;

        fn handleUser(id: UserId): Unit => ()

        export fn main(): Unit => {
            let o = OrderId(42);
            handleUser(o);
        }
    "#;
    let err2 = compile(type_cross_err_source, &[])
        .expect_err("Passing OrderId as UserId must be rejected");
    assert!(
        err2.contains("Type mismatch"),
        "Error should report type mismatch between OrderId and UserId: {}",
        err2
    );
}

#[test]
fn test_or_patterns_in_match() {
    let source = r#"
        type WebStatus =
            | Ok200
            | Created201
            | BadRequest400
            | NotFound404
            | InternalError500;

        fn categorize(status: WebStatus): String => {
            match status {
                Ok200 | Created201 => "success",
                BadRequest400 | NotFound404 => "client_error",
                InternalError500 => "server_error",
            }
        }

        fn digitName(d: Int): String => {
            match d {
                1 | 2 | 3 => "low",
                4 | 5 | 6 => "mid",
                _ => "other",
            }
        }

        export fn main(): Unit => {
            assert(categorize(Ok200) == "success", "Ok200 should be success");
            assert(categorize(Created201) == "success", "Created201 should be success");
            assert(categorize(NotFound404) == "client_error", "NotFound404 should be client_error");
            assert(categorize(InternalError500) == "server_error", "InternalError500 should be server_error");

            assert(digitName(2) == "low", "2 should be low");
            assert(digitName(5) == "mid", "5 should be mid");
            assert(digitName(9) == "other", "9 should be other");
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of Or-patterns should succeed");
    let mut js_runner = res.js_code.clone();
    js_runner.push_str("\n\nmain();\n");
    run_node(&js_runner);

    // Verify Go backend
    let go_code = compile_to_go(source).expect("Go codegen for Or-patterns should succeed");
    assert!(go_code.contains("Ok200") && go_code.contains("Created201"));
}

#[test]
fn test_cranelift_native_features_execution() {
    let source = r#"
        type UserId = distinct Int;

        type Status =
            | Active
            | Inactive;

        fn describeStatus(s: Status): String => {
            match s {
                Active | Inactive => "known",
            }
        }

        fn checkAge(age: Int): Int => {
            guard age >= 0 else {
                return 0;
            };
            return age * 2;
        }

        export fn main(): Unit => {
            let u = UserId(999);
            println(Int(u));

            let opt: Option<Int> = Some(123);
            let val = opt ?? 0;
            println(val);

            if let Some(x) = opt {
                println(x);
            } else {
                println(0);
            };

            let doubled = checkAge(21);
            println(doubled);

            let st = describeStatus(Active);
            println(st);
        }
    "#;

    let out_bin = std::path::Path::new("test_cranelift_features_bin");
    let res = aura_lang::build_native_binary(source, out_bin);
    assert!(res.is_ok(), "Native compilation failed: {:?}", res.err());
    assert!(out_bin.exists());

    let output = Command::new("./test_cranelift_features_bin")
        .output()
        .expect("Failed to run native binary");

    let _ = fs::remove_file(out_bin);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("999"), "stdout missing 999: {}", stdout);
    assert!(stdout.contains("123"), "stdout missing 123: {}", stdout);
    assert!(stdout.contains("42"), "stdout missing 42: {}", stdout);
    assert!(stdout.contains("known"), "stdout missing known: {}", stdout);
}
