use std::fs;
use std::process::Command;
use std::sync::Once;

static STAGE1_BOOTSTRAP: Once = Once::new();

fn run_stage1_compilation() {
    STAGE1_BOOTSTRAP.call_once(|| {
        let _ = fs::create_dir_all("dist");
        let modules = [
            ("src/aura_compiler/ast.aura", "dist/ast.mjs"),
            ("src/aura_compiler/lexer.aura", "dist/lexer.mjs"),
            ("src/aura_compiler/parser.aura", "dist/parser.mjs"),
            ("src/aura_compiler/codegen.aura", "dist/codegen.mjs"),
            ("src/aura_compiler/main.aura", "dist/aurac.mjs"),
        ];

        for (src, out) in &modules {
            let status = Command::new(env!("CARGO_BIN_EXE_aurac"))
                .args(["compile", src, "-o", out])
                .status()
                .expect("Failed to execute aurac compile");

            assert!(
                status.success(),
                "Failed to compile stage 1 module: {}",
                src
            );
            assert!(
                fs::metadata(out).is_ok(),
                "Output file does not exist: {}",
                out
            );
        }
    });
}

#[test]
fn test_bootstrap_stage1_rust_compiles_aura_compiler() {
    run_stage1_compilation();
}

#[test]
fn test_bootstrap_stage2_aura_compiler_compiles_sample_programs() {
    // 1. Ensure stage 1 compiler is compiled first (thread-safe synchronization)
    run_stage1_compilation();

    // 2. Compile hello.aura with the self-hosted compiler (dist/aurac.mjs)
    let compile_status = Command::new("node")
        .args([
            "dist/aurac.mjs",
            "examples/bootstrap_demo/hello.aura",
            "-o",
            "examples/bootstrap_demo/hello.js",
        ])
        .status()
        .expect("Failed to execute node dist/aurac.mjs");

    assert!(
        compile_status.success(),
        "Self-hosted compiler failed to compile hello.aura"
    );
    assert!(
        fs::metadata("examples/bootstrap_demo/hello.js").is_ok(),
        "Generated JS output does not exist"
    );

    // 3. Execute the compiled JS program and verify stdout
    let run_output = Command::new("node")
        .arg("examples/bootstrap_demo/hello.js")
        .output()
        .expect("Failed to execute node examples/bootstrap_demo/hello.js");

    assert!(
        run_output.status.success(),
        "Executing compiled hello.js failed"
    );
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        stdout.contains("Hello Developer from Self-Hosted Aura Compiler!"),
        "Stdout missing greeting: {}",
        stdout
    );
    assert!(
        stdout.contains("Sum of 1..100: 5050"),
        "Stdout missing computed sum: {}",
        stdout
    );
}

#[test]
fn test_bootstrap_stage3_native_binary_build_and_execution() {
    let _ = fs::create_dir_all("dist");
    let native_bin = "dist/test-self-hosted-bin";
    let compiled_js = "dist/hello_from_native.js";

    // 1. Build self-hosted compiler into a native standalone binary
    let build_status = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .args(["build", "src/aura_compiler/main.aura", "-o", native_bin])
        .status()
        .expect("Failed to execute aurac build");

    assert!(
        build_status.success(),
        "Failed to compile self-hosted compiler to standalone native binary"
    );
    assert!(
        fs::metadata(native_bin).is_ok(),
        "Native binary does not exist at {}",
        native_bin
    );

    // 2. Run the native binary to compile examples/hello.aura
    let run_bin_status = Command::new(format!("./{}", native_bin))
        .args(["examples/hello.aura", "-o", compiled_js])
        .status()
        .expect("Failed to execute native self-hosted compiler binary");

    assert!(
        run_bin_status.success(),
        "Native self-hosted compiler failed to compile examples/hello.aura"
    );
    assert!(
        fs::metadata(compiled_js).is_ok(),
        "Compiled JS output does not exist at {}",
        compiled_js
    );

    // 3. Execute the JS output with node and verify output
    let node_output = Command::new("node")
        .arg(compiled_js)
        .output()
        .expect("Failed to run node on compiled output");

    assert!(
        node_output.status.success(),
        "Node failed to execute compiled JS"
    );
    let stdout = String::from_utf8_lossy(&node_output.stdout);
    assert!(
        stdout.contains("Hello Developer from Self-Hosted Aura Compiler!"),
        "Stdout did not match expected greeting: {}",
        stdout
    );

    // Cleanup
    let _ = fs::remove_file(native_bin);
    let _ = fs::remove_file(compiled_js);
}

