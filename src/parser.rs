//! Recursive Descent & Pratt Parser for the Aura programming language.

use crate::ast::*;
use crate::lexer::{Lexer, RawTemplateSegment, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenKind {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos].kind
        } else {
            &TokenKind::Eof
        }
    }

    fn peek_next(&self) -> &TokenKind {
        if self.pos + 1 < self.tokens.len() {
            &self.tokens[self.pos + 1].kind
        } else {
            &TokenKind::Eof
        }
    }

    #[allow(dead_code)]
    fn peek_ahead(&self, offset: usize) -> Option<&TokenKind> {
        if self.pos + offset < self.tokens.len() {
            Some(&self.tokens[self.pos + offset].kind)
        } else {
            None
        }
    }

    fn advance(&mut self) -> TokenKind {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].kind.clone();
            self.pos += 1;
            tok
        } else {
            TokenKind::Eof
        }
    }

    fn match_token(&mut self, expected: &TokenKind) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), String> {
        let current = self.advance();
        if std::mem::discriminant(&current) == std::mem::discriminant(&expected) {
            Ok(())
        } else {
            Err(format!(
                "Expected token {:?}, found {:?}",
                expected, current
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            TokenKind::Ident(s) => Ok(s),
            TokenKind::Type => Ok("type".to_string()),
            TokenKind::From => Ok("from".to_string()),
            TokenKind::When => Ok("when".to_string()),
            TokenKind::Match => Ok("match".to_string()),
            TokenKind::Module => Ok("module".to_string()),
            TokenKind::Default => Ok("default".to_string()),
            TokenKind::Spawn => Ok("spawn".to_string()),
            TokenKind::Routine => Ok("routine".to_string()),
            TokenKind::Go => Ok("go".to_string()),
            TokenKind::Select => Ok("select".to_string()),
            TokenKind::While => Ok("while".to_string()),
            TokenKind::For => Ok("for".to_string()),
            TokenKind::In => Ok("in".to_string()),
            TokenKind::Break => Ok("break".to_string()),
            TokenKind::Continue => Ok("continue".to_string()),
            TokenKind::Interface => Ok("interface".to_string()),
            TokenKind::Panic => Ok("panic".to_string()),
            TokenKind::Recover => Ok("recover".to_string()),
            TokenKind::Underscore => Ok("_".to_string()),
            other => Err(format!("Expected identifier, found {:?}", other)),
        }
    }

    fn is_arrow_function_start(&self) -> bool {
        if self.peek() != &TokenKind::LParen {
            return false;
        }
        if self.pos + 1 < self.tokens.len() {
            match &self.tokens[self.pos + 1].kind {
                TokenKind::RParen | TokenKind::Ident(_) | TokenKind::LBrace | TokenKind::Spread => {
                }
                _ => return false,
            }
        }
        let mut depth = 0;
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        let next_idx = i + 1;
                        if next_idx < self.tokens.len() {
                            match &self.tokens[next_idx].kind {
                                TokenKind::FatArrow => return true,
                                TokenKind::Colon | TokenKind::Arrow => {
                                    let mut j = next_idx + 1;
                                    let mut type_depth = 0;
                                    let max_scan = (next_idx + 15).min(self.tokens.len());
                                    while j < max_scan {
                                        match &self.tokens[j].kind {
                                            TokenKind::Less
                                            | TokenKind::LParen
                                            | TokenKind::LBrace
                                            | TokenKind::LBracket => type_depth += 1,
                                            TokenKind::Greater
                                            | TokenKind::RParen
                                            | TokenKind::RBrace
                                            | TokenKind::RBracket => {
                                                if type_depth > 0 {
                                                    type_depth -= 1;
                                                }
                                            }
                                            TokenKind::FatArrow if type_depth == 0 => return true,
                                            TokenKind::Semicolon
                                            | TokenKind::Eof
                                            | TokenKind::Comma
                                            | TokenKind::PipeOp
                                                if type_depth == 0 =>
                                            {
                                                return false;
                                            }
                                            _ => {}
                                        }
                                        j += 1;
                                    }
                                    return false;
                                }
                                _ => return false,
                            }
                        }
                        return false;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        false
    }

    pub fn parse_module(&mut self) -> Result<Module, String> {
        let mut module_name = None;
        if self.match_token(&TokenKind::Module) {
            let mut name = self.expect_ident()?;
            while self.match_token(&TokenKind::Dot) {
                name.push('.');
                name.push_str(&self.expect_ident()?);
            }
            module_name = Some(name);
        }

        let mut items = Vec::new();
        while self.peek() != &TokenKind::Eof {
            items.push(self.parse_item()?);
        }

        Ok(Module {
            name: module_name,
            items,
        })
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        let is_exported = self.match_token(&TokenKind::Export);

        match self.peek() {
            TokenKind::Import => {
                let imp = self.parse_import()?;
                Ok(Item::Import(imp))
            }
            TokenKind::Extern => {
                let ext = self.parse_extern()?;
                Ok(Item::Extern(ext))
            }
            TokenKind::Fn | TokenKind::Async => {
                let func = self.parse_function(is_exported)?;
                self.match_token(&TokenKind::Semicolon);
                Ok(Item::Function(func))
            }
            TokenKind::Interface | TokenKind::Trait => {
                let iface = self.parse_interface(is_exported)?;
                self.match_token(&TokenKind::Semicolon);
                Ok(Item::Interface(iface))
            }
            TokenKind::Packed => {
                self.advance();
                let _ = self.match_token(&TokenKind::Struct);
                let name = self.expect_ident()?;
                let type_params = self.parse_optional_type_params()?;
                if self.peek() == &TokenKind::Equal {
                    self.advance();
                }
                let target = self.parse_type()?;
                self.match_token(&TokenKind::Semicolon);
                Ok(Item::TypeAlias(TypeAliasDecl {
                    name,
                    is_exported,
                    is_packed: true,
                    type_params,
                    target,
                }))
            }
            TokenKind::Struct => {
                self.advance();
                let name = self.expect_ident()?;
                let type_params = self.parse_optional_type_params()?;
                let target = self.parse_type()?;
                self.match_token(&TokenKind::Semicolon);
                Ok(Item::TypeAlias(TypeAliasDecl {
                    name,
                    is_exported,
                    is_packed: false,
                    type_params,
                    target,
                }))
            }
            TokenKind::Type => {
                self.advance();
                let name = self.expect_ident()?;
                let type_params = self.parse_optional_type_params()?;

                if self.peek() == &TokenKind::Equal {
                    self.advance();
                }
                let is_packed = self.match_token(&TokenKind::Packed);
                let _ = self.match_token(&TokenKind::Struct);

                let is_sum_type = if self.peek() == &TokenKind::PipeOp {
                    true
                } else if matches!(self.peek(), TokenKind::Ident(s) if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
                {
                    matches!(
                        self.peek_next(),
                        TokenKind::LParen | TokenKind::LBrace | TokenKind::PipeOp
                    )
                } else {
                    false
                };

                if is_sum_type {
                    let variants = self.parse_sum_type_variants()?;
                    self.match_token(&TokenKind::Semicolon);
                    Ok(Item::SumType(SumTypeDecl {
                        name,
                        is_exported,
                        type_params,
                        variants,
                    }))
                } else {
                    let target = self.parse_type()?;
                    self.match_token(&TokenKind::Semicolon);
                    Ok(Item::TypeAlias(TypeAliasDecl {
                        name,
                        is_exported,
                        is_packed,
                        type_params,
                        target,
                    }))
                }
            }
            TokenKind::Let => {
                let stmt = self.parse_statement()?;
                Ok(Item::Statement(stmt))
            }
            _ => {
                let stmt = self.parse_statement()?;
                Ok(Item::Statement(stmt))
            }
        }
    }

    fn parse_import(&mut self) -> Result<ImportDecl, String> {
        self.expect(TokenKind::Import)?;
        self.expect(TokenKind::LBrace)?;

        let mut items = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            let name = self.expect_ident()?;
            let mut alias = None;
            if self.match_token(&TokenKind::Ident("as".to_string())) {
                alias = Some(self.expect_ident()?);
            }
            items.push(ImportItem { name, alias });
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::From)?;

        let source = match self.advance() {
            TokenKind::String(s) => s,
            other => {
                return Err(format!(
                    "Expected string for import path, found {:?}",
                    other
                ));
            }
        };
        self.match_token(&TokenKind::Semicolon);

        Ok(ImportDecl { source, items })
    }

    fn parse_extern(&mut self) -> Result<ExternDecl, String> {
        self.expect(TokenKind::Extern)?;
        let module_name = match self.advance() {
            TokenKind::String(s) => s,
            other => {
                return Err(format!(
                    "Expected module string for extern, found {:?}",
                    other
                ));
            }
        };

        self.expect(TokenKind::LBrace)?;
        let mut functions = Vec::new();

        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            self.expect(TokenKind::Fn)?;
            let name = self.expect_ident()?;
            let type_params = self.parse_optional_type_params()?;
            let params = self.parse_param_list()?;
            self.expect(TokenKind::Colon)?;
            let return_type = self.parse_type()?;
            self.match_token(&TokenKind::Semicolon);

            functions.push(ExternFunction {
                name,
                type_params,
                params,
                return_type,
            });
        }
        self.expect(TokenKind::RBrace)?;
        self.match_token(&TokenKind::Semicolon);

        Ok(ExternDecl {
            module_name,
            functions,
        })
    }

    fn parse_function(&mut self, is_exported: bool) -> Result<FunctionDecl, String> {
        let is_async = self.match_token(&TokenKind::Async);
        self.expect(TokenKind::Fn)?;

        let receiver = if self.peek() == &TokenKind::LParen {
            self.advance();
            let recv_name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let target_type = self.parse_type()?;
            self.expect(TokenKind::RParen)?;
            Some(Receiver {
                name: recv_name,
                target_type,
            })
        } else {
            None
        };

        let name = self.expect_ident()?;
        let type_params = self.parse_optional_type_params()?;
        let params = self.parse_param_list()?;

        let mut return_type = None;
        if self.match_token(&TokenKind::Colon) || self.match_token(&TokenKind::Arrow) {
            return_type = Some(self.parse_type()?);
        }

        let body = if self.match_token(&TokenKind::FatArrow) {
            self.parse_expr()?
        } else if self.peek() == &TokenKind::LBrace {
            self.parse_block_expr()?
        } else {
            return Err(format!(
                "Expected '=>' or '{{' for function body, found {:?}",
                self.peek()
            ));
        };

        let is_exported = is_exported || name.chars().next().map_or(false, |c| c.is_uppercase());

        Ok(FunctionDecl {
            name,
            receiver,
            is_async,
            is_exported,
            type_params,
            params,
            return_type,
            body,
        })
    }

    fn parse_interface(&mut self, is_exported: bool) -> Result<InterfaceDecl, String> {
        if self.match_token(&TokenKind::Interface) || self.match_token(&TokenKind::Trait) {}
        let name = self.expect_ident()?;
        let is_exported = is_exported || name.chars().next().map_or(false, |c| c.is_uppercase());
        let type_params = self.parse_optional_type_params()?;
        self.expect(TokenKind::LBrace)?;
        let mut methods = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            self.match_token(&TokenKind::Fn);
            let method_name = self.expect_ident()?;
            let params = self.parse_param_list()?;
            let mut return_type = Type::Unit;
            if self.match_token(&TokenKind::Colon) || self.match_token(&TokenKind::Arrow) {
                return_type = self.parse_type()?;
            }
            self.match_token(&TokenKind::Semicolon);
            methods.push(InterfaceMethod {
                name: method_name,
                params,
                return_type,
            });
        }
        self.expect(TokenKind::RBrace)?;
        Ok(InterfaceDecl {
            name,
            is_exported,
            type_params,
            methods,
        })
    }

    fn parse_optional_type_params(&mut self) -> Result<Vec<String>, String> {
        if self.match_token(&TokenKind::Less) {
            let mut params = Vec::new();
            while self.peek() != &TokenKind::Greater && self.peek() != &TokenKind::Eof {
                params.push(self.expect_ident()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::Greater)?;
            Ok(params)
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, String> {
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            let name = if self.match_token(&TokenKind::LBrace) {
                let mut fields = Vec::new();
                while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                    fields.push(self.expect_ident()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBrace)?;
                format!("{{ {} }}", fields.join(", "))
            } else {
                self.expect_ident()?
            };

            let mut type_annotation = None;
            if self.match_token(&TokenKind::Colon) {
                type_annotation = Some(self.parse_type()?);
            }

            let mut default_value = None;
            if self.match_token(&TokenKind::Equal) {
                default_value = Some(self.parse_expr()?);
            }

            params.push(Param {
                name,
                type_annotation,
                default_value,
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(params)
    }

    fn parse_sum_type_variants(&mut self) -> Result<Vec<Variant>, String> {
        let mut variants = Vec::new();
        self.match_token(&TokenKind::PipeOp);

        loop {
            let name = self.expect_ident()?;
            let fields = if self.peek() == &TokenKind::LParen {
                self.advance();
                let mut types = Vec::new();
                while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                    types.push(self.parse_type()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                VariantFields::Tuple(types)
            } else if self.peek() == &TokenKind::LBrace {
                self.advance();
                let mut fields_vec = Vec::new();
                while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                    let field_name = self.expect_ident()?;
                    self.expect(TokenKind::Colon)?;
                    let field_type = self.parse_type()?;
                    fields_vec.push((field_name, field_type));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBrace)?;
                VariantFields::Record(fields_vec)
            } else {
                VariantFields::Unit
            };

            variants.push(Variant { name, fields });

            if !self.match_token(&TokenKind::PipeOp) {
                break;
            }
        }

        Ok(variants)
    }

    pub fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek() {
            TokenKind::LBracket => {
                self.advance();
                if self.match_token(&TokenKind::RBracket) {
                    let inner = self.parse_type()?;
                    Ok(Type::Named {
                        name: "List".to_string(),
                        type_args: vec![inner],
                    })
                } else {
                    let inner = self.parse_type()?;
                    self.expect(TokenKind::RBracket)?;
                    Ok(Type::Named {
                        name: "List".to_string(),
                        type_args: vec![inner],
                    })
                }
            }
            TokenKind::Packed | TokenKind::Struct => {
                self.advance();
                let _ = self.match_token(&TokenKind::Struct);
                self.parse_type()
            }
            TokenKind::Star => {
                self.advance();
                let inner = self.parse_type()?;
                Ok(Type::Pointer(Box::new(inner)))
            }
            TokenKind::ArrowLeft => {
                self.advance();
                if matches!(self.peek(), TokenKind::Ident(s) if s == "chan") {
                    self.advance();
                }
                let inner = self.parse_type()?;
                Ok(Type::RecvChannel(Box::new(inner)))
            }
            TokenKind::Ident(s) if s == "chan" && self.peek_next() == &TokenKind::ArrowLeft => {
                self.advance(); // chan
                self.advance(); // <-
                let inner = self.parse_type()?;
                Ok(Type::SendChannel(Box::new(inner)))
            }
            TokenKind::Ident(s) => {
                let name = s.clone();
                self.advance();

                let mut type_args = Vec::new();
                if self.match_token(&TokenKind::Less) {
                    while self.peek() != &TokenKind::Greater && self.peek() != &TokenKind::Eof {
                        type_args.push(self.parse_type()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::Greater)?;
                }

                if name == "SendChannel" && type_args.len() == 1 {
                    Ok(Type::SendChannel(Box::new(type_args.remove(0))))
                } else if name == "RecvChannel" && type_args.len() == 1 {
                    Ok(Type::RecvChannel(Box::new(type_args.remove(0))))
                } else {
                    Ok(Type::Named { name, type_args })
                }
            }
            TokenKind::LBrace => {
                self.advance();
                let mut fields = Vec::new();
                while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                    let field_name = self.expect_ident()?;
                    let is_opt = self.match_token(&TokenKind::Question);
                    self.expect(TokenKind::Colon)?;
                    let field_type = self.parse_type()?;
                    if matches!(
                        self.peek(),
                        TokenKind::TemplateString(_) | TokenKind::String(_)
                    ) {
                        self.advance();
                    }
                    let final_name = if is_opt {
                        format!("{}?", field_name)
                    } else {
                        field_name
                    };
                    fields.push((final_name, field_type));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Type::Record(fields))
            }
            TokenKind::LParen => {
                self.advance();
                if self.match_token(&TokenKind::RParen) {
                    if self.match_token(&TokenKind::Arrow) {
                        let ret = self.parse_type()?;
                        return Ok(Type::Function {
                            params: vec![],
                            return_type: Box::new(ret),
                        });
                    }
                    return Ok(Type::Unit);
                }

                let first = self.parse_type()?;
                if self.match_token(&TokenKind::Comma) {
                    let mut types = vec![first];
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        types.push(self.parse_type()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    if self.match_token(&TokenKind::Arrow) {
                        let ret = self.parse_type()?;
                        Ok(Type::Function {
                            params: types,
                            return_type: Box::new(ret),
                        })
                    } else {
                        Ok(Type::Tuple(types))
                    }
                } else {
                    self.expect(TokenKind::RParen)?;
                    if self.match_token(&TokenKind::Arrow) {
                        let ret = self.parse_type()?;
                        Ok(Type::Function {
                            params: vec![first],
                            return_type: Box::new(ret),
                        })
                    } else {
                        Ok(first)
                    }
                }
            }
            TokenKind::Fn => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let mut params = Vec::new();
                while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                    params.push(self.parse_type()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                self.expect(TokenKind::Arrow)?;
                let ret = self.parse_type()?;
                Ok(Type::Function {
                    params,
                    return_type: Box::new(ret),
                })
            }
            other => Err(format!("Unexpected token in type: {:?}", other)),
        }
    }

    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        if self.match_token(&TokenKind::Let) {
            let is_mut = self.match_token(&TokenKind::Mut);
            if self.peek() == &TokenKind::LParen || self.peek() == &TokenKind::LBrace {
                let pattern = self.parse_pattern()?;
                let mut type_annotation = None;
                if self.match_token(&TokenKind::Colon) {
                    type_annotation = Some(self.parse_type()?);
                }
                self.expect(TokenKind::Equal)?;
                let value = self.parse_expr()?;
                self.match_token(&TokenKind::Semicolon);
                return Ok(Statement::LetPattern {
                    pattern,
                    type_annotation,
                    value,
                });
            }
            let name = self.expect_ident()?;
            let mut type_annotation = None;
            if self.match_token(&TokenKind::Colon) {
                type_annotation = Some(self.parse_type()?);
            }
            self.expect(TokenKind::Equal)?;
            let value = self.parse_expr()?;
            self.match_token(&TokenKind::Semicolon);
            Ok(Statement::Let {
                name,
                is_mut,
                type_annotation,
                value,
            })
        } else if self.match_token(&TokenKind::Defer) {
            let expr = self.parse_expr()?;
            let final_expr = if self.match_token(&TokenKind::Equal) {
                let value = self.parse_expr()?;
                Expr::Block(vec![Statement::Assign {
                    target: expr,
                    value,
                }])
            } else {
                expr
            };
            self.match_token(&TokenKind::Semicolon);
            Ok(Statement::Defer(Box::new(final_expr)))
        } else if self.match_token(&TokenKind::ErrDefer) {
            let expr = self.parse_expr()?;
            let final_expr = if self.match_token(&TokenKind::Equal) {
                let value = self.parse_expr()?;
                Expr::Block(vec![Statement::Assign {
                    target: expr,
                    value,
                }])
            } else {
                expr
            };
            self.match_token(&TokenKind::Semicolon);
            Ok(Statement::ErrDefer(Box::new(final_expr)))
        } else if self.match_token(&TokenKind::Return) {
            let val = if self.peek() != &TokenKind::Semicolon && self.peek() != &TokenKind::RBrace {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.match_token(&TokenKind::Semicolon);
            Ok(Statement::Return(val))
        } else if self.peek() == &TokenKind::Fn && matches!(self.peek_next(), TokenKind::Ident(_)) {
            self.advance();
            let name = self.expect_ident()?;
            let params = self.parse_param_list()?;
            let mut return_type = None;
            if self.match_token(&TokenKind::Colon) || self.match_token(&TokenKind::Arrow) {
                return_type = Some(self.parse_type()?);
            }
            let body = if self.match_token(&TokenKind::FatArrow) {
                self.parse_expr()?
            } else if self.peek() == &TokenKind::LBrace {
                self.parse_block_expr()?
            } else {
                return Err("Expected '=>' or '{' for local function body".to_string());
            };
            self.match_token(&TokenKind::Semicolon);
            Ok(Statement::Let {
                name,
                is_mut: false,
                type_annotation: None,
                value: Expr::Lambda {
                    params,
                    return_type,
                    body: Box::new(body),
                },
            })
        } else {
            let expr = self.parse_expr()?;
            if self.match_token(&TokenKind::Equal) {
                let value = self.parse_expr()?;
                self.match_token(&TokenKind::Semicolon);
                Ok(Statement::Assign {
                    target: expr,
                    value,
                })
            } else {
                self.match_token(&TokenKind::Semicolon);
                Ok(Statement::Expr(expr))
            }
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_chan_send_expr()
    }

    fn parse_chan_send_expr(&mut self) -> Result<Expr, String> {
        let left = self.parse_ternary_expr()?;
        if self.match_token(&TokenKind::ArrowLeft) {
            let right = self.parse_chan_send_expr()?;
            Ok(Expr::ChanSend {
                channel: Box::new(left),
                value: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_ternary_expr(&mut self) -> Result<Expr, String> {
        let condition = self.parse_pipeline_expr()?;

        if self.match_token(&TokenKind::Question) {
            let then_branch = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            let else_branch = self.parse_expr()?;

            Ok(Expr::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            })
        } else {
            Ok(condition)
        }
    }

    fn parse_pipeline_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_logical_or()?;

        while self.match_token(&TokenKind::Pipe) {
            let right = self.parse_logical_or()?;
            left = Expr::Pipeline {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_logical_and()?;
        while self.match_token(&TokenKind::OrOr) {
            let right = self.parse_logical_and()?;
            left = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.match_token(&TokenKind::AndAnd) {
            let right = self.parse_equality()?;
            left = Expr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while let Some(op) = match self.peek() {
            TokenKind::EqualEqual => Some(BinOp::Equal),
            TokenKind::NotEqual => Some(BinOp::NotEqual),
            _ => None,
        } {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        while let Some(op) = match self.peek() {
            TokenKind::Less => Some(BinOp::LessThan),
            TokenKind::LessEqual => Some(BinOp::LessEqual),
            TokenKind::Greater => Some(BinOp::GreaterThan),
            TokenKind::GreaterEqual => Some(BinOp::GreaterEqual),
            _ => None,
        } {
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        while let Some(op) = match self.peek() {
            TokenKind::Plus => Some(BinOp::Add),
            TokenKind::Minus => Some(BinOp::Sub),
            _ => None,
        } {
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while let Some(op) = match self.peek() {
            TokenKind::Star => Some(BinOp::Mul),
            TokenKind::Slash => Some(BinOp::Div),
            TokenKind::Percent => Some(BinOp::Mod),
            _ => None,
        } {
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn is_generic_type_args_call(&self) -> bool {
        if self.peek() != &TokenKind::Less {
            return false;
        }
        let mut depth = 0;
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Less => depth += 1,
                TokenKind::Greater => {
                    depth -= 1;
                    if depth == 0 {
                        let next_idx = i + 1;
                        if next_idx < self.tokens.len() {
                            return matches!(&self.tokens[next_idx].kind, TokenKind::LParen);
                        }
                        return false;
                    }
                }
                TokenKind::Semicolon | TokenKind::Eof => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    fn is_ternary_question(&self) -> bool {
        if self.peek() != &TokenKind::Question {
            return false;
        }
        let mut depth = 0;
        let mut i = self.pos + 1;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => depth += 1,
                TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => {
                    if depth > 0 {
                        depth -= 1;
                    } else {
                        return false;
                    }
                }
                TokenKind::Colon if depth == 0 => return true,
                TokenKind::Semicolon | TokenKind::Eof if depth == 0 => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.parse_unary_with_opt(true)
    }

    fn parse_unary_with_opt(&mut self, allow_arrow_lambda: bool) -> Result<Expr, String> {
        if self.match_token(&TokenKind::Minus) {
            let expr = self.parse_unary_with_opt(allow_arrow_lambda)?;
            Ok(Expr::Unary {
                op: UnOp::Negate,
                expr: Box::new(expr),
            })
        } else if self.match_token(&TokenKind::Bang) {
            let expr = self.parse_unary_with_opt(allow_arrow_lambda)?;
            Ok(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(expr),
            })
        } else if self.match_token(&TokenKind::Await) {
            let expr = self.parse_unary_with_opt(allow_arrow_lambda)?;
            Ok(Expr::Await(Box::new(expr)))
        } else if self.match_token(&TokenKind::ArrowLeft) {
            let expr = self.parse_unary_with_opt(allow_arrow_lambda)?;
            Ok(Expr::ChanRecv(Box::new(expr)))
        } else if self.match_token(&TokenKind::Ampersand) {
            let expr = self.parse_unary_with_opt(allow_arrow_lambda)?;
            Ok(Expr::AddressOf(Box::new(expr)))
        } else if self.match_token(&TokenKind::Star) {
            let expr = self.parse_unary_with_opt(allow_arrow_lambda)?;
            Ok(Expr::Deref(Box::new(expr)))
        } else {
            self.parse_postfix_with_opt(allow_arrow_lambda)
        }
    }

    fn parse_postfix_with_opt(&mut self, allow_arrow_lambda: bool) -> Result<Expr, String> {
        let mut expr = self.parse_primary_with_opt(allow_arrow_lambda)?;

        loop {
            if self.match_token(&TokenKind::LParen) {
                let mut args = Vec::new();
                while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                    args.push(self.parse_expr()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                expr = Expr::FunctionCall {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.peek() == &TokenKind::Less && self.is_generic_type_args_call() {
                self.advance(); // <
                let mut type_args = Vec::new();
                while self.peek() != &TokenKind::Greater && self.peek() != &TokenKind::Eof {
                    type_args.push(self.parse_type()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::Greater)?;
                if self.match_token(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        args.push(self.parse_expr()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    expr = Expr::FunctionCall {
                        callee: Box::new(expr),
                        args,
                    };
                }
            } else if self.match_token(&TokenKind::Dot) {
                let member = self.expect_ident()?;
                expr = Expr::MemberAccess {
                    object: Box::new(expr),
                    member,
                };
            } else if self.match_token(&TokenKind::LBracket) {
                if self.match_token(&TokenKind::Colon) || self.match_token(&TokenKind::DotDot) {
                    let high = if self.peek() != &TokenKind::RBracket
                        && self.peek() != &TokenKind::Colon
                    {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    let max = if self.match_token(&TokenKind::Colon) {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    self.expect(TokenKind::RBracket)?;
                    expr = Expr::SliceAccess {
                        object: Box::new(expr),
                        low: None,
                        high,
                        max,
                    };
                } else {
                    let first = self.parse_expr()?;
                    if self.match_token(&TokenKind::Colon) || self.match_token(&TokenKind::DotDot) {
                        let high = if self.peek() != &TokenKind::RBracket
                            && self.peek() != &TokenKind::Colon
                        {
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        let max = if self.match_token(&TokenKind::Colon) {
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        self.expect(TokenKind::RBracket)?;
                        expr = Expr::SliceAccess {
                            object: Box::new(expr),
                            low: Some(Box::new(first)),
                            high,
                            max,
                        };
                    } else {
                        self.expect(TokenKind::RBracket)?;
                        expr = Expr::IndexAccess {
                            object: Box::new(expr),
                            index: Box::new(first),
                        };
                    }
                }
            } else if self.peek() == &TokenKind::Question && !self.is_ternary_question() {
                self.advance();
                expr = Expr::Try(Box::new(expr));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary_with_opt(&mut self, allow_arrow_lambda: bool) -> Result<Expr, String> {
        match self.peek().clone() {
            TokenKind::Int(n) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(n)))
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(n)))
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            TokenKind::TemplateString(raw_segments) => {
                self.advance();
                let mut segments = Vec::new();
                for raw in raw_segments {
                    match raw {
                        RawTemplateSegment::Text(t) => {
                            segments.push(TemplateSegment::Text(t));
                        }
                        RawTemplateSegment::ExprCode(code) => {
                            let mut inner_lexer = Lexer::new(&code);
                            let inner_tokens = inner_lexer.tokenize()?;
                            let mut inner_parser = Parser::new(inner_tokens);
                            let inner_expr = inner_parser.parse_expr()?;
                            segments.push(TemplateSegment::Expr(inner_expr));
                        }
                    }
                }
                Ok(Expr::Literal(Literal::TemplateString(segments)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            TokenKind::Underscore => {
                self.advance();
                Ok(Expr::Placeholder)
            }
            TokenKind::Embed | TokenKind::EmbedBytes => {
                let is_binary = self.peek() == &TokenKind::EmbedBytes;
                self.advance();
                self.expect(TokenKind::LParen)?;
                let path = match self.parse_expr()? {
                    Expr::Literal(Literal::String(s)) => s,
                    other => {
                        return Err(format!(
                            "embed() requires a string literal path, found {:?}",
                            other
                        ));
                    }
                };
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Embed { path, is_binary })
            }
            TokenKind::Ident(name) => {
                self.advance();
                if self.peek() == &TokenKind::Colon
                    && (self.peek_next() == &TokenKind::For
                        || self.peek_next() == &TokenKind::While)
                {
                    let label_name = name.clone();
                    self.advance(); // consume ':'
                    let loop_tok = self.advance(); // consume 'for' or 'while'
                    if loop_tok == TokenKind::For {
                        let has_paren = self.match_token(&TokenKind::LParen);
                        let mut index_name = None;
                        let mut var_name = self.expect_ident()?;
                        if self.match_token(&TokenKind::Comma) {
                            index_name = Some(var_name);
                            var_name = self.expect_ident()?;
                        }
                        let closed_paren_before_in =
                            has_paren && self.match_token(&TokenKind::RParen);
                        self.expect(TokenKind::In)?;
                        let iterable = self.parse_expr()?;
                        if has_paren && !closed_paren_before_in {
                            self.expect(TokenKind::RParen)?;
                        }
                        let body = self.parse_block_expr()?;
                        return Ok(Expr::ForIn {
                            label: Some(label_name),
                            index_name,
                            var_name,
                            iterable: Box::new(iterable),
                            body: Box::new(body),
                        });
                    } else {
                        let has_paren = self.match_token(&TokenKind::LParen);
                        let condition = self.parse_expr()?;
                        if has_paren {
                            self.expect(TokenKind::RParen)?;
                        }
                        let body = self.parse_block_expr()?;
                        return Ok(Expr::While {
                            label: Some(label_name),
                            condition: Box::new(condition),
                            body: Box::new(body),
                        });
                    }
                }
                if allow_arrow_lambda && self.peek() == &TokenKind::FatArrow {
                    self.advance();
                    let body = self.parse_expr()?;
                    return Ok(Expr::Lambda {
                        params: vec![Param {
                            name,
                            type_annotation: None,
                            default_value: None,
                        }],
                        return_type: None,
                        body: Box::new(body),
                    });
                }
                let is_uppercase = name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                if is_uppercase && self.peek() == &TokenKind::LBrace {
                    self.advance();
                    let mut fields = Vec::new();
                    while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                        let fname = self.expect_ident()?;
                        let fval = if self.match_token(&TokenKind::Colon) {
                            self.parse_expr()?
                        } else {
                            Expr::Identifier(fname.clone())
                        };
                        fields.push((fname, fval));
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    Ok(Expr::ConstructorCall {
                        name,
                        args: vec![Expr::RecordLiteral {
                            fields,
                            spread: None,
                        }],
                    })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            TokenKind::LParen => {
                if self.is_arrow_function_start() {
                    let params = self.parse_param_list()?;
                    let mut return_type = None;
                    if self.match_token(&TokenKind::Colon) || self.match_token(&TokenKind::Arrow) {
                        return_type = Some(self.parse_type()?);
                    }
                    self.expect(TokenKind::FatArrow)?;
                    let body = self.parse_expr()?;
                    return Ok(Expr::Lambda {
                        params,
                        return_type,
                        body: Box::new(body),
                    });
                }

                self.advance();
                if self.match_token(&TokenKind::RParen) {
                    return Ok(Expr::Literal(Literal::Unit));
                }
                let first = self.parse_expr()?;
                if self.match_token(&TokenKind::Comma) {
                    let mut items = vec![first];
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        items.push(self.parse_expr()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    Ok(Expr::TupleLiteral(items))
                } else {
                    self.expect(TokenKind::RParen)?;
                    Ok(first)
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                while self.peek() != &TokenKind::RBracket && self.peek() != &TokenKind::Eof {
                    items.push(self.parse_expr()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expr::ListLiteral(items))
            }
            TokenKind::LBrace => self.parse_block_or_record(),
            TokenKind::If => self.parse_if_expr(),
            TokenKind::Match => self.parse_match_expr(),
            TokenKind::Async => {
                self.advance();
                if self.peek() == &TokenKind::Fn {
                    let lambda = self.parse_lambda()?;
                    Ok(Expr::Async(Box::new(lambda)))
                } else if self.is_arrow_function_start() {
                    let params = self.parse_param_list()?;
                    let mut return_type = None;
                    if self.match_token(&TokenKind::Colon) || self.match_token(&TokenKind::Arrow) {
                        return_type = Some(self.parse_type()?);
                    }
                    let body = if self.match_token(&TokenKind::FatArrow) {
                        self.parse_expr()?
                    } else if self.peek() == &TokenKind::LBrace {
                        self.parse_block_expr()?
                    } else {
                        return Err(format!(
                            "Expected '=>' or '{{' for async arrow function body, found {:?}",
                            self.peek()
                        ));
                    };
                    Ok(Expr::Async(Box::new(Expr::Lambda {
                        params,
                        return_type,
                        body: Box::new(body),
                    })))
                } else if matches!(self.peek(), TokenKind::Ident(_))
                    && self.peek_next() == &TokenKind::FatArrow
                {
                    let name = self.expect_ident()?;
                    self.expect(TokenKind::FatArrow)?;
                    let body = self.parse_expr()?;
                    Ok(Expr::Async(Box::new(Expr::Lambda {
                        params: vec![Param {
                            name,
                            type_annotation: None,
                            default_value: None,
                        }],
                        return_type: None,
                        body: Box::new(body),
                    })))
                } else {
                    let body = self.parse_expr()?;
                    Ok(Expr::Async(Box::new(body)))
                }
            }
            TokenKind::While => {
                self.advance();
                let has_paren = self.match_token(&TokenKind::LParen);
                let condition = self.parse_expr()?;
                if has_paren {
                    self.expect(TokenKind::RParen)?;
                }
                let body = self.parse_block_expr()?;
                Ok(Expr::While {
                    label: None,
                    condition: Box::new(condition),
                    body: Box::new(body),
                })
            }
            TokenKind::For => {
                self.advance();
                let has_paren = self.match_token(&TokenKind::LParen);
                let mut index_name = None;
                let mut var_name = self.expect_ident()?;
                if self.match_token(&TokenKind::Comma) {
                    index_name = Some(var_name);
                    var_name = self.expect_ident()?;
                }
                let closed_paren_before_in = has_paren && self.match_token(&TokenKind::RParen);
                self.expect(TokenKind::In)?;
                let iterable = self.parse_expr()?;
                if has_paren && !closed_paren_before_in {
                    self.expect(TokenKind::RParen)?;
                }
                let body = self.parse_block_expr()?;
                Ok(Expr::ForIn {
                    label: None,
                    index_name,
                    var_name,
                    iterable: Box::new(iterable),
                    body: Box::new(body),
                })
            }
            TokenKind::Break => {
                self.advance();
                let label = if let TokenKind::Ident(lbl) = self.peek() {
                    let l = lbl.clone();
                    self.advance();
                    Some(l)
                } else {
                    None
                };
                Ok(Expr::Break(label))
            }
            TokenKind::Continue => {
                self.advance();
                let label = if let TokenKind::Ident(lbl) = self.peek() {
                    let l = lbl.clone();
                    self.advance();
                    Some(l)
                } else {
                    None
                };
                Ok(Expr::Continue(label))
            }
            TokenKind::Fn => self.parse_lambda(),
            TokenKind::Spawn | TokenKind::Routine | TokenKind::Go => {
                self.advance();
                let body = self.parse_expr()?;
                Ok(Expr::Spawn(Box::new(body)))
            }
            TokenKind::Select => self.parse_select_expr(),
            TokenKind::Panic => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let arg = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Panic(Box::new(arg)))
            }
            TokenKind::Recover => {
                self.advance();
                if self.match_token(&TokenKind::LParen) {
                    self.expect(TokenKind::RParen)?;
                }
                Ok(Expr::Recover)
            }
            other => Err(format!("Unexpected token in expression: {:?}", other)),
        }
    }

    fn is_record_key_token(kind: &TokenKind) -> bool {
        match kind {
            TokenKind::Ident(_)
            | TokenKind::String(_)
            | TokenKind::Type
            | TokenKind::From
            | TokenKind::When
            | TokenKind::Match
            | TokenKind::Module
            | TokenKind::Default
            | TokenKind::Spawn
            | TokenKind::Routine
            | TokenKind::Go
            | TokenKind::Select
            | TokenKind::While
            | TokenKind::For
            | TokenKind::In
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Underscore
            | TokenKind::Fn
            | TokenKind::Let
            | TokenKind::Mut
            | TokenKind::Async
            | TokenKind::Await
            | TokenKind::Import
            | TokenKind::Export
            | TokenKind::Extern
            | TokenKind::Trait
            | TokenKind::Impl
            | TokenKind::Return
            | TokenKind::True
            | TokenKind::False => true,
            _ => false,
        }
    }

    fn expect_record_key(&mut self) -> Result<String, String> {
        match self.advance() {
            TokenKind::Ident(s) => Ok(s),
            TokenKind::String(s) => Ok(format!(
                "\"{}\"",
                s.replace('\\', "\\\\").replace('"', "\\\"")
            )),
            TokenKind::Type => Ok("type".to_string()),
            TokenKind::From => Ok("from".to_string()),
            TokenKind::When => Ok("when".to_string()),
            TokenKind::Match => Ok("match".to_string()),
            TokenKind::Module => Ok("module".to_string()),
            TokenKind::Default => Ok("default".to_string()),
            TokenKind::Spawn => Ok("spawn".to_string()),
            TokenKind::Routine => Ok("routine".to_string()),
            TokenKind::Go => Ok("go".to_string()),
            TokenKind::Select => Ok("select".to_string()),
            TokenKind::While => Ok("while".to_string()),
            TokenKind::For => Ok("for".to_string()),
            TokenKind::In => Ok("in".to_string()),
            TokenKind::Break => Ok("break".to_string()),
            TokenKind::Continue => Ok("continue".to_string()),
            TokenKind::Underscore => Ok("_".to_string()),
            TokenKind::Fn => Ok("fn".to_string()),
            TokenKind::Let => Ok("let".to_string()),
            TokenKind::Mut => Ok("mut".to_string()),
            TokenKind::Async => Ok("async".to_string()),
            TokenKind::Await => Ok("await".to_string()),
            TokenKind::Import => Ok("import".to_string()),
            TokenKind::Export => Ok("export".to_string()),
            TokenKind::Extern => Ok("extern".to_string()),
            TokenKind::Trait => Ok("trait".to_string()),
            TokenKind::Impl => Ok("impl".to_string()),
            TokenKind::Return => Ok("return".to_string()),
            TokenKind::Defer => Ok("defer".to_string()),
            TokenKind::ErrDefer => Ok("errdefer".to_string()),
            TokenKind::Packed => Ok("packed".to_string()),
            TokenKind::Struct => Ok("struct".to_string()),
            TokenKind::True => Ok("true".to_string()),
            TokenKind::False => Ok("false".to_string()),
            other => Err(format!(
                "Expected record key or identifier, found {:?}",
                other
            )),
        }
    }

    fn parse_block_or_record(&mut self) -> Result<Expr, String> {
        self.expect(TokenKind::LBrace)?;

        if self.match_token(&TokenKind::RBrace) {
            return Ok(Expr::RecordLiteral {
                fields: Vec::new(),
                spread: None,
            });
        }

        if self.match_token(&TokenKind::Spread) {
            let spread_expr = self.parse_expr()?;
            let mut fields = Vec::new();
            if self.match_token(&TokenKind::Comma) {
                while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                    let fname = self.expect_record_key()?;
                    let fval = if self.match_token(&TokenKind::Colon) {
                        self.parse_expr()?
                    } else {
                        Expr::Identifier(fname.clone())
                    };
                    fields.push((fname, fval));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(Expr::RecordLiteral {
                fields,
                spread: Some(Box::new(spread_expr)),
            });
        }

        if Self::is_record_key_token(self.peek()) && self.peek_next() == &TokenKind::Colon {
            let mut fields = Vec::new();
            while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                let fname = self.expect_record_key()?;
                self.expect(TokenKind::Colon)?;
                let fval = self.parse_expr()?;
                fields.push((fname, fval));
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RBrace)?;
            Ok(Expr::RecordLiteral {
                fields,
                spread: None,
            })
        } else {
            let mut stmts = Vec::new();
            while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                stmts.push(self.parse_statement()?);
            }
            self.expect(TokenKind::RBrace)?;
            Ok(Expr::Block(stmts))
        }
    }

    fn parse_block_expr(&mut self) -> Result<Expr, String> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            stmts.push(self.parse_statement()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Block(stmts))
    }

    fn parse_if_expr(&mut self) -> Result<Expr, String> {
        self.expect(TokenKind::If)?;
        let condition = self.parse_expr()?;
        let then_branch = self.parse_block_expr()?;
        let else_branch = if self.match_token(&TokenKind::Else) {
            if self.peek() == &TokenKind::If {
                Some(Box::new(self.parse_if_expr()?))
            } else {
                Some(Box::new(self.parse_block_expr()?))
            }
        } else {
            None
        };

        Ok(Expr::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_match_expr(&mut self) -> Result<Expr, String> {
        self.expect(TokenKind::Match)?;
        let subject = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            let pattern = self.parse_pattern()?;
            let guard = if self.match_token(&TokenKind::When) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(TokenKind::FatArrow)?;
            let body = self.parse_expr()?;
            self.match_token(&TokenKind::Comma);

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });
        }
        self.expect(TokenKind::RBrace)?;

        Ok(Expr::Match {
            subject: Box::new(subject),
            arms,
        })
    }

    fn parse_select_expr(&mut self) -> Result<Expr, String> {
        self.expect(TokenKind::Select)?;
        self.expect(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        let mut default_arm = None;

        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            if self.match_token(&TokenKind::Default) {
                self.expect(TokenKind::FatArrow)?;
                let body = self.parse_expr()?;
                default_arm = Some(Box::new(body));
                self.match_token(&TokenKind::Comma);
            } else {
                if self.peek() == &TokenKind::Ident("case".to_string())
                    || self.peek() == &TokenKind::When
                {
                    self.advance();
                }

                let kind = if self.match_token(&TokenKind::ArrowLeft) {
                    let channel = self.parse_unary_with_opt(false)?;
                    SelectArmKind::Recv {
                        binding: None,
                        channel,
                    }
                } else if matches!(self.peek(), TokenKind::Ident(_))
                    && self.peek_next() == &TokenKind::Equal
                {
                    let binding = self.expect_ident()?;
                    self.expect(TokenKind::Equal)?;
                    self.expect(TokenKind::ArrowLeft)?;
                    let channel = self.parse_unary_with_opt(false)?;
                    SelectArmKind::Recv {
                        binding: Some(binding),
                        channel,
                    }
                } else {
                    let first = self.parse_unary_with_opt(false)?;
                    if self.match_token(&TokenKind::ArrowLeft) {
                        let value = self.parse_unary_with_opt(false)?;
                        SelectArmKind::Send {
                            channel: first,
                            value,
                        }
                    } else {
                        SelectArmKind::Timeout(first)
                    }
                };

                self.expect(TokenKind::FatArrow)?;
                let body = self.parse_expr()?;
                self.match_token(&TokenKind::Comma);

                arms.push(SelectArm { kind, body });
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Expr::Select {
            arms,
            default: default_arm,
        })
    }

    fn parse_lambda(&mut self) -> Result<Expr, String> {
        self.expect(TokenKind::Fn)?;
        if let TokenKind::Ident(_) = self.peek() {
            self.advance();
        }
        let params = self.parse_param_list()?;
        let mut return_type = None;
        if self.match_token(&TokenKind::Colon) || self.match_token(&TokenKind::Arrow) {
            return_type = Some(self.parse_type()?);
        }
        let body = if self.match_token(&TokenKind::FatArrow) {
            self.parse_expr()?
        } else if self.peek() == &TokenKind::LBrace {
            self.parse_block_expr()?
        } else {
            return Err(format!(
                "Expected '=>' or '{{' for lambda body, found {:?}",
                self.peek()
            ));
        };
        Ok(Expr::Lambda {
            params,
            return_type,
            body: Box::new(body),
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        match self.peek().clone() {
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::Int(n) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Int(n)))
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Float(n)))
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            TokenKind::LParen => {
                self.advance();
                let mut items = Vec::new();
                while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                    items.push(self.parse_pattern()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                Ok(Pattern::Tuple(items))
            }
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                let mut rest = None;
                while self.peek() != &TokenKind::RBracket && self.peek() != &TokenKind::Eof {
                    if self.match_token(&TokenKind::Spread) {
                        let rest_pat = self.parse_pattern()?;
                        rest = Some(Box::new(rest_pat));
                        break;
                    }
                    items.push(self.parse_pattern()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Pattern::List { items, rest })
            }
            TokenKind::Ident(name) => {
                self.advance();
                let is_uppercase = name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);

                if is_uppercase && self.peek() == &TokenKind::LParen {
                    self.advance();
                    let mut patterns = Vec::new();
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        patterns.push(self.parse_pattern()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    Ok(Pattern::Constructor { name, patterns })
                } else if is_uppercase && self.peek() == &TokenKind::LBrace {
                    self.advance();
                    let mut fields = Vec::new();
                    let mut has_rest = false;
                    while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                        if self.match_token(&TokenKind::DotDot) {
                            has_rest = true;
                            break;
                        }
                        let fname = self.expect_ident()?;
                        let fpat = if self.match_token(&TokenKind::Colon) {
                            self.parse_pattern()?
                        } else {
                            Pattern::Variable(fname.clone())
                        };
                        fields.push((fname, fpat));
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    Ok(Pattern::Record {
                        type_name: Some(name),
                        fields,
                        has_rest,
                    })
                } else if is_uppercase {
                    Ok(Pattern::Constructor {
                        name,
                        patterns: Vec::new(),
                    })
                } else {
                    Ok(Pattern::Variable(name))
                }
            }
            TokenKind::LBrace => {
                self.advance();
                let mut fields = Vec::new();
                let mut has_rest = false;
                while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                    if self.match_token(&TokenKind::DotDot) {
                        has_rest = true;
                        break;
                    }
                    let fname = self.expect_ident()?;
                    let fpat = if self.match_token(&TokenKind::Colon) {
                        self.parse_pattern()?
                    } else {
                        Pattern::Variable(fname.clone())
                    };
                    fields.push((fname, fpat));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Pattern::Record {
                    type_name: None,
                    fields,
                    has_rest,
                })
            }
            other => Err(format!("Unexpected token in pattern: {:?}", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_source(src: &str) -> Result<Module, String> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_module()
    }

    #[test]
    fn test_parse_functions() {
        let src = r#"
            export fn add(a: Int, b: Int = 0): Int {
                return a + b;
            }

            async fn fetch_user<T>(id: Int): T => api_get(id);
        "#;
        let module = parse_source(src).expect("failed to parse functions");
        assert_eq!(module.items.len(), 2);

        if let Item::Function(f) = &module.items[0] {
            assert_eq!(f.name, "add");
            assert!(f.is_exported);
            assert!(!f.is_async);
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name, "a");
            assert!(f.params[1].default_value.is_some());
        } else {
            panic!("Expected Function item");
        }

        if let Item::Function(f) = &module.items[1] {
            assert_eq!(f.name, "fetch_user");
            assert!(!f.is_exported);
            assert!(f.is_async);
            assert_eq!(f.type_params, vec!["T".to_string()]);
        } else {
            panic!("Expected Function item");
        }
    }

    #[test]
    fn test_parse_receiver_methods() {
        let src = r#"
            struct Person {
                name: String
            };

            fn (p: Person) greet(target: String): String => `Hello ${target}, I am ${p.name}`;

            fn (p: *Person) setName(newName: String): Unit => {
                p.name = newName;
            }
        "#;
        let module = parse_source(src).expect("failed to parse receiver methods");
        assert_eq!(module.items.len(), 3);

        if let Item::Function(f) = &module.items[1] {
            assert_eq!(f.name, "greet");
            assert!(f.receiver.is_some());
            let recv = f.receiver.as_ref().unwrap();
            assert_eq!(recv.name, "p");
            assert_eq!(
                recv.target_type,
                Type::Named {
                    name: "Person".to_string(),
                    type_args: vec![]
                }
            );
        } else {
            panic!("Expected greet method");
        }

        if let Item::Function(f) = &module.items[2] {
            assert_eq!(f.name, "setName");
            assert!(f.receiver.is_some());
            let recv = f.receiver.as_ref().unwrap();
            assert_eq!(recv.name, "p");
            assert_eq!(
                recv.target_type,
                Type::Pointer(Box::new(Type::Named {
                    name: "Person".to_string(),
                    type_args: vec![]
                }))
            );
        } else {
            panic!("Expected setName method");
        }
    }

    #[test]
    fn test_parse_sum_types_and_type_aliases() {
        let src = r#"
            type UserId = Int;

            export type Shape<T> =
                | Circle(Float)
                | Rectangle(Float, Float)
                | Point { x: Float, y: Float }
                | Empty;
        "#;
        let module = parse_source(src).expect("failed to parse types");
        assert_eq!(module.items.len(), 2);

        if let Item::TypeAlias(alias) = &module.items[0] {
            assert_eq!(alias.name, "UserId");
            assert_eq!(
                alias.target,
                Type::Named {
                    name: "Int".to_string(),
                    type_args: vec![],
                }
            );
        } else {
            panic!("Expected TypeAlias");
        }

        if let Item::SumType(sum) = &module.items[1] {
            assert_eq!(sum.name, "Shape");
            assert!(sum.is_exported);
            assert_eq!(sum.type_params, vec!["T".to_string()]);
            assert_eq!(sum.variants.len(), 4);
            assert_eq!(sum.variants[0].name, "Circle");
            assert_eq!(sum.variants[3].name, "Empty");
        } else {
            panic!("Expected SumType");
        }
    }

    #[test]
    fn test_parse_pattern_matching_and_when_guards() {
        let src = r#"
            fn eval(s: Shape): Float {
                match s {
                    Circle(r) when r > 0.0 => 3.14 * r * r,
                    Point { x, y } => x + y,
                    (a, b) => a + b,
                    _ => 0.0,
                }
            }
        "#;
        let module = parse_source(src).expect("failed to parse pattern matching");
        assert_eq!(module.items.len(), 1);

        if let Item::Function(f) = &module.items[0] {
            if let Expr::Block(stmts) = &f.body {
                if let Statement::Expr(Expr::Match { arms, .. }) = &stmts[0] {
                    assert_eq!(arms.len(), 4);
                    assert!(arms[0].guard.is_some());
                    assert!(arms[1].guard.is_none());
                    assert_eq!(arms[3].pattern, Pattern::Wildcard);
                } else {
                    panic!("Expected Match expr statement");
                }
            } else {
                panic!("Expected Block body");
            }
        } else {
            panic!("Expected Function");
        }
    }

    #[test]
    fn test_parse_pipeline_and_arrow_functions() {
        let src = r#"
            fn process(list: List<Int>): List<Int> {
                list
                |> filter((x) => x > 0)
                |> map((x: Int) -> Int => x * 2);
            }
        "#;
        let module = parse_source(src).expect("failed to parse pipeline");
        assert_eq!(module.items.len(), 1);

        if let Item::Function(f) = &module.items[0] {
            if let Expr::Block(stmts) = &f.body {
                if let Statement::Expr(Expr::Pipeline { .. }) = &stmts[0] {
                    // Successfully parsed nested pipeline
                } else {
                    panic!("Expected Pipeline expr");
                }
            }
        }
    }

    #[test]
    fn test_parse_csp_concurrency_constructs() {
        let src = r#"
            fn concurrent_flow() {
                let ch = Channel.make();
                spawn worker(ch);
                ch <- 42;
                let val = <-ch;

                select {
                    case msg = <-ch => println(msg),
                    case ch <- 100 => println("sent"),
                    default => println("timeout"),
                }
            }
        "#;
        let module = parse_source(src).expect("failed to parse CSP constructs");
        assert_eq!(module.items.len(), 1);
    }

    #[test]
    fn test_parse_imports_and_externs() {
        let src = r#"
            import { Router, Handler as handler } from "net/http";
            extern "math" {
                fn sin(x: Float): Float;
                fn cos(x: Float): Float;
            }
        "#;
        let module = parse_source(src).expect("failed to parse imports/externs");
        assert_eq!(module.items.len(), 2);

        if let Item::Import(imp) = &module.items[0] {
            assert_eq!(imp.source, "net/http");
            assert_eq!(imp.items.len(), 2);
            assert_eq!(imp.items[0].name, "Router");
            assert_eq!(imp.items[1].alias, Some("handler".to_string()));
        } else {
            panic!("Expected Import item");
        }

        if let Item::Extern(ext) = &module.items[1] {
            assert_eq!(ext.module_name, "math");
            assert_eq!(ext.functions.len(), 2);
        } else {
            panic!("Expected Extern item");
        }
    }

    #[test]
    fn test_parse_error_handling() {
        let src_unexpected = "fn 123() {}";
        assert!(parse_source(src_unexpected).is_err());

        let src_unmatched_brace = "fn test() { let x = 1;";
        assert!(parse_source(src_unmatched_brace).is_err());
    }
}
