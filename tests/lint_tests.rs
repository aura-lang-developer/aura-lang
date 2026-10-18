use aura_lang::linter::{lint_source, LintSeverity};

#[test]
fn test_linter_detects_sql_injection_template() {
    let code = r#"
        fn query(user_id: String) {
            let q = `SELECT * FROM users WHERE id = ${user_id}`;
        }
    "#;

    let diags = lint_source(code).expect("should parse");
    assert!(
        diags.iter().any(|d| d.code == "SEC001" && d.severity == LintSeverity::Error),
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
        diags.iter().any(|d| d.code == "SEC001" && d.severity == LintSeverity::Error),
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
        diags.iter().any(|d| d.code == "SEC002" && d.severity == LintSeverity::Warning),
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
        diags.iter().any(|d| d.code == "LINT001" && d.severity == LintSeverity::Warning),
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
