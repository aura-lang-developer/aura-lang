//! Static code analysis and security linter for Aura Language.
//!
//! Provides automated security and code quality checks:
//! - `SEC001`: Potential SQL injection via dynamic string interpolation/concatenation.
//! - `SEC002`: Hardcoded sensitive credentials or API keys.
//! - `SEC003`: Insecure PRNG (`Math.random`) usage for tokens, keys, passwords or cryptography.
//! - `LINT001`: Variable declared with `let mut` but never reassigned.
//! - `LINT002`: Exhaustive pattern match with empty arms.

use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::HashMap;

/// Severity level of a lint diagnostic finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    Warning,
    Error,
}

impl std::fmt::Display for LintSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintSeverity::Warning => write!(f, "WARNING"),
            LintSeverity::Error => write!(f, "ERROR"),
        }
    }
}

/// An individual issue detected by the linter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintDiagnostic {
    /// Diagnostic code (e.g. `SEC001`, `LINT001`).
    pub code: &'static str,
    /// Severity level.
    pub severity: LintSeverity,
    /// Human-readable explanation and suggestion.
    pub message: String,
}

impl std::fmt::Display for LintDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} - {}] {}", self.severity, self.code, self.message)
    }
}

/// AST visitor implementation executing lint rules.
#[derive(Default)]
pub struct Linter {
    /// Discovered diagnostics.
    pub diagnostics: Vec<LintDiagnostic>,
    /// Map of variables declared with `let mut` to whether they were ever reassigned.
    mut_vars: HashMap<String, bool>,
}

impl Linter {
    /// Creates a new linter instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyzes an AST Module and returns all discovered diagnostics.
    pub fn analyze(&mut self, module: &Module) -> Vec<LintDiagnostic> {
        self.diagnostics.clear();
        self.mut_vars.clear();

        self.visit_module(module);

        // Check for variables declared with `let mut` that were never reassigned
        for (name, assigned) in &self.mut_vars {
            if !*assigned && !name.starts_with('_') {
                self.diagnostics.push(LintDiagnostic {
                    code: "LINT001",
                    severity: LintSeverity::Warning,
                    message: format!(
                        "Variable '{}' is declared with 'let mut' but is never reassigned; consider using 'let'.",
                        name
                    ),
                });
            }
        }

        self.diagnostics.clone()
    }

    fn check_sql_injection_template(&mut self, segments: &[TemplateSegment]) {
        let has_expr = segments.iter().any(|s| matches!(s, TemplateSegment::Expr(_)));
        if !has_expr {
            return;
        }

        for seg in segments {
            if let TemplateSegment::Text(text) = seg {
                if contains_sql_keyword(text) {
                    self.diagnostics.push(LintDiagnostic {
                        code: "SEC001",
                        severity: LintSeverity::Error,
                        message: "Potential SQL injection risk: raw string interpolation detected with SQL query statements. Use parameterized query placeholders instead.".to_string(),
                    });
                    break;
                }
            }
        }
    }

    fn check_sql_injection_binary(&mut self, left: &Expr, right: &Expr) {
        let left_sql = match left {
            Expr::Literal(Literal::String(s)) => contains_sql_keyword(s),
            _ => false,
        };
        let right_sql = match right {
            Expr::Literal(Literal::String(s)) => contains_sql_keyword(s),
            _ => false,
        };

        if (left_sql && !matches!(right, Expr::Literal(Literal::String(_))))
            || (right_sql && !matches!(left, Expr::Literal(Literal::String(_))))
        {
            self.diagnostics.push(LintDiagnostic {
                code: "SEC001",
                severity: LintSeverity::Error,
                message: "Potential SQL injection risk: dynamic string concatenation detected with SQL query syntax. Use parameterized query placeholders instead.".to_string(),
            });
        }
    }

