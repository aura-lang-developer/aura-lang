//! Formatter for Aura, TypeScript, and JavaScript code.
//!
//! Provides a standardized, deterministic, and idempotent code formatting engine
//! similar to `gofmt` and modern formatters.

use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::exit;

/// Supported target languages for formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Aura,
    TypeScript,
    JavaScript,
}

impl Language {
    /// Detects the language based on file extension.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = path.as_ref();
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "aura" => Some(Language::Aura),
            "ts" | "dts" => Some(Language::TypeScript),
            "js" | "mjs" | "cjs" => Some(Language::JavaScript),
            _ => {
                let name = path.file_name()?.to_str()?.to_lowercase();
                if name.ends_with(".d.ts") {
                    Some(Language::TypeScript)
                } else {
                    None
                }
            }
        }
    }
}

/// Indentation style configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    Spaces(usize),
    Tabs,
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Spaces(2)
    }
}

/// Formatting options.
#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub indent_style: IndentStyle,
    pub max_blank_lines: usize,
    pub trim_trailing_whitespace: bool,
    pub insert_final_newline: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Spaces(2),
            max_blank_lines: 1,
            trim_trailing_whitespace: true,
            insert_final_newline: true,
        }
    }
}

impl FormatConfig {
    pub fn indent_str(&self, level: usize) -> String {
        match self.indent_style {
            IndentStyle::Spaces(n) => " ".repeat(n * level),
            IndentStyle::Tabs => "\t".repeat(level),
        }
    }
}

/// Result of formatting a file or source string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatResult {
    pub original: String,
    pub formatted: String,
    pub changed: bool,
}

#[derive(Debug)]
pub enum FormatError {
    IoError(io::Error),
    SyntaxError(String),
    UnsupportedLanguage(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::IoError(e) => write!(f, "IO Error: {}", e),
            FormatError::SyntaxError(msg) => write!(f, "Syntax Error: {}", msg),
            FormatError::UnsupportedLanguage(ext) => {
                write!(f, "Unsupported language / extension: {}", ext)
            }
        }
    }
}

impl std::error::Error for FormatError {}

impl From<io::Error> for FormatError {
    fn from(e: io::Error) -> Self {
        FormatError::IoError(e)
    }
}

// -----------------------------------------------------------------------------
// FORMATTER TOKENIZER & SCANNER
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum FmtTokenKind {
    Whitespace(String),
    Newline,
    LineComment(String),
    BlockComment(String),
    StringLiteral(String),
    TemplateLiteral(String),
    IdentifierOrKeyword(String),
    Number(String),
    Operator(String),
    Punctuation(char),
}

struct FmtScanner {
    chars: Vec<char>,
    pos: usize,
    len: usize,
}

