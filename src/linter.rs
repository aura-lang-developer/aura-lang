//! Static code analysis and security linter for Aura Language.
//!
//! Provides automated security and code quality checks:
//! - `SEC001`: Potential SQL injection via dynamic string interpolation/concatenation.
//! - `SEC002`: Hardcoded sensitive credentials or API keys.
//! - `SEC003`: Insecure PRNG (`Math.random`) usage for tokens, keys, passwords or cryptography.
//! - `LINT001`: Variable declared with `let mut` but never reassigned.
//! - `LINT002`: Exhaustive pattern match with empty arms.
//! - `LINT004`: Redundant `export` keyword on PascalCase symbol (symbols starting with an uppercase letter are already exported by Go-style convention).
//! - `LINT005`: Deprecated concurrency synonym (`go` or `routine`); use canonical `spawn`.
//! - `LINT006`: Deprecated `trait` keyword; use canonical `interface` for structural duck typing.

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
    /// Map of functions marked with `#[must_use]` or returning `Result`/`Option`.
    must_use_functions: HashMap<String, Option<String>>,
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
        self.must_use_functions.clear();

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
        let has_expr = segments
            .iter()
            .any(|s| matches!(s, TemplateSegment::Expr(_)));
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
            "password",
            "api_key",
            "apikey",
            "secret",
            "auth_token",
            "jwt_secret",
            "access_token",
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

        if sensitive_token_indicators
            .iter()
            .any(|k| name_lower.contains(k))
        {
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
    fn visit_module(&mut self, module: &Module) {
        for item in &module.items {
            match item {
                Item::Function(f) => {
                    let is_ret_must_use = matches!(
                        &f.return_type,
                        Some(Type::Named { name, .. }) if name == "Result" || name == "Option"
                    );
                    if f.is_must_use || is_ret_must_use {
                        self.must_use_functions
                            .insert(f.name.clone(), f.must_use_message.clone());
                    }
                    if f.has_explicit_export && f.name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        self.diagnostics.push(LintDiagnostic {
                            code: "LINT004",
                            severity: LintSeverity::Warning,
                            message: format!(
                                "Redundant 'export' keyword on PascalCase function '{}'. PascalCase symbols are automatically exported by Go-style visibility convention.",
                                f.name
                            ),
                        });
                    }
                }
                Item::TypeAlias(t) => {
                    if t.has_explicit_export && t.name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        self.diagnostics.push(LintDiagnostic {
                            code: "LINT004",
                            severity: LintSeverity::Warning,
                            message: format!(
                                "Redundant 'export' keyword on PascalCase type '{}'. PascalCase symbols are automatically exported by Go-style visibility convention.",
                                t.name
                            ),
                        });
                    }
                }
                Item::SumType(s) => {
                    if s.has_explicit_export && s.name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        self.diagnostics.push(LintDiagnostic {
                            code: "LINT004",
                            severity: LintSeverity::Warning,
                            message: format!(
                                "Redundant 'export' keyword on PascalCase sum type '{}'. PascalCase symbols are automatically exported by Go-style visibility convention.",
                                s.name
                            ),
                        });
                    }
                }
                Item::Interface(i) => {
                    if i.has_explicit_export && i.name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        self.diagnostics.push(LintDiagnostic {
                            code: "LINT004",
                            severity: LintSeverity::Warning,
                            message: format!(
                                "Redundant 'export' keyword on PascalCase interface '{}'. PascalCase symbols are automatically exported by Go-style visibility convention.",
                                i.name
                            ),
                        });
                    }
                }
                _ => {}
            }
        }
        walk_module(self, module);
    }

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
            Statement::Expr(expr) => {
                let callee_name = match expr {
                    Expr::FunctionCall { callee, .. } => match callee.as_ref() {
                        Expr::Identifier(name) => Some(name.clone()),
                        Expr::MemberAccess { member, .. } => Some(member.clone()),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(name) = callee_name {
                    if let Some(msg_opt) = self.must_use_functions.get(&name) {
                        let message = match msg_opt {
                            Some(m) => format!(
                                "Unused return value from #[must_use] function '{}': {}",
                                name, m
                            ),
                            None => format!(
                                "Unused return value from #[must_use] function '{}'; the result must be handled.",
                                name
                            ),
                        };
                        self.diagnostics.push(LintDiagnostic {
                            code: "LINT003",
                            severity: LintSeverity::Warning,
                            message,
                        });
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
    let mut parser = Parser::new(tokens.clone());
    let module = parser.parse_module()?;
    let mut linter = Linter::new();
    let mut diags = linter.analyze(&module);

    // Syntax-level deprecation checks for non-canonical synonyms
    for token in &tokens {
        match token.kind {
            crate::lexer::TokenKind::Go => {
                diags.push(LintDiagnostic {
                    code: "LINT005",
                    severity: LintSeverity::Warning,
                    message: format!(
                        "line {}:{}: 'go' is deprecated for CSP concurrency; use canonical 'spawn' instead.",
                        token.span.line, token.span.column
                    ),
                });
            }
            crate::lexer::TokenKind::Routine => {
                diags.push(LintDiagnostic {
                    code: "LINT005",
                    severity: LintSeverity::Warning,
                    message: format!(
                        "line {}:{}: 'routine' is deprecated for CSP concurrency; use canonical 'spawn' instead.",
                        token.span.line, token.span.column
                    ),
                });
            }
            crate::lexer::TokenKind::Trait => {
                diags.push(LintDiagnostic {
                    code: "LINT006",
                    severity: LintSeverity::Warning,
                    message: format!(
                        "line {}:{}: 'trait' is deprecated; Aura uses structural duck typing, use canonical 'interface' instead.",
                        token.span.line, token.span.column
                    ),
                });
            }
            _ => {}
        }
    }

    Ok(diags)
}

/// Automatically fixes lintable issues in source code (e.g. `let mut` -> `let`, deprecations).
pub fn fix_source(source: &str) -> (String, usize) {
    let mut lexer = Lexer::new(source);
    let Ok(tokens) = lexer.tokenize() else {
        return (source.to_string(), 0);
    };
    let mut parser = Parser::new(tokens.clone());
    let Ok(module) = parser.parse_module() else {
        return (source.to_string(), 0);
    };
    let mut linter = Linter::new();
    let diags = linter.analyze(&module);

    let mut unneeded_muts = Vec::new();
    let mut redundant_export_symbols = Vec::new();
    for d in diags {
        if d.code == "LINT001" {
            if let Some(start) = d.message.find('\'') {
                if let Some(end) = d.message[start + 1..].find('\'') {
                    let var_name = &d.message[start + 1..start + 1 + end];
                    unneeded_muts.push(var_name.to_string());
                }
            }
        } else if d.code == "LINT004" {
            if let Some(start) = d.message.find('\'') {
                if let Some(second) = d.message[start + 1..].find('\'') {
                    let rest = &d.message[start + 1 + second + 1..];
                    if let Some(third) = rest.find('\'') {
                        if let Some(fourth) = rest[third + 1..].find('\'') {
                            let sym_name = &rest[third + 1..third + 1 + fourth];
                            redundant_export_symbols.push(sym_name.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut has_deprecated_tokens = false;
    for token in &tokens {
        if matches!(
            token.kind,
            crate::lexer::TokenKind::Go
                | crate::lexer::TokenKind::Routine
                | crate::lexer::TokenKind::Trait
        ) {
            has_deprecated_tokens = true;
            break;
        }
    }

    if unneeded_muts.is_empty() && redundant_export_symbols.is_empty() && !has_deprecated_tokens {
        return (source.to_string(), 0);
    }

    let mut fixed = source.to_string();
    let mut count = 0;
    for var in unneeded_muts {
        let pattern = format!("let mut {}", var);
        let replacement = format!("let {}", var);
        if fixed.contains(&pattern) {
            fixed = fixed.replace(&pattern, &replacement);
            count += 1;
        }
    }

    for sym in redundant_export_symbols {
        // Match patterns like "export fn Symbol", "export struct Symbol", "export type Symbol", "export interface Symbol"
        for kw in &["fn", "struct", "type", "interface", "packed struct"] {
            let pattern = format!("export {} {}", kw, sym);
            let replacement = format!("{} {}", kw, sym);
            if fixed.contains(&pattern) {
                fixed = fixed.replace(&pattern, &replacement);
                count += 1;
            }
        }
    }

    // Auto-fix deprecated synonyms to canonical keywords
    if has_deprecated_tokens {
        // Fix 'routine {' and 'routine '
        // Replace keyword 'routine' by 'spawn' when followed by whitespace or '{'
        for token in tokens.iter().rev() {
            match token.kind {
                crate::lexer::TokenKind::Routine => {
                    let start = token.span.start;
                    let end = token.span.end;
                    if start < fixed.len() && end <= fixed.len() && &fixed[start..end] == "routine" {
                        fixed.replace_range(start..end, "spawn");
                        count += 1;
                    }
                }
                crate::lexer::TokenKind::Go => {
                    let start = token.span.start;
                    let end = token.span.end;
                    if start < fixed.len() && end <= fixed.len() && &fixed[start..end] == "go" {
                        fixed.replace_range(start..end, "spawn");
                        count += 1;
                    }
                }
                crate::lexer::TokenKind::Trait => {
                    let start = token.span.start;
                    let end = token.span.end;
                    if start < fixed.len() && end <= fixed.len() && &fixed[start..end] == "trait" {
                        fixed.replace_range(start..end, "interface");
                        count += 1;
                    }
                }
                _ => {}
            }
        }
    }

    (fixed, count)
}

/// Reads, auto-fixes, and saves a file in-place if issues were repaired.
pub fn fix_file(path: &std::path::Path) -> Result<usize, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (fixed, count) = fix_source(&content);
    if count > 0 {
        std::fs::write(path, fixed).map_err(|e| e.to_string())?;
    }
    Ok(count)
}
