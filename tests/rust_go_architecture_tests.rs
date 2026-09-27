//! Architecture Verification Tests: Rust + Golang Unified Foundations
//!
//! This suite validates that Aura retains its core dual heritage:
//! 1. Rust Foundation: Immutability by default, Algebraic Data Types (ADTs),
//!    exhaustive pattern matching, Option/Result with the ? try operator, pipelines, and TCO.
//! 2. Golang Foundation: CSP concurrency (spawn, channels, select), Go slices ([]T, s[low:high:max]),
//!    implicit structural duck typing interfaces, receivers, defer/errdefer, pointers, and struct tags.
//! 3. Clean Architecture: Rejection of overengineered dynamic Lisp primitives (untyped atoms,
//!    ad-hoc condition/restart systems that break static type safety).

use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use aura_lang::{compile_backend, compile_to_go};

#[test]
fn test_rust_adt_and_exhaustive_pattern_matching() {
    let source = r#"
        type Shape =
            | Circle(Float)
            | Rectangle(Float, Float)
            | Point;

        export fn area(s: Shape): Float => {
            match s {
                Circle(r) => 3.14159 * r * r,
                Rectangle(w, h) => w * h,
                Point => 0.0,
            }
        }

        export fn main(): Unit => {
            let c = Circle(5.0);
            let r = Rectangle(4.0, 6.0);
            let p = Point;
            let a = area(c);
            println(`Circle area: ${a}`);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Compilation of Rust ADTs should succeed");
    assert!(res.js_code.contains("Circle"));
    assert!(res.js_code.contains("Rectangle"));
    assert!(res.js_code.contains("Point"));

    // Also verify Go backend code generation
    let go_code = compile_to_go(source).expect("Go codegen for Rust ADTs should succeed");
    assert!(go_code.contains("Circle"));
    assert!(go_code.contains("Rectangle"));
}

#[test]
fn test_rust_result_option_and_try_operator() {
    let source = r#"
        fn parseId(raw: String): Result<Int, String> => {
            if raw == "invalid" {
                Err("Invalid ID format")
            } else {
                Ok(42)
            }
        }

        fn fetchRecord(raw: String): Result<String, String> => {
            let id = parseId(raw)?;
            Ok(`User record #${id}`)
        }

        export fn main(): Unit => {
            let res = fetchRecord("100");
            match res {
                Ok(msg) => println(msg),
                Err(err) => println(`Error: ${err}`),
            }
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Result with try operator must compile");
    assert!(res.js_code.contains("fetchRecord"));
    assert!(res.js_code.contains("parseId"));
}

#[test]
fn test_rust_pipeline_operator() {
    let source = r#"
        fn double(x: Int): Int => x * 2;
        fn addTen(x: Int): Int => x + 10;

        export fn calculate(init: Int): Int => {
            init |> double |> addTen
        }

        export fn main(): Unit => {
            let val = calculate(5);
            println(`Value: ${val}`);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Pipelines should compile cleanly");
    assert!(res.js_code.contains("calculate"));
}

#[test]
fn test_go_csp_concurrency_and_channels() {
    let source = r#"
        export fn main(): Unit => {
            let ch = Channel.make<String>(2);

            spawn {
                ch <- "hello from fiber";
            };

            let msg = <-ch;
            println(msg);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("CSP channels should compile");
    assert!(res.js_code.contains("Channel"));

    // Verify Go codegen generates native goroutines and channels
    let go_code = compile_to_go(source).expect("Go codegen should succeed for CSP");
    assert!(go_code.contains("make(chan"));
    assert!(go_code.contains("go func()"));
}

#[test]
fn test_go_slices_and_slicing_expressions() {
    let source = r#"
        export fn subSlice(data: []Int): []Int => {
            return data[1:3];
        }

        export fn main(): Unit => {
            let items: []Int = [10, 20, 30, 40];
            let sub = subSlice(items);
            let length = len(sub);
            println(`Length: ${length}`);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Go slices must compile");
    assert!(res.js_code.contains("subSlice"));

    let go_code = compile_to_go(source).expect("Go codegen for slices must succeed");
    assert!(go_code.contains("[]int"));
    assert!(go_code.contains("len("));
}

#[test]
fn test_go_defer_and_errdefer_transactional_cleanup() {
    let source = r#"
        export fn doWork(shouldFail: Bool): Result<String, String> => {
            defer println("cleanup executed");
            errdefer println("rollback on error");

            if shouldFail {
                return Err("Failed operation");
            }
            return Ok("Success");
        }

        export fn main(): Unit => {
            let _ = doWork(false);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Defer and errdefer must compile");
    assert!(res.js_code.contains("doWork"));
}

#[test]
fn test_go_implicit_interfaces_and_struct_receivers() {
    let source = r#"
        interface Describable {
            describe(): String;
        }

        struct Device {
            model: String,
            watts: Int
        };

        fn (d: Device) describe(): String => {
            `Device ${d.model} consumes ${d.watts}W`
        }

        export fn printDescription(d: Describable): String => {
            d.describe()
        }

        export fn main(): Unit => {
            let laptop: Device = { model: "Pro 16", watts: 96 };
            let desc = printDescription(laptop);
            println(desc);
        }
    "#;

    let res = compile_backend(source, None, &[]).expect("Implicit interfaces must compile");
    assert!(res.js_code.contains("printDescription"));

    let go_code = compile_to_go(source).expect("Go codegen for receivers must succeed");
    assert!(go_code.contains("func (d Device) Describe() string"));
}

#[test]
fn test_rejection_of_overengineered_lisp_primitives() {
    // 1. Untyped Lisp keyword atom syntax ':foo' is rejected by the parser
    let atom_source = r#"
        export fn main(): Unit => {
            let x = :status_ok;
        }
    "#;
    let mut lexer = Lexer::new(atom_source);
    let tokens = lexer.tokenize().expect("Lexing token stream");
    let mut parser = Parser::new(tokens);
    let parse_result = parser.parse_module();
    assert!(
        parse_result.is_err(),
        "Untyped Lisp atoms like :status_ok must be rejected"
    );

    // 2. Dynamic Lisp condition system built-in functions are not in scope and rejected by typechecker
    let condition_source = r#"
        export fn main(): Unit => {
            signalCondition("DiskFull", { "free": 0 });
        }
    "#;
    let compile_result = compile_backend(condition_source, None, &[]);
    assert!(
        compile_result.is_err(),
        "Dynamic signalCondition must be rejected by static typechecker"
    );

    // 3. Dynamic withRestarts is not in scope and rejected
    let restart_source = r#"
        export fn main(): Unit => {
            withRestarts({}, fn() => { 0 });
        }
    "#;
    let compile_result2 = compile_backend(restart_source, None, &[]);
    assert!(
        compile_result2.is_err(),
        "Dynamic withRestarts must be rejected by static typechecker"
    );
}
