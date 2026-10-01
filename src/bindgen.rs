//! C and Rust FFI Header Generator for Aura Language (`aurac bindgen`).
//! Automatically parses C header files (`.h`) and generates idiomatic, type-safe
//! Aura extern "C" declarations, packed C structs, enum sum types, and safe wrappers.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration options for Aura Bindgen.
#[derive(Debug, Clone)]
pub struct BindgenConfig {
    /// Path to the C header file to parse.
    pub input_header: String,
    /// Path to output `.aura` file (or None for stdout).
    pub output_file: Option<String>,
    /// Optional module name / prefix for generated bindings.
    pub module_name: Option<String>,
    /// Whether to generate ergonomic safe wrapper functions for string and pointer conversions.
    pub generate_wrappers: bool,
    /// Prefix to strip from C symbols (e.g. `sqlite3_` -> `open` instead of `sqlite3_open`).
    pub strip_prefix: Option<String>,
}

impl Default for BindgenConfig {
    fn default() -> Self {
        BindgenConfig {
            input_header: String::new(),
            output_file: None,
            module_name: None,
            generate_wrappers: true,
            strip_prefix: None,
        }
    }
}

/// Represents a parsed C function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CParam {
    pub name: String,
    pub c_type: String,
    pub aura_type: String,
}

/// Represents a parsed C function declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CFunction {
    pub name: String,
    pub return_c_type: String,
    pub return_aura_type: String,
    pub params: Vec<CParam>,
    pub is_variadic: bool,
}

/// Represents a field in a C struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CStructField {
    pub name: String,
    pub c_type: String,
    pub aura_type: String,
}

/// Represents a parsed C struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CStruct {
    pub name: String,
    pub fields: Vec<CStructField>,
}

/// Represents a parsed C enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CEnum {
    pub name: String,
    pub variants: Vec<(String, Option<i64>)>,
}

/// Represents a parsed C macro constant (`#define FOO 42`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CMacroConst {
    pub name: String,
    pub value: String,
    pub aura_type: String,
}

/// Result of parsing a C header file.
#[derive(Debug, Clone, Default)]
pub struct ParsedCHeader {
    pub functions: Vec<CFunction>,
    pub structs: Vec<CStruct>,
    pub enums: Vec<CEnum>,
    pub constants: Vec<CMacroConst>,
    pub typedefs: HashMap<String, String>,
}

/// Parser and code generator for C headers into Aura bindings.
pub struct AuraBindgen {
    config: BindgenConfig,
}

impl AuraBindgen {
    /// Creates a new AuraBindgen instance with the given configuration.
    pub fn new(config: BindgenConfig) -> Self {
        AuraBindgen { config }
    }

    /// Maps a C type string to its corresponding Aura type representation.
    pub fn map_c_type_to_aura(c_type: &str) -> String {
        let trimmed = c_type.trim();

        // Pointers
        if trimmed == "const char*" || trimmed == "char const*" {
            return "*const Char".to_string();
        }
        if trimmed == "char*" {
            return "*mut Char".to_string();
        }
        if trimmed == "void*" {
            return "*mut Unit".to_string();
        }
        if trimmed == "const void*" {
            return "*const Unit".to_string();
        }

        if let Some(inner) = trimmed.strip_prefix("const ")
            && let Some(base) = inner.strip_suffix('*')
        {
            return format!("*const {}", Self::map_c_type_to_aura(base.trim()));
        }

        if let Some(base) = trimmed.strip_suffix('*') {
            return format!("*mut {}", Self::map_c_type_to_aura(base.trim()));
        }

        // Primitive types
        match trimmed {
            "void" => "Unit".to_string(),
            "int" | "int32_t" | "int64_t" | "long" | "long long" | "short" | "int16_t"
            | "int8_t" | "signed int" | "signed" | "size_t" | "ssize_t" | "uintptr_t"
            | "intptr_t" => "Int".to_string(),
            "unsigned int" | "uint32_t" | "uint64_t" | "unsigned long" | "unsigned long long"
            | "unsigned short" | "uint16_t" | "uint8_t" | "unsigned char" => "Int".to_string(),
            "float" | "double" => "Float".to_string(),
            "char" | "signed char" => "Char".to_string(),
            "bool" | "_Bool" => "Bool".to_string(),
            other => {
                // If it starts with struct/enum, strip keyword
                let clean = other
                    .trim_start_matches("struct ")
                    .trim_start_matches("enum ")
                    .trim();
                clean.to_string()
            }
        }
    }

