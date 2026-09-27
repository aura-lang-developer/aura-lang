//! AST (Abstract Syntax Tree) definitions for the Aura language.
//!
//! Represents all syntactic constructs of Aura programs, including modules,
//! declarations, statements, expressions, patterns, and type annotations.

/// A compilation unit or module in the Aura language.
///
/// Contains optional module namespace declaration and a sequence of top-level items.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// Optional hierarchical module identifier (e.g. `module Aura.Compiler.Ast`).
    pub name: Option<String>,
    /// List of declarations and statements defined at the top-level of the module.
    pub items: Vec<Item>,
}

/// Top-level declaration or statement item in an Aura module.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Function declaration (standalone or struct receiver method).
    Function(FunctionDecl),
    /// Type alias declaration (`type Point = { x: Float, y: Float };`).
    TypeAlias(TypeAliasDecl),
    /// Algebraic Data Type (Sum type / tagged union) declaration.
    SumType(SumTypeDecl),
    /// Structural interface declaration with duck-typing support.
    Interface(InterfaceDecl),
    /// Module import declaration (`import { x, y } from "./math";`).
    Import(ImportDecl),
    /// External C ABI function interface (FFI) declaration.
    Extern(ExternDecl),
    /// Top-level executable statement.
    Statement(Statement),
}

/// Structural interface declaration supporting implicit implementation (Go-style duck typing).
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDecl {
    /// Interface name (e.g. `Reader`, `Writer`).
    pub name: String,
    /// Indicates whether the interface is visible outside its defining module.
    pub is_exported: bool,
    /// Generic type parameters (e.g. `['T', 'U']`).
    pub type_params: Vec<String>,
    /// Method signatures required by this interface.
    pub methods: Vec<InterfaceMethod>,
}

/// An individual method signature specified within an interface.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethod {
    /// Method name (e.g. `read`, `write`).
    pub name: String,
    /// Parameter list required for calling this method.
    pub params: Vec<Param>,
    /// Expected return type of the method.
    pub return_type: Type,
}

/// Method receiver binding a function to a struct/record type (Go receiver model).
#[derive(Debug, Clone, PartialEq)]
pub struct Receiver {
    /// Receiver variable name inside the method body (e.g. `self` or `u`).
    pub name: String,
    /// Target type on which this receiver method is defined (e.g. `User`).
    pub target_type: Type,
}

impl std::fmt::Display for Receiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.target_type)
    }
}

/// Function declaration in Aura.
///
/// Supports standalone functions, receiver methods, async functions, generics,
/// parameter defaults, and expression bodies.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    /// Function name identifier (e.g. `add`, `serve`).
    pub name: String,
    /// Optional struct receiver if defined as a method (`fn (u: User) getName(): String`).
    pub receiver: Option<Receiver>,
    /// Whether the function is asynchronous, returning a `Task<T, E>`.
    pub is_async: bool,
    /// Whether the function is public and exported to module consumers.
    pub is_exported: bool,
    /// Generic type parameters declared for this function (e.g. `<T, E>`).
    pub type_params: Vec<String>,
    /// List of formal parameters accepted by the function.
    pub params: Vec<Param>,
    /// Explicit return type annotation, or inferred if omitted.
    pub return_type: Option<Type>,
    /// Body expression of the function (often a `Block` or single expression).
    pub body: Expr,
}

/// A formal parameter in a function or lambda declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// Parameter identifier name (e.g. `x`, `req`).
    pub name: String,
    /// Optional explicit type annotation (e.g. `Int`, `Request`).
    pub type_annotation: Option<Type>,
    /// Optional default fallback expression if argument is omitted by caller.
    pub default_value: Option<Expr>,
}

/// Type alias declaration assigning a new name to an existing or composite type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDecl {
    /// Name of the defined type alias (e.g. `UserId`, `Point`).
    pub name: String,
    /// Whether this type alias is visible to importing modules.
    pub is_exported: bool,
    /// Whether the underlying struct has zero padding and packed binary alignment (Zig model).
    pub is_packed: bool,
    /// Generic type parameters (e.g. `<T>`).
    pub type_params: Vec<String>,
    /// Underlying target type representation.
    pub target: Type,
}

