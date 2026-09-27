use aura_lang::ast::Item;
use aura_lang::codegen::CodeGen;
use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use aura_lang::typechecker::TypeChecker;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn run_node(js_code: &str) -> String {
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp_path = format!("test_zig_{}_{}.mjs", std::process::id(), count);
    fs::write(&tmp_path, js_code).expect("Failed to write temp test file");

    let output = Command::new("node")
        .arg(&tmp_path)
        .output()
        .expect("Failed to execute node");

    let _ = fs::remove_file(&tmp_path);

    if !output.status.success() {
        panic!(
            "Node.js execution failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn test_errdefer_execution_on_error_and_skipped_on_success() {
    let src = r#"
        let mut events: []String = [];

        fn succeedOperation(): Result<String, String> => {
            defer events.push("defer_succeed");
            errdefer events.push("errdefer_succeed");

            Ok("success")
        }

        fn failOperation(): Result<String, String> => {
            defer events.push("defer_fail");
            errdefer events.push("errdefer_fail");

            Err("failure")
        }

        export fn main(): Unit => {
            let res1 = succeedOperation();
            let res2 = failOperation();
            println(events.join(","));
        }
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().expect("Parsing failed");

    let mut tc = TypeChecker::new();
    tc.check_module(&module).expect("Typecheck failed");

    let mut codegen = CodeGen::new();
    let (js, _) = codegen.generate(&module);

    let output = run_node(&format!("{}\nmain();", js));
    // On succeed: only "defer_succeed" executes.
    // On fail: both "errdefer_fail" and "defer_fail" execute in LIFO order!
    assert_eq!(output, "defer_succeed,errdefer_fail,defer_fail");
}

#[test]
fn test_errdefer_execution_with_try_operator() {
    let src = r#"
        let mut log: []String = [];

        fn stepOne(): Result<Int, String> => Ok(42);
        fn stepTwoFail(): Result<Int, String> => Err("network timeout");

        fn runPipeline(): Result<Int, String> => {
            defer log.push("cleanup_always");
            errdefer log.push("rollback_transaction");

            let a = stepOne()?;
            let b = stepTwoFail()?;

            Ok(a + b)
        }

        export fn main(): Unit => {
            let res = runPipeline();
            println(log.join(","));
        }
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().expect("Parsing failed");

    let mut tc = TypeChecker::new();
    tc.check_module(&module).expect("Typecheck failed");

    let mut codegen = CodeGen::new();
    let (js, _) = codegen.generate(&module);

    let output = run_node(&format!("{}\nmain();", js));
    // Because stepTwoFail() returns Err and ? unwraps it, errdefer and defer unwind in LIFO order!
    assert_eq!(output, "rollback_transaction,cleanup_always");
}

#[test]
fn test_packed_struct_syntax_and_ast() {
    let src = r#"
        export packed struct TcpHeader {
            src_port: Uint16,
            dst_port: Uint16,
            seq_no: Uint32,
            ack_no: Uint32,
        };

        export type Packet = packed struct {
            length: Uint16,
            checksum: Uint16,
        };

        export fn checkPacket(p: Packet): Uint16 => p.length;
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().expect("Parsing failed");

    assert_eq!(module.items.len(), 3);

    // Verify first item is packed TypeAliasDecl
    if let Item::TypeAlias(ref alias) = module.items[0] {
        assert_eq!(alias.name, "TcpHeader");
        assert!(alias.is_packed);
    } else {
        panic!("Expected TypeAliasDecl for TcpHeader");
    }

    // Verify second item is packed TypeAliasDecl
    if let Item::TypeAlias(ref alias) = module.items[1] {
        assert_eq!(alias.name, "Packet");
        assert!(alias.is_packed);
    } else {
        panic!("Expected TypeAliasDecl for Packet");
    }

    let mut tc = TypeChecker::new();
    tc.check_module(&module).expect("Typecheck failed");

    let mut codegen = CodeGen::new();
    let (js, dts) = codegen.generate(&module);

    assert!(js.contains("Packed struct: TcpHeader"));
    assert!(js.contains("Packed struct: Packet"));
    assert!(dts.contains("export type TcpHeader"));
}

#[test]
fn test_cross_compilation_target_parsing() {
    use aura_lang::codegen_cranelift::CraneliftBackend;

    // Musl Linux target
    let backend_musl = CraneliftBackend::with_target(Some("x86_64-unknown-linux-musl"));
    assert!(
        backend_musl.is_ok(),
        "Cranelift should configure musl linux target"
    );

    // ARM64 Linux target
    let backend_arm_linux = CraneliftBackend::with_target(Some("aarch64-unknown-linux-musl"));
    assert!(
        backend_arm_linux.is_ok(),
        "Cranelift should configure aarch64 linux target"
    );

    // macOS target
    let backend_macos = CraneliftBackend::with_target(Some("aarch64-apple-darwin"));
    assert!(
        backend_macos.is_ok(),
        "Cranelift should configure macOS darwin target"
    );

    // Short forms: linux/amd64
    let backend_short = CraneliftBackend::with_target(Some("linux/amd64"));
    assert!(
        backend_short.is_ok(),
        "Cranelift should configure short alias linux/amd64"
    );
}

#[test]
fn test_extern_c_function_declaration() {
    let src = r#"
        extern "C" {
            fn getpid(): Int;
            fn abs(n: Int): Int;
        }

        export fn myPid(): Int => getpid();
        export fn main(): Unit => {
            let _ = myPid();
        }
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().expect("Parsing failed");

    let mut tc = TypeChecker::new();
    tc.check_module(&module)
        .expect("Typecheck failed for extern C");

    // Verify Cranelift module compilation handles extern declarations
    let cl_backend = aura_lang::codegen_cranelift::CraneliftBackend::new().expect("Failed backend");
    let object_bytes = cl_backend.compile_module(&module);
    assert!(
        object_bytes.is_ok(),
        "Cranelift should successfully declare extern import functions"
    );
}
