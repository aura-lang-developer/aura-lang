use aura_lang::package::{ModuleFile, PackageManager, resolve_package_import};
use aura_lang::{compile_backend, compile_local_deps_to_mjs_from_source};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{}_{}", prefix, nanos));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_aura_mod_init_and_get() {
    let root = get_unique_temp_dir("aura_test_init");

    // 1. aurac mod init
    let mod_path =
        PackageManager::init(&root, Some("github.com/myorg/myapp")).expect("init failed");
    assert!(mod_path.exists());
    assert!(root.join("aura.mod").exists());
    assert!(root.join("aura.lock").exists());

    let content = fs::read_to_string(&mod_path).unwrap();
    assert!(content.contains("module github.com/myorg/myapp"));
    assert!(content.contains("aura 0.1.0"));

    // 2. aurac mod get
    let res = PackageManager::get(&root, "github.com/myorg/utils@v1.2.3").expect("get failed");
    assert!(res.contains("Added dependency 'github.com/myorg/utils'"));

    let mod_file = ModuleFile::load_from_dir(&root).unwrap().unwrap().1;
    assert_eq!(mod_file.require.len(), 1);
    assert_eq!(mod_file.require[0].path, "github.com/myorg/utils");
    assert_eq!(mod_file.require[0].version, "v1.2.3");

    let lock_content = fs::read_to_string(root.join("aura.lock")).unwrap();
    assert!(lock_content.contains("github.com/myorg/utils v1.2.3 h1:"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_aura_mod_vendor_and_tamper_verification() {
    let root = get_unique_temp_dir("aura_test_vendor");

    PackageManager::init(&root, Some("github.com/myorg/secureapp")).unwrap();
    PackageManager::get(&root, "github.com/myorg/auth@v2.0.0").unwrap();

    // 1. Vendor
    let count = PackageManager::vendor(&root).expect("vendor failed");
    assert_eq!(count, 1);
    let vendored_lib = root
        .join("vendor")
        .join("github.com/myorg/auth")
        .join("lib.aura");
    assert!(vendored_lib.exists(), "Vendored lib.aura must exist");

    // 2. Verify: should succeed
    let statuses = PackageManager::verify(&root).expect("verify failed");
    assert_eq!(statuses.len(), 1);
    assert!(statuses[0].ok, "Expected package to be verified");

    // 3. Tamper with vendored file
    fs::write(&vendored_lib, "// TAMPERED CODE MALICIOUS INJECTION\n").unwrap();

    // 4. Verify again: should detect tampering
    let tampered_statuses = PackageManager::verify(&root).expect("verify run");
    assert_eq!(tampered_statuses.len(), 1);
    assert!(!tampered_statuses[0].ok, "Tampering must be detected");
    assert!(tampered_statuses[0].message.contains("SECURITY MISMATCH"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_aura_mod_tidy_auto_discovery_and_pruning() {
    let root = get_unique_temp_dir("aura_test_tidy");

    PackageManager::init(&root, Some("github.com/myorg/tidytest")).unwrap();

    // Create a source file importing an external package
    let app_file = root.join("app.aura");
    fs::write(
        &app_file,
        r#"
import { Hash } from "github.com/crypto/hash";

export fn main(): Unit => {
    println("Using hash");
}
"#,
    )
    .unwrap();

    // 1. Run tidy: should auto-discover github.com/crypto/hash
    let (added, removed) = PackageManager::tidy(&root).expect("tidy failed");
    assert_eq!(added, 1);
    assert_eq!(removed, 0);

    let mod_file = ModuleFile::load_from_dir(&root).unwrap().unwrap().1;
    assert_eq!(mod_file.require.len(), 1);
    assert_eq!(mod_file.require[0].path, "github.com/crypto/hash");

    // 2. Remove the import from app.aura
    fs::write(
        &app_file,
        r#"
export fn main(): Unit => {
    println("No imports anymore");
}
"#,
    )
    .unwrap();

    // 3. Run tidy again: should prune the unused dependency
    let (added2, removed2) = PackageManager::tidy(&root).expect("tidy prune failed");
    assert_eq!(added2, 0);
    assert_eq!(removed2, 1);

    let mod_file2 = ModuleFile::load_from_dir(&root).unwrap().unwrap().1;
    assert_eq!(mod_file2.require.len(), 0, "Dependency should be pruned");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_compiler_package_import_and_execution() {
    let root = get_unique_temp_dir("aura_test_compile_pkg");

    PackageManager::init(&root, Some("github.com/myorg/app")).unwrap();

    // Create a vendored package
    let pkg_dir = root
        .join("vendor")
        .join("github.com")
        .join("acme")
        .join("math");
    fs::create_dir_all(&pkg_dir).unwrap();

    let pkg_aura = pkg_dir.join("lib.aura");
    fs::write(
        &pkg_aura,
        r#"
export fn add(a: Int, b: Int): Int => a + b;
export fn multiply(a: Int, b: Int): Int => a * b;
"#,
    )
    .unwrap();

    // Verify resolve_package_import locates it
    let resolved = resolve_package_import("github.com/acme/math", Some(&root));
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap(), pkg_aura);

    // Create main app importing the package
    let app_aura = root.join("main.aura");
    let app_source = r#"
import { add, multiply } from "github.com/acme/math";

export fn main(): Unit => {
    let sum = add(10, 20);
    let prod = multiply(5, 6);
    println(`Sum: ${sum}, Prod: ${prod}`);
}
"#;
    fs::write(&app_aura, app_source).unwrap();

    // Typecheck and compile main module
    let res = compile_backend(app_source, Some(&root), &[]).expect("Compilation must succeed");
    assert!(res.js_code.contains("add(10, 20)"));

    // Compile local deps to .mjs
    let compiled_deps = compile_local_deps_to_mjs_from_source(app_source, Some(&root));
    assert!(!compiled_deps.is_empty());
    assert!(pkg_dir.join("lib.mjs").exists());

    // Execute generated bundle with node to verify end-to-end execution
    let main_mjs = root.join("main.mjs");
    let mut runner_code = res.js_code.clone();
    runner_code.push_str("\nmain();\n");
    fs::write(&main_mjs, &runner_code).unwrap();

    let node_out = Command::new("node").arg(&main_mjs).output();
    if let Ok(out) = node_out {
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("Sum: 30, Prod: 30"),
            "Output was: {}",
            stdout
        );
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_security_package_path_validation() {
    let malicious_paths = [
        "../escape",
        "../../etc/passwd",
        "/etc/shadow",
        "\\Windows\\System32",
        "C:\\autoexec.bat",
        "-u--upload-pack=evil",
        "--config=protocol.ext.allow=always",
        "pkg;rm -rf /",
        "foo|bar",
        "pkg$(whoami)",
    ];

    for path in &malicious_paths {
        assert!(
            aura_lang::package::validate_security_package_path(path).is_err(),
            "Path '{}' must be rejected for security reasons",
            path
        );
    }

    let malicious_versions = ["--upload-pack=evil", "-b", ";evil", "v1.0.0;rm -rf /"];

    for ver in &malicious_versions {
        assert!(
            aura_lang::package::validate_security_version(ver).is_err(),
            "Version '{}' must be rejected for security reasons",
            ver
        );
    }

    assert!(aura_lang::package::validate_security_package_path("github.com/myorg/utils").is_ok());
    assert!(aura_lang::package::validate_security_package_path("gitlab.com/pkg/mod_v2").is_ok());
    assert!(aura_lang::package::validate_security_version("v1.2.3").is_ok());
    assert!(aura_lang::package::validate_security_version("latest").is_ok());
}