/// Algebraic Data Type (Sum type / tagged union) declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct SumTypeDecl {
    /// Name of the sum type (e.g. `Option`, `Result`, `Shape`).
    pub name: String,
    /// Whether the sum type is exported from the module.
    pub is_exported: bool,
    /// Generic type parameters (e.g. `<T, E>`).
    pub type_params: Vec<String>,
    /// List of mutually exclusive variants comprising this sum type.
    pub variants: Vec<Variant>,
}

/// An individual variant arm of a sum type.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    /// Name identifier of the variant (e.g. `Some`, `None`, `Circle`).
    pub name: String,
    /// Associated payload fields stored by this variant.
    pub fields: VariantFields,
}

/// Data payload fields associated with a sum type variant.
#[derive(Debug, Clone, PartialEq)]
pub enum VariantFields {
    /// Unit variant with no attached payload data (e.g. `None`, `Point`).
    Unit,
    /// Positional tuple fields attached to variant (e.g. `Circle(Float)`).
    Tuple(Vec<Type>),
    /// Named record fields attached to variant (e.g. `Rectangle { w: Float, h: Float }`).
    Record(Vec<(String, Type)>),
}

/// Module import declaration importing symbols from another file or package.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    /// Import source path or package identifier (e.g. `"./math"`, `"net/http"`).
    pub source: String,
    /// Individual imported symbols and their optional local aliases.
    pub items: Vec<ImportItem>,
}

/// An individual imported symbol within an import declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportItem {
    /// Name of the symbol in the source module (e.g. `Server`).
    pub name: String,
    /// Optional alias name for the imported symbol in the local module (e.g. `HttpServer`).
    pub alias: Option<String>,
}

/// External C ABI / foreign function interface (FFI) declaration block.
#[derive(Debug, Clone, PartialEq)]
pub struct ExternDecl {
    /// Shared library or C module name (e.g. `"c"`, `"sqlite3"`).
    pub module_name: String,
    /// Foreign function signatures declared within this external module.
    pub functions: Vec<ExternFunction>,
}

/// An individual foreign C ABI function signature.
#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    /// Foreign C function symbol name (e.g. `puts`, `malloc`).
    pub name: String,
    /// Generic type parameters.
    pub type_params: Vec<String>,
    /// Formal parameters required by the external function.
    pub params: Vec<Param>,
    /// Return type produced by the external C function.
    pub return_type: Type,
}

// -----------------------------------------------------------------------------
// TYPES
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named {
        name: String,
        type_args: Vec<Type>,
    },
    Record(Vec<(String, Type)>),
    Tuple(Vec<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Pointer(Box<Type>),
    SendChannel(Box<Type>),
    RecvChannel(Box<Type>),
    Unit,
    TypeVar(String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named { name, type_args } => {
                if type_args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    let args: Vec<String> = type_args.iter().map(|t| format!("{}", t)).collect();
                    write!(f, "{}<{}>", name, args.join(", "))
                }
            }
            Type::Record(fields) => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{ {} }}", fs.join(", "))
            }
            Type::Tuple(elems) => {
                let es: Vec<String> = elems.iter().map(|e| format!("{}", e)).collect();
                write!(f, "({})", es.join(", "))
            }
            Type::Function {
                params,
                return_type,
            } => {
                let ps: Vec<String> = params.iter().map(|p| format!("{}", p)).collect();
                write!(f, "fn({}) -> {}", ps.join(", "), return_type)
            }
            Type::Pointer(inner) => write!(f, "*{}", inner),
            Type::SendChannel(inner) => write!(f, "chan<- {}", inner),
            Type::RecvChannel(inner) => write!(f, "<-chan {}", inner),
            Type::Unit => write!(f, "()"),
            Type::TypeVar(tv) => write!(f, "{}", tv),
        }
    }
}

// -----------------------------------------------------------------------------
// STATEMENTS
// -----------------------------------------------------------------------------

