//! Lexer for the Aura programming language.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RawTemplateSegment {
    Text(String),
    ExprCode(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Mut,
    Type,
    Match,
    When,
    If,
    Else,
    Async,
    Await,
    Import,
    Export,
    From,
    Extern,
    Trait,
    Impl,
    Return,
    Module,
    Spawn,
    Routine,
    Go,
    Select,
    Default,
    True,
    False,
    While,
    For,
    In,
    Break,
    Continue,
    Defer,
    ErrDefer,
    Embed,
    EmbedBytes,
    Interface,
    Panic,
    Recover,
    Struct,
    Packed,

    // Identifiers & Literals
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),
    TemplateString(Vec<RawTemplateSegment>),

    // Operators
    Pipe,         // |>
    FatArrow,     // =>
    Arrow,        // ->
    ArrowLeft,    // <-
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Ampersand,    // &
    AndAnd,       // &&
    OrOr,         // ||
    Bang,         // !
    PipeOp,       // | (single pipe for types / match)
    Question,     // ?

    // Delimiters & Punctuation
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Colon,      // :
    Comma,      // ,
    Semicolon,  // ;
    Dot,        // .
    DotDot,     // ..
    Spread,     // ...
    Underscore, // _
    Hash,       // #

    // EOF
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::When => write!(f, "when"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Async => write!(f, "async"),
            TokenKind::Await => write!(f, "await"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Extern => write!(f, "extern"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::Impl => write!(f, "impl"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Module => write!(f, "module"),
            TokenKind::Spawn => write!(f, "spawn"),
            TokenKind::Routine => write!(f, "routine"),
            TokenKind::Go => write!(f, "go"),
            TokenKind::Select => write!(f, "select"),
            TokenKind::Default => write!(f, "default"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Defer => write!(f, "defer"),
            TokenKind::ErrDefer => write!(f, "errdefer"),
            TokenKind::Embed => write!(f, "embed"),
            TokenKind::EmbedBytes => write!(f, "embedBytes"),
            TokenKind::Interface => write!(f, "interface"),
            TokenKind::Panic => write!(f, "panic"),
            TokenKind::Recover => write!(f, "recover"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Packed => write!(f, "packed"),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Int(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Pipe => write!(f, "|>"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::ArrowLeft => write!(f, "<-"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::PipeOp => write!(f, "|"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::Spread => write!(f, "..."),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::Hash => write!(f, "#"),
            TokenKind::Eof => write!(f, "<EOF>"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    _input: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            _input: input,
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek_next() == Some('/') {
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else if ch == '/' && self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                while let Some(c) = self.peek() {
                    if c == '*' && self.peek_next() == Some('/') {
                        self.advance();
                        self.advance();
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        let start_pos = self.pos;
        let line = self.line;
        let column = self.column;

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::Eof,
                    span: Span {
                        start: start_pos,
                        end: start_pos,
                        line,
                        column,
                    },
                });
            }
        };

        if ch.is_alphabetic() || ch == '_' {
            let mut s = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    s.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let kind = match s.as_str() {
                "fn" => TokenKind::Fn,
                "let" => TokenKind::Let,
                "mut" => TokenKind::Mut,
                "type" => TokenKind::Type,
                "match" => TokenKind::Match,
                "when" => TokenKind::When,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "async" => TokenKind::Async,
                "await" => TokenKind::Await,
                "import" => TokenKind::Import,
                "export" => TokenKind::Export,
                "from" => TokenKind::From,
                "extern" => TokenKind::Extern,
                "trait" => TokenKind::Trait,
                "impl" => TokenKind::Impl,
                "return" => TokenKind::Return,
                "module" => TokenKind::Module,
                "spawn" => TokenKind::Spawn,
                "routine" => TokenKind::Routine,
                "go" => TokenKind::Go,
                "select" => TokenKind::Select,
                "default" => TokenKind::Default,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "while" => TokenKind::While,
                "for" => TokenKind::For,
                "in" => TokenKind::In,
                "break" => TokenKind::Break,
                "continue" => TokenKind::Continue,
                "defer" => TokenKind::Defer,
                "errdefer" => TokenKind::ErrDefer,
                "embed" => TokenKind::Embed,
                "embedBytes" => TokenKind::EmbedBytes,
                "interface" => TokenKind::Interface,
                "panic" => TokenKind::Panic,
                "recover" => TokenKind::Recover,
                "struct" => TokenKind::Struct,
                "packed" => TokenKind::Packed,
                "_" => TokenKind::Underscore,
                _ => TokenKind::Ident(s),
            };

            return Ok(Token {
                kind,
                span: Span {
                    start: start_pos,
                    end: self.pos,
                    line,
                    column,
                },
            });
        }

        if ch.is_ascii_digit() {
            let mut s = String::new();
            let mut has_dot = false;

            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    if c != '_' {
                        s.push(c);
                    }
                    self.advance();
                } else if c == '.'
                    && !has_dot
                    && self
                        .peek_next()
                        .map(|n| n.is_ascii_digit())
                        .unwrap_or(false)
                {
                    has_dot = true;
                    s.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let kind = if has_dot {
                let val: f64 = s
                    .parse()
                    .map_err(|e| format!("Invalid float {}: {}", s, e))?;
                TokenKind::Float(val)
            } else {
                let val: i64 = s.parse().map_err(|e| format!("Invalid int {}: {}", s, e))?;
                TokenKind::Int(val)
            };

            return Ok(Token {
                kind,
                span: Span {
                    start: start_pos,
                    end: self.pos,
                    line,
                    column,
                },
            });
        }

        if ch == '"' {
            self.advance();
            let mut s = String::new();
            while let Some(c) = self.peek() {
                if c == '"' {
                    self.advance();
                    return Ok(Token {
                        kind: TokenKind::String(s),
                        span: Span {
                            start: start_pos,
                            end: self.pos,
                            line,
                            column,
                        },
                    });
                } else if c == '\\' {
                    self.advance();
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        Some(other) => s.push(other),
                        None => return Err("Unterminated string escape".to_string()),
                    }
                } else {
                    s.push(c);
                    self.advance();
                }
            }
            return Err("Unterminated string literal".to_string());
        }

        if ch == '`' {
            self.advance();
            let mut segments = Vec::new();
            let mut current_text = String::new();

            while let Some(c) = self.peek() {
                if c == '`' {
                    self.advance();
                    if !current_text.is_empty() || segments.is_empty() {
                        segments.push(RawTemplateSegment::Text(current_text));
                    }
                    return Ok(Token {
                        kind: TokenKind::TemplateString(segments),
                        span: Span {
                            start: start_pos,
                            end: self.pos,
                            line,
                            column,
                        },
                    });
                } else if c == '$' && self.peek_next() == Some('{') {
                    self.advance(); // $
                    self.advance(); // {
                    if !current_text.is_empty() {
                        segments.push(RawTemplateSegment::Text(current_text.clone()));
                        current_text.clear();
                    }

                    let mut expr_code = String::new();
                    let mut brace_depth = 1;
                    while let Some(ec) = self.advance() {
                        if ec == '{' {
                            brace_depth += 1;
                            expr_code.push(ec);
                        } else if ec == '}' {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                break;
                            }
                            expr_code.push(ec);
                        } else {
                            expr_code.push(ec);
                        }
                    }
                    segments.push(RawTemplateSegment::ExprCode(expr_code));
                } else if c == '\\' {
                    self.advance();
                    match self.advance() {
                        Some('n') => current_text.push('\n'),
                        Some('`') => current_text.push('`'),
                        Some('$') => current_text.push('$'),
                        Some(other) => current_text.push(other),
                        None => return Err("Unterminated template escape".to_string()),
                    }
                } else {
                    current_text.push(c);
                    self.advance();
                }
            }
            return Err("Unterminated template literal".to_string());
        }

        let kind = match ch {
            '?' => {
                self.advance();
                TokenKind::Question
            }
            '|' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Pipe
                } else if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::OrOr
                } else {
                    TokenKind::PipeOp
                }
            }
            '=' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::NotEqual
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LessEqual
                } else if self.peek() == Some('-') {
                    self.advance();
                    TokenKind::ArrowLeft
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '&' => {
                self.advance();
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::AndAnd
                } else {
                    TokenKind::Ampersand
                }
            }
            '.' => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        TokenKind::Spread
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }
            '/' => {
                self.advance();
                TokenKind::Slash
            }
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '*' => {
                self.advance();
                TokenKind::Star
            }
            '%' => {
                self.advance();
                TokenKind::Percent
            }
            '(' => {
                self.advance();
                TokenKind::LParen
            }
            ')' => {
                self.advance();
                TokenKind::RParen
            }
            '{' => {
                self.advance();
                TokenKind::LBrace
            }
            '}' => {
                self.advance();
                TokenKind::RBrace
            }
            '[' => {
                self.advance();
                TokenKind::LBracket
            }
            ']' => {
                self.advance();
                TokenKind::RBracket
            }
            ':' => {
                self.advance();
                TokenKind::Colon
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            ';' => {
                self.advance();
                TokenKind::Semicolon
            }
            '#' => {
                self.advance();
                TokenKind::Hash
            }
            _ => {
                self.advance();
                return Err(format!(
                    "Unexpected character '{}' at {}:{}",
                    ch, line, column
                ));
            }
        };

        Ok(Token {
            kind,
            span: Span {
                start: start_pos,
                end: self.pos,
                line,
                column,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let input = "fn let mut type match when if else async await import export from extern trait impl return module spawn routine go select default true false";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("failed to tokenize keywords");
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Fn,
                TokenKind::Let,
                TokenKind::Mut,
                TokenKind::Type,
                TokenKind::Match,
                TokenKind::When,
                TokenKind::If,
                TokenKind::Else,
                TokenKind::Async,
                TokenKind::Await,
                TokenKind::Import,
                TokenKind::Export,
                TokenKind::From,
                TokenKind::Extern,
                TokenKind::Trait,
                TokenKind::Impl,
                TokenKind::Return,
                TokenKind::Module,
                TokenKind::Spawn,
                TokenKind::Routine,
                TokenKind::Go,
                TokenKind::Select,
                TokenKind::Default,
                TokenKind::True,
                TokenKind::False,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_identifiers_and_numbers() {
        let input = "foo_bar _hidden count42 0 123 1_000_000 3.14 0.001 1_000.5";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("failed to tokenize");
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("foo_bar".to_string()),
                TokenKind::Ident("_hidden".to_string()),
                TokenKind::Ident("count42".to_string()),
                TokenKind::Int(0),
                TokenKind::Int(123),
                TokenKind::Int(1_000_000),
                TokenKind::Float(3.14),
                TokenKind::Float(0.001),
                TokenKind::Float(1000.5),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_strings_and_escapes() {
        let input = r#""hello" "hello\nworld" "tab\tquote\"slash\\""#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("failed to tokenize strings");
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::String("hello".to_string()),
                TokenKind::String("hello\nworld".to_string()),
                TokenKind::String("tab\tquote\"slash\\".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_unterminated_string() {
        let input = r#""unterminated string"#;
        let mut lexer = Lexer::new(input);
        assert!(lexer.tokenize().is_err());
    }

    #[test]
    fn test_template_strings() {
        let input = "`Hello ${name}! 1 + 1 = ${1 + 1}` `plain template`";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("failed to tokenize template");
        assert_eq!(tokens.len(), 3); // 2 templates + EOF
        if let TokenKind::TemplateString(segments) = &tokens[0].kind {
            assert_eq!(
                segments,
                &vec![
                    RawTemplateSegment::Text("Hello ".to_string()),
                    RawTemplateSegment::ExprCode("name".to_string()),
                    RawTemplateSegment::Text("! 1 + 1 = ".to_string()),
                    RawTemplateSegment::ExprCode("1 + 1".to_string()),
                ]
            );
        } else {
            panic!("Expected TemplateString token, got {:?}", tokens[0].kind);
        }

        if let TokenKind::TemplateString(segments) = &tokens[1].kind {
            assert_eq!(
                segments,
                &vec![RawTemplateSegment::Text("plain template".to_string())]
            );
        } else {
            panic!("Expected plain TemplateString, got {:?}", tokens[1].kind);
        }
    }

    #[test]
    fn test_operators_and_delimiters() {
        let input =
            "+ - * / % |> => -> <- = == != < <= > >= && || ! | ? ( ) { } [ ] : , ; . .. ... _ #";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("failed to tokenize operators");
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Pipe,
                TokenKind::FatArrow,
                TokenKind::Arrow,
                TokenKind::ArrowLeft,
                TokenKind::Equal,
                TokenKind::EqualEqual,
                TokenKind::NotEqual,
                TokenKind::Less,
                TokenKind::LessEqual,
                TokenKind::Greater,
                TokenKind::GreaterEqual,
                TokenKind::AndAnd,
                TokenKind::OrOr,
                TokenKind::Bang,
                TokenKind::PipeOp,
                TokenKind::Question,
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::Colon,
                TokenKind::Comma,
                TokenKind::Semicolon,
                TokenKind::Dot,
                TokenKind::DotDot,
                TokenKind::Spread,
                TokenKind::Underscore,
                TokenKind::Hash,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_comments() {
        let input = "// line comment\nlet x = 10; /* block comment */ let y = 20;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("failed to tokenize with comments");
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Let,
                TokenKind::Ident("x".to_string()),
                TokenKind::Equal,
                TokenKind::Int(10),
                TokenKind::Semicolon,
                TokenKind::Let,
                TokenKind::Ident("y".to_string()),
                TokenKind::Equal,
                TokenKind::Int(20),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_spans_and_positions() {
        let input = "let x = 42;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("failed to tokenize");
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 3); // "let"
    }

    #[test]
    fn test_invalid_character_error() {
        let input = "let @bad = 1;";
        let mut lexer = Lexer::new(input);
        assert!(lexer.tokenize().is_err());
    }

    #[test]
    fn test_display_tokens() {
        assert_eq!(format!("{}", TokenKind::Fn), "fn");
        assert_eq!(format!("{}", TokenKind::Pipe), "|>");
        assert_eq!(format!("{}", TokenKind::Int(42)), "42");
        assert_eq!(
            format!("{}", TokenKind::String("test".to_string())),
            "\"test\""
        );
    }
}