impl FmtScanner {
    fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();
        Self { chars, pos: 0, len }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.len {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        if self.pos + offset < self.len {
            Some(self.chars[self.pos + offset])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.len {
            let ch = self.chars[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn tokenize(&mut self) -> Vec<FmtTokenKind> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
                tokens.push(FmtTokenKind::Newline);
            } else if ch == '\r' {
                self.advance();
                if self.peek() == Some('\n') {
                    self.advance();
                }
                tokens.push(FmtTokenKind::Newline);
            } else if ch.is_whitespace() {
                let mut ws = String::new();
                while let Some(c) = self.peek() {
                    if c.is_whitespace() && c != '\n' && c != '\r' {
                        ws.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                tokens.push(FmtTokenKind::Whitespace(ws));
            } else if ch == '/' && self.peek_ahead(1) == Some('/') {
                let mut comment = String::new();
                while let Some(c) = self.peek() {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    comment.push(c);
                    self.advance();
                }
                tokens.push(FmtTokenKind::LineComment(comment));
            } else if ch == '/' && self.peek_ahead(1) == Some('*') {
                let mut comment = String::new();
                comment.push(self.advance().unwrap());
                comment.push(self.advance().unwrap());
                while let Some(c) = self.peek() {
                    if c == '*' && self.peek_ahead(1) == Some('/') {
                        comment.push(self.advance().unwrap());
                        comment.push(self.advance().unwrap());
                        break;
                    }
                    comment.push(self.advance().unwrap());
                }
                tokens.push(FmtTokenKind::BlockComment(comment));
            } else if ch == '"' || ch == '\'' {
                let quote = ch;
                let mut s = String::new();
                s.push(self.advance().unwrap());
                let mut escaped = false;
                while let Some(c) = self.peek() {
                    s.push(c);
                    self.advance();
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == quote {
                        break;
                    }
                }
                tokens.push(FmtTokenKind::StringLiteral(s));
            } else if ch == '`' {
                let mut s = String::new();
                s.push(self.advance().unwrap());
                let mut escaped = false;
                while let Some(c) = self.peek() {
                    s.push(c);
                    self.advance();
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '`' {
                        break;
                    }
                }
                tokens.push(FmtTokenKind::TemplateLiteral(s));
            } else if ch.is_alphabetic() || ch == '_' || ch == '$' {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '$' {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                tokens.push(FmtTokenKind::IdentifierOrKeyword(ident));
            } else if ch.is_numeric() {
                let mut num = String::new();
                let mut has_dot = false;
                while let Some(c) = self.peek() {
                    if c.is_numeric() {
                        num.push(c);
                        self.advance();
                    } else if c == '.'
                        && !has_dot
                        && self.peek_ahead(1).map_or(false, |d| d.is_numeric())
                    {
                        has_dot = true;
                        num.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                tokens.push(FmtTokenKind::Number(num));
            } else {
                let two_char = if let Some(next) = self.peek_ahead(1) {
                    format!("{}{}", ch, next)
                } else {
                    String::new()
                };

                let three_char =
                    if let (Some(n1), Some(n2)) = (self.peek_ahead(1), self.peek_ahead(2)) {
                        format!("{}{}{}", ch, n1, n2)
                    } else {
                        String::new()
                    };

                if three_char == "==="
                    || three_char == "!=="
                    || three_char == "..."
                    || three_char == ">>>"
                {
                    for _ in 0..3 {
                        self.advance();
                    }
                    tokens.push(FmtTokenKind::Operator(three_char));
                } else if matches!(
                    two_char.as_str(),
                    "|>" | "=>"
                        | "->"
                        | "<-"
                        | "=="
                        | "!="
                        | "<="
                        | ">="
                        | "&&"
                        | "||"
                        | "++"
                        | "--"
                        | "+="
                        | "-="
                        | "*="
                        | "/="
                        | "%="
                        | "?."
                        | "??"
                        | "::"
                ) {
                    for _ in 0..2 {
                        self.advance();
                    }
                    tokens.push(FmtTokenKind::Operator(two_char));
                } else if matches!(
                    ch,
                    '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | '.' | ':' | '?'
                ) {
                    self.advance();
                    tokens.push(FmtTokenKind::Punctuation(ch));
                } else if matches!(
                    ch,
                    '+' | '-'
                        | '*'
                        | '/'
                        | '%'
                        | '='
                        | '<'
                        | '>'
                        | '!'
                        | '&'
                        | '|'
                        | '^'
                        | '~'
                        | '@'
                        | '#'
                ) {
                    self.advance();
                    tokens.push(FmtTokenKind::Operator(ch.to_string()));
                } else {
                    self.advance();
                    tokens.push(FmtTokenKind::Punctuation(ch));
                }
            }
        }

        tokens
    }
}

// -----------------------------------------------------------------------------
// CORE FORMATTER IMPLEMENTATION
// -----------------------------------------------------------------------------

/// Formats source code for a specific language using the provided configuration.
pub fn format_source(
    source: &str,
    lang: Language,
    config: &FormatConfig,
) -> Result<String, FormatError> {
    match lang {
        Language::Aura => format_aura_internal(source, config),
        Language::TypeScript | Language::JavaScript => format_js_ts_internal(source, lang, config),
    }
}

/// Convenience function to format Aura code with default options.
pub fn format_aura(source: &str) -> Result<String, FormatError> {
    format_source(source, Language::Aura, &FormatConfig::default())
}

/// Convenience function to format TypeScript code with default options.
pub fn format_ts(source: &str) -> Result<String, FormatError> {
    format_source(source, Language::TypeScript, &FormatConfig::default())
}

/// Convenience function to format JavaScript code with default options.
pub fn format_js(source: &str) -> Result<String, FormatError> {
    format_source(source, Language::JavaScript, &FormatConfig::default())
}

/// Formats a file on disk or returns its formatted content.
pub fn format_file<P: AsRef<Path>>(
    path: P,
    config: &FormatConfig,
) -> Result<FormatResult, FormatError> {
    let path = path.as_ref();
    let lang = Language::from_path(path)
        .ok_or_else(|| FormatError::UnsupportedLanguage(path.display().to_string()))?;
    let original = fs::read_to_string(path)?;
    let formatted = format_source(&original, lang, config)?;
    let changed = original != formatted;
    Ok(FormatResult {
        original,
        formatted,
        changed,
    })
}

// -----------------------------------------------------------------------------
// AURA SPECIFIC FORMATTER
// -----------------------------------------------------------------------------

fn format_aura_internal(source: &str, config: &FormatConfig) -> Result<String, FormatError> {
    let mut scanner = FmtScanner::new(source);
    let raw_tokens = scanner.tokenize();

    let mut out = String::with_capacity(source.len() + 128);
    let mut indent_level: usize = 0;
    let mut at_line_start = true;
    let mut blank_line_count = 0;
    let mut inside_paren = 0;
    let mut inside_bracket = 0;

    let len = raw_tokens.len();
    let mut i = 0;

    while i < len {
        let token = &raw_tokens[i];

        match token {
            FmtTokenKind::Newline => {
                if at_line_start {
                    if blank_line_count < config.max_blank_lines {
                        out.push('\n');
                        blank_line_count += 1;
                    }
                } else {
                    while out.ends_with(' ') || out.ends_with('\t') {
                        out.pop();
                    }
                    out.push('\n');
                    at_line_start = true;
                    blank_line_count = 0;
                }
                i += 1;
                continue;
            }
            FmtTokenKind::Whitespace(_) => {
                i += 1;
                continue;
            }
            _ => {}
        }

        match token {
            FmtTokenKind::Punctuation('}') => {
                if indent_level > 0 {
                    indent_level -= 1;
                }
            }
            FmtTokenKind::Punctuation(']') => {
                if inside_bracket > 0 {
                    inside_bracket -= 1;
                }
            }
            FmtTokenKind::Punctuation(')') => {
                if inside_paren > 0 {
                    inside_paren -= 1;
                }
            }
            _ => {}
        }

        if at_line_start {
            out.push_str(&config.indent_str(indent_level));
            at_line_start = false;
            blank_line_count = 0;
        }

        match token {
            FmtTokenKind::LineComment(c) => {
                out.push_str(c);
            }
            FmtTokenKind::BlockComment(c) => {
                if c.contains('\n') {
                    let lines: Vec<&str> = c.lines().collect();
                    for (idx, line) in lines.iter().enumerate() {
                        let trimmed = line.trim();
                        if idx == 0 {
                            out.push_str(trimmed);
                        } else {
                            out.push('\n');
                            out.push_str(&config.indent_str(indent_level));
                            if trimmed.starts_with('*') {
                                out.push(' ');
                            }
                            out.push_str(trimmed);
                        }
                    }
                } else {
                    out.push_str(c);
                }
            }
            FmtTokenKind::StringLiteral(s) => {
                out.push_str(s);
            }
            FmtTokenKind::TemplateLiteral(s) => {
                out.push_str(s);
            }
            FmtTokenKind::Number(n) => {
                out.push_str(n);
            }
            FmtTokenKind::IdentifierOrKeyword(ident) => {
                out.push_str(ident);
            }
            FmtTokenKind::Operator(op) => {
                if op == "<" || op == ">" {
                    out.push_str(op);
                } else if op == "<-" {
                    // Unary receive vs channel send
                    if out.ends_with('=')
                        || out.ends_with("= ")
                        || out.ends_with("let ")
                        || out.ends_with('(')
                        || out.ends_with(',')
                        || at_line_start
                    {
                        if !out.ends_with(' ')
                            && !out.ends_with('(')
                            && !out.ends_with(',')
                            && !out.ends_with('\n')
                        {
                            out.push(' ');
                        }
                        out.push_str(op);
                    } else {
                        if !out.ends_with(' ') && !out.ends_with('\n') {
                            out.push(' ');
                        }
                        out.push_str(op);
                        if let Some(next) = get_next_non_whitespace(&raw_tokens, i + 1) {
                            if !matches!(next, FmtTokenKind::Newline) {
                                out.push(' ');
                            }
                        }
                    }
                } else if op == "!" || op == "++" || op == "--" || op == "~" {
                    out.push_str(op);
                } else if op == "." || op == "..." || op == "?." || op == "::" {
                    out.push_str(op);
                } else {
                    if !out.ends_with(' ')
                        && !out.ends_with('\n')
                        && !out.ends_with('(')
                        && !out.ends_with('{')
                    {
                        out.push(' ');
                    }
                    out.push_str(op);
                }
            }
            FmtTokenKind::Punctuation(p) => match p {
                '{' => {
                    if !out.ends_with(' ')
                        && !out.ends_with('\n')
                        && !out.ends_with('(')
                        && !out.ends_with('{')
                    {
                        out.push(' ');
                    }
                    out.push('{');
                    indent_level += 1;
                }
                '}' => {
                    out.push('}');
                }
                '(' => {
                    let needs_space = out.ends_with("if")
                        || out.ends_with("when")
                        || out.ends_with("match")
                        || out.ends_with("select")
                        || out.ends_with("for")
                        || out.ends_with("while")
                        || out.ends_with("catch");
                    if needs_space && !out.ends_with(' ') {
                        out.push(' ');
                    }
                    out.push('(');
                    inside_paren += 1;
                }
                ')' => {
                    out.push(')');
                }
                '[' => {
                    out.push('[');
                    inside_bracket += 1;
                }
                ']' => {
                    out.push(']');
                }
                ':' => {
                    out.push(':');
                }
                ',' => {
                    out.push(',');
                }
                ';' => {
                    out.push(';');
                }
                '.' => {
                    out.push('.');
                }
                '?' => {
                    if !out.ends_with(' ') && !out.ends_with('\n') {
                        out.push(' ');
                    }
                    out.push('?');
                    if let Some(next) = get_next_non_whitespace(&raw_tokens, i + 1) {
                        if !matches!(
                            next,
                            FmtTokenKind::Newline
                                | FmtTokenKind::Punctuation(':')
                                | FmtTokenKind::Operator(_)
                        ) {
                            out.push(' ');
                        }
                    }
                }
                _ => {
                    out.push(*p);
                }
            },
            _ => {}
        }

        if let Some(next_tok) = get_next_non_whitespace(&raw_tokens, i + 1) {
            match token {
                FmtTokenKind::IdentifierOrKeyword(kw) => {
                    let is_keyword = matches!(
                        kw.as_str(),
                        "fn" | "let"
                            | "mut"
                            | "type"
                            | "match"
                            | "when"
                            | "if"
                            | "else"
                            | "async"
                            | "await"
                            | "import"
                            | "export"
                            | "from"
                            | "extern"
                            | "trait"
                            | "impl"
                            | "return"
                            | "module"
                            | "spawn"
                            | "routine"
                            | "go"
                            | "select"
                            | "default"
                            | "const"
                            | "var"
                            | "function"
                            | "class"
                            | "interface"
                            | "defer"
                            | "errdefer"
                            | "struct"
                            | "packed"
                            | "enum"
                            | "case"
                    );
                    if is_keyword {
                        match next_tok {
                            FmtTokenKind::Newline
                            | FmtTokenKind::Punctuation(';')
                            | FmtTokenKind::Punctuation('(')
                            | FmtTokenKind::Punctuation(',')
                            | FmtTokenKind::Punctuation(':') => {}
                            _ => {
                                if !out.ends_with(' ') {
                                    out.push(' ');
                                }
                            }
                        }
                    } else {
                        match next_tok {
                            FmtTokenKind::IdentifierOrKeyword(_)
                            | FmtTokenKind::Number(_)
                            | FmtTokenKind::StringLiteral(_) => {
                                if !out.ends_with(' ') {
                                    out.push(' ');
                                }
                            }
                            _ => {}
                        }
                    }
                }
                FmtTokenKind::StringLiteral(_) => {}
                FmtTokenKind::Punctuation('}') => {}
                FmtTokenKind::Punctuation(':') | FmtTokenKind::Punctuation(',') => match next_tok {
                    FmtTokenKind::Newline => {}
                    _ => {
                        if !out.ends_with(' ') {
                            out.push(' ');
                        }
                    }
                },
                FmtTokenKind::Punctuation(';') => {
                    if inside_paren > 0 {
                        match next_tok {
                            FmtTokenKind::Newline => {}
                            _ => {
                                if !out.ends_with(' ') {
                                    out.push(' ');
                                }
                            }
                        }
                    }
                }
                FmtTokenKind::Operator(op) => {
                    if op == "."
                        || op == "..."
                        || op == "?."
                        || op == "!"
                        || op == "::"
                        || op == "<"
                        || op == ">"
                    {
                    } else if op == "<-" {
                        // Unary receive: no space after <- if followed by identifier
                    } else {
                        match next_tok {
                            FmtTokenKind::Newline => {}
                            _ => {
                                if !out.ends_with(' ') {
                                    out.push(' ');
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        i += 1;
    }

    let mut cleaned = String::new();
    let lines: Vec<&str> = out.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed_end = if config.trim_trailing_whitespace {
            line.trim_end()
        } else {
            *line
        };
        cleaned.push_str(trimmed_end);
        if idx + 1 < lines.len() {
            cleaned.push('\n');
        }
    }

    if config.insert_final_newline && !cleaned.is_empty() {
        cleaned.push('\n');
    }

    Ok(cleaned)
}

// -----------------------------------------------------------------------------
// JAVASCRIPT / TYPESCRIPT FORMATTER
// -----------------------------------------------------------------------------

fn format_js_ts_internal(
    source: &str,
    _lang: Language,
    config: &FormatConfig,
) -> Result<String, FormatError> {
    let mut scanner = FmtScanner::new(source);
    let raw_tokens = scanner.tokenize();

    let mut out = String::with_capacity(source.len() + 128);
    let mut indent_level: usize = 0;
    let mut at_line_start = true;
    let mut blank_line_count = 0;
    let mut inside_paren = 0;
    let mut inside_bracket = 0;

    let len = raw_tokens.len();
    let mut i = 0;

    while i < len {
        let token = &raw_tokens[i];

        match token {
            FmtTokenKind::Newline => {
                if at_line_start {
                    if blank_line_count < config.max_blank_lines {
                        out.push('\n');
                        blank_line_count += 1;
                    }
                } else {
                    while out.ends_with(' ') || out.ends_with('\t') {
                        out.pop();
                    }
                    out.push('\n');
                    at_line_start = true;
                    blank_line_count = 0;
                }
                i += 1;
                continue;
            }
            FmtTokenKind::Whitespace(_) => {
                i += 1;
                continue;
            }
            _ => {}
        }

        match token {
            FmtTokenKind::Punctuation('}') => {
                if indent_level > 0 {
                    indent_level -= 1;
                }
            }
            FmtTokenKind::Punctuation(']') => {
                if inside_bracket > 0 {
                    inside_bracket -= 1;
                }
            }
            FmtTokenKind::Punctuation(')') => {
                if inside_paren > 0 {
                    inside_paren -= 1;
                }
            }
            _ => {}
        }

        if at_line_start {
            out.push_str(&config.indent_str(indent_level));
            at_line_start = false;
            blank_line_count = 0;
        }

        match token {
            FmtTokenKind::LineComment(c) => {
                out.push_str(c);
            }
            FmtTokenKind::BlockComment(c) => {
                if c.contains('\n') {
                    let lines: Vec<&str> = c.lines().collect();
                    for (idx, line) in lines.iter().enumerate() {
                        let trimmed = line.trim();
                        if idx == 0 {
                            out.push_str(trimmed);
                        } else {
                            out.push('\n');
                            out.push_str(&config.indent_str(indent_level));
                            if trimmed.starts_with('*') {
                                out.push(' ');
                            }
                            out.push_str(trimmed);
                        }
                    }
                } else {
                    out.push_str(c);
                }
            }
            FmtTokenKind::StringLiteral(s) => {
                out.push_str(s);
            }
            FmtTokenKind::TemplateLiteral(s) => {
                out.push_str(s);
            }
            FmtTokenKind::Number(n) => {
                out.push_str(n);
            }
            FmtTokenKind::IdentifierOrKeyword(ident) => {
                out.push_str(ident);
            }
            FmtTokenKind::Operator(op) => {
                if op == "<" || op == ">" {
                    out.push_str(op);
                } else if op == "."
                    || op == "..."
                    || op == "?."
                    || op == "!"
                    || op == "++"
                    || op == "--"
                    || op == "~"
                {
                    out.push_str(op);
                } else {
                    if !out.ends_with(' ')
                        && !out.ends_with('\n')
                        && !out.ends_with('(')
                        && !out.ends_with('{')
                    {
                        out.push(' ');
                    }
                    out.push_str(op);
                }
            }
            FmtTokenKind::Punctuation(p) => match p {
                '{' => {
                    if !out.ends_with(' ')
                        && !out.ends_with('\n')
                        && !out.ends_with('(')
                        && !out.ends_with('{')
                    {
                        out.push(' ');
                    }
                    out.push('{');
                    indent_level += 1;

                    // If single line destructuring or object e.g. ({ label, onClick })
                    if let Some(next) = get_next_non_whitespace(&raw_tokens, i + 1) {
                        if !matches!(next, FmtTokenKind::Newline | FmtTokenKind::Punctuation('}')) {
                            out.push(' ');
                        }
                    }
                }
                '}' => {
                    if !out.ends_with(' ')
                        && !out.ends_with('\n')
                        && !out.ends_with('{')
                        && !out.ends_with(';')
                    {
                        out.push(' ');
                    }
                    out.push('}');
                }
                '(' => {
                    let needs_space = out.ends_with("if")
                        || out.ends_with("for")
                        || out.ends_with("while")
                        || out.ends_with("switch")
                        || out.ends_with("catch")
                        || out.ends_with("with");
                    if needs_space && !out.ends_with(' ') {
                        out.push(' ');
                    }
                    out.push('(');
                    inside_paren += 1;
                }
                ')' => {
                    out.push(')');
                }
                '[' => {
                    out.push('[');
                    inside_bracket += 1;
                }
                ']' => {
                    out.push(']');
                }
                ':' => {
                    out.push(':');
                }
                ',' => {
                    out.push(',');
                }
                ';' => {
                    out.push(';');
                }
                '.' => {
                    out.push('.');
                }
                '?' => {
                    if !out.ends_with(' ') && !out.ends_with('\n') {
                        out.push(' ');
                    }
                    out.push('?');
                    if let Some(next) = get_next_non_whitespace(&raw_tokens, i + 1) {
                        if !matches!(
                            next,
                            FmtTokenKind::Newline
                                | FmtTokenKind::Punctuation(':')
                                | FmtTokenKind::Operator(_)
                        ) {
                            out.push(' ');
                        }
                    }
                }
                _ => {
                    out.push(*p);
                }
            },
            _ => {}
        }

        if let Some(next_tok) = get_next_non_whitespace(&raw_tokens, i + 1) {
            match token {
                FmtTokenKind::IdentifierOrKeyword(kw) => {
                    let is_keyword = matches!(
                        kw.as_str(),
                        "const"
                            | "let"
                            | "var"
                            | "function"
                            | "class"
                            | "interface"
                            | "type"
                            | "enum"
                            | "import"
                            | "export"
                            | "from"
                            | "as"
                            | "default"
                            | "extends"
                            | "implements"
                            | "return"
                            | "if"
                            | "else"
                            | "switch"
                            | "case"
                            | "for"
                            | "while"
                            | "do"
                            | "try"
                            | "catch"
                            | "finally"
                            | "throw"
                            | "typeof"
                            | "instanceof"
                            | "yield"
                            | "async"
                            | "await"
                            | "new"
                            | "delete"
                            | "in"
                            | "of"
                    );
                    if is_keyword {
                        match next_tok {
                            FmtTokenKind::Newline
                            | FmtTokenKind::Punctuation(';')
                            | FmtTokenKind::Punctuation('(')
                            | FmtTokenKind::Punctuation(',')
                            | FmtTokenKind::Punctuation(':') => {}
                            _ => {
                                if !out.ends_with(' ') {
                                    out.push(' ');
                                }
                            }
                        }
                    } else {
                        match next_tok {
                            FmtTokenKind::IdentifierOrKeyword(_)
                            | FmtTokenKind::Number(_)
                            | FmtTokenKind::StringLiteral(_) => {
                                if !out.ends_with(' ') {
                                    out.push(' ');
                                }
                            }
                            _ => {}
                        }
                    }
                }
                FmtTokenKind::Punctuation(':') | FmtTokenKind::Punctuation(',') => match next_tok {
                    FmtTokenKind::Newline => {}
                    _ => {
                        if !out.ends_with(' ') {
                            out.push(' ');
                        }
                    }
                },
                FmtTokenKind::Punctuation(';') => {
                    if inside_paren > 0 {
                        match next_tok {
                            FmtTokenKind::Newline => {}
                            _ => {
                                if !out.ends_with(' ') {
                                    out.push(' ');
                                }
                            }
                        }
                    }
                }
                FmtTokenKind::Operator(op) => {
                    if op == "."
                        || op == "..."
                        || op == "?."
                        || op == "!"
                        || op == "++"
                        || op == "--"
                        || op == "~"
                        || op == "<"
                        || op == ">"
                    {
                    } else {
                        match next_tok {
                            FmtTokenKind::Newline => {}
                            _ => {
                                if !out.ends_with(' ') {
                                    out.push(' ');
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        i += 1;
    }

    let mut cleaned = String::new();
    let lines: Vec<&str> = out.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed_end = if config.trim_trailing_whitespace {
            line.trim_end()
        } else {
            *line
        };
        cleaned.push_str(trimmed_end);
        if idx + 1 < lines.len() {
            cleaned.push('\n');
        }
    }

    if config.insert_final_newline && !cleaned.is_empty() {
        cleaned.push('\n');
    }

    Ok(cleaned)
}

fn get_next_non_whitespace(tokens: &[FmtTokenKind], start_idx: usize) -> Option<&FmtTokenKind> {
    let mut i = start_idx;
    while i < tokens.len() {
        match &tokens[i] {
            FmtTokenKind::Whitespace(_) => {
                i += 1;
            }
            other => return Some(other),
        }
    }
    None
}

// -----------------------------------------------------------------------------
// UNIFIED DIFF UTILITIES (FOR -d / --diff FLAG)
// -----------------------------------------------------------------------------

/// Generates a unified diff between original and formatted source code.
pub fn generate_unified_diff(filename: &str, original: &str, formatted: &str) -> String {
    if original == formatted {
        return String::new();
    }

    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();

    let mut diff = String::new();
    diff.push_str(&format!("--- {}.orig\n", filename));
    diff.push_str(&format!("+++ {}\n", filename));

    let max_len = orig_lines.len().max(fmt_lines.len());
    let mut i = 0;
    while i < max_len {
        let o_line = orig_lines.get(i);
        let f_line = fmt_lines.get(i);

        match (o_line, f_line) {
            (Some(o), Some(f)) if o == f => {
                diff.push_str(&format!(" {}\n", o));
            }
            (Some(o), Some(f)) => {
                diff.push_str(&format!("-{}\n", o));
                diff.push_str(&format!("+{}\n", f));
            }
            (Some(o), None) => {
                diff.push_str(&format!("-{}\n", o));
            }
            (None, Some(f)) => {
                diff.push_str(&format!("+{}\n", f));
            }
            (None, None) => break,
        }
        i += 1;
    }

    diff
}

// -----------------------------------------------------------------------------
// CLI RUNNER & DISPATCH (Shared between aurac fmt and aurafmt)
// -----------------------------------------------------------------------------

pub fn print_usage() {
    println!("Aura Code Formatter (aurafmt / aurac fmt) v0.1.0");
    println!("Opinionated, deterministic formatting for Aura, TypeScript, and JavaScript.");
    println!();
    println!("Usage:");
    println!("  aurafmt [flags] [path ...]");
    println!("  aurac fmt [flags] [path ...]");
    println!();
    println!("Flags:");
    println!("  -w, --write       Write result to (source) file instead of stdout");
    println!("  -l, --list        List files whose formatting differs from aurafmt's");
    println!("  -d, --diff        Display diffs instead of rewriting files");
    println!("  -c, --check       Check formatting; exit with code 1 if unformatted files found");
    println!("  -s, --stdin       Read code from stdin and format to stdout");
    println!("      --tabs        Use tabs for indentation instead of spaces");
    println!("      --indent <N>  Number of spaces per indentation level (default: 2)");
    println!("  -h, --help        Show this help message");
    println!();
    println!("Examples:");
    println!(
        "  aurafmt -w .                 # Format all Aura/TS/JS files in current dir in-place"
    );
    println!("  aurac fmt -d src/main.aura   # View formatting diff for a file");
    println!("  aurafmt -c src/              # Verify all files are formatted (CI mode)");
    println!("  cat foo.aura | aurafmt -s    # Format code from stdin");
}

pub fn collect_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let path = path.as_ref();

    if path.is_file() {
        if Language::from_path(path).is_some() {
            files.push(path.to_path_buf());
        }
    } else if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let file_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if file_name.starts_with('.')
                    || file_name == "node_modules"
                    || file_name == "target"
                {
                    continue;
                }
                if entry_path.is_dir() {
                    files.extend(collect_files(&entry_path));
                } else if Language::from_path(&entry_path).is_some() {
                    files.push(entry_path);
                }
            }
        }
    }

    files
}

pub fn run_cli(args: &[String]) {
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }

    let mut write_mode = false;
    let mut list_mode = false;
    let mut diff_mode = false;
    let mut check_mode = false;
    let mut stdin_mode = false;
    let mut use_tabs = false;
    let mut indent_spaces = 2;
    let mut paths = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-w" | "--write" => write_mode = true,
            "-l" | "--list" => list_mode = true,
            "-d" | "--diff" => diff_mode = true,
            "-c" | "--check" => check_mode = true,
            "-s" | "--stdin" | "-" => stdin_mode = true,
            "--tabs" => use_tabs = true,
            "--indent" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        indent_spaces = n;
                    }
                    i += 1;
                }
            }
            _ => {
                if !arg.starts_with('-') {
                    paths.push(arg.clone());
                }
            }
        }
        i += 1;
    }

    let config = FormatConfig {
        indent_style: if use_tabs {
            IndentStyle::Tabs
        } else {
            IndentStyle::Spaces(indent_spaces)
        },
        max_blank_lines: 1,
        trim_trailing_whitespace: true,
        insert_final_newline: true,
    };

    if stdin_mode || (paths.is_empty() && (write_mode || list_mode || diff_mode || check_mode)) {
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            eprintln!("Error reading stdin: {}", e);
            exit(1);
        }
        match format_source(&buffer, Language::Aura, &config) {
            Ok(formatted) => {
                print!("{}", formatted);
            }
            Err(e) => {
                eprintln!("Formatting error: {}", e);
                exit(1);
            }
        }
        return;
    }

    if paths.is_empty() {
        print_usage();
        return;
    }

    let mut all_files = Vec::new();
    for p in &paths {
        all_files.extend(collect_files(p));
    }

    if all_files.is_empty() {
        eprintln!("No matching Aura, TypeScript, or JavaScript files found.");
        return;
    }

    let mut unformatted_count = 0;

    for file in &all_files {
        match format_file(file, &config) {
            Ok(res) => {
                if res.changed {
                    unformatted_count += 1;

                    if list_mode {
                        println!("{}", file.display());
                    }

                    if diff_mode {
                        let diff = generate_unified_diff(
                            &file.display().to_string(),
                            &res.original,
                            &res.formatted,
                        );
                        print!("{}", diff);
                    }

                    if write_mode {
                        if let Err(e) = fs::write(file, &res.formatted) {
                            eprintln!("Error writing {}: {}", file.display(), e);
                        } else if !list_mode && !diff_mode {
                            println!("Formatted: {}", file.display());
                        }
                    } else if !list_mode && !diff_mode && !check_mode {
                        print!("{}", res.formatted);
                    }
                } else if !write_mode
                    && !list_mode
                    && !diff_mode
                    && !check_mode
                    && paths.len() == 1
                    && Path::new(&paths[0]).is_file()
                {
                    print!("{}", res.formatted);
                }
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", file.display(), e);
            }
        }
    }

    if check_mode {
        if unformatted_count > 0 {
            eprintln!(
                "✕ Check failed: {} unformatted file(s) found. Run 'aurafmt -w' to format.",
                unformatted_count
            );
            exit(1);
        } else {
            println!("✓ All {} file(s) are properly formatted.", all_files.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_path("src/main.aura"), Some(Language::Aura));
        assert_eq!(
            Language::from_path("types/api.d.ts"),
            Some(Language::TypeScript)
        );
        assert_eq!(
            Language::from_path("server/api.ts"),
            Some(Language::TypeScript)
        );
        assert_eq!(
            Language::from_path("server/index.js"),
            Some(Language::JavaScript)
        );
        assert_eq!(
            Language::from_path("server/index.mjs"),
            Some(Language::JavaScript)
        );
        assert_eq!(Language::from_path("docs/readme.txt"), None);
    }

    #[test]
    fn test_indent_styles() {
        let default_indent = IndentStyle::default();
        assert_eq!(default_indent, IndentStyle::Spaces(2));

        let config_4 = FormatConfig {
            indent_style: IndentStyle::Spaces(4),
            ..FormatConfig::default()
        };
        let src = "fn test() {\nlet x = 1;\n}";
        let res = format_source(src, Language::Aura, &config_4).expect("format failed");
        assert!(res.contains("    let x = 1;"));
    }

    #[test]
    fn test_format_aura_idempotency_and_comments() {
        let src =
            "// Main entrypoint\nfn main() {\n  // Print greeting\n  println(\"hello\");\n}\n";
        let formatted = format_aura(src).expect("format failed");
        assert_eq!(formatted, src);

        // Formatting again produces exact same output
        let second = format_aura(&formatted).expect("second format");
        assert_eq!(formatted, second);
    }

    #[test]
    fn test_format_js_and_ts() {
        let js_src = "function add(a,b){\nlet x=a+b;\nreturn x;\n}";
        let formatted_js = format_js(js_src).expect("format js");
        assert!(formatted_js.contains("  let x = a + b;"));

        let ts_src = "export interface UserProps<T>{\nid:number;\nname:string;\n}";
        let formatted_ts = format_ts(ts_src).expect("format ts");
        assert!(formatted_ts.contains("export interface UserProps<T> {"));
        assert!(formatted_ts.contains("  id: number;"));
        assert!(formatted_ts.contains("  name: string;"));
    }

    #[test]
    fn test_unified_diff() {
        let orig = "fn main() {\nlet x = 1;\n}\n";
        let formatted = "fn main() {\n  let x = 1;\n}\n";
        let diff = generate_unified_diff("main.aura", orig, formatted);
        assert!(diff.contains("--- main.aura.orig"));
        assert!(diff.contains("+++ main.aura"));
        assert!(diff.contains("-let x = 1;"));
        assert!(diff.contains("+  let x = 1;"));

        let empty_diff = generate_unified_diff("same.aura", formatted, formatted);
        assert!(empty_diff.is_empty());
    }
}