/// Statements in Aura that execute sequentially or control program execution.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Variable binding statement (`let x = 10;` or `let mut counter: Int = 0;`).
    Let {
        /// Bound variable identifier name.
        name: String,
        /// Indicates whether the variable can be reassigned (`mut`). Immutable by default.
        is_mut: bool,
        /// Optional explicit type annotation.
        type_annotation: Option<Type>,
        /// Initializer expression assigned to the variable.
        value: Expr,
    },
    /// Pattern destructuring variable binding statement (`let (a, b) = tuple;` or `let { name, age } = user;`).
    LetPattern {
        /// Destructuring pattern matching against value.
        pattern: Pattern,
        /// Optional explicit type annotation.
        type_annotation: Option<Type>,
        /// Evaluated expression whose components are bound to pattern variables.
        value: Expr,
    },
    /// Assignment expression updating an existing mutable variable, field, or index (`x = 20;`, `rec.f = v;`).
    Assign {
        /// Target l-value being modified (variable identifier, member access, or index access).
        target: Expr,
        /// Value assigned to the target l-value.
        value: Expr,
    },
    /// Deterministic cleanup statement executed in LIFO order upon function exit (`defer file.close();`).
    Defer(Box<Expr>),
    /// Deterministic cleanup statement executed in LIFO order ONLY if an error or panic occurs (Zig model).
    ErrDefer(Box<Expr>),
    /// Standalone expression evaluated for its side effects (e.g. `println("Hello");`).
    Expr(Expr),
    /// Function return statement with an optional return value expression (`return 42;`).
    Return(Option<Expr>),
}

// -----------------------------------------------------------------------------
// EXPRESSIONS
// -----------------------------------------------------------------------------

