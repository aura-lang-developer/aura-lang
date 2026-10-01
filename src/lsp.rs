//! Aura Language Server Protocol (LSP) Engine
//!
//! Implements JSON-RPC 2.0 communication, document synchronization,
//! diagnostics publishing, Hover information, Go to Definition,
//! Contextual Completions, Document Symbols, and Code Formatting.

use crate::ast::*;
use crate::formatter::{FormatConfig, format_aura};
use crate::lexer::{Lexer, Span, TokenKind};
use crate::parser::Parser;
use crate::typechecker::{ConcreteType, TypeChecker};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};

/// Represents a 0-indexed position in a text document (LSP standard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

/// Represents a text range with start and end positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

impl LspRange {
    pub fn new(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Self {
        Self {
            start: LspPosition {
                line: start_line,
                character: start_char,
            },
            end: LspPosition {
                line: end_line,
                character: end_char,
            },
        }
    }

    pub fn from_span(span: &Span, length: usize) -> Self {
        let line = if span.line > 0 {
            (span.line - 1) as u32
        } else {
            0
        };
        let char_start = if span.column > 0 {
            (span.column - 1) as u32
        } else {
            0
        };
        let char_end = char_start + (length as u32);
        Self::new(line, char_start, line, char_end)
    }

    pub fn contains(&self, pos: &LspPosition) -> bool {
        if pos.line < self.start.line || pos.line > self.end.line {
            return false;
        }
        if pos.line == self.start.line && pos.character < self.start.character {
            return false;
        }
        if pos.line == self.end.line && pos.character > self.end.character {
            return false;
        }
        true
    }
}

/// Diagnostic severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// A diagnostic message (error/warning) reported to the client editor.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub range: LspRange,
    pub severity: DiagnosticSeverity,
    pub source: String,
    pub message: String,
}

/// Symbol kind for LSP document symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Constructor = 9,
    Enum = 10,
    Interface = 11,
    Function = 12,
    Variable = 13,
    Constant = 14,
    String = 15,
    Number = 16,
    Boolean = 17,
    Array = 18,
    Object = 19,
    Key = 20,
    Null = 21,
    EnumMember = 22,
    Struct = 23,
    Event = 24,
    Operator = 25,
    TypeParameter = 26,
}

/// Document symbol definition for outline view.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentSymbol {
    pub name: String,
    pub detail: Option<String>,
    pub kind: SymbolKind,
    pub range: LspRange,
    pub selection_range: LspRange,
    pub children: Vec<DocumentSymbol>,
}

/// A hover response card.
#[derive(Debug, Clone, PartialEq)]
pub struct HoverResult {
    pub contents: String,
    pub range: Option<LspRange>,
}

/// A location for Go to Definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub uri: String,
    pub range: LspRange,
}

/// Completion item kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

/// Completion item.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

/// Text edit for formatting.
#[derive(Debug, Clone, PartialEq)]
pub struct TextEdit {
    pub range: LspRange,
    pub new_text: String,
}

/// Definition information recorded during symbol indexing.
#[derive(Debug, Clone)]
pub struct SymbolDef {
    pub name: String,
    pub kind_name: String,
    pub type_str: String,
    pub doc_comment: Option<String>,
    pub def_range: LspRange,
    pub uri: String,
}

/// Identifier reference in source code.
#[derive(Debug, Clone)]
pub struct SymbolRef {
    pub name: String,
    pub range: LspRange,
    pub def_name: String,
}

/// Parameter information in a signature help card.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterInformation {
    pub label: String,
    pub documentation: Option<String>,
}

/// A signature representation for function and method calls.
#[derive(Debug, Clone, PartialEq)]
pub struct SignatureInformation {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<ParameterInformation>,
}

/// Active signature and parameter information for the call at cursor.
#[derive(Debug, Clone, PartialEq)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: u32,
    pub active_parameter: u32,
}

/// Highlighting occurrence of a symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentHighlight {
    pub range: LspRange,
    pub kind: u32, // 1 = Text, 2 = Read, 3 = Write
}

/// Inlay hint displaying inferred types or parameter names.
#[derive(Debug, Clone, PartialEq)]
pub struct InlayHint {
    pub position: LspPosition,
    pub label: String,
    pub kind: u32, // 1 = Type, 2 = Parameter
    pub padding_left: bool,
    pub padding_right: bool,
}

/// A Code Action / Quick Fix proposed to the user.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeAction {
    pub title: String,
    pub kind: String, // "quickfix", "refactor", "source.organizeImports"
    pub diagnostics: Vec<Diagnostic>,
    pub edit: Option<HashMap<String, Vec<TextEdit>>>,
    pub is_preferred: bool,
}

/// Workspace-level symbol definition.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub container_name: Option<String>,
}

/// Command attached to a Code Lens button.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeLensCommand {
    pub title: String,
    pub command: String,
    pub arguments: Vec<String>,
}

/// A Code Lens action button positioned above a line of code.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeLens {
    pub range: LspRange,
    pub command: Option<CodeLensCommand>,
}

/// Folding range for collapsible code regions.
#[derive(Debug, Clone, PartialEq)]
pub struct FoldingRange {
    pub start_line: u32,
    pub end_line: u32,
    pub kind: Option<String>, // "comment", "imports", "region"
}

/// Call hierarchy item representing a function or method.
#[derive(Debug, Clone, PartialEq)]
pub struct CallHierarchyItem {
    pub name: String,
    pub kind: SymbolKind,
    pub uri: String,
    pub range: LspRange,
    pub selection_range: LspRange,
}

/// Incoming call site in call hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct CallHierarchyIncomingCall {
    pub from: CallHierarchyItem,
    pub from_ranges: Vec<LspRange>,
}

/// Outgoing call target in call hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct CallHierarchyOutgoingCall {
    pub to: CallHierarchyItem,
    pub from_ranges: Vec<LspRange>,
}

/// Analysis state for an open Aura document.
pub struct DocumentState {
    pub uri: String,
    pub source: String,
    pub version: i64,
    pub lines: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub symbols: Vec<DocumentSymbol>,
    pub definitions: HashMap<String, SymbolDef>,
    pub references: Vec<SymbolRef>,
    pub type_env: HashMap<String, ConcreteType>,
}

impl DocumentState {
    pub fn new(uri: String, source: String, version: i64) -> Self {
        let lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
        let mut state = Self {
            uri,
            source,
            version,
            lines,
            diagnostics: Vec::new(),
            symbols: Vec::new(),
            definitions: HashMap::new(),
            references: Vec::new(),
            type_env: HashMap::new(),
        };
        state.analyze();
        state
    }

    pub fn update(&mut self, source: String, version: i64) {
        self.source = source;
        self.lines = self.source.lines().map(|s| s.to_string()).collect();
        self.version = version;
        self.analyze();
    }

    /// Performs lexical, syntactic, and semantic indexing of the document.
    pub fn analyze(&mut self) {
        self.diagnostics.clear();
        self.symbols.clear();
        self.definitions.clear();
        self.references.clear();
        self.type_env.clear();

        // 1. Lexical Analysis
        let mut lexer = Lexer::new(&self.source);
        let tokens = match lexer.tokenize() {
            Ok(toks) => toks,
            Err(lex_err) => {
                let range = parse_error_range(&lex_err, &self.lines);
                self.diagnostics.push(Diagnostic {
                    range,
                    severity: DiagnosticSeverity::Error,
                    source: "aura-lexer".to_string(),
                    message: lex_err,
                });
                return;
            }
        };

        // 2. Syntax Parsing
        let mut parser = Parser::new(tokens.clone());
        let module = match parser.parse_module() {
            Ok(m) => m,
            Err(parse_err) => {
                let range = parse_error_range(&parse_err, &self.lines);
                self.diagnostics.push(Diagnostic {
                    range,
                    severity: DiagnosticSeverity::Error,
                    source: "aura-parser".to_string(),
                    message: parse_err,
                });
                return;
            }
        };

        // 3. Type Checking
        let mut typechecker = TypeChecker::new();
        if let Err(type_err) = typechecker.check_module(&module) {
            let range = parse_error_range(&type_err, &self.lines);
            self.diagnostics.push(Diagnostic {
                range,
                severity: DiagnosticSeverity::Error,
                source: "aura-typechecker".to_string(),
                message: type_err,
            });
        }

        // Store inferred/declared types from environment
        for (var_name, var_type) in typechecker.env().all_variables() {
            self.type_env.insert(var_name.clone(), var_type.clone());
        }

        // 4. Extract symbols, definitions, and references
        self.index_symbols_and_definitions(&module, &tokens);
    }

