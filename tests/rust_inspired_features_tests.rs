use aura_lang::compile;
use aura_lang::linter::{fix_source, lint_source};
use aura_lang::package::ModuleFile;
use aura_lang::testing::extract_doctests;
use std::path::Path;

#[test]
fn test_derive_attribute_parsing_and_codegen() {
    let source = r#"
        #[derive(Debug, Clone, Eq, Json, Hash)]
        export type User = {
            id: Int,
            name: String,
        };

        export fn createUser(id: Int, name: String): User => {
            { id: id, name: name }
        };
    "#;

    let res = compile(source, &[]).expect("Compilation with @derive failed");
    // Verify companion methods on User
    assert!(res.js_code.contains("User.debug = "));
    assert!(res.js_code.contains("User.clone = "));
    assert!(res.js_code.contains("User.equals = "));
    assert!(res.js_code.contains("User.toJson = "));
    assert!(res.js_code.contains("User.fromJson = "));
    assert!(res.js_code.contains("User.hash = "));

    // Verify globalThis receiver methods are registered
    assert!(res.js_code.contains("__aura_methods[\"User\"][\"debug\"]"));
    assert!(res.js_code.contains("__aura_methods[\"User\"][\"clone\"]"));
    assert!(res.js_code.contains("__aura_methods[\"User\"][\"equals\"]"));
    assert!(res.js_code.contains("__aura_methods[\"User\"][\"toJson\"]"));
}

#[test]
fn test_derive_rust_style_hash_bracket() {
    let source = r#"
        #[derive(Debug, Clone, Eq)]
        export type Product = {
            sku: String,
            price: Float,
        };
    "#;

    let res = compile(source, &[]).expect("Compilation with #[derive(...)] failed");
    assert!(res.js_code.contains("Product.debug = "));
    assert!(res.js_code.contains("Product.clone = "));
    assert!(res.js_code.contains("Product.equals = "));
}

#[test]
fn test_derive_on_sum_types() {
    let source = r#"
        #[derive(Debug, Clone, Eq, Json)]
        export type Status =
            | Active
            | Suspended(String);
    "#;

    let res = compile(source, &[]).expect("Compilation with #[derive] on sum types failed");
    assert!(res.js_code.contains("Status.debug = "));
    assert!(res.js_code.contains("Status.clone = "));
    assert!(res.js_code.contains("Suspended.clone = "));
}

#[test]
fn test_must_use_attribute_and_linter_detection() {
    let source = r#"
        #[must_use = "Transaction must be committed or rolled back"]
        fn beginTx(): Int => {
            return 42;
        }

        fn main(): Unit => {
            beginTx(); // Discarded call! Should trigger LINT003
        }
    "#;

    let diags = lint_source(source).expect("Linter failed on source");
    let must_use_finding = diags.iter().find(|d| d.code == "LINT003");
    assert!(must_use_finding.is_some(), "Expected LINT003 diagnostic finding");
    let msg = &must_use_finding.unwrap().message;
    assert!(msg.contains("Transaction must be committed or rolled back"));
}

#[test]
fn test_linter_auto_fix() {
    let source = r#"
        fn compute(): Int => {
            let mut a = 10;
            let mut b = 20;
            b = 30;
            return a + b;
        }
    "#;

    let (fixed, count) = fix_source(source);
    assert_eq!(count, 1, "Only 'a' should be fixed from let mut to let");
    assert!(fixed.contains("let a = 10;"));
    assert!(fixed.contains("let mut b = 20;"));
}

#[test]
fn test_doctest_extraction() {
    let source = r#"
        /// Adds two numbers together.
        /// ```aura
        /// let res = add(10, 20);
        /// assert_eq(res, 30);
        /// ```
        pub fn add(a: Int, b: Int): Int => {
            a + b
        }
    "#;

    let (doctests, synthetic_code) = extract_doctests(source, Path::new("math.aura"));
    assert_eq!(doctests.len(), 1);
    assert_eq!(doctests[0].kind, aura_lang::testing::TestKind::DocTest);
    assert!(synthetic_code.contains("export fn __aura_doctest_1"));
    assert!(synthetic_code.contains("let res = add(10, 20);"));
}

#[test]
fn test_aura_mod_features_parsing_and_serialization() {
    let mod_content = r#"
module my/backend/service

aura 0.8.3

require (
    github.com/aura/db v1.2.0 [postgres, sqlite]
    github.com/aura/metrics v0.4.0
)
"#;

    let parsed = ModuleFile::parse(mod_content).expect("Failed to parse aura.mod with features");
    assert_eq!(parsed.require.len(), 2);
    let db_dep = parsed.require.iter().find(|d| d.path == "github.com/aura/db").unwrap();
    assert_eq!(db_dep.features, vec!["postgres".to_string(), "sqlite".to_string()]);

    let metrics_dep = parsed.require.iter().find(|d| d.path == "github.com/aura/metrics").unwrap();
    assert!(metrics_dep.features.is_empty());

    // Verify roundtrip serialization preserves features
    let serialized = parsed.to_string();
    assert!(serialized.contains("github.com/aura/db v1.2.0 [postgres, sqlite]"));
    assert!(serialized.contains("github.com/aura/metrics v0.4.0"));
}

#[test]
fn test_e2e_derive_methods_execution() {
    let source = r#"
        #[derive(Debug, Clone, Eq, Json, Hash)]
        export type Point = {
            x: Int,
            y: Int,
        };

        export fn testPoint(): Bool => {
            let p1 = { x: 10, y: 20 };
            let p2 = Point.clone(p1);
            let eq = Point.equals(p1, p2);
            let json = Point.toJson(p1);
            return eq;
        };
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let script = format!(
        "{}\nif (!testPoint()) process.exit(1);\nconsole.log('DERIVE_E2E_OK');\n",
        res.js_code
    );
    let tmp_path = std::env::temp_dir().join("test_derive_e2e.mjs");
    std::fs::write(&tmp_path, script).unwrap();
    let output = std::process::Command::new("node")
        .arg(&tmp_path)
        .output()
        .expect("Failed to execute node");
    let _ = std::fs::remove_file(&tmp_path);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DERIVE_E2E_OK"));
}