/// Expressions in Aura. All computations, operations, and control structures are expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Constant literal value (integer, float, string, boolean, or unit).
    Literal(Literal),
    /// Identifier referencing a variable, parameter, or constant in scope.
    Identifier(String),
    /// Binary operator computation (`left + right`, `left == right`, etc.).
    Binary {
        /// Binary operator being applied.
        op: BinOp,
        /// Left-hand side operand expression.
        left: Box<Expr>,
        /// Right-hand side operand expression.
        right: Box<Expr>,
    },
    /// Unary operator computation (`-expr` negation or `!expr` logical not).
    Unary {
        /// Unary operator being applied.
        op: UnOp,
        /// Target operand expression.
        expr: Box<Expr>,
    },
    /// Pipeline operator (`data |> transform`), passing the left operand as the first argument to the right function.
    Pipeline {
        /// Left-hand expression providing the data value to pipe.
        left: Box<Expr>,
        /// Right-hand function or callable expression receiving the piped value.
        right: Box<Expr>,
    },
    /// Function or closure call invocation (`callee(arg1, arg2)`).
    FunctionCall {
        /// Callable expression (function identifier, closure, or member access).
        callee: Box<Expr>,
        /// Arguments supplied to the function call.
        args: Vec<Expr>,
    },
    /// Property or field access on a struct, record, or module (`object.member`).
    MemberAccess {
        /// Target object expression.
        object: Box<Expr>,
        /// Property or field identifier name being accessed.
        member: String,
    },
    /// Element access by index on lists, arrays, or maps (`object[index]`).
    IndexAccess {
        /// Target collection expression.
        object: Box<Expr>,
        /// Index expression indicating the element to retrieve.
        index: Box<Expr>,
    },
    /// Slicing expression extracting sub-slice (`object[low:high]` or `object[low:high:max]`).
    SliceAccess {
        /// Target sliceable collection expression (List or Slice).
        object: Box<Expr>,
        /// Optional starting index (defaults to 0 if omitted).
        low: Option<Box<Expr>>,
        /// Optional ending index (defaults to collection length if omitted).
        high: Option<Box<Expr>>,
        /// Optional maximum capacity slice limiter (Go 3-index slice model).
        max: Option<Box<Expr>>,
    },
    /// Struct or record construction literal (`{ x: 10, y: 20 }`).
    RecordLiteral {
        /// Key-value fields defining the record properties.
        fields: Vec<(String, Expr)>,
        /// Optional record spread expression (`{ ...base, override: 1 }`).
        spread: Option<Box<Expr>>,
    },
    /// Heterogeneous fixed-size tuple constructor (`(1, "hello", true)`).
    TupleLiteral(Vec<Expr>),
    /// Dynamic array/list constructor (`[1, 2, 3]`).
    ListLiteral(Vec<Expr>),
    /// Anonymous lambda function closure (`fn(x: Int): Int => x * 2`).
    Lambda {
        /// Parameters accepted by the closure.
        params: Vec<Param>,
        /// Optional explicit return type annotation.
        return_type: Option<Type>,
        /// Body expression evaluated when the closure is invoked.
        body: Box<Expr>,
    },
    /// Exhaustive pattern matching expression (`match subject { Pattern => Body, ... }`).
    Match {
        /// Subject expression being evaluated and matched against pattern arms.
        subject: Box<Expr>,
        /// List of pattern matching branches and their resulting bodies.
        arms: Vec<MatchArm>,
    },
    /// Conditional branching expression (`if condition => then_branch else => else_branch`).
    If {
        /// Boolean condition expression.
        condition: Box<Expr>,
        /// Expression evaluated if condition is true.
        then_branch: Box<Expr>,
        /// Optional expression evaluated if condition is false.
        else_branch: Option<Box<Expr>>,
    },
    /// Lexical block scope containing statements and evaluating to a final result value.
    Block(Vec<Statement>),
    /// Asynchronous task creation expression yielding a `Task<T, E>`.
    Async(Box<Expr>),
    /// Asynchronous suspension expression awaiting `Task<T, E>` completion.
    Await(Box<Expr>),
    /// Lightweight fiber / goroutine spawn running work concurrently on the M:N scheduler.
    Spawn(Box<Expr>),
    /// CSP channel send expression (`channel <- value`).
    ChanSend {
        /// Target channel expression.
        channel: Box<Expr>,
        /// Value expression to transmit across the channel.
        value: Box<Expr>,
    },
    /// CSP channel receive expression (`<-channel`).
    ChanRecv(Box<Expr>),
    /// Channel multiplexing select expression coordinating asynchronous communication across multiple channels.
    Select {
        /// Communication arms evaluated for channel readiness.
        arms: Vec<SelectArm>,
        /// Optional non-blocking fallback expression if no channel is ready.
        default: Option<Box<Expr>>,
    },
    /// Algebraic Data Type constructor invocation (`Some(42)`, `Ok("success")`, `Circle(3.14)`).
    ConstructorCall {
        /// Name of the variant constructor.
        name: String,
        /// Arguments supplied to the variant constructor.
        args: Vec<Expr>,
    },
    /// Try / unwrap operator expression (`expr?`), propagating `Err(e)` or `None` up the call stack.
    Try(Box<Expr>),
    /// Iterative while loop with optional label for labeled break/continue.
    While {
        /// Optional loop label identifier (`'outer: while ...`).
        label: Option<String>,
        /// Loop continuation condition expression.
        condition: Box<Expr>,
        /// Loop body expression executed on each iteration.
        body: Box<Expr>,
    },
    /// Iterative for-in loop traversing a list, range, or CSP channel.
    ForIn {
        /// Optional loop label identifier.
        label: Option<String>,
        /// Optional index counter variable name (`for i, item in items`).
        index_name: Option<String>,
        /// Variable name bound to each element in the collection.
        var_name: String,
        /// Iterable collection or channel expression being traversed.
        iterable: Box<Expr>,
        /// Loop body expression executed for each element.
        body: Box<Expr>,
    },
    /// Break loop statement terminating the innermost or targeted labeled loop (`break` or `break 'outer`).
    Break(Option<String>),
    /// Continue loop statement advancing to the next iteration of the innermost or labeled loop (`continue`).
    Continue(Option<String>),
    /// Compile-time static asset embedding macro (`embed("assets/logo.png")`).
    Embed {
        /// Path to the asset file relative to the source module.
        path: String,
        /// Whether the file is embedded as binary bytes (`Uint8Array`) or UTF-8 text string.
        is_binary: bool,
    },
    /// Reference / address-of operator generating a pointer (`&variable`).
    AddressOf(Box<Expr>),
    /// Pointer dereference operator accessing pointed value (`*pointer`).
    Deref(Box<Expr>),
    /// Abnormal termination / stack unwinding panic expression (`panic("fatal error")`).
    Panic(Box<Expr>),
    /// Active panic interceptor callable within defer blocks to catch errors (`recover()`).
    Recover,
    /// Associative key-value map literal constructor (`#{ "key" => value }`).
    MapLiteral(Vec<(Expr, Expr)>),
    /// Set literal constructor containing unique values (`#[ item1, item2 ]`).
    SetLiteral(Vec<Expr>),
    /// Internal placeholder node used during AST transformations and type inference.
    Placeholder,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    TemplateString(Vec<TemplateSegment>),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateSegment {
    Text(String),
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Negate,
    Not,
}