    fn index_symbols_and_definitions(&mut self, module: &Module, tokens: &[crate::lexer::Token]) {
        // Collect comments above definitions
        let doc_comments = extract_doc_comments(&self.source);

        // Find token positions for fast range resolution
        let mut ident_spans: Vec<(&str, LspRange)> = Vec::new();
        for tok in tokens {
            if let TokenKind::Ident(ref name) = tok.kind {
                let r = LspRange::from_span(&tok.span, name.len());
                ident_spans.push((name.as_str(), r));
            }
        }

        for item in &module.items {
            match item {
                Item::Function(f) => {
                    let param_types: Vec<String> = f
                        .params
                        .iter()
                        .map(|p| {
                            let ty_str = p
                                .type_annotation
                                .as_ref()
                                .map(|t| format!("{}", t))
                                .unwrap_or_else(|| "Any".to_string());
                            format!("{}: {}", p.name, ty_str)
                        })
                        .collect();
                    let ret_str = f
                        .return_type
                        .as_ref()
                        .map(|t| format!(" -> {}", t))
                        .unwrap_or_default();
                    let recv_prefix = if let Some(ref r) = f.receiver {
                        format!("({}) ", r)
                    } else {
                        String::new()
                    };
                    let sig = format!(
                        "fn {}{}({}){}",
                        recv_prefix,
                        f.name,
                        param_types.join(", "),
                        ret_str
                    );

                    let fn_range = find_ident_range(&ident_spans, &f.name, 0);
                    let doc = doc_comments.get(&f.name).cloned();

                    self.definitions.insert(
                        f.name.clone(),
                        SymbolDef {
                            name: f.name.clone(),
                            kind_name: "function".to_string(),
                            type_str: sig.clone(),
                            doc_comment: doc.clone(),
                            def_range: fn_range,
                            uri: self.uri.clone(),
                        },
                    );

                    // Index parameters
                    for p in &f.params {
                        let p_ty = p
                            .type_annotation
                            .as_ref()
                            .map(|t| format!("{}", t))
                            .unwrap_or_else(|| "Any".to_string());
                        let p_range = find_ident_range(&ident_spans, &p.name, fn_range.start.line);
                        let p_key = format!("{}::{}", f.name, p.name);
                        self.definitions.insert(
                            p_key.clone(),
                            SymbolDef {
                                name: p.name.clone(),
                                kind_name: "parameter".to_string(),
                                type_str: format!("{}: {}", p.name, p_ty),
                                doc_comment: None,
                                def_range: p_range,
                                uri: self.uri.clone(),
                            },
                        );
                        self.definitions.insert(
                            p.name.clone(),
                            SymbolDef {
                                name: p.name.clone(),
                                kind_name: "parameter".to_string(),
                                type_str: format!("{}: {}", p.name, p_ty),
                                doc_comment: None,
                                def_range: p_range,
                                uri: self.uri.clone(),
                            },
                        );
                    }

                    self.symbols.push(DocumentSymbol {
                        name: f.name.clone(),
                        detail: Some(sig),
                        kind: SymbolKind::Function,
                        range: fn_range,
                        selection_range: fn_range,
                        children: Vec::new(),
                    });
                }
                Item::SumType(s) => {
                    let s_range = find_ident_range(&ident_spans, &s.name, 0);
                    let doc = doc_comments.get(&s.name).cloned();
                    let var_names: Vec<String> =
                        s.variants.iter().map(|v| v.name.clone()).collect();
                    let sig = format!("type {} = {}", s.name, var_names.join(" | "));

                    self.definitions.insert(
                        s.name.clone(),
                        SymbolDef {
                            name: s.name.clone(),
                            kind_name: "enum".to_string(),
                            type_str: sig.clone(),
                            doc_comment: doc,
                            def_range: s_range,
                            uri: self.uri.clone(),
                        },
                    );

                    let mut var_symbols = Vec::new();
                    for v in &s.variants {
                        let v_range = find_ident_range(&ident_spans, &v.name, s_range.start.line);
                        let v_sig = format!("variant {}::{}", s.name, v.name);
                        self.definitions.insert(
                            v.name.clone(),
                            SymbolDef {
                                name: v.name.clone(),
                                kind_name: "variant".to_string(),
                                type_str: v_sig.clone(),
                                doc_comment: None,
                                def_range: v_range,
                                uri: self.uri.clone(),
                            },
                        );
                        var_symbols.push(DocumentSymbol {
                            name: v.name.clone(),
                            detail: Some(v_sig),
                            kind: SymbolKind::EnumMember,
                            range: v_range,
                            selection_range: v_range,
                            children: Vec::new(),
                        });
                    }

                    self.symbols.push(DocumentSymbol {
                        name: s.name.clone(),
                        detail: Some(sig),
                        kind: SymbolKind::Enum,
                        range: s_range,
                        selection_range: s_range,
                        children: var_symbols,
                    });
                }
                Item::TypeAlias(t) => {
                    let t_range = find_ident_range(&ident_spans, &t.name, 0);
                    let sig = format!("type {} = {}", t.name, t.target);
                    let doc = doc_comments.get(&t.name).cloned();

                    self.definitions.insert(
                        t.name.clone(),
                        SymbolDef {
                            name: t.name.clone(),
                            kind_name: "type alias".to_string(),
                            type_str: sig.clone(),
                            doc_comment: doc,
                            def_range: t_range,
                            uri: self.uri.clone(),
                        },
                    );

                    self.symbols.push(DocumentSymbol {
                        name: t.name.clone(),
                        detail: Some(sig),
                        kind: SymbolKind::Interface,
                        range: t_range,
                        selection_range: t_range,
                        children: Vec::new(),
                    });
                }
                Item::Import(imp) => {
                    for it in &imp.items {
                        let it_name = it.alias.as_ref().unwrap_or(&it.name);
                        let it_range = find_ident_range(&ident_spans, it_name, 0);

                        let mut resolved_sig = format!("import {} from '{}'", it_name, imp.source);
                        let mut resolved_doc = None;
                        let mut resolved_uri = self.uri.clone();

                        if (imp.source.starts_with("./") || imp.source.starts_with("../"))
                            && self.uri.starts_with("file://")
                        {
                            let curr_path = self.uri.trim_start_matches("file://");
                            if let Some(parent_dir) = std::path::Path::new(curr_path).parent() {
                                let mut candidate = parent_dir.join(&imp.source);
                                if !candidate.exists() && candidate.with_extension("aura").exists()
                                {
                                    candidate = candidate.with_extension("aura");
                                }
                                if candidate.is_file() {
                                    if let Ok(src) = std::fs::read_to_string(&candidate) {
                                        for line in src.lines() {
                                            let trimmed = line.trim();
                                            if (trimmed.starts_with("export fn")
                                                || trimmed.starts_with("fn"))
                                                && trimmed.contains(&it.name)
                                            {
                                                resolved_sig = format!(
                                                    "// Defined in {}\n{}",
                                                    imp.source,
                                                    trimmed
                                                        .trim_end_matches('{')
                                                        .trim_end_matches("=>")
                                                        .trim()
                                                );
                                                resolved_doc = Some(format!(
                                                    "Underlying definition in `{}`",
                                                    candidate
                                                        .file_name()
                                                        .unwrap_or_default()
                                                        .to_string_lossy()
                                                ));
                                                resolved_uri = format!(
                                                    "file://{}",
                                                    candidate.to_string_lossy()
                                                );
                                                break;
                                            } else if (trimmed.starts_with("export type")
                                                || trimmed.starts_with("type"))
                                                && trimmed.contains(&it.name)
                                            {
                                                resolved_sig = format!(
                                                    "// Defined in {}\n{}",
                                                    imp.source, trimmed
                                                );
                                                resolved_doc = Some(format!(
                                                    "Underlying type in `{}`",
                                                    candidate
                                                        .file_name()
                                                        .unwrap_or_default()
                                                        .to_string_lossy()
                                                ));
                                                resolved_uri = format!(
                                                    "file://{}",
                                                    candidate.to_string_lossy()
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        self.definitions.insert(
                            it_name.clone(),
                            SymbolDef {
                                name: it_name.clone(),
                                kind_name: "import".to_string(),
                                type_str: resolved_sig,
                                doc_comment: resolved_doc,
                                def_range: it_range,
                                uri: resolved_uri,
                            },
                        );
                    }
                }
                Item::Statement(Statement::Let {
                    name,
                    is_mut: _,
                    type_annotation,
                    value: _,
                }) => {
                    let v_range = find_ident_range(&ident_spans, name, 0);
                    let ty_str = if let Some(t) = type_annotation {
                        format!("{}", t)
                    } else if let Some(conc_ty) = self.type_env.get(name) {
                        format!("{}", conc_ty)
                    } else {
                        "Any".to_string()
                    };
                    let sig = format!("let {}: {}", name, ty_str);

                    self.definitions.insert(
                        name.clone(),
                        SymbolDef {
                            name: name.clone(),
                            kind_name: "variable".to_string(),
                            type_str: sig.clone(),
                            doc_comment: None,
                            def_range: v_range,
                            uri: self.uri.clone(),
                        },
                    );

                    self.symbols.push(DocumentSymbol {
                        name: name.clone(),
                        detail: Some(sig),
                        kind: SymbolKind::Variable,
                        range: v_range,
                        selection_range: v_range,
                        children: Vec::new(),
                    });
                }

                _ => {}
            }
        }

        // Map all identifier tokens to symbol references
        for (ident, range) in &ident_spans {
            if self.definitions.contains_key(*ident) {
                self.references.push(SymbolRef {
                    name: ident.to_string(),
                    range: *range,
                    def_name: ident.to_string(),
                });
            }
        }
    }

    /// Provides hover information for a position in the document.
    pub fn hover(&self, pos: &LspPosition) -> Option<HoverResult> {
        // 1. Check if hovering over a reference or definition
        for s_ref in &self.references {
            if s_ref.range.contains(pos) {
                if let Some(def) = self.definitions.get(&s_ref.def_name) {
                    let mut md = format!("```aura\n{}\n```", def.type_str);
                    if let Some(ref doc) = def.doc_comment {
                        md.push_str("\n\n---\n");
                        md.push_str(doc);
                    }
                    return Some(HoverResult {
                        contents: md,
                        range: Some(s_ref.range),
                    });
                }
            }
        }

        // 2. Check if hovering over standard keywords or built-ins
        let word = self
            .word_at_position(pos)
            .or_else(|| self.operator_at_position(pos))?;
        if let Some(keyword_hover) = get_keyword_hover(&word.0) {
            return Some(HoverResult {
                contents: keyword_hover,
                range: Some(word.1),
            });
        }

        // 3. Check if hovering over built-in types or standard library
        if let Some(builtin_hover) = get_builtin_hover(&word.0) {
            return Some(HoverResult {
                contents: builtin_hover,
                range: Some(word.1),
            });
        }

        None
    }

    /// Resolves Go to Definition for a position in the document.
    pub fn definition(&self, pos: &LspPosition) -> Option<Location> {
        for s_ref in &self.references {
            if s_ref.range.contains(pos) {
                if let Some(def) = self.definitions.get(&s_ref.def_name) {
                    return Some(Location {
                        uri: def.uri.clone(),
                        range: def.def_range,
                    });
                }
            }
        }

        // Check if cursor is directly on a definition symbol
        for def in self.definitions.values() {
            if def.def_range.contains(pos) {
                return Some(Location {
                    uri: def.uri.clone(),
                    range: def.def_range,
                });
            }
        }

        None
    }

    /// Provides completions at a position in the document.
    pub fn completions(&self, _pos: &LspPosition) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // 1. In-scope definitions
        for def in self.definitions.values() {
            let kind = match def.kind_name.as_str() {
                "function" => CompletionItemKind::Function,
                "enum" => CompletionItemKind::Enum,
                "variant" => CompletionItemKind::EnumMember,
                "type alias" => CompletionItemKind::Interface,
                "parameter" => CompletionItemKind::Variable,
                _ => CompletionItemKind::Variable,
            };

            items.push(CompletionItem {
                label: def.name.clone(),
                kind,
                detail: Some(def.type_str.clone()),
                documentation: def.doc_comment.clone(),
                insert_text: None,
            });
        }

        // 2. Aura Language Keywords
        let keywords = [
            (
                "fn",
                "Declare a function",
                "fn ${1:name}(${2:params}) -> ${3:Type} {\n\t$0\n}",
            ),
            ("let", "Declare a variable", "let ${1:name} = ${2:value};"),
            ("mut", "Mutable variable modifier", "mut "),
            (
                "match",
                "Pattern match expression",
                "match ${1:expr} {\n\t${2:pattern} => ${3:result},\n}",
            ),
            ("when", "Guard condition in match", "when ${1:condition}"),
            (
                "if",
                "Conditional branching",
                "if ${1:condition} {\n\t$0\n} else {\n}",
            ),
            ("else", "Else branch", "else {\n\t$0\n}"),
            ("async", "Asynchronous function modifier", "async "),
            ("await", "Await a Task or Promise", "await "),
            (
                "spawn",
                "Spawn a lightweight concurrent task",
                "spawn {\n\t$0\n}",
            ),
            (
                "routine",
                "Launch an Aura Routine (lightweight concurrent task)",
                "routine {\n\t$0\n}",
            ),
            (
                "go",
                "Launch a goroutine (lightweight concurrent task)",
                "go {\n\t$0\n}",
            ),
            (
                "select",
                "Select multiplexing on channels",
                "select {\n\t${1:chan} <- msg => {\n\t\t$0\n\t}\n}",
            ),
            (
                "import",
                "Import declarations from module",
                "import { ${1:item} } from \"${2:source}\";",
            ),
            ("export", "Export declaration", "export "),
            (
                "type",
                "Declare type alias or sum type",
                "type ${1:Name} = $0;",
            ),
            ("while", "While loop", "while ${1:condition} {\n\t$0\n}"),
            (
                "for",
                "For-in loop",
                "for ${1:item} in ${2:iterable} {\n\t$0\n}",
            ),
            (
                "defer",
                "Defer execution until function exit (LIFO)",
                "defer $0;",
            ),
            (
                "embed",
                "Embed static text file at compile time",
                "embed(\"${1:path}\")",
            ),
            (
                "embedBytes",
                "Embed static binary asset as Uint8Array at compile time",
                "embedBytes(\"${1:path}\")",
            ),
            ("return", "Return statement", "return $0;"),
        ];

        for (kw, doc, snippet) in keywords {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some(format!("keyword: {}", kw)),
                documentation: Some(doc.to_string()),
                insert_text: Some(snippet.to_string()),
            });
        }

        // 3. Built-in types
        let builtin_types = [
            "Int",
            "Float",
            "String",
            "Bool",
            "Option",
            "Result",
            "Task",
            "Channel",
            "List",
            "Array",
            "Iterator",
            "Interator",
            "Map",
            "Set",
        ];
        for ty in builtin_types {
            items.push(CompletionItem {
                label: ty.to_string(),
                kind: CompletionItemKind::Class,
                detail: Some(format!("type: {}", ty)),
                documentation: Some(format!("Built-in Aura type {}", ty)),
                insert_text: None,
            });
        }

        // 4. Built-in functions
        let builtin_funcs = [
            ("println", "println(value: Any): Unit", "println($0)"),
            ("print", "print(value: Any): Unit", "print($0)"),
            ("eprintln", "eprintln(value: Any): Unit", "eprintln($0)"),
            ("len", "len(container: Any): Int", "len($0)"),
            (
                "map",
                "map(coll, fn): Transform collection or iterator",
                "map(${1:coll}, ${2:fn})",
            ),
            (
                "filter",
                "filter(coll, pred): Filter collection or iterator",
                "filter(${1:coll}, ${2:pred})",
            ),
            (
                "sort",
                "sort(coll, cmp?): Sort collection or iterator",
                "sort(${1:coll})",
            ),
            ("iter", "iter(coll): Convert to Iterator", "iter(${1:coll})"),
        ];
        for (f, doc, snippet) in builtin_funcs {
            items.push(CompletionItem {
                label: f.to_string(),
                kind: CompletionItemKind::Function,
                detail: Some(format!("built-in function: {}", f)),
                documentation: Some(doc.to_string()),
                insert_text: Some(snippet.to_string()),
            });
        }

        // Deduplicate by label
        let mut seen = std::collections::HashSet::new();
        items.retain(|it| seen.insert(it.label.clone()));

        items
    }

    /// Formats the document.
    pub fn format(&self, _config: Option<FormatConfig>) -> Result<Vec<TextEdit>, String> {
        let formatted = format_aura(&self.source).map_err(|e| format!("{}", e))?;
        if formatted == self.source {
            return Ok(Vec::new());
        }

        let total_lines = self.lines.len() as u32;
        let last_line_len = self.lines.last().map(|l| l.len() as u32).unwrap_or(0);

        Ok(vec![TextEdit {
            range: LspRange::new(0, 0, total_lines, last_line_len),
            new_text: formatted,
        }])
    }

    fn word_at_position(&self, pos: &LspPosition) -> Option<(String, LspRange)> {
        let line_idx = pos.line as usize;
        if line_idx >= self.lines.len() {
            return None;
        }
        let line = &self.lines[line_idx];
        let col = pos.character as usize;
        if col > line.len() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
            return None;
        }

        let target_col = if col >= chars.len() {
            chars.len().saturating_sub(1)
        } else {
            col
        };
        if !is_ident_char(chars[target_col]) {
            return None;
        }

        let mut start = target_col;
        while start > 0 && is_ident_char(chars[start - 1]) {
            start -= 1;
        }

        let mut end = target_col;
        while end + 1 < chars.len() && is_ident_char(chars[end + 1]) {
            end += 1;
        }

        let word: String = chars[start..=end].iter().collect();
        let range = LspRange::new(pos.line, start as u32, pos.line, (end + 1) as u32);
        Some((word, range))
    }

    fn operator_at_position(&self, pos: &LspPosition) -> Option<(String, LspRange)> {
        let line_idx = pos.line as usize;
        if line_idx >= self.lines.len() {
            return None;
        }
        let line = &self.lines[line_idx];
        let col = pos.character as usize;
        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() || col >= chars.len() {
            return None;
        }

        // Two-character operators
        if col + 1 < chars.len() {
            let two: String = chars[col..=col + 1].iter().collect();
            if matches!(
                two.as_str(),
                "|>" | "<-" | "==" | "!=" | "<=" | ">=" | "&&" | "||" | "=>" | "->"
            ) {
                return Some((
                    two,
                    LspRange::new(pos.line, col as u32, pos.line, (col + 2) as u32),
                ));
            }
        }
        if col > 0 && col < chars.len() {
            let two: String = chars[col - 1..=col].iter().collect();
            if matches!(
                two.as_str(),
                "|>" | "<-" | "==" | "!=" | "<=" | ">=" | "&&" | "||" | "=>" | "->"
            ) {
                return Some((
                    two,
                    LspRange::new(pos.line, (col - 1) as u32, pos.line, (col + 1) as u32),
                ));
            }
        }

        // Single-character operators
        let ch = chars[col];
        if matches!(
            ch,
            '?' | '|' | '<' | '>' | '=' | '+' | '-' | '*' | '/' | '%' | '!'
        ) {
            return Some((
                ch.to_string(),
                LspRange::new(pos.line, col as u32, pos.line, (col + 1) as u32),
            ));
        }

        None
    }

    /// Finds all references to the symbol at the given position.
    pub fn find_references(&self, pos: &LspPosition, include_declaration: bool) -> Vec<Location> {
        let mut target_name = None;
        let mut target_def = None;

        for s_ref in &self.references {
            if s_ref.range.contains(pos) {
                target_name = Some(s_ref.def_name.clone());
                target_def = self.definitions.get(&s_ref.def_name);
                break;
            }
        }

        if target_name.is_none() {
            for def in self.definitions.values() {
                if def.def_range.contains(pos) {
                    target_name = Some(def.name.clone());
                    target_def = Some(def);
                    break;
                }
            }
        }

        if target_name.is_none() {
            if let Some((word, _)) = self.word_at_position(pos) {
                if self.definitions.contains_key(&word) {
                    target_name = Some(word.clone());
                    target_def = self.definitions.get(&word);
                } else if self
                    .references
                    .iter()
                    .any(|r| r.name == word || r.def_name == word)
                {
                    target_name = Some(word);
                }
            }
        }

        let name = match target_name {
            Some(n) => n,
            None => return Vec::new(),
        };

        let mut results = Vec::new();

        if include_declaration {
            if let Some(def) = target_def {
                results.push(Location {
                    uri: def.uri.clone(),
                    range: def.def_range,
                });
            }
        }

        for s_ref in &self.references {
            if s_ref.name == name || s_ref.def_name == name {
                if !include_declaration {
                    if let Some(def) = target_def {
                        if s_ref.range == def.def_range {
                            continue;
                        }
                    }
                }
                results.push(Location {
                    uri: self.uri.clone(),
                    range: s_ref.range,
                });
            }
        }

        let mut seen = HashSet::new();
        results.retain(|loc| {
            let key = (
                loc.uri.clone(),
                loc.range.start.line,
                loc.range.start.character,
                loc.range.end.line,
                loc.range.end.character,
            );
            seen.insert(key)
        });

        results
    }

    /// Prepares rename refactoring, returning the range of the symbol to rename and its placeholder name.
    pub fn prepare_rename(&self, pos: &LspPosition) -> Option<(LspRange, String)> {
        let (word, word_range) = self.word_at_position(pos)?;

        // Disallow renaming keywords and built-in identifiers
        if get_keyword_hover(&word).is_some() || get_builtin_hover(&word).is_some() {
            return None;
        }

        if let Some(def) = self.definitions.get(&word) {
            let range = if def.def_range.contains(pos) {
                def.def_range
            } else {
                word_range
            };
            return Some((range, word));
        }

        for s_ref in &self.references {
            if s_ref.range.contains(pos) {
                return Some((s_ref.range, word));
            }
        }

        None
    }

    /// Provides semantic document highlights for the symbol at the given position.
    pub fn document_highlight(&self, pos: &LspPosition) -> Vec<DocumentHighlight> {
        let mut target_name = None;
        for s_ref in &self.references {
            if s_ref.range.contains(pos) {
                target_name = Some(s_ref.def_name.clone());
                break;
            }
        }
        if target_name.is_none() {
            for def in self.definitions.values() {
                if def.def_range.contains(pos) {
                    target_name = Some(def.name.clone());
                    break;
                }
            }
        }
        if target_name.is_none() {
            if let Some((word, _)) = self.word_at_position(pos) {
                if self.definitions.contains_key(&word)
                    || self.references.iter().any(|r| r.name == word)
                {
                    target_name = Some(word);
                }
            }
        }

        let name = match target_name {
            Some(n) => n,
            None => return Vec::new(),
        };

        let mut highlights = Vec::new();

        if let Some(def) = self.definitions.get(&name) {
            let kind = if def.kind_name == "variable" || def.kind_name == "parameter" {
                3 // Write
            } else {
                1 // Text
            };
            highlights.push(DocumentHighlight {
                range: def.def_range,
                kind,
            });
        }

        for s_ref in &self.references {
            if s_ref.name == name || s_ref.def_name == name {
                let kind = if self.is_write_reference(&s_ref.range) {
                    3 // Write
                } else {
                    2 // Read
                };
                highlights.push(DocumentHighlight {
                    range: s_ref.range,
                    kind,
                });
            }
        }

        let mut seen = HashSet::new();
        highlights.retain(|h| {
            let key = (
                h.range.start.line,
                h.range.start.character,
                h.range.end.line,
                h.range.end.character,
            );
            seen.insert(key)
        });

        highlights
    }

    /// Checks if a reference range is a write/mutation site (e.g. assignment target).
    fn is_write_reference(&self, range: &LspRange) -> bool {
        let line_idx = range.start.line as usize;
        if line_idx >= self.lines.len() {
            return false;
        }
        let line = &self.lines[line_idx];

        let end_char = range.end.character as usize;
        if end_char < line.len() {
            let remainder = line[end_char..].trim_start();
            if remainder.starts_with('=')
                && !remainder.starts_with("==")
                && !remainder.starts_with("=>")
            {
                return true;
            }
        }

        let start_char = range.start.character as usize;
        if start_char <= line.len() {
            let prefix = line[..start_char].trim_end();
            if prefix.ends_with("let mut") || prefix.ends_with("let") {
                return true;
            }
        }

        false
    }

    /// Computes signature help and active parameter index for the call at cursor.
    pub fn signature_help(&self, pos: &LspPosition) -> Option<SignatureHelp> {
        let line_idx = pos.line as usize;
        if line_idx >= self.lines.len() {
            return None;
        }
        let line = &self.lines[line_idx];
        let col = (pos.character as usize).min(line.len());
        let before_cursor = &line[..col];

        let mut paren_depth = 0;
        let mut open_paren_idx = None;
        let mut active_param = 0;

        for (idx, ch) in before_cursor.char_indices().rev() {
            match ch {
                ')' => paren_depth += 1,
                '(' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    } else {
                        open_paren_idx = Some(idx);
                        break;
                    }
                }
                ',' if paren_depth == 0 => {
                    active_param += 1;
                }
                _ => {}
            }
        }

        let paren_idx = open_paren_idx?;
        let prefix = line[..paren_idx].trim_end();

        let fn_name_start = prefix
            .rfind(|c: char| !is_ident_char(c) && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);
        let fn_name = prefix[fn_name_start..].trim();
        let short_name = fn_name.rsplit('.').next().unwrap_or(fn_name);

        if let Some(def) = self.definitions.get(short_name) {
            let params = extract_param_names_from_sig(&def.type_str);
            let sig_info = SignatureInformation {
                label: def.type_str.clone(),
                documentation: def.doc_comment.clone(),
                parameters: params
                    .into_iter()
                    .map(|p| ParameterInformation {
                        label: p,
                        documentation: None,
                    })
                    .collect(),
            };
            return Some(SignatureHelp {
                signatures: vec![sig_info],
                active_signature: 0,
                active_parameter: active_param,
            });
        }

        if let Some(hover) = get_builtin_hover(short_name) {
            let first_line = hover
                .lines()
                .find(|l| l.contains("```aura"))
                .and_then(|_| hover.lines().skip_while(|l| !l.contains("```aura")).nth(1))
                .unwrap_or(short_name);
            let params = extract_param_names_from_sig(first_line);
            let sig_info = SignatureInformation {
                label: first_line.to_string(),
                documentation: Some(hover),
                parameters: params
                    .into_iter()
                    .map(|p| ParameterInformation {
                        label: p,
                        documentation: None,
                    })
                    .collect(),
            };
            return Some(SignatureHelp {
                signatures: vec![sig_info],
                active_signature: 0,
                active_parameter: active_param,
            });
        }

        None
    }

    /// Generates inferred type inlay hints for variable declarations.
    pub fn inlay_hints(&self, range: Option<&LspRange>) -> Vec<InlayHint> {
        let mut hints = Vec::new();

        for def in self.definitions.values() {
            if def.kind_name == "variable" {
                if let Some(r) = range {
                    if def.def_range.start.line < r.start.line
                        || def.def_range.end.line > r.end.line
                    {
                        continue;
                    }
                }
                let line_idx = def.def_range.start.line as usize;
                if line_idx < self.lines.len() {
                    let line = &self.lines[line_idx];
                    let after_id = if (def.def_range.end.character as usize) <= line.len() {
                        &line[def.def_range.end.character as usize..]
                    } else {
                        ""
                    };
                    let trimmed_after = after_id.trim_start();
                    if !trimmed_after.starts_with(':') {
                        if let Some(conc_ty) = self.type_env.get(&def.name) {
                            hints.push(InlayHint {
                                position: def.def_range.end,
                                label: format!(": {}", conc_ty),
                                kind: 1, // Type
                                padding_left: false,
                                padding_right: true,
                            });
                        } else if def.type_str.contains(": ") {
                            let ty_part = def.type_str.split(": ").nth(1).unwrap_or("").trim();
                            if !ty_part.is_empty() && ty_part != "Any" {
                                hints.push(InlayHint {
                                    position: def.def_range.end,
                                    label: format!(": {}", ty_part),
                                    kind: 1, // Type
                                    padding_left: false,
                                    padding_right: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        hints.sort_by_key(|h| (h.position.line, h.position.character));
        hints
    }

    /// Produces context-sensitive code actions and quick fixes.
    pub fn code_actions(&self, _range: &LspRange, diagnostics: &[Diagnostic]) -> Vec<CodeAction> {
        let mut actions = Vec::new();

        for diag in diagnostics {
            let msg = &diag.message;

            if msg.contains("immutable")
                || msg.contains("cannot assign twice")
                || msg.contains("reassignment")
            {
                let line_idx = diag.range.start.line as usize;
                if line_idx < self.lines.len() {
                    let line = &self.lines[line_idx];
                    if let Some(let_pos) = line.find("let ") {
                        let edit_range = LspRange::new(
                            diag.range.start.line,
                            let_pos as u32,
                            diag.range.start.line,
                            (let_pos + 4) as u32,
                        );
                        let mut edits = HashMap::new();
                        edits.insert(
                            self.uri.clone(),
                            vec![TextEdit {
                                range: edit_range,
                                new_text: "let mut ".to_string(),
                            }],
                        );
                        actions.push(CodeAction {
                            title: "Make variable mutable with 'let mut'".to_string(),
                            kind: "quickfix".to_string(),
                            diagnostics: vec![diag.clone()],
                            edit: Some(edits),
                            is_preferred: true,
                        });
                    }
                }
            }

            if msg.contains("Option") || msg.contains("Result") || msg.contains("unwrap") {
                let mut edits = HashMap::new();
                let insert_range = LspRange::new(
                    diag.range.end.line,
                    diag.range.end.character,
                    diag.range.end.line,
                    diag.range.end.character,
                );
                edits.insert(
                    self.uri.clone(),
                    vec![TextEdit {
                        range: insert_range,
                        new_text: "?".to_string(),
                    }],
                );
                actions.push(CodeAction {
                    title: "Unwrap Result/Option with '?' operator".to_string(),
                    kind: "quickfix".to_string(),
                    diagnostics: vec![diag.clone()],
                    edit: Some(edits),
                    is_preferred: true,
                });
            }
        }

        if let Ok(formatted) = format_aura(&self.source) {
            if formatted != self.source {
                let total_lines = self.lines.len() as u32;
                let last_len = self.lines.last().map(|l| l.len() as u32).unwrap_or(0);
                let mut edits = HashMap::new();
                edits.insert(
                    self.uri.clone(),
                    vec![TextEdit {
                        range: LspRange::new(0, 0, total_lines, last_len),
                        new_text: formatted,
                    }],
                );
                actions.push(CodeAction {
                    title: "Format Document (Aura Formatter)".to_string(),
                    kind: "source.fixAll".to_string(),
                    diagnostics: Vec::new(),
                    edit: Some(edits),
                    is_preferred: false,
                });
            }
        }

        actions
    }

    /// Produces delta-encoded semantic tokens for syntax highlighting.
    pub fn semantic_tokens_full(&self) -> Vec<u32> {
        #[derive(Debug, Clone)]
        struct RawToken {
            line: u32,
            start_char: u32,
            length: u32,
            token_type: u32,
            token_modifiers: u32,
        }

        let mut raw_tokens = Vec::new();

        // 1. Line comments
        for (i, line) in self.lines.iter().enumerate() {
            if let Some(comment_pos) = line.find("//") {
                raw_tokens.push(RawToken {
                    line: i as u32,
                    start_char: comment_pos as u32,
                    length: (line.len() - comment_pos) as u32,
                    token_type: 14, // comment
                    token_modifiers: 0,
                });
            }
        }

        // 2. Lexer tokens
        let mut lexer = Lexer::new(&self.source);
        if let Ok(toks) = lexer.tokenize() {
            for tok in toks {
                let line = if tok.span.line > 0 {
                    (tok.span.line - 1) as u32
                } else {
                    0
                };
                let start_char = if tok.span.column > 0 {
                    (tok.span.column - 1) as u32
                } else {
                    0
                };
                let length = (tok.span.end - tok.span.start) as u32;

                match tok.kind {
                    TokenKind::Fn
                    | TokenKind::Let
                    | TokenKind::Mut
                    | TokenKind::Type
                    | TokenKind::Match
                    | TokenKind::When
                    | TokenKind::If
                    | TokenKind::Else
                    | TokenKind::Async
                    | TokenKind::Await
                    | TokenKind::Import
                    | TokenKind::Export
                    | TokenKind::From
                    | TokenKind::Extern
                    | TokenKind::Trait
                    | TokenKind::Impl
                    | TokenKind::Return
                    | TokenKind::Module
                    | TokenKind::Spawn
                    | TokenKind::Routine
                    | TokenKind::Go
                    | TokenKind::Select
                    | TokenKind::Default
                    | TokenKind::While
                    | TokenKind::For
                    | TokenKind::In
                    | TokenKind::Break
                    | TokenKind::Continue
                    | TokenKind::Defer
                    | TokenKind::ErrDefer
                    | TokenKind::Embed
                    | TokenKind::EmbedBytes
                    | TokenKind::Interface
                    | TokenKind::Panic
                    | TokenKind::Recover
                    | TokenKind::Struct
                    | TokenKind::Packed => {
                        raw_tokens.push(RawToken {
                            line,
                            start_char,
                            length,
                            token_type: 12, // keyword
                            token_modifiers: 0,
                        });
                    }
                    TokenKind::True | TokenKind::False => {
                        raw_tokens.push(RawToken {
                            line,
                            start_char,
                            length,
                            token_type: 12, // keyword
                            token_modifiers: 0,
                        });
                    }
                    TokenKind::Int(_) | TokenKind::Float(_) => {
                        raw_tokens.push(RawToken {
                            line,
                            start_char,
                            length,
                            token_type: 16, // number
                            token_modifiers: 0,
                        });
                    }
                    TokenKind::String(_) | TokenKind::TemplateString(_) => {
                        raw_tokens.push(RawToken {
                            line,
                            start_char,
                            length,
                            token_type: 15, // string
                            token_modifiers: 0,
                        });
                    }
                    TokenKind::Pipe
                    | TokenKind::FatArrow
                    | TokenKind::Arrow
                    | TokenKind::ArrowLeft
                    | TokenKind::Equal
                    | TokenKind::EqualEqual
                    | TokenKind::NotEqual
                    | TokenKind::Less
                    | TokenKind::LessEqual
                    | TokenKind::Greater
                    | TokenKind::GreaterEqual
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Star
                    | TokenKind::Slash
                    | TokenKind::Percent
                    | TokenKind::Ampersand
                    | TokenKind::AndAnd
                    | TokenKind::OrOr
                    | TokenKind::Bang
                    | TokenKind::PipeOp
                    | TokenKind::Question => {
                        raw_tokens.push(RawToken {
                            line,
                            start_char,
                            length,
                            token_type: 17, // operator
                            token_modifiers: 0,
                        });
                    }
                    TokenKind::Ident(ref name) => {
                        let (token_type, token_modifiers) =
                            if let Some(def) = self.definitions.get(name) {
                                match def.kind_name.as_str() {
                                    "function" => (10, 2),  // function, definition
                                    "enum" => (2, 2),       // enum, definition
                                    "variant" => (9, 2),    // enumMember, definition
                                    "type alias" => (0, 2), // type, definition
                                    "parameter" => (6, 0),  // parameter
                                    "variable" => (7, 0),   // variable
                                    _ => (7, 0),
                                }
                            } else if get_builtin_hover(name).is_some() {
                                (10, 16) // function, defaultLibrary
                            } else if [
                                "Int", "Float", "String", "Bool", "Option", "Result", "Task",
                                "Channel", "List", "Array", "Map", "Set", "Unit",
                            ]
                            .contains(&name.as_str())
                            {
                                (0, 16) // type, defaultLibrary
                            } else {
                                (7, 0) // variable
                            };

                        raw_tokens.push(RawToken {
                            line,
                            start_char,
                            length: name.len() as u32,
                            token_type,
                            token_modifiers,
                        });
                    }
                    _ => {}
                }
            }
        }

        raw_tokens.sort_by(|a, b| (a.line, a.start_char).cmp(&(b.line, b.start_char)));

        let mut deduped: Vec<RawToken> = Vec::new();
        for tok in raw_tokens {
            if let Some(last) = deduped.last() {
                if last.line == tok.line && tok.start_char < last.start_char + last.length {
                    continue;
                }
            }
            deduped.push(tok);
        }

        let mut encoded = Vec::new();
        let mut prev_line = 0;
        let mut prev_char = 0;

        for tok in deduped {
            let delta_line = tok.line - prev_line;
            let delta_char = if delta_line == 0 {
                tok.start_char - prev_char
            } else {
                tok.start_char
            };

            encoded.push(delta_line);
            encoded.push(delta_char);
            encoded.push(tok.length);
            encoded.push(tok.token_type);
            encoded.push(tok.token_modifiers);

            prev_line = tok.line;
            prev_char = tok.start_char;
        }

        encoded
    }

    /// Generates code lenses (e.g. Run Main and Reference counts).
    pub fn code_lens(&self) -> Vec<CodeLens> {
        let mut lenses = Vec::new();

        for def in self.definitions.values() {
            if def.kind_name == "function" && def.name == "main" {
                lenses.push(CodeLens {
                    range: def.def_range,
                    command: Some(CodeLensCommand {
                        title: "▶ Run main".to_string(),
                        command: "aura.runMain".to_string(),
                        arguments: vec![self.uri.clone()],
                    }),
                });
            } else if def.kind_name == "function"
                || def.kind_name == "enum"
                || def.kind_name == "type alias"
            {
                let ref_count = self
                    .references
                    .iter()
                    .filter(|r| r.name == def.name || r.def_name == def.name)
                    .count();
                let title = if ref_count == 1 {
                    "1 reference".to_string()
                } else {
                    format!("{} references", ref_count)
                };
                lenses.push(CodeLens {
                    range: def.def_range,
                    command: Some(CodeLensCommand {
                        title,
                        command: "aura.showReferences".to_string(),
                        arguments: vec![
                            self.uri.clone(),
                            def.def_range.start.line.to_string(),
                            def.def_range.start.character.to_string(),
                        ],
                    }),
                });
            }
        }

        lenses.sort_by_key(|l| l.range.start.line);
        lenses
    }

    /// Identifies collapsible code ranges (blocks, multi-line comments).
    pub fn folding_ranges(&self) -> Vec<FoldingRange> {
        let mut ranges = Vec::new();
        let mut brace_stack = Vec::new();

        for (line_idx, line) in self.lines.iter().enumerate() {
            let u_line = line_idx as u32;
            for ch in line.chars() {
                if ch == '{' {
                    brace_stack.push(u_line);
                } else if ch == '}' {
                    if let Some(start_line) = brace_stack.pop() {
                        if u_line > start_line {
                            ranges.push(FoldingRange {
                                start_line,
                                end_line: u_line,
                                kind: Some("region".to_string()),
                            });
                        }
                    }
                }
            }
        }

        let mut comment_start = None;
        for (line_idx, line) in self.lines.iter().enumerate() {
            let u_line = line_idx as u32;
            if line.contains("/*") {
                comment_start = Some(u_line);
            }
            if line.contains("*/") {
                if let Some(start_line) = comment_start.take() {
                    if u_line > start_line {
                        ranges.push(FoldingRange {
                            start_line,
                            end_line: u_line,
                            kind: Some("comment".to_string()),
                        });
                    }
                }
            }
        }

        ranges.sort_by_key(|r| r.start_line);
        ranges
    }

    /// Prepares call hierarchy for the function at cursor.
    pub fn prepare_call_hierarchy(&self, pos: &LspPosition) -> Option<CallHierarchyItem> {
        for def in self.definitions.values() {
            if def.kind_name == "function" && def.def_range.contains(pos) {
                return Some(CallHierarchyItem {
                    name: def.name.clone(),
                    kind: SymbolKind::Function,
                    uri: self.uri.clone(),
                    range: def.def_range,
                    selection_range: def.def_range,
                });
            }
        }
        for s_ref in &self.references {
            if s_ref.range.contains(pos) {
                if let Some(def) = self.definitions.get(&s_ref.def_name) {
                    if def.kind_name == "function" {
                        return Some(CallHierarchyItem {
                            name: def.name.clone(),
                            kind: SymbolKind::Function,
                            uri: self.uri.clone(),
                            range: def.def_range,
                            selection_range: def.def_range,
                        });
                    }
                }
            }
        }
        None
    }

    /// Finds incoming calls targeting the given function name.
    pub fn incoming_calls(&self, item_name: &str) -> Vec<CallHierarchyIncomingCall> {
        let mut calls = Vec::new();

        for caller_def in self.definitions.values() {
            if caller_def.kind_name == "function" && caller_def.name != item_name {
                let call_ranges: Vec<LspRange> = self
                    .references
                    .iter()
                    .filter(|r| {
                        (r.name == item_name || r.def_name == item_name)
                            && r.range.start.line >= caller_def.def_range.start.line
                    })
                    .map(|r| r.range)
                    .collect();

                if !call_ranges.is_empty() {
                    calls.push(CallHierarchyIncomingCall {
                        from: CallHierarchyItem {
                            name: caller_def.name.clone(),
                            kind: SymbolKind::Function,
                            uri: self.uri.clone(),
                            range: caller_def.def_range,
                            selection_range: caller_def.def_range,
                        },
                        from_ranges: call_ranges,
                    });
                }
            }
        }

        calls
    }

    /// Finds outgoing calls made from within the given function.
    pub fn outgoing_calls(&self, item_name: &str) -> Vec<CallHierarchyOutgoingCall> {
        let mut calls = Vec::new();
        let target_def = match self.definitions.get(item_name) {
            Some(d) if d.kind_name == "function" => d,
            _ => return Vec::new(),
        };

        for called_def in self.definitions.values() {
            if called_def.kind_name == "function" && called_def.name != item_name {
                let from_ranges: Vec<LspRange> = self
                    .references
                    .iter()
                    .filter(|r| {
                        (r.name == called_def.name || r.def_name == called_def.name)
                            && r.range.start.line >= target_def.def_range.start.line
                    })
                    .map(|r| r.range)
                    .collect();

                if !from_ranges.is_empty() {
                    calls.push(CallHierarchyOutgoingCall {
                        to: CallHierarchyItem {
                            name: called_def.name.clone(),
                            kind: SymbolKind::Function,
                            uri: self.uri.clone(),
                            range: called_def.def_range,
                            selection_range: called_def.def_range,
                        },
                        from_ranges,
                    });
                }
            }
        }

        calls
    }
}

fn extract_param_names_from_sig(sig: &str) -> Vec<String> {
    if let Some(start) = sig.find('(') {
        if let Some(end) = sig.find(')') {
            if end > start {
                let inner = &sig[start + 1..end];
                if inner.trim().is_empty() {
                    return Vec::new();
                }
                return inner
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn find_ident_range(spans: &[(&str, LspRange)], name: &str, min_line: u32) -> LspRange {
    for (ident, range) in spans {
        if *ident == name && range.start.line >= min_line {
            return *range;
        }
    }
    LspRange::new(min_line, 0, min_line, name.len() as u32)
}

fn extract_doc_comments(source: &str) -> HashMap<String, String> {
    let mut comments = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();

    for i in 0..lines.len() {
        let line = lines[i].trim();
        if line.starts_with("fn ")
            || line.starts_with("pub fn ")
            || line.starts_with("export fn ")
            || line.starts_with("type ")
            || line.starts_with("export type ")
        {
            // Gather doc lines above
            let mut doc_lines = Vec::new();
            let mut j = i;
            while j > 0 {
                j -= 1;
                let prev = lines[j].trim();
                if prev.starts_with("///") {
                    doc_lines.push(prev.trim_start_matches("///").trim().to_string());
                } else if prev.starts_with("//") {
                    doc_lines.push(prev.trim_start_matches("//").trim().to_string());
                } else {
                    break;
                }
            }

            if !doc_lines.is_empty() {
                doc_lines.reverse();
                // Extract symbol name
                if let Some(sym_name) = extract_name_from_decl(line) {
                    comments.insert(sym_name, doc_lines.join("\n"));
                }
            }
        }
    }

    comments
}

fn extract_name_from_decl(decl: &str) -> Option<String> {
    let tokens: Vec<&str> = decl.split_whitespace().collect();
    for i in 0..tokens.len() {
        if (tokens[i] == "fn" || tokens[i] == "type") && i + 1 < tokens.len() {
            let raw_name = tokens[i + 1];
            let clean_name = raw_name
                .split(|c| c == '(' || c == '<' || c == '=' || c == '{' || c == ':')
                .next()
                .unwrap_or("");
            if !clean_name.is_empty() {
                return Some(clean_name.to_string());
            }
        }
    }
    None
}

fn parse_error_range(err: &str, lines: &[String]) -> LspRange {
    // Try to match line N, col M patterns
    if let Some(pos) = extract_line_col_from_error(err) {
        let line = if pos.0 > 0 { pos.0 - 1 } else { 0 };
        let col = if pos.1 > 0 { pos.1 - 1 } else { 0 };
        let line_len = lines
            .get(line as usize)
            .map(|l| l.len() as u32)
            .unwrap_or(col + 5);
        return LspRange::new(line, col, line, line_len.max(col + 1));
    }
    LspRange::new(0, 0, 0, 1)
}

fn extract_line_col_from_error(err: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = err
        .split(|c: char| !c.is_numeric())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() >= 2 {
        if let (Ok(l), Ok(c)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            if l < 10000 && c < 500 {
                return Some((l, c));
            }
        }
    }
    None
}

fn get_keyword_hover(kw: &str) -> Option<String> {
    let doc = match kw {
        "fn" => {
            "### `fn` *(Keyword)*\n\n**`fn` keyword** — Declares a named function or lambda closure in Aura.\n\n```aura\nfn <name>(<params>): <ReturnType> => <Body>\n```\n\nFunctions are first-class citizens, support lexical scoping, and tail-recursive calls are automatically compiled with **Tail-Call Optimization (TCO)** into iterative `while (true)` loops.\n\n---\n#### 📥 Parameters / Entrada\n* `params`: Comma-separated typed parameters (`param: Type` or `param: Type = defaultExpr`).\n* `ReturnType`: Output type annotation (`Int`, `String`, `Unit`, `Result<T, E>`, etc.).\n\n---\n#### 📤 Returns\n* Evaluates to the result of the body expression or `Unit` (`()`).\n\n---\n#### 💡 Example\n```aura\nfn add(a: Int, b: Int): Int => {\n    a + b\n}\n```"
        }
        "let" => {
            "### `let` *(Keyword)*\n\nDeclares an **immutable** variable binding (default in Aura).\n\n```aura\nlet <name>: <Type> = <value>;\n```\n\nOnce bound, an immutable variable cannot be reassigned or mutated, guaranteeing thread safety and eliminating race conditions.\n\n---\n#### 📥 Parameters / Operandos\n* `name`: Identifier of the variable.\n* `Type` (opcional): Explicit type annotation (inferred if omitted).\n* `value`: Initializer expression.\n\n---\n#### 💡 Example\n```aura\nlet port: Int = 8080;\nlet greeting = \"Hello, World!\";\n```"
        }
        "mut" => {
            "### `mut` *(Keyword)*\n\nDeclares a **mutable** variable binding modifier when combined with `let`.\n\n```aura\nlet mut <name>: <Type> = <initial_value>;\n```\n\nAllows subsequent reassignments to the variable using the `=` assignment operator.\n\n---\n#### 📥 Parameters / Operandos\n* `name`: Identifier of the mutable variable.\n* `initial_value`: Initial expression value.\n\n---\n#### 💡 Example\n```aura\nlet mut counter = 0;\ncounter = counter + 1;\n```"
        }
        "type" => {
            "### `type` *(Keyword)*\n\nDeclares an **Algebraic Data Type (ADT / Sum Type)**, record type, or type alias.\n\n```aura\ntype <Name> = | <Variant1>(<Type>) | <Variant2>;\n```\n\nVariants are checked for exhaustiveness at compile time inside `match` expressions.\n\n---\n#### 📥 Parameters / Definition\n* `Name`: PascalCase type identifier.\n* `variants`: Pipe-separated constructors (`Variant(Payload)` or `UnitVariant`).\n\n---\n#### 💡 Example\n```aura\ntype Shape =\n    | Circle(Float)\n    | Rectangle(Float, Float)\n    | Point;\n```"
        }
        "interface" => {
            "### `interface` *(Keyword)*\n\nDeclares a **structural interface** (Go-style duck typing).\n\n```aura\ninterface <Name> {\n    fn <method>(<params>): <ReturnType>;\n}\n```\n\nAny type that implements all required method signatures implicitly satisfies the interface without explicit inheritance or `implements` declarations.\n\n---\n#### 📥 Definition\n* `Name`: Interface identifier.\n* `methods`: Function signatures required by the interface contract.\n\n---\n#### 💡 Example\n```aura\ninterface Reader {\n    fn read(buf: List<Byte>): Result<Int, Error>;\n}\n```"
        }
        "match" => {
            "### `match` *(Keyword / Expression)*\n\nExhaustive pattern matching expression with sum type destructuring and pattern guards.\n\n```aura\nmatch <expr> {\n    <Pattern> => <Branch>,\n    <Pattern> when <guard> => <Branch>,\n    _ => <Default>\n}\n```\n\nEvaluates to the value of the branch corresponding to the first matching pattern.\n\n---\n#### 📥 Operandos\n* `expr`: Target expression evaluated and destructured.\n* `Pattern`: Constructor, literal, tuple, list, or wildcard (`_`).\n* `guard` (opcional): Boolean predicate via `when`.\n\n---\n#### 📤 Returns\n* Result of the matched branch expression. All branches must unify to the same type.\n\n---\n#### 💡 Example\n```aura\nmatch res {\n    Ok(data) => println(`Received: ${data}`),\n    Err(err) => eprintln(`Error: ${err}`),\n}\n```"
        }
        "when" => {
            "### `when` *(Keyword)*\n\nPattern guard condition attached to a `match` arm.\n\n```aura\n<Pattern> when <boolean_condition> => <Branch>\n```\n\nThe arm is selected only if the pattern matches AND `<boolean_condition>` evaluates to `true`.\n\n---\n#### 📥 Operandos\n* `boolean_condition`: Expression evaluating to `Bool`.\n\n---\n#### 💡 Example\n```aura\nmatch n {\n    x when x > 0 => \"positive\",\n    0 => \"zero\",\n    _ => \"negative\",\n}\n```"
        }
        "defer" => {
            "### `defer` *(Keyword)*\n\nDefers statement execution until the enclosing function returns.\n\n```aura\ndefer <statement_or_block>;\n```\n\nExecutes in deterministic **LIFO (Last-In, First-Out)** order regardless of whether the function returns normally or early via `?` or `return`.\n\n---\n#### 📥 Operando\n* `statement`: Cleanup action executed upon function exit.\n\n---\n#### 💡 Example\n```aura\nlet file = openFile(\"data.txt\")?;\ndefer file.close();\n```"
        }
        "errdefer" => {
            "### `errdefer` *(Keyword)*\n\nDefers statement execution until the enclosing function returns, **only if an error or panic occurs** (Zig model).\n\n```aura\nerrdefer <statement_or_block>;\n```\n\nExecutes in deterministic LIFO order only during failure unwinding. Ideal for transactional rollbacks.\n\n---\n#### 📥 Operando\n* `statement`: Cleanup action executed solely on failure.\n\n---\n#### 💡 Example\n```aura\nlet tx = db.beginTransaction()?;\nerrdefer tx.rollback();\n```"
        }
        "packed" => {
            "### `packed` *(Keyword)*\n\nDeclares a **packed structure** with zero padding and exact binary memory alignment (Zig model).\n\n```aura\npacked struct <Name> {\n    <field>: <Type>,\n}\n```\n\nIdeal for binary network protocols, file formats, and zero-copy deserialization.\n\n---\n#### 💡 Example\n```aura\npacked struct IpHeader {\n    version: Byte,\n    ttl: Byte,\n    protocol: Byte,\n}\n```"
        }
        "spawn" | "routine" | "go" => {
            "### `go` / `routine` / `spawn` *(Keyword)*\n\nLaunches a **Goroutine / Aura Routine** (lightweight fiber).\n\n```aura\ngo {\n    <concurrent_work>\n};\n```\n\nExecutes on an M:N cooperative scheduler with low memory footprint (~4KB stack). Communicates via typed CSP channels (`Channel<T>`).\n\n---\n#### 📥 Operando\n* `block`: Closure or function call executed asynchronously.\n\n---\n#### 💡 Example\n```aura\ngo worker(ch);\n```"
        }
        "select" => {
            "### `select` *(Keyword)*\n\nMultiplexes on multiple concurrent channel operations (CSP model).\n\n```aura\nselect {\n    case msg <- ch => { ... },\n    case ch2 <- out => { ... },\n    default => { ... }\n}\n```\n\nBlocks until one communication is ready. If `default` is provided, executes non-blocking.\n\n---\n#### 💡 Example\n```aura\nselect {\n    case msg <- inbox => println(`Received: ${msg}`),\n    default => println(\"No message pending\"),\n}\n```"
        }
        "embed" => {
            "### `embed` *(Macro / Keyword)*\n\nEmbeds a static text asset directly into the compiled standalone binary at build time as a UTF-8 `String`.\n\n```aura\nembed(\"<relative_path>\"): String\n```\n\n---\n#### 📥 Parameters\n* `path` (`String`): Relative filesystem path to the text file.\n\n---\n#### 📤 Returns\n* UTF-8 `String` embedded in binary executable.\n\n---\n#### 💡 Example\n```aura\nlet schema = embed(\"schema.sql\");\n```"
        }
        "embedBytes" => {
            "### `embedBytes` *(Macro / Keyword)*\n\nEmbeds a static binary file directly into the compiled output as a byte array (`List<Byte>`).\n\n```aura\nembedBytes(\"<relative_path>\"): List<Byte>\n```\n\n---\n#### 📥 Parameters\n* `path` (`String`): Relative filesystem path to the binary file.\n\n---\n#### 📤 Returns\n* Immutable `List<Byte>` buffer.\n\n---\n#### 💡 Example\n```aura\nlet iconData = embedBytes(\"assets/icon.png\");\n```"
        }
        "async" => {
            "### `async` *(Keyword)*\n\nMarks a function as asynchronous, returning a `Task<T, E>`.\n\n```aura\nasync fn <name>(<params>): Task<T, E> => <Body>\n```\n\nEnables cooperative concurrency that can be awaited using `await`."
        }
        "await" => {
            "### `await` *(Keyword)*\n\nSuspends fiber execution until the awaited `Task<T, E>` resolves, unwrapping its inner value.\n\n```aura\nlet result = await <task_expression>;\n```\n\n---\n#### 📥 Operando\n* `task_expression`: An expression of type `Task<T, E>`.\n\n---\n#### 📤 Returns\n* Unwrapped value of type `T` upon successful task completion."
        }
        "panic" => {
            "### `panic` *(Built-in Function)*\n\nStops normal execution and initiates stack unwinding.\n\n```aura\npanic(message: String): Never\n```\n\nCan be intercepted with `recover()` inside a `defer` block.\n\n---\n#### 📥 Parameters\n* `message` (`String`): Diagnostic error message.\n\n---\n#### 📤 Returns\n* `Never` (never returns normally)."
        }
        "recover" => {
            "### `recover` *(Built-in Function)*\n\nCatches an active panic during unwinding inside a `defer` block.\n\n```aura\nrecover(): Option<Any>\n```\n\n---\n#### 📤 Returns\n* `Some(error)` if a panic was caught, or `None` if execution was normal."
        }
        "if" => {
            "### `if` *(Keyword / Expression)*\n\nConditional branching expression.\n\n```aura\nif <cond> => <ThenExpr> else => <ElseExpr>\n```\n\nIn Aura, `if` is an expression yielding a value. Both branches must unify to the same type.\n\n---\n#### 📥 Operandos\n* `cond` (`Bool`): Branch condition.\n* `ThenExpr`: Value if `cond` is `true`.\n* `ElseExpr`: Value if `cond` is `false`.\n\n---\n#### 💡 Example\n```aura\nlet max = if a > b => a else => b;\n```"
        }
        "else" => {
            "### `else` *(Keyword)*\n\nAlternative branch executed when an `if` condition evaluates to `false`."
        }
        "while" => {
            "### `while` *(Keyword)*\n\nIterative loop. Runs while `<condition>` is `true`.\n\n```aura\nwhile <condition> { <body> }\n```\n\n---\n#### 📥 Operandos\n* `condition` (`Bool`): Loop continuation test.\n* `body`: Statements executed sequentially per iteration."
        }
        "for" => {
            "### `for` *(Keyword)*\n\nIterates over a list, array, range, or CSP channel.\n\n```aura\nfor <item> in <iterable_or_channel> { <body> }\n```\n\n---\n#### 📥 Operandos\n* `item`: Variable bound to each element.\n* `iterable_or_channel`: Source collection or channel.\n\n---\n#### 💡 Example\n```aura\nfor num in [1, 2, 3, 4] {\n    println(num);\n}\n```"
        }
        "in" => {
            "### `in` *(Keyword)*\n\nSpecifies the collection, range, or channel to iterate over in a `for` loop."
        }
        "break" => {
            "### `break` *(Keyword)*\n\nTerminates the innermost or labeled enclosing loop immediately."
        }
        "continue" => {
            "### `continue` *(Keyword)*\n\nSkips to the next iteration of the innermost or labeled enclosing loop."
        }
        "return" => {
            "### `return` *(Keyword)*\n\nExits early from the enclosing function, returning an optional expression value.\n\n```aura\nreturn <value>;\n```"
        }
        "import" => {
            "### `import` *(Keyword)*\n\nImports functions, types, and interfaces from relative `.aura` modules or standard packages.\n\n```aura\nimport { Symbol1, Symbol2 as Alias } from \"<path_or_module>\";\n```"
        }
        "export" => {
            "### `export` *(Keyword)*\n\nMarks declarations as publicly visible to other modules and external consumers.\n\n```aura\nexport fn calculate(): Int => 42;\nexport type User = { id: Int, name: String };\n```"
        }
        "from" => {
            "### `from` *(Keyword)*\n\nSpecifies the source path or package identifier in an `import` declaration."
        }
        "as" => {
            "### `as` *(Keyword)*\n\nAliases an imported symbol locally: `import { Original as Alias } from \"module\";`."
        }
        "module" => {
            "### `module` *(Keyword)*\n\nDeclares the hierarchical module namespace of the source file.\n\n```aura\nmodule Aura.Net.Http;\n```"
        }
        "case" => {
            "### `case` *(Keyword)*\n\nSpecifies a communication branch inside a `select` multiplexer block:\n\n```aura\ncase msg <- ch => { ... }\n```"
        }
        "default" => {
            "### `default` *(Keyword)*\n\nNon-blocking fallback branch in `select` multiplexing."
        }
        "trait" => {
            "### `trait` *(Keyword)*\n\nDefines a set of method signatures that types can implement."
        }
        "impl" => {
            "### `impl` *(Keyword)*\n\nImplements a trait or attaches methods directly to a data type."
        }
        "const" => {
            "### `const` *(Keyword)*\n\nDeclares an immutable compile-time constant evaluated during semantic analysis."
        }
        "|>" => {
            "### `|>` *(Pipeline Operator)*\n\nChains transformations cleanly by passing the left-hand expression as the first argument of the right-hand function call.\n\n```aura\ndata |> step1 |> step2(extra_arg)\n```\n\n---\n#### 📥 Operandos\n* `left`: Data value expression.\n* `right`: Callable taking the data value as its first parameter.\n\n---\n#### 📤 Returns\n* Result of `right(left)`.\n\n---\n#### 💡 Example\n```aura\nlet evens = numbers\n    |> filter(fn(x) => x % 2 == 0)\n    |> map(fn(x) => x * 10);\n```"
        }
        "?" => {
            "### `?` *(Try / Propagate Operator)*\n\nUnwraps `Ok(v)` from `Result<T, E>` or `Some(v)` from `Option<T>`. If the value is `Err(e)` or `None`, early returns from the enclosing function.\n\n```aura\nlet file = openFile(\"data.txt\")?;\n```\n\n---\n#### 📥 Operando\n* `inner`: An expression of type `Result<T, E>` or `Option<T>`.\n\n---\n#### 📤 Returns\n* Unwrapped value of type `T` on success.\n\n---\n#### 💡 Example\n```aura\nfn loadConfig(): Result<Config, Error> => {\n    let text = readFile(\"config.json\")?;\n    let cfg = json.parse(text)?;\n    Ok(cfg)\n}\n```"
        }
        "<-" => {
            "### `<-` *(Channel Send / Receive Operator)*\n\nCSP channel communication operator.\n\n```aura\nch <- message;  // Send (binary)\nlet msg = <-ch; // Receive (unary prefix)\n```\n\n---\n#### 📥 Operandos\n* Send: `channel <- value` (suspends until buffer has space or receiver is ready).\n* Receive: `<-channel` (suspends until a message is available).\n\n---\n#### 💡 Example\n```aura\nlet ch = Channel<String>::new(5);\nch <- \"ping\";\nlet response = <-ch;\n```"
        }
        _ => return None,
    };
    Some(doc.to_string())
}

fn get_builtin_hover(name: &str) -> Option<String> {
    let doc = match name {
        "Int" => {
            "### `Int` *(Core Primitive Type)*\n\n64-bit signed integer (`i64`). Supports arithmetic (+, -, *, /, %), bitwise operators, and comparisons.\n\n```aura\nlet x: Int = 42;\n```"
        }
        "Float" => {
            "### `Float` *(Core Primitive Type)*\n\n64-bit IEEE 754 floating-point number (`f64`). Supports standard mathematical operations.\n\n```aura\nlet pi: Float = 3.1415926535;\n```"
        }
        "String" => {
            "### `String` *(Core Primitive Type)*\n\nImmutable UTF-8 string with template interpolation support (`${expr}`).\n\n```aura\nlet msg: String = `Hello, ${name}!`;\n```"
        }
        "Bool" => {
            "### `Bool` *(Core Primitive Type)*\n\nBoolean truth value (`true` or `false`). Supports logical operators `&&`, `||`, and `!`."
        }
        "Unit" => {
            "### `Unit` *(Core Primitive Type)*\n\nUnit type `()`. Denotes the absence of a meaningful value (equivalent to `void`)."
        }
        "Byte" => {
            "### `Byte` *(Core Primitive Type)*\n\n8-bit unsigned integer (`0..255`). Used for raw binary buffers and byte arrays."
        }
        "Option" => {
            "### `Option<T>` *(Standard Sum Type)*\n\n```aura\ntype Option<T> = | Some(T) | None\n```\n\nEliminates null and undefined pointers entirely. Unwrapped via `match` or `?` operator."
        }
        "Some" => {
            "### `Some(T)` *(Option Constructor)*\n\nWraps an existing value in an `Option<T>`.\n\n```aura\nSome(value: T): Option<T>\n```"
        }
        "None" => {
            "### `None` *(Option Constructor)*\n\nRepresents the absence of a value in an `Option<T>`.\n\n```aura\nNone: Option<Never>\n```"
        }
        "Result" => {
            "### `Result<T, E>` *(Standard Sum Type)*\n\n```aura\ntype Result<T, E> = | Ok(T) | Err(E)\n```\n\nRepresents success (`Ok(T)`) or failure (`Err(E)`). Propagated with the `?` operator."
        }
        "Ok" => {
            "### `Ok(T)` *(Result Constructor)*\n\nWraps a successful computation payload in a `Result<T, E>`.\n\n```aura\nOk(value: T): Result<T, Never>\n```"
        }
        "Err" => {
            "### `Err(E)` *(Result Constructor)*\n\nWraps an error diagnostic payload in a `Result<T, E>`.\n\n```aura\nErr(error: E): Result<Never, E>\n```"
        }
        "Task" => {
            "### `Task<T, E>` *(Async Computation)*\n\nRepresents an asynchronous computation that can be awaited with `await`.\n\n```aura\nlet res: T = await task;\n```"
        }
        "Channel" => {
            "### `Channel<T>` *(CSP Concurrency)*\n\nTyped concurrent CSP channel for inter-fiber communication.\n\n```aura\nChannel<T>::new(buffer_size: Int = 0): Channel<T>\n```\n\n---\n#### 📥 Parameters\n* `buffer_size` (`Int`, optional): Buffer capacity. 0 denotes an unbuffered rendezvous channel.\n\n---\n#### 📤 Operations\n* Send: `ch <- val`\n* Receive: `let val = <-ch`"
        }
        "SendChannel" => {
            "### `SendChannel<T>` *(Write-Only Channel)*\n\nWrite-only channel view. Only sends (`ch <- val`) permitted."
        }
        "RecvChannel" => {
            "### `RecvChannel<T>` *(Read-Only Channel)*\n\nRead-only channel view. Only receives (`<-ch`) permitted."
        }
        "Context" => {
            "### `Context` *(Structured Concurrency)*\n\nPropagates cancellation signals and deadlines across concurrent fibers."
        }
        "WaitGroup" => {
            "### `WaitGroup` *(Sync Primitive)*\n\nCoordinates and waits for a group of concurrent fibers to complete via `add(n)`, `done()`, and `wait()`."
        }
        "Mutex" => {
            "### `Mutex` *(Sync Primitive)*\n\nMutual exclusion lock with `lock()` and `unlock()`."
        }
        "RWMutex" => {
            "### `RWMutex` *(Sync Primitive)*\n\nRead-write mutual exclusion lock with `rlock()` for shared readers and `lock()` for exclusive writers."
        }
        "Once" => {
            "### `Once` *(Sync Primitive)*\n\nExecutes an initialization routine exactly once across concurrent fibers."
        }
        "List" => {
            "### `List<T>` *(Collection)*\n\nImmutable indexed sequence of elements of type `T`. Supports `.map()`, `.filter()`, `.sort()`, indexing (`list[i]`), and pipeline transformations."
        }
        "Array" => {
            "### `Array<T>` *(Collection)*\n\nOrdered collection of elements of type `T` (alias for `List<T>`). Supports `.map()`, `.filter()`, `.sort()`, and pipeline transformations."
        }
        "Iterator" | "Interator" => {
            "### `Iterator<T>` *(Interface)*\n\n```aura\ninterface Iterator<T> {\n  next(): Option<T>;\n}\n```\n\nInterface for sequential item iteration. Provides `.map()`, `.filter()`, `.sort()`, `.toList()`, and `for..in` consumption."
        }
        "Map" => {
            "### `Map<K, V>` *(Collection)*\n\nAssociative key-value hash map collection. Supports indexing `map[key]`, `.get(key)`, and `.has(key)`."
        }
        "Set" => {
            "### `Set<T>` *(Collection)*\n\nCollection of unique values of type `T`. Supports `.has(item)` and `.add(item)`."
        }
        "println" => {
            "### `println` *(Built-in Function)*\n\nPrints a formatted value followed by a newline to standard output (`stdout`).\n\n```aura\nprintln(value: Any): Unit\n```\n\n---\n#### 📥 Parameters\n* `value` (`Any`): The value, string, or data structure to display.\n\n---\n#### 📤 Returns\n* `Unit` (`()`).\n\n---\n#### 💡 Example\n```aura\nprintln(\"Hello, World!\");\n```"
        }
        "print" => {
            "### `print` *(Built-in Function)*\n\nPrints a formatted value without appending a newline to standard output (`stdout`).\n\n```aura\nprint(value: Any): Unit\n```\n\n---\n#### 📥 Parameters\n* `value` (`Any`): The value to display.\n\n---\n#### 📤 Returns\n* `Unit` (`()`)."
        }
        "eprintln" => {
            "### `eprintln` *(Built-in Function)*\n\nPrints a formatted value followed by a newline to standard error (`stderr`).\n\n```aura\neprintln(value: Any): Unit\n```\n\n---\n#### 📥 Parameters\n* `value` (`Any`): The error message or diagnostic payload.\n\n---\n#### 📤 Returns\n* `Unit` (`()`)."
        }
        "eprint" => {
            "### `eprint` *(Built-in Function)*\n\nPrints a formatted value without appending a newline to standard error (`stderr`).\n\n```aura\neprint(value: Any): Unit\n```\n\n---\n#### 📥 Parameters\n* `value` (`Any`): Error text to display.\n\n---\n#### 📤 Returns\n* `Unit` (`()`)."
        }
        "len" => {
            "### `len` *(Built-in Function)*\n\nReturns the element count or character length of a collection, string, map, or channel.\n\n```aura\nlen(container: Any): Int\n```\n\n---\n#### 📥 Parameters\n* `container` (`List<T> | String | Map<K, V> | Channel<T>`): Target data structure.\n\n---\n#### 📤 Returns\n* `Int`: Non-negative 64-bit integer count.\n\n---\n#### 💡 Example\n```aura\nlet count = len([10, 20, 30]); // 3\n```"
        }
        "cap" => {
            "### `cap` *(Built-in Function)*\n\nReturns the allocated buffer capacity of a channel or preallocated collection.\n\n```aura\ncap(container: Any): Int\n```\n\n---\n#### 📥 Parameters\n* `container` (`Channel<T> | List<T>`): Target container.\n\n---\n#### 📤 Returns\n* `Int`: Buffer capacity."
        }
        "map" => {
            "### `map` *(Collection & Iterator Transform)*\n\nTransforms each element of a collection or iterator by applying a mapping closure.\n\n```aura\nmap(coll: Array<T>, fn: (item: T) => U): Array<U>\nmap(coll: Iterator<T>, fn: (item: T) => U): Iterator<U>\n```\n\n---\n#### 📥 Parameters\n* `coll` (`Array<T> | Iterator<T>`): Source collection.\n* `fn` (`(item: T) => U`): Mapping transformation closure.\n\n---\n#### 📤 Returns\n* Collection or iterator containing mapped elements.\n\n---\n#### 💡 Example\n```aura\nlet doubled = [1, 2, 3].map(fn(x) => x * 2);\n```"
        }
        "filter" => {
            "### `filter` *(Collection & Iterator Filter)*\n\nSelects elements matching a predicate closure.\n\n```aura\nfilter(coll: Array<T>, pred: (item: T) => Bool): Array<T>\nfilter(coll: Iterator<T>, pred: (item: T) => Bool): Iterator<T>\n```\n\n---\n#### 📥 Parameters\n* `coll` (`Array<T> | Iterator<T>`): Source collection.\n* `pred` (`(item: T) => Bool`): Filter predicate.\n\n---\n#### 📤 Returns\n* Collection or iterator containing only matching elements.\n\n---\n#### 💡 Example\n```aura\nlet evens = numbers.filter(fn(x) => x % 2 == 0);\n```"
        }
        "sort" => {
            "### `sort` *(Collection & Iterator Sort)*\n\nReturns elements sorted numerically or by a custom comparator closure. Pure and non-mutating.\n\n```aura\nsort(coll: Array<T>, cmp?: (a: T, b: T) => Int): Array<T>\nsort(coll: Iterator<T>, cmp?: (a: T, b: T) => Int): Iterator<T>\n```\n\n---\n#### 📥 Parameters\n* `coll` (`Array<T> | Iterator<T>`): Source collection.\n* `cmp` (`(a: T, b: T) => Int`, optional): Ordering comparator returning `< 0`, `0`, or `> 0`.\n\n---\n#### 📤 Returns\n* Sorted collection copy.\n\n---\n#### 💡 Example\n```aura\nlet sortedAsc = scores.sort();\nlet sortedDesc = scores.sort(fn(a, b) => b - a);\n```"
        }
        "iter" => {
            "### `iter` *(Built-in Function)*\n\nWraps an array, list, or iterable into a lazy `Iterator<T>` for streaming pipelines.\n\n```aura\niter(coll: Array<T> | Iterator<T>): Iterator<T>\n```\n\n---\n#### 📥 Parameters\n* `coll` (`Array<T>`): Source collection.\n\n---\n#### 📤 Returns\n* `Iterator<T>`."
        }
        "sleep" => {
            "### `sleep` *(Built-in Function)*\n\nPauses execution of the current fiber for the specified duration in milliseconds.\n\n```aura\nsleep(ms: Int): Unit\n```\n\n---\n#### 📥 Parameters\n* `ms` (`Int`): Sleep duration in milliseconds.\n\n---\n#### 📤 Returns\n* `Unit` (`()`)."
        }
        "yield" => {
            "### `yield` *(Built-in Function)*\n\nCooperatively yields the current fiber's execution slice back to the scheduler.\n\n```aura\nyield(): Unit\n```\n\n---\n#### 📤 Returns\n* `Unit` (`()`)."
        }
        "assert" => {
            "### `assert` *(Built-in Function)*\n\nVerifies that a boolean invariant is `true`. Panics if the condition evaluates to `false`.\n\n```aura\nassert(condition: Bool, message?: String): Unit\n```\n\n---\n#### 📥 Parameters\n* `condition` (`Bool`): The assertion condition to validate.\n* `message` (`String`, optional): Diagnostic message printed if the assertion fails.\n\n---\n#### 📤 Returns\n* `Unit` (`()`)."
        }
        "assert_eq" => {
            "### `assert_eq` *(Built-in Function)*\n\nPanics if two values are not structurally equal, printing a diagnostic comparison diff.\n\n```aura\nassert_eq(actual: Any, expected: Any, message?: String): Unit\n```\n\n---\n#### 📥 Parameters\n* `actual` (`Any`): Computed result.\n* `expected` (`Any`): Anticipated expected value.\n* `message` (`String`, optional): Diagnostic context message.\n\n---\n#### 📤 Returns\n* `Unit` (`()`)."
        }
        "append" => {
            "### `append` *(Built-in Function)*\n\nAppends an element to a list and returns the updated list.\n\n```aura\nappend(list: List<T>, item: T): List<T>\n```\n\n---\n#### 📥 Parameters\n* `list` (`List<T>`): Target list.\n* `item` (`T`): Element to append.\n\n---\n#### 📤 Returns\n* `List<T>`: Updated list."
        }
        "parseInt" => {
            "### `parseInt` *(Built-in Function)*\n\nParses a decimal integer string into a 64-bit integer `Int`.\n\n```aura\nparseInt(s: String): Int\n```\n\n---\n#### 📥 Parameters\n* `s` (`String`): Numeric text (e.g. `\"123\"`).\n\n---\n#### 📤 Returns\n* `Int`: Parsed 64-bit integer."
        }
        "parseFloat" => {
            "### `parseFloat` *(Built-in Function)*\n\nParses a floating-point string into a 64-bit float `Float`.\n\n```aura\nparseFloat(s: String): Float\n```\n\n---\n#### 📥 Parameters\n* `s` (`String`): Floating-point numeric string (e.g. `\"3.1415\"`).\n\n---\n#### 📤 Returns\n* `Float`: Parsed 64-bit float."
        }
        "openFile" => {
            "### `openFile` *(Built-in Function)*\n\nOpens a filesystem file returning a file handle wrapped in `Result<File, Error>`.\n\n```aura\nopenFile(path: String, mode?: String): Result<File, Error>\n```\n\n---\n#### 📥 Parameters\n* `path` (`String`): Path to file.\n* `mode` (`String`, optional): Access mode (`\"r\"`, `\"w\"`, `\"a\"`, default `\"r\"`).\n\n---\n#### 📤 Returns\n* `Result<File, Error>`."
        }
        "Server" => {
            "### `Server` *(Native HTTP Server)*\n\nNative high-performance HTTP web server instance.\n\n```aura\nServer.new(addr: String): Server\n```\n\nSupports routing, middleware chains, OpenAPI 3.0 / Swagger UI auto-generation, and fiber concurrency.\n\n---\n#### 📥 Parameters\n* `addr` (`String`): TCP bind address (e.g. `\":8080\"`).\n\n---\n#### 💡 Example\n```aura\nlet server = Server.new(\":8080\");\nserver.handle(\"/api/users\", handleUsers);\nserver.enableSwagger(\"/swagger\");\nserver.listen();\n```"
        }
        "Request" => {
            "### `Request` *(HTTP Request)*\n\nRepresents an incoming HTTP request.\n\n```aura\ntype Request = {\n    method: String,\n    url: String,\n    headers: Map<String, String>,\n    body: String\n}\n```"
        }
        "Response" => {
            "### `Response` *(HTTP Response)*\n\nHTTP response constructor.\n\n```aura\nResponse.json(data: Any, status: Int = 200): Response\nResponse.html(html: String, status: Int = 200): Response\nResponse.text(text: String, status: Int = 200): Response\n```\n\n---\n#### 📥 Parameters\n* `data` (`Any`): Payload serialized as JSON.\n* `status` (`Int`, optional): HTTP status code (default 200)."
        }
        "http" => {
            "### `http` *(Native Server & Client)*\n\n```aura\nimport { Server, Request, Response } from \"net/http\";\n```\n\nStandard library module for high-performance HTTP networking."
        }
        "pg" | "postgres" => {
            "### `pg` *(Native Database Driver)*\n\n```aura\npg::connect(url: String): Result<PgClient, Error>\n```\n\nPostgreSQL database driver with connection pooling and transactions."
        }
        "redis" => {
            "### `redis` *(Native Database Driver)*\n\n```aura\nredis::connect(url: String): Result<RedisClient, Error>\n```\n\nRedis client for in-memory caching, KV storage, and Pub/Sub messaging."
        }
        "mysql" => {
            "### `mysql` *(Native Database Driver)*\n\n```aura\nmysql::connect(url: String): Result<MySqlClient, Error>\n```\n\nMySQL database client with connection pooling."
        }
        "mongo" | "mongodb" => {
            "### `mongo` *(Native Database Driver)*\n\n```aura\nmongo::connect(url: String): Result<MongoClient, Error>\n```\n\nMongoDB document database driver."
        }
        _ => return None,
    };
    Some(doc.to_string())
}

/// The Aura LSP Server managing multiple documents and processing JSON-RPC messages.
pub struct LspServer {
    pub documents: HashMap<String, DocumentState>,
    pub is_initialized: bool,
    pub is_shutdown: bool,
}

impl LspServer {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            is_initialized: false,
            is_shutdown: false,
        }
    }

    /// Performs symbol rename across all open documents.
    pub fn rename(
        &self,
        uri: &str,
        pos: &LspPosition,
        new_name: &str,
    ) -> Option<HashMap<String, Vec<TextEdit>>> {
        let doc = self.documents.get(uri)?;
        let (_, old_name) = doc.prepare_rename(pos)?;

        if new_name.is_empty() || !new_name.chars().all(is_ident_char) {
            return None;
        }

        let mut changes = HashMap::new();

        for (doc_uri, d) in &self.documents {
            let mut edits = Vec::new();
            if let Some(def) = d.definitions.get(&old_name) {
                edits.push(TextEdit {
                    range: def.def_range,
                    new_text: new_name.to_string(),
                });
            }
            for s_ref in &d.references {
                if s_ref.name == old_name || s_ref.def_name == old_name {
                    edits.push(TextEdit {
                        range: s_ref.range,
                        new_text: new_name.to_string(),
                    });
                }
            }

            let mut seen = HashSet::new();
            edits.retain(|e| {
                let key = (
                    e.range.start.line,
                    e.range.start.character,
                    e.range.end.line,
                    e.range.end.character,
                );
                seen.insert(key)
            });

            if !edits.is_empty() {
                changes.insert(doc_uri.clone(), edits);
            }
        }

        if changes.is_empty() {
            None
        } else {
            Some(changes)
        }
    }

    /// Searches for workspace-level symbols across all open documents.
    pub fn workspace_symbols(&self, query: &str) -> Vec<WorkspaceSymbol> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for (uri, doc) in &self.documents {
            for sym in &doc.symbols {
                if query_lower.is_empty() || sym.name.to_lowercase().contains(&query_lower) {
                    results.push(WorkspaceSymbol {
                        name: sym.name.clone(),
                        kind: sym.kind,
                        location: Location {
                            uri: uri.clone(),
                            range: sym.range,
                        },
                        container_name: None,
                    });
                }
                for child in &sym.children {
                    if query_lower.is_empty() || child.name.to_lowercase().contains(&query_lower) {
                        results.push(WorkspaceSymbol {
                            name: child.name.clone(),
                            kind: child.kind,
                            location: Location {
                                uri: uri.clone(),
                                range: child.range,
                            },
                            container_name: Some(sym.name.clone()),
                        });
                    }
                }
            }
        }

        results
    }

    /// Finds all references to a symbol across all open documents.
    pub fn find_references_workspace(
        &self,
        uri: &str,
        pos: &LspPosition,
        include_declaration: bool,
    ) -> Vec<Location> {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Vec::new(),
        };

        let local_refs = doc.find_references(pos, include_declaration);
        let target_name = doc.word_at_position(pos).map(|(w, _)| w).or_else(|| {
            doc.definitions
                .values()
                .find(|d| d.def_range.contains(pos))
                .map(|d| d.name.clone())
        });

        let name = match target_name {
            Some(n) => n,
            None => return local_refs,
        };

        let mut all_refs = local_refs;

        for (other_uri, other_doc) in &self.documents {
            if other_uri == uri {
                continue;
            }
            for s_ref in &other_doc.references {
                if s_ref.name == name || s_ref.def_name == name {
                    all_refs.push(Location {
                        uri: other_uri.clone(),
                        range: s_ref.range,
                    });
                }
            }
        }

        all_refs
    }

    /// Handles an incoming JSON-RPC raw request string and produces an optional JSON-RPC response.
    pub fn handle_message(&mut self, message: &str) -> Option<String> {
        let (id, method, params) = parse_json_rpc(message)?;

        match method.as_str() {
            "initialize" => {
                self.is_initialized = true;
                let result = r#"{"capabilities":{"textDocumentSync":1,"hoverProvider":true,"definitionProvider":true,"completionProvider":{"resolveProvider":false,"triggerCharacters":[".",":"]},"documentSymbolProvider":true,"documentFormattingProvider":true,"referencesProvider":true,"renameProvider":{"prepareProvider":true},"signatureHelpProvider":{"triggerCharacters":["(",","]},"documentHighlightProvider":true,"inlayHintProvider":true,"codeActionProvider":true,"semanticTokensProvider":{"legend":{"tokenTypes":["type","class","enum","interface","struct","typeParameter","parameter","variable","property","enumMember","function","method","keyword","modifier","comment","string","number","operator"],"tokenModifiers":["declaration","definition","readonly","static","defaultLibrary","modification"]},"full":true},"workspaceSymbolProvider":true,"codeLensProvider":{"resolveProvider":false},"foldingRangeProvider":true,"callHierarchyProvider":true}}"#;
                Some(format_json_response(id, result))
            }
            "initialized" => None,
            "shutdown" => {
                self.is_shutdown = true;
                Some(format_json_response(id, "null"))
            }
            "exit" => None,
            "textDocument/didOpen" => {
                if let (Some(uri), Some(text)) = (
                    extract_json_string(&params, "uri"),
                    extract_json_string(&params, "text"),
                ) {
                    let version = extract_json_int(&params, "version").unwrap_or(1);
                    let state = DocumentState::new(uri.clone(), text, version);
                    let diag_notification = format_publish_diagnostics(&uri, &state.diagnostics);
                    self.documents.insert(uri, state);
                    return Some(diag_notification);
                }
                None
            }
            "textDocument/didChange" => {
                if let (Some(uri), Some(text)) = (
                    extract_json_string(&params, "uri"),
                    extract_json_string(&params, "text"),
                ) {
                    let version = extract_json_int(&params, "version").unwrap_or(1);
                    if let Some(doc) = self.documents.get_mut(&uri) {
                        doc.update(text, version);
                        return Some(format_publish_diagnostics(&uri, &doc.diagnostics));
                    } else {
                        let state = DocumentState::new(uri.clone(), text, version);
                        let diag = format_publish_diagnostics(&uri, &state.diagnostics);
                        self.documents.insert(uri, state);
                        return Some(diag);
                    }
                }
                None
            }
            "textDocument/didClose" => {
                if let Some(uri) = extract_json_string(&params, "uri") {
                    self.documents.remove(&uri);
                }
                None
            }
            "textDocument/hover" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line")? as u32;
                let char_idx = extract_json_int(&params, "character")? as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };

                if let Some(doc) = self.documents.get(&uri) {
                    if let Some(hover) = doc.hover(&pos) {
                        let escaped_contents = escape_json_string(&hover.contents);
                        let range_json = if let Some(r) = hover.range {
                            format!(
                                r#","range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}}"#,
                                r.start.line, r.start.character, r.end.line, r.end.character
                            )
                        } else {
                            String::new()
                        };
                        let res = format!(
                            r#"{{"contents":{{"kind":"markdown","value":"{}"}}{}}}"#,
                            escaped_contents, range_json
                        );
                        return Some(format_json_response(id, &res));
                    }
                }
                Some(format_json_response(id, "null"))
            }
            "textDocument/definition" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line")? as u32;
                let char_idx = extract_json_int(&params, "character")? as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };

                if let Some(doc) = self.documents.get(&uri) {
                    if let Some(loc) = doc.definition(&pos) {
                        let res = format!(
                            r#"{{"uri":"{}","range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}}}}"#,
                            escape_json_string(&loc.uri),
                            loc.range.start.line,
                            loc.range.start.character,
                            loc.range.end.line,
                            loc.range.end.character
                        );
                        return Some(format_json_response(id, &res));
                    }
                }
                Some(format_json_response(id, "null"))
            }
            "textDocument/completion" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line").unwrap_or(0) as u32;
                let char_idx = extract_json_int(&params, "character").unwrap_or(0) as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };

                if let Some(doc) = self.documents.get(&uri) {
                    let items = doc.completions(&pos);
                    let mut items_json = Vec::new();
                    for item in items {
                        let detail_str = item
                            .detail
                            .map(|d| format!(r#","detail":"{}""#, escape_json_string(&d)))
                            .unwrap_or_default();
                        let doc_str = item
                            .documentation
                            .map(|d| format!(r#","documentation":"{}""#, escape_json_string(&d)))
                            .unwrap_or_default();
                        let insert_str = item
                            .insert_text
                            .map(|i| format!(r#","insertText":"{}""#, escape_json_string(&i)))
                            .unwrap_or_default();
                        items_json.push(format!(
                            r#"{{"label":"{}","kind":{}{}{}{}}}"#,
                            escape_json_string(&item.label),
                            item.kind as u32,
                            detail_str,
                            doc_str,
                            insert_str
                        ));
                    }
                    let res = format!("[{}]", items_json.join(","));
                    return Some(format_json_response(id, &res));
                }
                Some(format_json_response(id, "[]"))
            }
            "textDocument/formatting" => {
                let uri = extract_json_string(&params, "uri")?;
                if let Some(doc) = self.documents.get(&uri) {
                    if let Ok(edits) = doc.format(None) {
                        let mut edits_json = Vec::new();
                        for edit in edits {
                            edits_json.push(format!(
                                r#"{{"range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}},"newText":"{}"}}"#,
                                edit.range.start.line,
                                edit.range.start.character,
                                edit.range.end.line,
                                edit.range.end.character,
                                escape_json_string(&edit.new_text)
                            ));
                        }
                        return Some(format_json_response(
                            id,
                            &format!("[{}]", edits_json.join(",")),
                        ));
                    }
                }
                Some(format_json_response(id, "[]"))
            }
            "textDocument/documentSymbol" => {
                let uri = extract_json_string(&params, "uri")?;
                if let Some(doc) = self.documents.get(&uri) {
                    let symbols_json: Vec<String> =
                        doc.symbols.iter().map(format_document_symbol).collect();
                    return Some(format_json_response(
                        id,
                        &format!("[{}]", symbols_json.join(",")),
                    ));
                }
                Some(format_json_response(id, "[]"))
            }
            "textDocument/references" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line")? as u32;
                let char_idx = extract_json_int(&params, "character")? as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };
                let include_decl = !params.contains("\"includeDeclaration\":false");
                let refs = self.find_references_workspace(&uri, &pos, include_decl);
                let locs_json: Vec<String> = refs.iter().map(format_location).collect();
                Some(format_json_response(
                    id,
                    &format!("[{}]", locs_json.join(",")),
                ))
            }
            "textDocument/prepareRename" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line")? as u32;
                let char_idx = extract_json_int(&params, "character")? as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };
                if let Some(doc) = self.documents.get(&uri) {
                    if let Some((range, placeholder)) = doc.prepare_rename(&pos) {
                        let res = format!(
                            r#"{{"range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}},"placeholder":"{}"}}"#,
                            range.start.line,
                            range.start.character,
                            range.end.line,
                            range.end.character,
                            escape_json_string(&placeholder)
                        );
                        return Some(format_json_response(id, &res));
                    }
                }
                Some(format_json_response(id, "null"))
            }
            "textDocument/rename" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line")? as u32;
                let char_idx = extract_json_int(&params, "character")? as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };
                let new_name = extract_json_string(&params, "newName")?;
                if let Some(changes) = self.rename(&uri, &pos, &new_name) {
                    let res = format_workspace_edit(&changes);
                    Some(format_json_response(id, &res))
                } else {
                    Some(format_json_response(id, "null"))
                }
            }
            "textDocument/signatureHelp" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line")? as u32;
                let char_idx = extract_json_int(&params, "character")? as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };
                if let Some(doc) = self.documents.get(&uri) {
                    if let Some(sig_help) = doc.signature_help(&pos) {
                        let mut sigs_json = Vec::new();
                        for sig in &sig_help.signatures {
                            let mut params_json = Vec::new();
                            for p in &sig.parameters {
                                let doc_str = p
                                    .documentation
                                    .as_ref()
                                    .map(|d| {
                                        format!(r#","documentation":"{}""#, escape_json_string(d))
                                    })
                                    .unwrap_or_default();
                                params_json.push(format!(
                                    r#"{{"label":"{}"{}}}"#,
                                    escape_json_string(&p.label),
                                    doc_str
                                ));
                            }
                            let doc_str = sig
                                .documentation
                                .as_ref()
                                .map(|d| format!(r#","documentation":"{}""#, escape_json_string(d)))
                                .unwrap_or_default();
                            sigs_json.push(format!(
                                r#"{{"label":"{}"{},"parameters":[{}]}}"#,
                                escape_json_string(&sig.label),
                                doc_str,
                                params_json.join(",")
                            ));
                        }
                        let res = format!(
                            r#"{{"signatures":[{}],"activeSignature":{},"activeParameter":{}}}"#,
                            sigs_json.join(","),
                            sig_help.active_signature,
                            sig_help.active_parameter
                        );
                        return Some(format_json_response(id, &res));
                    }
                }
                Some(format_json_response(id, "null"))
            }
            "textDocument/documentHighlight" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line")? as u32;
                let char_idx = extract_json_int(&params, "character")? as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };
                if let Some(doc) = self.documents.get(&uri) {
                    let highlights = doc.document_highlight(&pos);
                    let mut hl_json = Vec::new();
                    for h in highlights {
                        hl_json.push(format!(
                            r#"{{"range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}},"kind":{}}}"#,
                            h.range.start.line,
                            h.range.start.character,
                            h.range.end.line,
                            h.range.end.character,
                            h.kind
                        ));
                    }
                    return Some(format_json_response(
                        id,
                        &format!("[{}]", hl_json.join(",")),
                    ));
                }
                Some(format_json_response(id, "[]"))
            }
            "textDocument/inlayHint" => {
                let uri = extract_json_string(&params, "uri")?;
                let range = extract_json_range(&params);
                if let Some(doc) = self.documents.get(&uri) {
                    let hints = doc.inlay_hints(range.as_ref());
                    let mut hints_json = Vec::new();
                    for hint in hints {
                        hints_json.push(format!(
                            r#"{{"position":{{"line":{},"character":{}}},"label":"{}","kind":{},"paddingLeft":{},"paddingRight":{}}}"#,
                            hint.position.line,
                            hint.position.character,
                            escape_json_string(&hint.label),
                            hint.kind,
                            hint.padding_left,
                            hint.padding_right
                        ));
                    }
                    return Some(format_json_response(
                        id,
                        &format!("[{}]", hints_json.join(",")),
                    ));
                }
                Some(format_json_response(id, "[]"))
            }
            "textDocument/codeAction" => {
                let uri = extract_json_string(&params, "uri")?;
                let range = extract_json_range(&params).unwrap_or_default();
                if let Some(doc) = self.documents.get(&uri) {
                    let actions = doc.code_actions(&range, &doc.diagnostics);
                    let mut actions_json = Vec::new();
                    for a in actions {
                        let edit_str = if let Some(ref edits) = a.edit {
                            format!(r#","edit":{}"#, format_workspace_edit(edits))
                        } else {
                            String::new()
                        };
                        actions_json.push(format!(
                            r#"{{"title":"{}","kind":"{}","isPreferred":{}{}}}"#,
                            escape_json_string(&a.title),
                            escape_json_string(&a.kind),
                            a.is_preferred,
                            edit_str
                        ));
                    }
                    return Some(format_json_response(
                        id,
                        &format!("[{}]", actions_json.join(",")),
                    ));
                }
                Some(format_json_response(id, "[]"))
            }
            "textDocument/semanticTokens/full" => {
                let uri = extract_json_string(&params, "uri")?;
                if let Some(doc) = self.documents.get(&uri) {
                    let data = doc.semantic_tokens_full();
                    let data_strs: Vec<String> = data.iter().map(|n| n.to_string()).collect();
                    let res = format!(r#"{{"data":[{}]}}"#, data_strs.join(","));
                    return Some(format_json_response(id, &res));
                }
                Some(format_json_response(id, r#"{"data":[]}"#))
            }
            "workspace/symbol" => {
                let query = extract_json_string(&params, "query").unwrap_or_default();
                let syms = self.workspace_symbols(&query);
                let mut syms_json = Vec::new();
                for s in syms {
                    let container_str = s
                        .container_name
                        .as_ref()
                        .map(|c| format!(r#","containerName":"{}""#, escape_json_string(c)))
                        .unwrap_or_default();
                    syms_json.push(format!(
                        r#"{{"name":"{}","kind":{},"location":{}{}}}"#,
                        escape_json_string(&s.name),
                        s.kind as u32,
                        format_location(&s.location),
                        container_str
                    ));
                }
                Some(format_json_response(
                    id,
                    &format!("[{}]", syms_json.join(",")),
                ))
            }
            "textDocument/codeLens" => {
                let uri = extract_json_string(&params, "uri")?;
                if let Some(doc) = self.documents.get(&uri) {
                    let lenses = doc.code_lens();
                    let mut lenses_json = Vec::new();
                    for lens in lenses {
                        let cmd_str = if let Some(cmd) = lens.command {
                            let args_json: Vec<String> = cmd
                                .arguments
                                .iter()
                                .map(|a| format!("\"{}\"", escape_json_string(a)))
                                .collect();
                            format!(
                                r#","command":{{"title":"{}","command":"{}","arguments":[{}]}}"#,
                                escape_json_string(&cmd.title),
                                escape_json_string(&cmd.command),
                                args_json.join(",")
                            )
                        } else {
                            String::new()
                        };
                        lenses_json.push(format!(
                            r#"{{"range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}}{}}}"#,
                            lens.range.start.line,
                            lens.range.start.character,
                            lens.range.end.line,
                            lens.range.end.character,
                            cmd_str
                        ));
                    }
                    return Some(format_json_response(
                        id,
                        &format!("[{}]", lenses_json.join(",")),
                    ));
                }
                Some(format_json_response(id, "[]"))
            }
            "textDocument/foldingRange" => {
                let uri = extract_json_string(&params, "uri")?;
                if let Some(doc) = self.documents.get(&uri) {
                    let ranges = doc.folding_ranges();
                    let mut ranges_json = Vec::new();
                    for r in ranges {
                        let kind_str = r
                            .kind
                            .as_ref()
                            .map(|k| format!(r#","kind":"{}""#, escape_json_string(k)))
                            .unwrap_or_default();
                        ranges_json.push(format!(
                            r#"{{"startLine":{},"endLine":{}{}}}"#,
                            r.start_line, r.end_line, kind_str
                        ));
                    }
                    return Some(format_json_response(
                        id,
                        &format!("[{}]", ranges_json.join(",")),
                    ));
                }
                Some(format_json_response(id, "[]"))
            }
            "textDocument/prepareCallHierarchy" => {
                let uri = extract_json_string(&params, "uri")?;
                let line = extract_json_int(&params, "line")? as u32;
                let char_idx = extract_json_int(&params, "character")? as u32;
                let pos = LspPosition {
                    line,
                    character: char_idx,
                };
                if let Some(doc) = self.documents.get(&uri) {
                    if let Some(item) = doc.prepare_call_hierarchy(&pos) {
                        return Some(format_json_response(
                            id,
                            &format!("[{}]", format_call_hierarchy_item(&item)),
                        ));
                    }
                }
                Some(format_json_response(id, "[]"))
            }
            "callHierarchy/incomingCalls" => {
                let uri = extract_json_string(&params, "uri").unwrap_or_default();
                let item_name = extract_json_string(&params, "name").unwrap_or_default();
                if let Some(doc) = self.documents.get(&uri) {
                    let calls = doc.incoming_calls(&item_name);
                    let mut calls_json = Vec::new();
                    for c in calls {
                        let from_ranges: Vec<String> = c
                            .from_ranges
                            .iter()
                            .map(|r| {
                                format!(
                                    r#"{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}}"#,
                                    r.start.line, r.start.character, r.end.line, r.end.character
                                )
                            })
                            .collect();
                        calls_json.push(format!(
                            r#"{{"from":{},"fromRanges":[{}]}}"#,
                            format_call_hierarchy_item(&c.from),
                            from_ranges.join(",")
                        ));
                    }
                    return Some(format_json_response(
                        id,
                        &format!("[{}]", calls_json.join(",")),
                    ));
                }
                Some(format_json_response(id, "[]"))
            }
            "callHierarchy/outgoingCalls" => {
                let uri = extract_json_string(&params, "uri").unwrap_or_default();
                let item_name = extract_json_string(&params, "name").unwrap_or_default();
                if let Some(doc) = self.documents.get(&uri) {
                    let calls = doc.outgoing_calls(&item_name);
                    let mut calls_json = Vec::new();
                    for c in calls {
                        let from_ranges: Vec<String> = c
                            .from_ranges
                            .iter()
                            .map(|r| {
                                format!(
                                    r#"{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}}"#,
                                    r.start.line, r.start.character, r.end.line, r.end.character
                                )
                            })
                            .collect();
                        calls_json.push(format!(
                            r#"{{"to":{},"fromRanges":[{}]}}"#,
                            format_call_hierarchy_item(&c.to),
                            from_ranges.join(",")
                        ));
                    }
                    return Some(format_json_response(
                        id,
                        &format!("[{}]", calls_json.join(",")),
                    ));
                }
                Some(format_json_response(id, "[]"))
            }
            _ => {
                if id.is_some() {
                    Some(format_json_response(id, "null"))
                } else {
                    None
                }
            }
        }
    }

    /// Runs the stdio LSP server loop.
    pub fn run_stdio(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut reader = stdin.lock();

        loop {
            let mut content_length: Option<usize> = None;
            let mut header_line = String::new();

            loop {
                header_line.clear();
                let bytes_read = reader.read_line(&mut header_line)?;
                if bytes_read == 0 {
                    return Ok(()); // EOF
                }

                let trimmed = header_line.trim();
                if trimmed.is_empty() {
                    break; // Header section finished
                }

                if trimmed.to_lowercase().starts_with("content-length:") {
                    if let Some(len_str) = trimmed.split(':').nth(1) {
                        content_length = len_str.trim().parse::<usize>().ok();
                    }
                }
            }

            if let Some(len) = content_length {
                let mut body_buf = vec![0u8; len];
                io::Read::read_exact(&mut reader, &mut body_buf)?;
                let body_str = String::from_utf8_lossy(&body_buf);

                if let Some(response) = self.handle_message(&body_str) {
                    let response_bytes = response.as_bytes();
                    write!(stdout, "Content-Length: {}\r\n\r\n", response_bytes.len())?;
                    stdout.write_all(response_bytes)?;
                    stdout.flush()?;
                }

                if self.is_shutdown {
                    break;
                }
            }
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// JSON-RPC Parser and Formatter Helpers
// -----------------------------------------------------------------------------

fn parse_json_rpc(input: &str) -> Option<(Option<String>, String, String)> {
    let method = extract_json_string(input, "method")?;
    let id = extract_json_raw_field(input, "id");
    let params = extract_json_object_or_self(input, "params");
    Some((id, method, params))
}

fn format_json_response(id: Option<String>, result: &str) -> String {
    let id_str = id.unwrap_or_else(|| "null".to_string());
    format!(r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#, id_str, result)
}

fn format_publish_diagnostics(uri: &str, diags: &[Diagnostic]) -> String {
    let mut diag_items = Vec::new();
    for d in diags {
        diag_items.push(format!(
            r#"{{"range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}},"severity":{},"source":"{}","message":"{}"}}"#,
            d.range.start.line,
            d.range.start.character,
            d.range.end.line,
            d.range.end.character,
            d.severity as u32,
            escape_json_string(&d.source),
            escape_json_string(&d.message)
        ));
    }
    format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"uri":"{}","diagnostics":[{}]}}}}"#,
        escape_json_string(uri),
        diag_items.join(",")
    )
}

fn format_document_symbol(sym: &DocumentSymbol) -> String {
    let detail_str = sym
        .detail
        .as_ref()
        .map(|d| format!(r#","detail":"{}""#, escape_json_string(d)))
        .unwrap_or_default();
    let children_json: Vec<String> = sym.children.iter().map(format_document_symbol).collect();
    let children_str = if !children_json.is_empty() {
        format!(r#","children":[{}]"#, children_json.join(","))
    } else {
        String::new()
    };

    format!(
        r#"{{"name":"{}","kind":{}{},"range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}},"selectionRange":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}}{}}}"#,
        escape_json_string(&sym.name),
        sym.kind as u32,
        detail_str,
        sym.range.start.line,
        sym.range.start.character,
        sym.range.end.line,
        sym.range.end.character,
        sym.selection_range.start.line,
        sym.selection_range.start.character,
        sym.selection_range.end.line,
        sym.selection_range.end.character,
        children_str
    )
}

fn extract_json_string(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\"", field);
    let field_pos = json.find(&pattern)?;
    let after_field = &json[field_pos + pattern.len()..];
    let colon_pos = after_field.find(':')?;
    let after_colon = &after_field[colon_pos + 1..].trim_start();

    if after_colon.starts_with('"') {
        let mut s = String::new();
        let mut escaped = false;
        for c in after_colon[1..].chars() {
            if escaped {
                match c {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    _ => s.push(c),
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                return Some(s);
            } else {
                s.push(c);
            }
        }
    }
    None
}

fn extract_json_int(json: &str, field: &str) -> Option<i64> {
    let pattern = format!("\"{}\"", field);
    let field_pos = json.find(&pattern)?;
    let after_field = &json[field_pos + pattern.len()..];
    let colon_pos = after_field.find(':')?;
    let after_colon = &after_field[colon_pos + 1..].trim_start();

    let num_str: String = after_colon
        .chars()
        .take_while(|c| c.is_digit(10) || *c == '-')
        .collect();
    num_str.parse::<i64>().ok()
}

fn extract_json_raw_field(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\"", field);
    let field_pos = json.find(&pattern)?;
    let after_field = &json[field_pos + pattern.len()..];
    let colon_pos = after_field.find(':')?;
    let after_colon = &after_field[colon_pos + 1..].trim_start();

    if after_colon.starts_with('"') {
        return extract_json_string(json, field).map(|s| format!("\"{}\"", escape_json_string(&s)));
    }

    let raw: String = after_colon
        .chars()
        .take_while(|c| c.is_digit(10) || *c == '-' || *c == 'n' || *c == 'u' || *c == 'l')
        .collect();
    if !raw.is_empty() { Some(raw) } else { None }
}

fn extract_json_object_or_self(json: &str, field: &str) -> String {
    let pattern = format!("\"{}\"", field);
    if let Some(field_pos) = json.find(&pattern) {
        let after_field = &json[field_pos + pattern.len()..];
        if let Some(colon_pos) = after_field.find(':') {
            return after_field[colon_pos + 1..].trim_start().to_string();
        }
    }
    json.to_string()
}

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn format_location(loc: &Location) -> String {
    format!(
        r#"{{"uri":"{}","range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}}}}"#,
        escape_json_string(&loc.uri),
        loc.range.start.line,
        loc.range.start.character,
        loc.range.end.line,
        loc.range.end.character
    )
}

fn format_text_edit(edit: &TextEdit) -> String {
    format!(
        r#"{{"range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}},"newText":"{}"}}"#,
        edit.range.start.line,
        edit.range.start.character,
        edit.range.end.line,
        edit.range.end.character,
        escape_json_string(&edit.new_text)
    )
}

fn format_workspace_edit(changes: &HashMap<String, Vec<TextEdit>>) -> String {
    let mut entries = Vec::new();
    for (uri, edits) in changes {
        let edits_json: Vec<String> = edits.iter().map(format_text_edit).collect();
        entries.push(format!(
            r#""{}":[{}]"#,
            escape_json_string(uri),
            edits_json.join(",")
        ));
    }
    format!(r#"{{"changes":{{{}}}}}"#, entries.join(","))
}

fn format_call_hierarchy_item(item: &CallHierarchyItem) -> String {
    format!(
        r#"{{"name":"{}","kind":{},"uri":"{}","range":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}},"selectionRange":{{"start":{{"line":{},"character":{}}},"end":{{"line":{},"character":{}}}}}}}"#,
        escape_json_string(&item.name),
        item.kind as u32,
        escape_json_string(&item.uri),
        item.range.start.line,
        item.range.start.character,
        item.range.end.line,
        item.range.end.character,
        item.selection_range.start.line,
        item.selection_range.start.character,
        item.selection_range.end.line,
        item.selection_range.end.character
    )
}

fn extract_json_range(params: &str) -> Option<LspRange> {
    if let Some(pos) = params.find("\"range\"") {
        let after = &params[pos..];
        let start_pos = after.find("\"start\"")?;
        let start_sub = &after[start_pos..];
        let s_line = extract_json_int(start_sub, "line")? as u32;
        let s_char = extract_json_int(start_sub, "character")? as u32;

        let end_pos = after.find("\"end\"")?;
        let end_sub = &after[end_pos..];
        let e_line = extract_json_int(end_sub, "line")? as u32;
        let e_char = extract_json_int(end_sub, "character")? as u32;

        Some(LspRange::new(s_line, s_char, e_line, e_char))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_hover_and_definition() {
        let src = r#"
/// Adds two numbers together
export fn add(x: Int, y: Int): Int => x + y;

let result = add(10, 20);
"#;
        let state = DocumentState::new("file:///main.aura".to_string(), src.to_string(), 1);
        assert_eq!(state.diagnostics.len(), 0);

        // Test hover on 'add' in definition
        let hover_def = state.hover(&LspPosition {
            line: 2,
            character: 12,
        });
        assert!(hover_def.is_some());
        let val = hover_def.unwrap().contents;
        assert!(val.contains("fn add(x: Int, y: Int) -> Int"));
        assert!(val.contains("Adds two numbers together"));

        // Test Go to Definition on 'add' call at line 4
        let def_loc = state.definition(&LspPosition {
            line: 4,
            character: 14,
        });
        assert!(def_loc.is_some());
        let loc = def_loc.unwrap();
        assert_eq!(loc.uri, "file:///main.aura");
        assert_eq!(loc.range.start.line, 2);
    }

    #[test]
    fn test_lsp_json_rpc_lifecycle() {
        let mut server = LspServer::new();

        // Initialize
        let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let init_resp = server
            .handle_message(init_req)
            .expect("Expected init response");
        assert!(init_resp.contains("capabilities"));
        assert!(init_resp.contains("hoverProvider"));

        // didOpen
        let did_open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"uri":"file:///test.aura","text":"fn hello(): String => \"world\";","version":1}}"#;
        let did_open_resp = server
            .handle_message(did_open)
            .expect("Expected diagnostics notification");
        assert!(did_open_resp.contains("publishDiagnostics"));

        // hover
        let hover_req = r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"uri":"file:///test.aura","line":0,"character":4}}"#;
        let hover_resp = server
            .handle_message(hover_req)
            .expect("Expected hover response");
        assert!(hover_resp.contains("hello"));

        // completions
        let comp_req = r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/completion","params":{"uri":"file:///test.aura","line":0,"character":5}}"#;
        let comp_resp = server
            .handle_message(comp_req)
            .expect("Expected comp response");
        assert!(comp_resp.contains("hello"));
        assert!(comp_resp.contains("async"));
    }
}
