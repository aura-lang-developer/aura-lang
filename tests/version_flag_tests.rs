use std::process::Command;

#[test]
fn test_aurac_version_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .arg("--version")
        .output()
        .expect("Failed to execute aurac --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("aurac {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_aurac_short_v_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .arg("-v")
        .output()
        .expect("Failed to execute aurac -v");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("aurac {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_aurac_capital_v_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .arg("-V")
        .output()
        .expect("Failed to execute aurac -V");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("aurac {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_aurac_version_subcommand() {
    let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
        .arg("version")
        .output()
        .expect("Failed to execute aurac version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("aurac {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_aurac_subcommands_version_flag() {
    let subcommands = ["build", "run", "check", "emit-go", "watch"];
    for subcmd in subcommands {
        let output = Command::new(env!("CARGO_BIN_EXE_aurac"))
            .args([subcmd, "--version"])
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute aurac {} --version", subcmd));

        assert!(
            output.status.success(),
            "aurac {} --version failed with status: {:?}",
            subcmd,
            output.status
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.trim(),
            format!("aurac {}", env!("CARGO_PKG_VERSION")),
            "Unexpected output for aurac {} --version",
            subcmd
        );
    }
}

#[test]
fn test_toolchain_binaries_version_flag() {
    let binaries = [
        ("auratest", env!("CARGO_BIN_EXE_auratest")),
        ("aurafmt", env!("CARGO_BIN_EXE_aurafmt")),
        ("auramod", env!("CARGO_BIN_EXE_auramod")),
        ("aurabindgen", env!("CARGO_BIN_EXE_aurabindgen")),
        ("auralsp", env!("CARGO_BIN_EXE_auralsp")),
    ];

    for (name, exe) in binaries {
        let output = Command::new(exe)
            .arg("--version")
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute {} --version", name));

        assert!(
            output.status.success(),
            "{} --version failed with status: {:?}",
            name,
            output.status
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.trim(),
            format!("{} {}", name, env!("CARGO_PKG_VERSION")),
            "Unexpected output for {} --version",
            name
        );
    }
}

#[test]
fn test_aura_lang_version_constants() {
    assert_eq!(aura_lang::VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(aura_lang::version(), env!("CARGO_PKG_VERSION"));
}
