use std::process::Command;

#[test]
fn test_aurac_test_flag_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .args(["--test", "--help"])
        .output()
        .expect("Failed to execute aurac --test --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Aura Test Runner"),
        "Help output should identify the test runner"
    );
    assert!(
        stdout.contains("aurac --test [flags] [packages/paths...]"),
        "Help output should include aurac --test usage"
    );
}

#[test]
fn test_aurac_test_flag_single_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .args(["--test", "examples/tests/math_unit_test.aura"])
        .output()
        .expect("Failed to execute aurac --test <file>");

    assert!(
        output.status.success(),
        "aurac --test <file> failed with status: {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("PASS"),
        "Test run should report PASS on math_unit_test.aura"
    );
}

#[test]
fn test_aurac_file_then_test_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .args(["examples/tests/math_unit_test.aura", "--test"])
        .output()
        .expect("Failed to execute aurac <file> --test");

    assert!(
        output.status.success(),
        "aurac <file> --test failed with status: {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("PASS"),
        "Test run should report PASS on math_unit_test.aura"
    );
}

#[test]
fn test_aurac_dash_test_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .args(["-test", "examples/tests/math_unit_test.aura"])
        .output()
        .expect("Failed to execute aurac -test <file>");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
}

#[test]
fn test_aurac_test_flag_filter() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .args([
            "--test",
            "-run",
            "test_addition",
            "examples/tests/math_unit_test.aura",
        ])
        .output()
        .expect("Failed to execute aurac --test -run test_addition");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(
        stdout.contains("Ran 1 tests"),
        "Should run only 1 filtered test"
    );
}

#[test]
fn test_aurac_test_flag_verbose() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .args(["examples/tests/math_unit_test.aura", "--test", "-v"])
        .output()
        .expect("Failed to execute aurac <file> --test -v");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(
        stdout.contains("=== RUN"),
        "Verbose mode should print '=== RUN' header"
    );
}

#[test]
fn test_aurac_test_flag_coverage() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .args(["--test", "--coverage", "examples/tests/math_unit_test.aura"])
        .output()
        .expect("Failed to execute aurac --test --coverage");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(
        stdout.contains("Coverage"),
        "Output should include coverage table"
    );
    assert!(
        stdout.contains("coverage:"),
        "Output should include overall coverage percentage"
    );
}