    fn check_hardcoded_secret(&mut self, name: &str, value: &Expr) {
        let name_lower = name.to_lowercase();
        let sensitive_keywords = [
            "password", "api_key", "apikey", "secret", "auth_token", "jwt_secret", "access_token",
            "private_key",
        ];

        if sensitive_keywords.iter().any(|k| name_lower.contains(k)) {
            if let Expr::Literal(Literal::String(s)) = value {
                let s_lower = s.to_lowercase();
                let is_placeholder = s.is_empty()
                    || s_lower.contains("dummy")
                    || s_lower.contains("test")
                    || s_lower.contains("example")
                    || s_lower.contains("placeholder")
                    || s_lower.contains("changeme")
                    || s_lower.contains("localhost");

                if !is_placeholder && s.len() >= 8 {
                    self.diagnostics.push(LintDiagnostic {
                        code: "SEC002",
                        severity: LintSeverity::Warning,
                        message: format!(
                            "Possible hardcoded credential or secret detected in variable '{}'. Consider loading from environment variables or a secret vault.",
                            name
                        ),
                    });
                }
            }
        }
    }

    fn check_insecure_random_usage(&mut self, name: &str, value: &Expr) {
        let name_lower = name.to_lowercase();
        let sensitive_token_indicators = [
            "token", "secret", "salt", "iv", "nonce", "key", "password", "session", "auth",
        ];

        if sensitive_token_indicators.iter().any(|k| name_lower.contains(k)) {
            let mut uses_math_random = false;
            if let Expr::FunctionCall { callee, .. } = value {
                if let Expr::MemberAccess { object, member } = callee.as_ref() {
                    if let Expr::Identifier(obj_id) = object.as_ref() {
                        if obj_id == "Math" && member == "random" {
                            uses_math_random = true;
                        }
                    }
                }
            }

            if uses_math_random {
                self.diagnostics.push(LintDiagnostic {
                    code: "SEC003",
                    severity: LintSeverity::Error,
                    message: format!(
                        "Insecure randomness detected in security-sensitive variable '{}'. 'Math.random()' is pseudo-random and predictable. Use 'crypto.randomBytes()' or 'crypto.randomUUID()' instead.",
                        name
                    ),
                });
            }
        }
    }
}

impl AstVisitor for Linter {
    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let {
                name,
                is_mut,
                value,
                ..
            } => {
                if *is_mut {
                    self.mut_vars.insert(name.clone(), false);
                }
                self.check_hardcoded_secret(name, value);
                self.check_insecure_random_usage(name, value);
            }
            Statement::Assign { target, .. } => {
                if let Expr::Identifier(name) = target {
                    if let Some(entry) = self.mut_vars.get_mut(name) {
                        *entry = true;
                    }
                }
            }
            _ => {}
        }
        walk_statement(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(Literal::TemplateString(segments)) => {
                self.check_sql_injection_template(segments);
            }
            Expr::Binary {
                op: BinOp::Add,
                left,
                right,
            } => {
                self.check_sql_injection_binary(left, right);
            }
            Expr::Match { arms, .. } => {
                if arms.is_empty() {
                    self.diagnostics.push(LintDiagnostic {
                        code: "LINT002",
                        severity: LintSeverity::Warning,
                        message: "Match expression has no arms and is redundant.".to_string(),
                    });
                }
            }
            _ => {}
        }
        walk_expr(self, expr);
    }
}

fn contains_sql_keyword(s: &str) -> bool {
    let upper = s.to_uppercase();
    upper.contains("SELECT ")
        || upper.contains("INSERT INTO ")
        || upper.contains("UPDATE ")
        || upper.contains("DELETE FROM ")
        || upper.contains("DROP TABLE ")
}

/// Parses and lints Aura source code, returning diagnostics or a parser error.
pub fn lint_source(source: &str) -> Result<Vec<LintDiagnostic>, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module()?;
    let mut linter = Linter::new();
    Ok(linter.analyze(&module))
}
