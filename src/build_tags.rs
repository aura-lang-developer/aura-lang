//! Build tags parser and evaluator for Aura Lang (supports //aura:build and //go:build syntax)

use std::collections::HashSet;

/// Evaluates if an Aura source file should be included in the build based on build tags.
pub fn should_build_source(source: &str, custom_tags: &[&str]) -> bool {
    let active_tags = get_active_tags(custom_tags);
    should_build_with_exact_tags(source, &active_tags)
}

/// Evaluates if an Aura source file should be included based on an exact set of active tags.
pub fn should_build_with_exact_tags(source: &str, active_tags: &HashSet<String>) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with("//") {
            // Build tags must appear at the top of the file before code
            break;
        }

        if let Some(expr_str) = trimmed
            .strip_prefix("//aura:build")
            .or_else(|| trimmed.strip_prefix("//go:build"))
        {
            let expr = expr_str.trim();
            if !expr.is_empty() {
                if !eval_tag_expression(expr, active_tags) {
                    return false;
                }
            }
        } else if let Some(expr_str) = trimmed.strip_prefix("// +build") {
            let expr = expr_str.trim();
            if !expr.is_empty() {
                if !eval_plus_build(expr, active_tags) {
                    return false;
                }
            }
        }
    }

    true
}

/// Returns the set of default and user-specified tags.
pub fn get_active_tags(custom_tags: &[&str]) -> HashSet<String> {
    let mut tags = HashSet::new();

    // OS tags
    #[cfg(target_os = "macos")]
    {
        tags.insert("darwin".to_string());
        tags.insert("macos".to_string());
        tags.insert("unix".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        tags.insert("linux".to_string());
        tags.insert("unix".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        tags.insert("windows".to_string());
    }

    // Architecture tags
    #[cfg(target_arch = "x86_64")]
    {
        tags.insert("amd64".to_string());
        tags.insert("x86_64".to_string());
    }
    #[cfg(target_arch = "aarch64")]
    {
        tags.insert("arm64".to_string());
        tags.insert("aarch64".to_string());
    }

    // Aura environment tags
    tags.insert("aura".to_string());
    tags.insert("es6".to_string());
    tags.insert("node".to_string());

    for tag in custom_tags {
        tags.insert(tag.trim().to_string());
    }

    tags
}

fn split_top_level<'a>(expr: &'a str, op: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut last = 0;
    let bytes = expr.as_bytes();
    let op_bytes = op.as_bytes();

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            depth += 1;
            i += 1;
        } else if bytes[i] == b')' {
            if depth > 0 {
                depth -= 1;
            }
            i += 1;
        } else if depth == 0
            && i + op_bytes.len() <= bytes.len()
            && &bytes[i..i + op_bytes.len()] == op_bytes
        {
            parts.push(expr[last..i].trim());
            i += op_bytes.len();
            last = i;
        } else {
            i += 1;
        }
    }
    if last < expr.len() {
        parts.push(expr[last..].trim());
    }
    parts
}

/// Evaluates a boolean expression (e.g. `(linux && amd64) || darwin || !windows`)
fn eval_tag_expression(expr: &str, active_tags: &HashSet<String>) -> bool {
    let mut expr = expr.trim();
    if expr.is_empty() {
        return true;
    }

    // Strip enclosing balanced parentheses
    while expr.starts_with('(') && expr.ends_with(')') {
        let mut depth = 0;
        let mut balanced_all = true;
        for (idx, ch) in expr.char_indices() {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 && idx < expr.len() - 1 {
                    balanced_all = false;
                    break;
                }
            }
        }
        if balanced_all {
            expr = expr[1..expr.len() - 1].trim();
        } else {
            break;
        }
    }

    let or_parts = split_top_level(expr, "||");
    if or_parts.len() > 1 {
        for part in or_parts {
            if eval_tag_expression(part, active_tags) {
                return true;
            }
        }
        return false;
    }

    let and_parts = split_top_level(expr, "&&");
    if and_parts.len() > 1 {
        for part in and_parts {
            if !eval_tag_expression(part, active_tags) {
                return false;
            }
        }
        return true;
    }

    if let Some(inner) = expr.strip_prefix('!') {
        return !eval_tag_expression(inner.trim(), active_tags);
    }

    active_tags.contains(expr)
}

/// Evaluates old-style `// +build tag1,tag2 tag3` (spaces are OR, commas are AND)
fn eval_plus_build(line: &str, active_tags: &HashSet<String>) -> bool {
    let or_clauses: Vec<&str> = line.split_whitespace().collect();
    if or_clauses.is_empty() {
        return true;
    }

    for or_clause in or_clauses {
        let and_clauses: Vec<&str> = or_clause.split(',').collect();
        let mut and_match = true;
        for and_tag in and_clauses {
            let tag = and_tag.trim();
            if let Some(negated) = tag.strip_prefix('!') {
                if active_tags.contains(negated) {
                    and_match = false;
                    break;
                }
            } else if !active_tags.contains(tag) {
                and_match = false;
                break;
            }
        }
        if and_match {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tags_eval() {
        let tags: HashSet<String> = vec![
            "darwin".to_string(),
            "arm64".to_string(),
            "prod".to_string(),
        ]
        .into_iter()
        .collect();

        assert!(eval_tag_expression("darwin", &tags));
        assert!(!eval_tag_expression("linux", &tags));
        assert!(eval_tag_expression("!linux", &tags));
        assert!(eval_tag_expression("darwin && arm64", &tags));
        assert!(eval_tag_expression("linux || darwin", &tags));
        assert!(eval_tag_expression(
            "(linux && amd64) || (darwin && arm64)",
            &tags
        ));
        assert!(!eval_tag_expression("prod && staging", &tags));
    }

    #[test]
    fn test_should_build_source() {
        let source_included = "//aura:build darwin || linux\nlet x = 10;";
        assert!(should_build_source(source_included, &[]));

        let source_excluded = "//go:build windows && !darwin\nlet x = 10;";
        assert!(!should_build_source(source_excluded, &[]));
    }
}