    /// Parses C header content into a structured representation.
    pub fn parse_header(&self, content: &str) -> ParsedCHeader {
        let mut header = ParsedCHeader::default();

        // 1. Remove block comments /* ... */ and line comments // ...
        let sanitized = self.strip_comments(content);

        // 2. Parse line by line for macros (#define) and filter preprocessor lines
        let mut non_preprocessor = String::with_capacity(sanitized.len());
        for line in sanitized.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#define") {
                if let Some(mc) = self.parse_macro_define(trimmed) {
                    header.constants.push(mc);
                }
            } else if !trimmed.starts_with('#') {
                non_preprocessor.push_str(line);
                non_preprocessor.push('\n');
            }
        }

        // 3. Tokenize/Parse declarations: structs, enums, functions from pure C declarations
        self.parse_declarations(&non_preprocessor, &mut header);

        header
    }

    /// Strips single-line and multi-line comments from C code.
    fn strip_comments(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '/' {
                if let Some(&'*') = chars.peek() {
                    chars.next(); // skip *
                    // in block comment
                    while let Some(bc) = chars.next() {
                        if bc == '*' && chars.peek() == Some(&'/') {
                            chars.next(); // skip /
                            break;
                        }
                    }
                    result.push(' ');
                    continue;
                } else if let Some(&'/') = chars.peek() {
                    chars.next(); // skip /
                    // in line comment
                    for lc in chars.by_ref() {
                        if lc == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                    continue;
                }
            }
            result.push(c);
        }

        result
    }

    /// Parses `#define NAME VALUE` numeric and string constants.
    fn parse_macro_define(&self, line: &str) -> Option<CMacroConst> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "#define" {
            let name = parts[1];
            // Discard function-like macros #define FOO(x)
            if name.contains('(') {
                return None;
            }

            let val_str = parts[2..].join(" ").trim().to_string();

            // Check if integer
            if val_str.parse::<i64>().is_ok() {
                return Some(CMacroConst {
                    name: name.to_string(),
                    value: val_str,
                    aura_type: "Int".to_string(),
                });
            }
            // Hex integer (e.g. 0x01)
            if (val_str.starts_with("0x") || val_str.starts_with("0X"))
                && let Ok(num) = i64::from_str_radix(
                    val_str
                        .trim_start_matches("0x")
                        .trim_start_matches("0X")
                        .trim_end_matches("U")
                        .trim_end_matches("L"),
                    16,
                )
            {
                return Some(CMacroConst {
                    name: name.to_string(),
                    value: num.to_string(),
                    aura_type: "Int".to_string(),
                });
            }
            // Check if float
            if val_str.parse::<f64>().is_ok() {
                return Some(CMacroConst {
                    name: name.to_string(),
                    value: val_str,
                    aura_type: "Float".to_string(),
                });
            }
            // String literal
            if val_str.starts_with('"') && val_str.ends_with('"') {
                return Some(CMacroConst {
                    name: name.to_string(),
                    value: val_str,
                    aura_type: "String".to_string(),
                });
            }
        }
        None
    }

    /// Parses structs, enums, and functions from sanitized C header text.
    fn parse_declarations(&self, content: &str, header: &mut ParsedCHeader) {
        let mut declarations = Vec::new();
        let mut current = String::new();
        let mut brace_depth = 0;

        for c in content.chars() {
            if c == '{' {
                brace_depth += 1;
            } else if c == '}' && brace_depth > 0 {
                brace_depth -= 1;
            }

            if c == ';' && brace_depth == 0 {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    declarations.push(trimmed.to_string());
                }
                current.clear();
            } else {
                current.push(c);
            }
        }
        let trimmed = current.trim();
        if !trimmed.is_empty() {
            declarations.push(trimmed.to_string());
        }

        for item in declarations {
            let item = item.trim();
            if item.is_empty() || item.starts_with('#') {
                continue;
            }

            // Parse enum
            if item.contains("enum ")
                && let Some(c_enum) = self.parse_enum(item)
            {
                header.enums.push(c_enum);
                continue;
            }

            // Parse struct
            if item.contains("struct ")
                && item.contains('{')
                && item.contains('}')
                && let Some(c_struct) = self.parse_struct(item)
            {
                header.structs.push(c_struct);
                continue;
            }

            // Parse function declaration
            if item.contains('(')
                && item.contains(')')
                && let Some(c_fn) = self.parse_function(item)
            {
                header.functions.push(c_fn);
            }
        }
    }

    /// Parses an enum declaration.
    fn parse_enum(&self, item: &str) -> Option<CEnum> {
        let brace_open = item.find('{')?;
        let brace_close = item.rfind('}')?;

        let body = &item[brace_open + 1..brace_close];
        let post = item[brace_close + 1..].trim();

        // Enum name: either after 'enum' or at end of typedef
        let enum_name = if !post.is_empty() {
            post.trim_end_matches(';').trim().to_string()
        } else {
            let pre = &item[..brace_open];
            let after_enum = pre.split("enum").nth(1)?.trim();
            after_enum.split_whitespace().next()?.to_string()
        };

        if enum_name.is_empty() {
            return None;
        }

        let mut variants = Vec::new();
        let mut current_val = 0i64;

        for part in body.split(',') {
            let p = part.trim();
            if p.is_empty() {
                continue;
            }

            if let Some((v_name, val_str)) = p.split_once('=') {
                let v_name = v_name.trim().to_string();
                if let Ok(val) = val_str.trim().parse::<i64>() {
                    current_val = val;
                    variants.push((v_name, Some(val)));
                    current_val += 1;
                } else {
                    variants.push((v_name, None));
                }
            } else {
                variants.push((p.to_string(), Some(current_val)));
                current_val += 1;
            }
        }

        Some(CEnum {
            name: enum_name,
            variants,
        })
    }

    /// Parses a struct declaration.
    fn parse_struct(&self, item: &str) -> Option<CStruct> {
        let brace_open = item.find('{')?;
        let brace_close = item.rfind('}')?;

        let body = &item[brace_open + 1..brace_close];
        let post = item[brace_close + 1..].trim();

        let struct_name = if !post.is_empty() {
            post.trim_end_matches(';').trim().to_string()
        } else {
            let pre = &item[..brace_open];
            let after_struct = pre.split("struct").nth(1)?.trim();
            after_struct.split_whitespace().next()?.to_string()
        };

        if struct_name.is_empty() {
            return None;
        }

        let mut fields = Vec::new();
        for field_line in body.split(';') {
            let f = field_line.trim();
            if f.is_empty() {
                continue;
            }

            let parts: Vec<&str> = f.split_whitespace().collect();
            if parts.len() >= 2 {
                let field_name_raw = parts.last().unwrap();
                let ptr_count = field_name_raw.chars().take_while(|&c| c == '*').count();
                let field_name = field_name_raw[ptr_count..].to_string();

                let type_parts = &parts[..parts.len() - 1];
                let mut c_type = type_parts.join(" ");
                for _ in 0..ptr_count {
                    c_type.push('*');
                }

                let aura_type = Self::map_c_type_to_aura(&c_type);
                fields.push(CStructField {
                    name: field_name,
                    c_type,
                    aura_type,
                });
            }
        }

        Some(CStruct {
            name: struct_name,
            fields,
        })
    }

    /// Parses a function prototype `int add(int a, int b)`.
    fn parse_function(&self, item: &str) -> Option<CFunction> {
        let paren_open = item.find('(')?;
        let paren_close = item.rfind(')')?;

        let pre = item[..paren_open].trim();
        let args_str = &item[paren_open + 1..paren_close].trim();

        // Clean out `extern` keyword
        let clean_pre = pre.strip_prefix("extern ").unwrap_or(pre).trim();

        let parts: Vec<&str> = clean_pre.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let fn_name_raw = parts.last().unwrap();
        let ret_ptr_count = fn_name_raw.chars().take_while(|&c| c == '*').count();
        let fn_name = fn_name_raw[ret_ptr_count..].to_string();

        let ret_parts = &parts[..parts.len() - 1];
        let mut ret_c_type = ret_parts.join(" ");
        for _ in 0..ret_ptr_count {
            ret_c_type.push('*');
        }

        let ret_aura_type = Self::map_c_type_to_aura(&ret_c_type);

        let mut params = Vec::new();
        let mut is_variadic = false;

        if !args_str.is_empty() && *args_str != "void" {
            for arg_part in args_str.split(',') {
                let ap = arg_part.trim();
                if ap == "..." {
                    is_variadic = true;
                    continue;
                }

                let p_parts: Vec<&str> = ap.split_whitespace().collect();
                if p_parts.len() >= 2 {
                    let p_name_raw = p_parts.last().unwrap();
                    let ptr_count = p_name_raw.chars().take_while(|&c| c == '*').count();
                    let p_name = p_name_raw[ptr_count..].to_string();

                    let t_parts = &p_parts[..p_parts.len() - 1];
                    let mut p_c_type = t_parts.join(" ");
                    for _ in 0..ptr_count {
                        p_c_type.push('*');
                    }

                    let p_aura_type = Self::map_c_type_to_aura(&p_c_type);
                    params.push(CParam {
                        name: p_name,
                        c_type: p_c_type,
                        aura_type: p_aura_type,
                    });
                } else if p_parts.len() == 1 {
                    let p_c_type = p_parts[0].to_string();
                    let p_aura_type = Self::map_c_type_to_aura(&p_c_type);
                    params.push(CParam {
                        name: format!("arg_{}", params.len()),
                        c_type: p_c_type,
                        aura_type: p_aura_type,
                    });
                }
            }
        }

        Some(CFunction {
            name: fn_name,
            return_c_type: ret_c_type,
            return_aura_type: ret_aura_type,
            params,
            is_variadic,
        })
    }

    /// Generates idiomatic Aura binding code from the parsed header.
    pub fn generate_aura_code(&self, header: &ParsedCHeader) -> String {
        let mut out = String::new();

        out.push_str(
            "// =============================================================================\n",
        );
        out.push_str("// Auto-generated by Aura Bindgen (`aurac bindgen`)\n");
        if let Some(ref mod_name) = self.config.module_name {
            out.push_str(&format!("// Module: {}\n", mod_name));
        }
        out.push_str(
            "// Standalone FFI Interoperability for C and Rust (Zero Runtime Dependencies)\n",
        );
        out.push_str(
            "// =============================================================================\n\n",
        );

        // 1. Constants
        if !header.constants.is_empty() {
            out.push_str("// --- Constants & Macro Definitions ---\n");
            for c in &header.constants {
                out.push_str(&format!(
                    "export let {}: {} = {};\n",
                    c.name, c.aura_type, c.value
                ));
            }
            out.push('\n');
        }

        // 2. Enums as Sum Types
        if !header.enums.is_empty() {
            out.push_str("// --- Enumerations (Algebraic Sum Types) ---\n");
            for e in &header.enums {
                out.push_str(&format!("export type {} =\n", e.name));
                for (v, _) in &e.variants {
                    out.push_str(&format!("    | {}\n", v));
                }
                out.push_str(";\n\n");
            }
        }

        // 3. Structs
        if !header.structs.is_empty() {
            out.push_str("// --- C Struct Layouts (Packed Memory) ---\n");
            for s in &header.structs {
                out.push_str(&format!("export type {} = {{\n", s.name));
                for f in &s.fields {
                    out.push_str(&format!("    {}: {},\n", f.name, f.aura_type));
                }
                out.push_str("};\n\n");
            }
        }

        // 4. Raw Foreign C Function Declarations
        if !header.functions.is_empty() {
            out.push_str("// --- Foreign C Function Declarations ---\n");
            for f in &header.functions {
                let params_str = f
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.aura_type))
                    .collect::<Vec<_>>()
                    .join(", ");

                out.push_str(&format!(
                    "export extern \"C\" fn {}({}): {};\n",
                    f.name, params_str, f.return_aura_type
                ));
            }
            out.push('\n');
        }

        // 5. Ergonomic Safe Wrappers
        if self.config.generate_wrappers {
            let string_fns: Vec<&CFunction> = header
                .functions
                .iter()
                .filter(|f| f.params.iter().any(|p| p.aura_type == "*const Char"))
                .collect();

            if !string_fns.is_empty() {
                out.push_str("// --- Ergonomic High-Level Safe Wrappers ---\n");
                for f in string_fns {
                    let wrapper_name = if let Some(ref pfx) = self.config.strip_prefix {
                        f.name.strip_prefix(pfx).unwrap_or(&f.name)
                    } else {
                        &f.name
                    };

                    let wrapper_fn_name = format!("safe_{}", wrapper_name);

                    let aura_params = f
                        .params
                        .iter()
                        .map(|p| {
                            if p.aura_type == "*const Char" {
                                format!("{}: String", p.name)
                            } else {
                                format!("{}: {}", p.name, p.aura_type)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    let call_args = f
                        .params
                        .iter()
                        .map(|p| {
                            if p.aura_type == "*const Char" {
                                format!("__c_str({})", p.name)
                            } else {
                                p.name.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    out.push_str(&format!(
                        "export fn {}({}): {} => {{\n",
                        wrapper_fn_name, aura_params, f.return_aura_type
                    ));
                    out.push_str(&format!("    {}({})\n", f.name, call_args));
                    out.push_str("}\n\n");
                }
            }
        }

        out
    }

    /// Executes the full bindgen pipeline and outputs to configured file or stdout.
    pub fn run(&self) -> Result<String, String> {
        let content = fs::read_to_string(&self.config.input_header).map_err(|e| {
            format!(
                "Failed to read C header file '{}': {}",
                self.config.input_header, e
            )
        })?;

        let header = self.parse_header(&content);
        let aura_code = self.generate_aura_code(&header);

        if let Some(ref out_path) = self.config.output_file {
            if let Some(parent) = Path::new(out_path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(out_path, &aura_code).map_err(|e| {
                format!("Failed to write generated Aura file '{}': {}", out_path, e)
            })?;
        }

        Ok(aura_code)
    }
}

/// CLI runner for `aurac bindgen` or standalone `aurabindgen`.
pub fn run_cli(args: &[String]) {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Aura Language C/Rust FFI Generator (`aurac bindgen`)");
        println!(
            "Usage: aurac bindgen <header.h> [-o <output.aura>] [--name <module>] [--strip-prefix <pfx>] [--no-wrappers]"
        );
        return;
    }

    let mut config = BindgenConfig::default();
    let mut i = 0;

    while i < args.len() {
        if args[i] == "-o" && i + 1 < args.len() {
            config.output_file = Some(args[i + 1].clone());
            i += 2;
        } else if (args[i] == "--name" || args[i] == "-n") && i + 1 < args.len() {
            config.module_name = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--strip-prefix" && i + 1 < args.len() {
            config.strip_prefix = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--no-wrappers" {
            config.generate_wrappers = false;
            i += 1;
        } else if config.input_header.is_empty() && !args[i].starts_with('-') {
            config.input_header = args[i].clone();
            i += 1;
        } else {
            i += 1;
        }
    }

    if config.input_header.is_empty() {
        eprintln!("Error: Missing input C header file.");
        eprintln!("Usage: aurac bindgen <header.h> [-o <output.aura>]");
        std::process::exit(1);
    }

    let bindgen = AuraBindgen::new(config.clone());
    match bindgen.run() {
        Ok(code) => {
            if let Some(ref out_path) = config.output_file {
                println!(
                    "✨ Successfully generated Aura FFI bindings in '{}' from '{}'",
                    out_path, config.input_header
                );
            } else {
                print!("{}", code);
            }
        }
        Err(e) => {
            eprintln!("✕ Bindgen error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_type_mapping() {
        assert_eq!(AuraBindgen::map_c_type_to_aura("int"), "Int");
        assert_eq!(AuraBindgen::map_c_type_to_aura("double"), "Float");
        assert_eq!(
            AuraBindgen::map_c_type_to_aura("const char*"),
            "*const Char"
        );
        assert_eq!(AuraBindgen::map_c_type_to_aura("void*"), "*mut Unit");
        assert_eq!(AuraBindgen::map_c_type_to_aura("Point*"), "*mut Point");
    }

    #[test]
    fn test_parse_c_header_functions_structs_enums() {
        let header_content = r#"
            #define SQLITE_OK 0
            #define MAX_TIMEOUT 5000

            typedef enum {
                STATUS_IDLE = 0,
                STATUS_ACTIVE = 1,
                STATUS_ERROR = 2
            } Status;

            typedef struct {
                int id;
                double balance;
            } Account;

            int sqlite3_open(const char *filename, void **ppDb);
            int sqlite3_close(void *pDb);
        "#;

        let config = BindgenConfig {
            input_header: "sqlite3.h".to_string(),
            output_file: None,
            module_name: Some("sqlite3".to_string()),
            generate_wrappers: true,
            strip_prefix: Some("sqlite3_".to_string()),
        };

        let bindgen = AuraBindgen::new(config);
        let parsed = bindgen.parse_header(header_content);

        assert_eq!(parsed.constants.len(), 2);
        assert_eq!(parsed.constants[0].name, "SQLITE_OK");
        assert_eq!(parsed.constants[0].value, "0");

        assert_eq!(parsed.enums.len(), 1);
        assert_eq!(parsed.enums[0].name, "Status");
        assert_eq!(parsed.enums[0].variants.len(), 3);

        assert_eq!(parsed.structs.len(), 1);
        assert_eq!(parsed.structs[0].name, "Account");
        assert_eq!(parsed.structs[0].fields.len(), 2);

        assert_eq!(parsed.functions.len(), 2);
        assert_eq!(parsed.functions[0].name, "sqlite3_open");
        assert_eq!(parsed.functions[0].params.len(), 2);

        let aura_code = bindgen.generate_aura_code(&parsed);
        assert!(aura_code.contains("export let SQLITE_OK: Int = 0;"));
        assert!(aura_code.contains("export type Status ="));
        assert!(aura_code.contains("export type Account = {"));
        assert!(aura_code.contains("export extern \"C\" fn sqlite3_open"));
        assert!(aura_code.contains("export fn safe_open"));
    }
}
