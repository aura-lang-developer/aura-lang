//! Desugaring and Canonical AST Transformation Pass.
//!
//! This module lowers high-level syntactic sugar (e.g. `Expr::Pipeline`,
//! pattern matching helpers, etc.) into canonical, primitive AST structures
//! before code generation.
//!
//! By running this pass after type checking and prior to backends (`codegen.rs`,
//! `codegen_go.rs`, `codegen_cranelift.rs`), all backends share identical
//! evaluation semantics, TCO behavior, and pipeline expansions without duplicating logic.

use crate::ast::*;

/// Transforms a typed AST module into its canonical, desugared form.
pub fn desugar_module(mut module: Module) -> Module {
    for item in &mut module.items {
        desugar_item(item);
    }
    module
}

/// Recursively desugars an item within a module.
pub fn desugar_item(item: &mut Item) {
    match item {
        Item::Function(func) => {
            desugar_expr(&mut func.body);
        }
        Item::Statement(stmt) => {
            desugar_statement(stmt);
        }
        Item::TypeAlias(_)
        | Item::SumType(_)
        | Item::Interface(_)
        | Item::Import(_)
        | Item::Extern(_) => {}
    }
}

/// Desugars statements in-place.
pub fn desugar_statement(stmt: &mut Statement) {
    match stmt {
        Statement::Let { value, .. } => {
            desugar_expr(value);
        }
        Statement::LetPattern { value, .. } => {
            desugar_expr(value);
        }
        Statement::Assign { target, value } => {
            desugar_expr(target);
            desugar_expr(value);
        }
        Statement::Defer(expr) | Statement::ErrDefer(expr) => {
            desugar_expr(expr);
        }
        Statement::Expr(expr) => {
            desugar_expr(expr);
        }
        Statement::Return(Some(expr)) => {
            desugar_expr(expr);
        }
        Statement::Return(None) => {}
        Statement::Guard {
            condition,
            else_body,
        } => {
            desugar_expr(condition);
            for s in else_body {
                desugar_statement(s);
            }
        }
    }
}

