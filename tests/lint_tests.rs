use aura_lang::linter::{LintSeverity, lint_source};

#[test]
fn test_linter_detects_sql_injection_template() {
    let code = r#"
        fn query(user_id: String) {
            let q = `SELECT * FROM users WHERE id = ${user_id}`;
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    assert!(
        diags
            .iter()
            .any(|d| d.code == "SEC001" && d.severity == LintSeverity::Error),
        "Expected SEC001 error for SQL injection in template string, got: {:?}",
        diags
    );
}

#[test]
fn test_linter_detects_sql_injection_concat() {
    let code = r#"
        fn query(user_id: String) {
            let q = "DELETE FROM items WHERE id = " + user_id;
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    assert!(
        diags
            .iter()
            .any(|d| d.code == "SEC001" && d.severity == LintSeverity::Error),
        "Expected SEC001 error for SQL injection in string concat, got: {:?}",
        diags
    );
}

#[test]
fn test_linter_ignores_safe_parameterized_query() {
    let code = r#"
        fn query() {
            let q = "SELECT * FROM users WHERE id = $1";
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    assert!(
        !diags.iter().any(|d| d.code == "SEC001"),
        "Did not expect SEC001 for static parameterized query, got: {:?}",
        diags
    );
}

#[test]
fn test_linter_detects_hardcoded_secrets() {
    let code = r#"
        fn config() {
            let api_key = "super_secret_production_jwt_key_9999";
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    assert!(
        diags
            .iter()
            .any(|d| d.code == "SEC002" && d.severity == LintSeverity::Warning),
        "Expected SEC002 for hardcoded api_key, got: {:?}",
        diags
    );
}

#[test]
fn test_linter_allows_placeholder_secrets() {
    let code = r#"
        fn config() {
            let api_key = "test_dummy_placeholder";
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    assert!(
        !diags.iter().any(|d| d.code == "SEC002"),
        "Did not expect SEC002 for dummy placeholder, got: {:?}",
        diags
    );
}

#[test]
fn test_linter_detects_unused_let_mut() {
    let code = r#"
        fn calc() {
            let mut count = 0;
            println(count);
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    assert!(
        diags
            .iter()
            .any(|d| d.code == "LINT001" && d.severity == LintSeverity::Warning),
        "Expected LINT001 for unused let mut count, got: {:?}",
        diags
    );
}

#[test]
fn test_linter_allows_used_let_mut() {
    let code = r#"
        fn calc() {
            let mut count = 0;
            count = 10;
            println(count);
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    assert!(
        !diags.iter().any(|d| d.code == "LINT001"),
        "Did not expect LINT001 for mutated let mut count, got: {:?}",
        diags
    );
}

#[test]
fn test_linter_detects_redundant_export_and_auto_fixes() {
    use aura_lang::linter::fix_source;

    let code = r#"
        export fn Add(a: Int, b: Int): Int => a + b;
        export struct User { id: Int };
        export type Status = | Active | Inactive;
        export interface Greeter { fn Greet(): String; };
        export fn helper(x: Int): Int => x;
    "#;

    let diags = lint_source(code).expect("should parse");
    let redundant_count = diags.iter().filter(|d| d.code == "LINT004").count();
    assert_eq!(redundant_count, 4, "Expected 4 LINT004 diagnostics, got: {:?}", diags);

    let (fixed, count) = fix_source(code);
    assert_eq!(count, 4, "Expected 4 auto-fixes");
    assert!(fixed.contains("fn Add(a: Int, b: Int)"), "Should remove redundant export from PascalCase fn Add");
    assert!(fixed.contains("struct User"), "Should remove redundant export from PascalCase struct User");
    assert!(fixed.contains("type Status ="), "Should remove redundant export from PascalCase type Status");
    assert!(fixed.contains("interface Greeter"), "Should remove redundant export from PascalCase interface Greeter");
    assert!(fixed.contains("export fn helper"), "Should preserve necessary export for camelCase helper");
}

#[test]
fn test_linter_detects_deprecated_synonyms_and_auto_fixes() {
    use aura_lang::linter::fix_source;

    let code = r#"
        trait Greeter {
            fn greet(): String;
        }

        fn run_tasks() {
            routine {
                println("worker 1");
            }
            go {
                println("worker 2");
            }
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    let deprecations: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "LINT005" || d.code == "LINT006")
        .collect();
    assert_eq!(deprecations.len(), 3, "Expected 3 deprecation diagnostics, got: {:?}", diags);

    let (fixed, count) = fix_source(code);
    assert_eq!(count, 3, "Expected 3 auto-fixes for deprecated synonyms");
    assert!(fixed.contains("interface Greeter"), "trait should be replaced with interface");
    assert!(!fixed.contains("trait Greeter"));
    assert!(fixed.contains("spawn {\n                println(\"worker 1\");"));
    assert!(fixed.contains("spawn {\n                println(\"worker 2\");"));
    assert!(!fixed.contains("routine {"));
    assert!(!fixed.contains("go {"));
}
