use std::process::Command;

/// Both `aura` and `aurac` support the same test runner flags.
/// We test with `aura` as the primary toolchain entrypoint and `aurac` as the compiler alias.
const TEST_BINARIES: &[&str] = &[env!("CARGO_BIN_EXE_aura"), env!("CARGO_BIN_EXE_aurac")];

#[test]
fn test_aura_test_flag_help() {
    for bin in TEST_BINARIES {
        let output = Command::new(bin)
            .args(["--test", "--help"])
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute {} --test --help", bin));

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Aura Test Runner"),
            "Help output should identify the test runner for binary {}",
            bin
        );
        assert!(
            stdout.contains("aurac --test [flags] [packages/paths...]"),
            "Help output should include test usage for binary {}",
            bin
        );
    }
}

#[test]
fn test_aura_test_flag_single_file() {
    for bin in TEST_BINARIES {
        let output = Command::new(bin)
            .args(["--test", "examples/tests/math_unit_test.aura"])
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute {} --test <file>", bin));

        assert!(
            output.status.success(),
            "{} --test <file> failed with status: {:?}",
            bin,
            output.status
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("PASS"),
            "Test run should report PASS on math_unit_test.aura for binary {}",
            bin
        );
    }
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
fn test_aura_dash_test_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_aura"))
        .args(["-test", "examples/tests/math_unit_test.aura"])
        .output()
        .expect("Failed to execute aura -test <file>");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
}

#[test]
fn test_aura_test_flag_filter() {
    let output = Command::new(env!("CARGO_BIN_EXE_aura"))
        .args([
            "--test",
            "-run",
            "test_addition",
            "examples/tests/math_unit_test.aura",
        ])
        .output()
        .expect("Failed to execute aura --test -run test_addition");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(
        stdout.contains("Ran 1 tests"),
        "Should run only 1 filtered test"
    );
}

#[test]
fn test_aura_test_flag_verbose() {
    let output = Command::new(env!("CARGO_BIN_EXE_aura"))
        .args(["examples/tests/math_unit_test.aura", "--test", "-v"])
        .output()
        .expect("Failed to execute aura <file> --test -v");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS"));
    assert!(
        stdout.contains("=== RUN"),
        "Verbose mode should print '=== RUN' header"
    );
}

#[test]
fn test_aura_test_flag_coverage() {
    for bin in TEST_BINARIES {
        let output = Command::new(bin)
            .args(["--test", "--coverage", "examples/tests/math_unit_test.aura"])
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute {} --test --coverage", bin));

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("PASS"));
        assert!(
            stdout.contains("Coverage"),
            "Output should include coverage table for binary {}",
            bin
        );
        assert!(
            stdout.contains("coverage:"),
            "Output should include overall coverage percentage for binary {}",
            bin
        );
    }
}
