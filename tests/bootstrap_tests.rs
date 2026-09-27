use std::fs;
use std::process::Command;

#[test]
fn test_bootstrap_stage1_rust_compiles_aura_compiler() {
    let modules = [
        ("src/aura_compiler/ast.aura", "dist/ast.mjs"),
        ("src/aura_compiler/lexer.aura", "dist/lexer.mjs"),
        ("src/aura_compiler/parser.aura", "dist/parser.mjs"),
        ("src/aura_compiler/codegen.aura", "dist/codegen.mjs"),
        ("src/aura_compiler/main.aura", "dist/aurac.mjs"),
    ];

    for (src, out) in &modules {
        let status = Command::new("cargo")
            .args(["run", "--bin", "aurac", "--", "compile", src, "-o", out])
            .status()
            .expect("Failed to execute cargo run aurac compile");

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
}

#[test]
fn test_bootstrap_stage2_aura_compiler_compiles_sample_programs() {
    // 1. Compile hello.aura with the self-hosted compiler (dist/aurac.mjs)
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

    // 2. Execute the compiled JS program and verify stdout
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