// -----------------------------------------------------------------------------
// CONCURRENCY (CSP)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct SelectArm {
    pub kind: SelectArmKind,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectArmKind {
    Recv {
        binding: Option<String>,
        channel: Expr,
    },
    Send {
        channel: Expr,
        value: Expr,
    },
    Timeout(Expr),
}

// -----------------------------------------------------------------------------
// PATTERN MATCHING
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    Variable(String),
    Constructor {
        name: String,
        patterns: Vec<Pattern>,
    },
    Record {
        type_name: Option<String>,
        fields: Vec<(String, Pattern)>,
        has_rest: bool,
    },
    Tuple(Vec<Pattern>),
    List {
        items: Vec<Pattern>,
        rest: Option<Box<Pattern>>,
    },
}

// -----------------------------------------------------------------------------
// DESIGN PATTERN: VISITOR PATTERN (AST TRAVERSAL & ANALYSIS)
// -----------------------------------------------------------------------------

pub trait AstVisitor {
    fn visit_module(&mut self, module: &Module) {
        walk_module(self, module);
    }

    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item);
    }

    fn visit_function(&mut self, func: &FunctionDecl) {
        self.visit_expr(&func.body);
    }

    fn visit_statement(&mut self, stmt: &Statement) {
        walk_statement(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    fn visit_pattern(&mut self, pattern: &Pattern) {
        walk_pattern(self, pattern);
    }
}

pub fn walk_module<V: AstVisitor + ?Sized>(visitor: &mut V, module: &Module) {
    for item in &module.items {
        visitor.visit_item(item);
    }
}

pub fn walk_item<V: AstVisitor + ?Sized>(visitor: &mut V, item: &Item) {
    match item {
        Item::Function(func) => visitor.visit_function(func),
        Item::Statement(stmt) => visitor.visit_statement(stmt),
        Item::TypeAlias(_)
        | Item::SumType(_)
        | Item::Interface(_)
        | Item::Import(_)
        | Item::Extern(_) => {}
    }
}

pub fn walk_statement<V: AstVisitor + ?Sized>(visitor: &mut V, stmt: &Statement) {
    match stmt {
        Statement::Let { value, .. } => visitor.visit_expr(value),
        Statement::LetPattern { pattern, value, .. } => {
            visitor.visit_pattern(pattern);
            visitor.visit_expr(value);
        }
        Statement::Assign { target, value } => {
            visitor.visit_expr(target);
            visitor.visit_expr(value);
        }
        Statement::Defer(expr) | Statement::ErrDefer(expr) => visitor.visit_expr(expr),
        Statement::Expr(expr) => visitor.visit_expr(expr),
        Statement::Return(opt_expr) => {
            if let Some(expr) = opt_expr {
                visitor.visit_expr(expr);
            }
        }
    }
}

pub fn walk_expr<V: AstVisitor + ?Sized>(visitor: &mut V, expr: &Expr) {
    match expr {
        Expr::Literal(_)
        | Expr::Identifier(_)
        | Expr::Break(_)
        | Expr::Continue(_)
        | Expr::Embed { .. }
        | Expr::Recover
        | Expr::Placeholder => {}
        Expr::Binary { left, right, .. } | Expr::Pipeline { left, right } => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        Expr::Unary { expr, .. }
        | Expr::Async(expr)
        | Expr::Await(expr)
        | Expr::Spawn(expr)
        | Expr::ChanRecv(expr)
        | Expr::Try(expr)
        | Expr::AddressOf(expr)
        | Expr::Deref(expr)
        | Expr::Panic(expr) => {
            visitor.visit_expr(expr);
        }
        Expr::FunctionCall { callee, args } => {
            visitor.visit_expr(callee);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        Expr::MemberAccess { object, .. } => visitor.visit_expr(object),
        Expr::IndexAccess { object, index } => {
            visitor.visit_expr(object);
            visitor.visit_expr(index);
        }
        Expr::SliceAccess {
            object,
            low,
            high,
            max,
        } => {
            visitor.visit_expr(object);
            if let Some(l) = low {
                visitor.visit_expr(l);
            }
            if let Some(h) = high {
                visitor.visit_expr(h);
            }
            if let Some(m) = max {
                visitor.visit_expr(m);
            }
        }
        Expr::RecordLiteral { fields, spread } => {
            for (_, val) in fields {
                visitor.visit_expr(val);
            }
            if let Some(s) = spread {
                visitor.visit_expr(s);
            }
        }
        Expr::TupleLiteral(items) | Expr::ListLiteral(items) | Expr::SetLiteral(items) => {
            for item in items {
                visitor.visit_expr(item);
            }
        }
        Expr::MapLiteral(entries) => {
            for (k, v) in entries {
                visitor.visit_expr(k);
                visitor.visit_expr(v);
            }
        }
        Expr::Lambda { body, .. } => visitor.visit_expr(body),
        Expr::Match { subject, arms } => {
            visitor.visit_expr(subject);
            for arm in arms {
                visitor.visit_pattern(&arm.pattern);
                if let Some(g) = &arm.guard {
                    visitor.visit_expr(g);
                }
                visitor.visit_expr(&arm.body);
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            visitor.visit_expr(condition);
            visitor.visit_expr(then_branch);
            if let Some(eb) = else_branch {
                visitor.visit_expr(eb);
            }
        }
        Expr::Block(stmts) => {
            for stmt in stmts {
                visitor.visit_statement(stmt);
            }
        }
        Expr::ChanSend { channel, value } => {
            visitor.visit_expr(channel);
            visitor.visit_expr(value);
        }
        Expr::Select { arms, default } => {
            for arm in arms {
                match &arm.kind {
                    SelectArmKind::Recv { channel, .. } => visitor.visit_expr(channel),
                    SelectArmKind::Send { channel, value } => {
                        visitor.visit_expr(channel);
                        visitor.visit_expr(value);
                    }
                    SelectArmKind::Timeout(t) => visitor.visit_expr(t),
                }
                visitor.visit_expr(&arm.body);
            }
            if let Some(d) = default {
                visitor.visit_expr(d);
            }
        }
        Expr::ConstructorCall { args, .. } => {
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        Expr::While {
            condition, body, ..
        } => {
            visitor.visit_expr(condition);
            visitor.visit_expr(body);
        }
        Expr::ForIn { iterable, body, .. } => {
            visitor.visit_expr(iterable);
            visitor.visit_expr(body);
        }
    }
}

pub fn walk_pattern<V: AstVisitor + ?Sized>(visitor: &mut V, pattern: &Pattern) {
    match pattern {
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::Variable(_) => {}
        Pattern::Constructor { patterns, .. } => {
            for p in patterns {
                visitor.visit_pattern(p);
            }
        }
        Pattern::Record { fields, .. } => {
            for (_, p) in fields {
                visitor.visit_pattern(p);
            }
        }
        Pattern::Tuple(patterns) => {
            for p in patterns {
                visitor.visit_pattern(p);
            }
        }
        Pattern::List { items, rest } => {
            for item in items {
                visitor.visit_pattern(item);
            }
            if let Some(r) = rest {
                visitor.visit_pattern(r);
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AstMetrics {
    pub function_count: usize,
    pub statement_count: usize,
    pub expression_count: usize,
    pub pattern_count: usize,
}

#[derive(Default)]
pub struct AstMetricsCollector {
    pub metrics: AstMetrics,
}

impl AstMetricsCollector {
    pub fn collect(module: &Module) -> AstMetrics {
        let mut collector = Self::default();
        collector.visit_module(module);
        collector.metrics
    }
}

impl AstVisitor for AstMetricsCollector {
    fn visit_function(&mut self, func: &FunctionDecl) {
        self.metrics.function_count += 1;
        self.visit_expr(&func.body);
    }

    fn visit_statement(&mut self, stmt: &Statement) {
        self.metrics.statement_count += 1;
        walk_statement(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        self.metrics.expression_count += 1;
        walk_expr(self, expr);
    }

    fn visit_pattern(&mut self, pattern: &Pattern) {
        self.metrics.pattern_count += 1;
        walk_pattern(self, pattern);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_module_and_items_creation() {
        let module = Module {
            name: Some("test_mod".to_string()),
            items: vec![
                Item::TypeAlias(TypeAliasDecl {
                    name: "Id".to_string(),
                    is_exported: true,
                    is_packed: false,
                    type_params: vec![],
                    target: Type::Named {
                        name: "Int".to_string(),
                        type_args: vec![],
                    },
                }),
                Item::SumType(SumTypeDecl {
                    name: "Result".to_string(),
                    is_exported: true,
                    type_params: vec!["T".to_string(), "E".to_string()],
                    variants: vec![
                        Variant {
                            name: "Ok".to_string(),
                            fields: VariantFields::Tuple(vec![Type::TypeVar("T".to_string())]),
                        },
                        Variant {
                            name: "Err".to_string(),
                            fields: VariantFields::Tuple(vec![Type::TypeVar("E".to_string())]),
                        },
                    ],
                }),
            ],
        };

        assert_eq!(module.name, Some("test_mod".to_string()));
        assert_eq!(module.items.len(), 2);
    }

    #[test]
    fn test_ast_expressions_and_statements() {
        let let_stmt = Statement::Let {
            name: "x".to_string(),
            is_mut: false,
            type_annotation: Some(Type::Named {
                name: "Int".to_string(),
                type_args: vec![],
            }),
            value: Expr::Literal(Literal::Int(42)),
        };

        let binary_expr = Expr::Binary {
            op: BinOp::Add,
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::Literal(Literal::Int(1))),
        };

        assert_eq!(
            let_stmt,
            Statement::Let {
                name: "x".to_string(),
                is_mut: false,
                type_annotation: Some(Type::Named {
                    name: "Int".to_string(),
                    type_args: vec![],
                }),
                value: Expr::Literal(Literal::Int(42)),
            }
        );

        if let Expr::Binary { op, .. } = binary_expr {
            assert_eq!(op, BinOp::Add);
        } else {
            panic!("Expected binary expr");
        }
    }

    #[test]
    fn test_ast_pattern_structures() {
        let pat_wildcard = Pattern::Wildcard;
        let pat_tuple = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Int(1)),
            Pattern::Variable("y".to_string()),
        ]);
        let pat_record = Pattern::Record {
            type_name: Some("User".to_string()),
            fields: vec![("name".to_string(), Pattern::Variable("n".to_string()))],
            has_rest: true,
        };

        assert_eq!(pat_wildcard, Pattern::Wildcard);
        if let Pattern::Tuple(items) = pat_tuple {
            assert_eq!(items.len(), 2);
        }
        if let Pattern::Record { has_rest, .. } = pat_record {
            assert!(has_rest);
        }
    }

    #[test]
    fn test_ast_visitor_metrics() {
        let module = Module {
            name: Some("visitor_test".to_string()),
            items: vec![Item::Function(FunctionDecl {
                name: "compute".to_string(),
                receiver: None,
                is_async: false,
                is_exported: true,
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Expr::Block(vec![
                    Statement::Let {
                        name: "a".to_string(),
                        is_mut: false,
                        type_annotation: None,
                        value: Expr::Literal(Literal::Int(10)),
                    },
                    Statement::Expr(Expr::Binary {
                        op: BinOp::Add,
                        left: Box::new(Expr::Identifier("a".to_string())),
                        right: Box::new(Expr::Literal(Literal::Int(20))),
                    }),
                ]),
            })],
        };

        let metrics = AstMetricsCollector::collect(&module);
        assert_eq!(metrics.function_count, 1);
        assert_eq!(metrics.statement_count, 2);
        assert!(metrics.expression_count >= 3);
    }
}
