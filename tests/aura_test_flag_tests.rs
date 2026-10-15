use std::process::Command;

#[test]
fn test_aura_test_flag_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_aura"))
        .args(["--test", "--help"])
        .output()
        .expect("Failed to execute aura --test --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Aura Test Runner"));
    assert!(stdout.contains("aurac --test [flags] [packages/paths...]"));
}

#[test]
fn test_aura_test_flag_single_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_aura"))
        .args(["--test", "examples/tests/math_unit_test.aura"])
        .output()
        .expect("Failed to execute aura --test <file>");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
}

#[test]
fn test_aura_file_then_test_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_aura"))
        .args(["examples/tests/math_unit_test.aura", "--test"])
        .output()
        .expect("Failed to execute aura <file> --test");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
}

#[test]
fn test_aura_test_flag_coverage() {
    let output = Command::new(env!("CARGO_BIN_EXE_aura"))
        .args(["--test", "--coverage", "examples/tests/math_unit_test.aura"])
        .output()
        .expect("Failed to execute aura --test --coverage");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(stdout.contains("Coverage"));
}
