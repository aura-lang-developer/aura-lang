//! Tests for Cranelift Native Backend and Standalone Aura Runtime.

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_native_cranelift_execution() {
    let source = r#"
        export fn main(): Unit => {
            println("Hello from Cranelift Native Backend!");
            let a = 15;
            let b = 27;
            println(a + b);
        }
    "#;

    let out_bin = Path::new("test_cranelift_bin");
    let res = aura_lang::build_native_binary(source, out_bin);
    assert!(res.is_ok(), "Native compilation failed: {:?}", res.err());
    assert!(out_bin.exists());

    let output = Command::new("./test_cranelift_bin")
        .output()
        .expect("Failed to run native binary");

    let _ = fs::remove_file(out_bin);

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Output: {}", stdout);
    assert!(stdout.contains("Hello from Cranelift Native Backend!"));
    assert!(stdout.contains("42"));
}

#[test]
fn test_native_fibers_and_concurrency() {
    let source = r#"
        export fn main(): Unit => {
            println("Parent fiber running");
            spawn {
                println("Spawned green thread running!");
            };
            yield();
            println("Parent fiber finished");
        }
    "#;

    let out_bin = Path::new("test_cranelift_fiber_bin");
    let res = aura_lang::build_native_binary(source, out_bin);
    assert!(res.is_ok(), "Native compilation failed: {:?}", res.err());
    assert!(out_bin.exists());

    let output = Command::new("./test_cranelift_fiber_bin")
        .output()
        .expect("Failed to run native binary");

    let _ = fs::remove_file(out_bin);

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Output: {}", stdout);
    assert!(stdout.contains("Parent fiber running"));
    assert!(stdout.contains("Spawned green thread running!"));
    assert!(stdout.contains("Parent fiber finished"));
}

#[test]
fn test_native_cranelift_release_mode() {
    let source = r#"
        export fn main(): Unit => {
            println("Hello from Cranelift Release Binary!");
            let a = 100;
            let b = 250;
            println(a + b);
        }
    "#;

    let out_bin = Path::new("test_cranelift_release_bin");
    let res = aura_lang::build_native_binary_with_options(source, out_bin, None, None, true);
    assert!(res.is_ok(), "Native release compilation failed: {:?}", res.err());
    assert!(out_bin.exists());

    let output = Command::new("./test_cranelift_release_bin")
        .output()
        .expect("Failed to run release native binary");

    let _ = fs::remove_file(out_bin);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Cranelift Release Binary!"));
    assert!(stdout.contains("350"));
}

