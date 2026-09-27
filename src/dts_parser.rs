//! Direct TypeScript Declaration (.d.ts) Parser and Type Environment Importer.
//! Enables seamless, zero-boilerplate consumption of npm packages in Aura.

use crate::typechecker::{ConcreteType, TypeEnv};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DtsModule {
    pub name: String,
    pub types: HashMap<String, ConcreteType>,
    pub functions: HashMap<String, ConcreteType>,
}

pub struct DtsParser<'a> {
    _input: &'a str,
    pos: usize,
    chars: Vec<char>,
}

impl<'a> DtsParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            _input: input,
            pos: 0,
            chars: input.chars().collect(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.chars.get(self.pos + 1) == Some(&'/') {
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else if ch == '/' && self.chars.get(self.pos + 1) == Some(&'*') {
                self.advance();
                self.advance();
                while let Some(c) = self.peek() {
                    if c == '*' && self.chars.get(self.pos + 1) == Some(&'/') {
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

    fn read_ident(&mut self) -> String {
        self.skip_whitespace_and_comments();
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn match_str(&mut self, s: &str) -> bool {
        self.skip_whitespace_and_comments();
        let len = s.chars().count();
        if self.pos + len <= self.chars.len() {
            let slice: String = self.chars[self.pos..self.pos + len].iter().collect();
            if slice == s {
                self.pos += len;
                return true;
            }
        }
        false
    }

    pub fn parse_dts(&mut self, module_name: &str) -> Result<DtsModule, String> {
        let mut types = HashMap::new();
        let mut functions = HashMap::new();

        while self.pos < self.chars.len() {
            self.skip_whitespace_and_comments();
            if self.pos >= self.chars.len() {
                break;
            }

            while self.match_str("export") || self.match_str("declare") || self.match_str("default")
            {
                self.skip_whitespace_and_comments();
            }

            let word = self.read_ident();
            match word.as_str() {
                "interface" => {
                    let name = self.read_ident();
                    if self.match_str("<") {
                        while let Some(c) = self.peek() {
                            self.advance();
                            if c == '>' {
                                break;
                            }
                        }
                    }
                    let record_ty = self.parse_interface_body()?;
                    types.insert(name, record_ty);
                }
                "type" => {
                    let name = self.read_ident();
                    if self.match_str("<") {
                        while let Some(c) = self.peek() {
                            self.advance();
                            if c == '>' {
                                break;
                            }
                        }
                    }
                    if self.match_str("=") {
                        let ty = self.parse_ts_type()?;
                        types.insert(name, ty);
                    }
                    self.match_str(";");
                }
                "function" => {
                    let name = self.read_ident();
                    if self.match_str("<") {
                        while let Some(c) = self.peek() {
                            self.advance();
                            if c == '>' {
                                break;
                            }
                        }
                    }
                    let fn_ty = self.parse_function_signature()?;
                    functions.insert(name, fn_ty);
                    self.match_str(";");
                }
                "const" | "let" => {
                    let name = self.read_ident();
                    if self.match_str(":") {
                        let ty = self.parse_ts_type()?;
                        functions.insert(name, ty);
                    }
                    self.match_str(";");
                }
                "" => {
                    self.advance();
                }
                _ => {
                    while let Some(c) = self.peek() {
                        self.advance();
                        if c == ';' || c == '\n' {
                            break;
                        }
                    }
                }
            }
        }

        Ok(DtsModule {
            name: module_name.to_string(),
            types,
            functions,
        })
    }

    fn parse_interface_body(&mut self) -> Result<ConcreteType, String> {
        self.skip_whitespace_and_comments();
        if !self.match_str("{") {
            return Err("Expected '{' in interface body".to_string());
        }

        let mut fields = HashMap::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.match_str("}") {
                break;
            }

            let field_name = self.read_ident();
            if field_name.is_empty() {
                self.advance();
                continue;
            }

            let is_optional = self.match_str("?");
            if !self.match_str(":") {
                return Err(format!("Expected ':' after field '{}'", field_name));
            }

            let mut field_ty = self.parse_ts_type()?;
            if is_optional {
                field_ty = ConcreteType::Option(Box::new(field_ty));
            }

            fields.insert(field_name, field_ty);
            self.match_str(";");
            self.match_str(",");
        }

        Ok(ConcreteType::Record(fields))
    }

    fn parse_function_signature(&mut self) -> Result<ConcreteType, String> {
        self.skip_whitespace_and_comments();
        if !self.match_str("(") {
            return Err("Expected '(' for function params".to_string());
        }

        let mut params = Vec::new();
        while !self.match_str(")") && self.pos < self.chars.len() {
            let param_name = self.read_ident();
            if param_name.is_empty() {
                break;
            }
            let is_optional = self.match_str("?");
            if self.match_str(":") {
                let mut pty = self.parse_ts_type()?;
                if is_optional {
                    pty = ConcreteType::Option(Box::new(pty));
                }
                params.push(pty);
            } else {
                params.push(ConcreteType::Any);
            }
            self.match_str(",");
        }

        let return_type = if self.match_str(":") {
            self.parse_ts_type()?
        } else {
            ConcreteType::Unit
        };

        Ok(ConcreteType::Function {
            params,
            return_type: Box::new(return_type),
        })
    }

    pub fn parse_ts_type(&mut self) -> Result<ConcreteType, String> {
        self.skip_whitespace_and_comments();

        let ident = self.read_ident();
        let mut base = match ident.as_str() {
            "string" => ConcreteType::String,
            "number" => ConcreteType::Float,
            "boolean" => ConcreteType::Bool,
            "void" | "undefined" | "null" => ConcreteType::Unit,
            "any" | "unknown" => ConcreteType::Any,
            "Promise" => {
                if self.match_str("<") {
                    let inner = self.parse_ts_type()?;
                    self.match_str(">");
                    ConcreteType::Task(
                        Box::new(inner),
                        Box::new(ConcreteType::TypeVar("Error".to_string())),
                    )
                } else {
                    ConcreteType::Task(
                        Box::new(ConcreteType::Any),
                        Box::new(ConcreteType::TypeVar("Error".to_string())),
                    )
                }
            }
            "Array" => {
                if self.match_str("<") {
                    let inner = self.parse_ts_type()?;
                    self.match_str(">");
                    ConcreteType::List(Box::new(inner))
                } else {
                    ConcreteType::List(Box::new(ConcreteType::Any))
                }
            }
            other if !other.is_empty() => {
                if self.match_str("<") {
                    let _inner = self.parse_ts_type()?;
                    self.match_str(">");
                }
                ConcreteType::TypeVar(other.to_string())
            }
            _ => ConcreteType::Any,
        };

        while self.match_str("[]") {
            base = ConcreteType::List(Box::new(base));
        }

        Ok(base)
    }

    pub fn import_into_env(dts_module: &DtsModule, env: &mut TypeEnv) {
        for (name, ty) in &dts_module.types {
            env.insert_type(name.clone(), ty.clone());
        }
        for (name, ty) in &dts_module.functions {
            env.insert_var(name.clone(), ty.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dts_interface() {
        let dts = r#"
            export interface User {
                id: number;
                name: string;
                age?: number;
                tags: string[];
            }
        "#;
        let mut parser = DtsParser::new(dts);
        let module = parser
            .parse_dts("user_mod")
            .expect("failed to parse interface");
        assert_eq!(module.name, "user_mod");
        assert!(module.types.contains_key("User"));

        if let Some(ConcreteType::Record(fields)) = module.types.get("User") {
            assert_eq!(fields.get("id"), Some(&ConcreteType::Float));
            assert_eq!(fields.get("name"), Some(&ConcreteType::String));
            assert_eq!(
                fields.get("age"),
                Some(&ConcreteType::Option(Box::new(ConcreteType::Float)))
            );
            assert_eq!(
                fields.get("tags"),
                Some(&ConcreteType::List(Box::new(ConcreteType::String)))
            );
        } else {
            panic!("Expected Record type for interface User");
        }
    }

    #[test]
    fn test_parse_dts_functions_and_promises() {
        let dts = r#"
            export declare function add(a: number, b: number): number;
            export function fetchData(url: string): Promise<string>;
            export const API_URL: string;
        "#;
        let mut parser = DtsParser::new(dts);
        let module = parser.parse_dts("api").expect("failed to parse functions");

        assert!(module.functions.contains_key("add"));
        assert!(module.functions.contains_key("fetchData"));
        assert!(module.functions.contains_key("API_URL"));

        if let Some(ConcreteType::Function {
            params,
            return_type,
        }) = module.functions.get("add")
        {
            assert_eq!(params, &vec![ConcreteType::Float, ConcreteType::Float]);
            assert_eq!(return_type.as_ref(), &ConcreteType::Float);
        } else {
            panic!("Expected Function for add");
        }

        if let Some(ConcreteType::Function {
            params,
            return_type,
        }) = module.functions.get("fetchData")
        {
            assert_eq!(params, &vec![ConcreteType::String]);
            assert_eq!(
                return_type.as_ref(),
                &ConcreteType::Task(
                    Box::new(ConcreteType::String),
                    Box::new(ConcreteType::TypeVar("Error".to_string()))
                )
            );
        } else {
            panic!("Expected Task return for fetchData");
        }

        assert_eq!(module.functions.get("API_URL"), Some(&ConcreteType::String));
    }

    #[test]
    fn test_parse_dts_type_aliases() {
        let dts = r#"
            export type ID = string;
            export type StringArray = Array<string>;
        "#;
        let mut parser = DtsParser::new(dts);
        let module = parser.parse_dts("types").expect("failed to parse types");

        assert_eq!(module.types.get("ID"), Some(&ConcreteType::String));
        assert_eq!(
            module.types.get("StringArray"),
            Some(&ConcreteType::List(Box::new(ConcreteType::String)))
        );
    }

    #[test]
    fn test_import_into_env() {
        let dts = r#"
            export interface Config {
                port: number;
            }
            export function startServer(cfg: Config): boolean;
        "#;
        let mut parser = DtsParser::new(dts);
        let dts_mod = parser.parse_dts("server").expect("parse server");
        let mut env = TypeEnv::new();
        DtsParser::import_into_env(&dts_mod, &mut env);

        assert!(env.lookup_type("Config").is_some());
        assert!(env.lookup_var("startServer").is_some());
    }
}