/// Recursively desugars an expression in-place.
pub fn desugar_expr(expr: &mut Expr) {
    // 1. Desugar children first (bottom-up transformation)
    match expr {
        Expr::Literal(lit) => {
            if let Literal::TemplateString(segments) = lit {
                for seg in segments {
                    if let TemplateSegment::Expr(e) = seg {
                        desugar_expr(e);
                    }
                }
            }
        }
        Expr::Identifier(_) | Expr::Placeholder | Expr::Recover => {}
        Expr::Binary { left, right, .. } => {
            desugar_expr(left);
            desugar_expr(right);
        }
        Expr::Unary { expr: inner, .. } => {
            desugar_expr(inner);
        }
        Expr::Pipeline { left, right } => {
            desugar_expr(left);
            desugar_expr(right);
        }
        Expr::FunctionCall { callee, args } => {
            desugar_expr(callee);
            for arg in args {
                desugar_expr(arg);
            }
        }
        Expr::MemberAccess { object, .. } => {
            desugar_expr(object);
        }
        Expr::IndexAccess { object, index } => {
            desugar_expr(object);
            desugar_expr(index);
        }
        Expr::SliceAccess {
            object,
            low,
            high,
            max,
        } => {
            desugar_expr(object);
            if let Some(l) = low {
                desugar_expr(l);
            }
            if let Some(h) = high {
                desugar_expr(h);
            }
            if let Some(m) = max {
                desugar_expr(m);
            }
        }
        Expr::RecordLiteral { fields, spread } => {
            for (_, val) in fields {
                desugar_expr(val);
            }
            if let Some(s) = spread {
                desugar_expr(s);
            }
        }
        Expr::TupleLiteral(elements) | Expr::ListLiteral(elements) => {
            for el in elements {
                desugar_expr(el);
            }
        }
        Expr::Lambda { body, .. } => {
            desugar_expr(body);
        }
        Expr::Match { subject, arms } => {
            desugar_expr(subject);
            for arm in arms {
                if let Some(ref mut guard) = arm.guard {
                    desugar_expr(guard);
                }
                desugar_expr(&mut arm.body);
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            desugar_expr(condition);
            desugar_expr(then_branch);
            if let Some(el) = else_branch {
                desugar_expr(el);
            }
        }
        Expr::Block(stmts) => {
            for stmt in stmts {
                desugar_statement(stmt);
            }
        }
        Expr::Async(inner)
        | Expr::Await(inner)
        | Expr::Spawn(inner)
        | Expr::ChanRecv(inner)
        | Expr::Try(inner)
        | Expr::AddressOf(inner)
        | Expr::Deref(inner)
        | Expr::Panic(inner) => {
            desugar_expr(inner);
        }
        Expr::ChanSend { channel, value } => {
            desugar_expr(channel);
            desugar_expr(value);
        }
        Expr::Select { arms, default } => {
            for arm in arms {
                match &mut arm.kind {
                    SelectArmKind::Recv { channel, .. } => desugar_expr(channel),
                    SelectArmKind::Send { channel, value } => {
                        desugar_expr(channel);
                        desugar_expr(value);
                    }
                    SelectArmKind::Timeout(duration) => desugar_expr(duration),
                }
                desugar_expr(&mut arm.body);
            }
            if let Some(def_body) = default {
                desugar_expr(def_body);
            }
        }
        Expr::ConstructorCall { args, .. } => {
            for arg in args {
                desugar_expr(arg);
            }
        }
        Expr::While { condition, body, .. } => {
            desugar_expr(condition);
            desugar_expr(body);
        }
        Expr::ForIn { iterable, body, .. } => {
            desugar_expr(iterable);
            desugar_expr(body);
        }
        Expr::Break(_) | Expr::Continue(_) | Expr::Embed { .. } => {}
        Expr::MapLiteral(entries) => {
            for (k, v) in entries {
                desugar_expr(k);
                desugar_expr(v);
            }
        }
        Expr::SetLiteral(elements) => {
            for el in elements {
                desugar_expr(el);
            }
        }
    }

    // 2. Perform node-level rewrite for syntactic sugar
    if matches!(expr, Expr::Pipeline { .. }) {
        let dummy = Expr::Placeholder;
        let taken = std::mem::replace(expr, dummy);
        if let Expr::Pipeline { left, right } = taken {
            *expr = desugar_pipeline(*left, *right);
        }
    }
}

/// Canonical rewrite for pipeline expressions: `left |> right`.
///
/// If `right` is `Expr::FunctionCall { callee, args }`:
///   - If `args` contains an explicit placeholder `_`, replace the placeholder with `left`.
///   - Otherwise, prepend `left` as the first argument: `callee(left, ...args)`.
///
/// If `right` is any other expression `f`:
///   - Lower directly to `f(left)`.
pub fn desugar_pipeline(left: Expr, right: Expr) -> Expr {
    match right {
        Expr::FunctionCall { callee, mut args } => {
            let mut substituted = false;
            for arg in &mut args {
                if matches!(arg, Expr::Placeholder) {
                    *arg = left.clone();
                    substituted = true;
                    break;
                }
            }
            if !substituted {
                args.insert(0, left);
            }
            Expr::FunctionCall { callee, args }
        }
        other => Expr::FunctionCall {
            callee: Box::new(other),
            args: vec![left],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desugar_pipeline_simple() {
        // x |> f  =>  f(x)
        let mut expr = Expr::Pipeline {
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::Identifier("f".to_string())),
        };
        desugar_expr(&mut expr);

        match expr {
            Expr::FunctionCall { callee, args } => {
                assert!(matches!(*callee, Expr::Identifier(ref name) if name == "f"));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expr::Identifier(ref name) if name == "x"));
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_desugar_pipeline_call_prepend() {
        // x |> f(y, z)  =>  f(x, y, z)
        let mut expr = Expr::Pipeline {
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::FunctionCall {
                callee: Box::new(Expr::Identifier("f".to_string())),
                args: vec![
                    Expr::Identifier("y".to_string()),
                    Expr::Identifier("z".to_string()),
                ],
            }),
        };
        desugar_expr(&mut expr);

        match expr {
            Expr::FunctionCall { callee, args } => {
                assert!(matches!(*callee, Expr::Identifier(ref name) if name == "f"));
                assert_eq!(args.len(), 3);
                assert!(matches!(args[0], Expr::Identifier(ref name) if name == "x"));
                assert!(matches!(args[1], Expr::Identifier(ref name) if name == "y"));
                assert!(matches!(args[2], Expr::Identifier(ref name) if name == "z"));
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_desugar_pipeline_placeholder() {
        // x |> f(y, _)  =>  f(y, x)
        let mut expr = Expr::Pipeline {
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::FunctionCall {
                callee: Box::new(Expr::Identifier("f".to_string())),
                args: vec![Expr::Identifier("y".to_string()), Expr::Placeholder],
            }),
        };
        desugar_expr(&mut expr);

        match expr {
            Expr::FunctionCall { callee, args } => {
                assert!(matches!(*callee, Expr::Identifier(ref name) if name == "f"));
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Expr::Identifier(ref name) if name == "y"));
                assert!(matches!(args[1], Expr::Identifier(ref name) if name == "x"));
            }
            _ => panic!("Expected FunctionCall"),
        }
    }
}
