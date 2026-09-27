//! Aura Language Compiler Core Library.

pub mod ast;
pub mod backend;
pub mod bindgen;
pub mod build_tags;
pub mod codegen;
pub mod codegen_cranelift;
pub mod codegen_go;
pub mod dts_parser;
pub mod formatter;
pub mod lexer;
pub mod lsp;
pub mod package;
pub mod parser;
pub mod playground;
pub mod sourcemap;
pub mod testing;
pub mod typechecker;
pub mod watcher;

pub use ast::{
    AstMetrics, AstMetricsCollector, AstVisitor, walk_expr, walk_item, walk_module, walk_pattern,
    walk_statement,
};
pub use backend::{
    BackendFactory, BackendKind, BackendOptions, BackendStrategy, ExecutableBuildResult,
    build_backend_executable, build_backend_executable_with_base_path,
    build_backend_executable_with_target, compile_backend, compile_backend_with_sourcemap,
    validate_backend_module,
};
pub use bindgen::{AuraBindgen, BindgenConfig};
use codegen::CodeGen;
pub use codegen_cranelift::{
    build_native_binary, build_native_binary_with_base_path, build_native_binary_with_options,
    build_native_binary_with_target,
};
use dts_parser::DtsParser;
pub use formatter::{
    FormatConfig, FormatError, FormatResult, IndentStyle, Language, format_aura, format_file,
    format_js, format_source, format_ts, generate_unified_diff,
};
use lexer::Lexer;
pub use package::{
    LockEntry, LockFile, ModuleFile, PackageInfo, PackageManager, VerificationStatus,
    resolve_package_import,
};
use parser::Parser;
pub use testing::{
    BenchmarkResult, CoverageReport, FileCoverage, TestCaseResult, TestConfig, TestFile,
    TestFunction, TestKind, TestStatus, TestSummary, discover_test_files, run_tests,
};
use typechecker::TypeChecker;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub js_code: String,
    pub dts_code: String,
    pub source_map: Option<String>,
}

// -----------------------------------------------------------------------------
// DESIGN PATTERN: PIPELINE & BUILDER PATTERN FOR COMPILATION
// -----------------------------------------------------------------------------

/// Pipeline & Builder pattern orchestrating multi-phase Aura compilation:
/// 1. Tokenization (`Lexer`)
/// 2. Syntactic Parsing (`Parser`)
/// 3. Static Type Checking & Module Resolution (`TypeChecker`)
/// 4. Code Generation (`CodeGen` / `BackendStrategy`)
pub struct CompilerPipeline<'a> {
    base_path: Option<&'a Path>,
    sourcemap_info: Option<(&'a str, &'a str)>,
    dts_inputs: Vec<(&'a str, &'a str)>,
    backend_strategy: Option<Box<dyn backend::BackendStrategy>>,
}

impl<'a> Default for CompilerPipeline<'a> {
    fn default() -> Self {
        Self {
            base_path: None,
            sourcemap_info: None,
            dts_inputs: Vec::new(),
            backend_strategy: None,
        }
    }
}

impl<'a> CompilerPipeline<'a> {
    /// Creates a new default `CompilerPipeline` builder.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the base filesystem path for resolving relative module imports (`./`, `../`).
    /// - `base_path`: Directory path containing the root source file.
    pub fn with_base_path(mut self, base_path: Option<&'a Path>) -> Self {
        self.base_path = base_path;
        self
    }

    /// Configures the pipeline to generate Source Maps V3 for step-through debugging.
    /// - `js_filename`: Target JavaScript output file name.
    /// - `aura_filename`: Original Aura input file name.
    pub fn with_sourcemap(mut self, js_filename: &'a str, aura_filename: &'a str) -> Self {
        self.sourcemap_info = Some((js_filename, aura_filename));
        self
    }

    /// Ingests TypeScript declaration files (`.d.ts`) to provide type checking for foreign modules.
    /// - `dts_inputs`: Slice of `(module_name, dts_content)` pairs.
    pub fn with_dts_inputs(mut self, dts_inputs: &[(&'a str, &'a str)]) -> Self {
        self.dts_inputs = dts_inputs.to_vec();
        self
    }

    /// Configures an explicit backend strategy (Cranelift native, Go native, or JS module).
    /// - `strategy`: Boxed `BackendStrategy` instance.
    pub fn with_backend(mut self, strategy: Box<dyn backend::BackendStrategy>) -> Self {
        self.backend_strategy = Some(strategy);
        self
    }

    /// Phase 1: Lexical Analysis
    pub fn tokenize(&self, source: &str) -> Result<Vec<lexer::Token>, String> {
        let mut lexer = Lexer::new(source);
        lexer.tokenize()
    }

    /// Phase 2: Syntax Parsing
    pub fn parse(&self, tokens: Vec<lexer::Token>) -> Result<ast::Module, String> {
        let mut parser = Parser::new(tokens);
        parser.parse_module()
    }

    /// Phase 3: Type Checking & Dependency Resolution
    pub fn typecheck(&self, module: &ast::Module) -> Result<TypeChecker, String> {
        let mut typechecker = TypeChecker::new();

        for (mod_name, dts_content) in &self.dts_inputs {
            let mut dts_parser = DtsParser::new(dts_content);
            let dts_mod = dts_parser.parse_dts(mod_name)?;
            DtsParser::import_into_env(&dts_mod, typechecker.env_mut());
        }

        resolve_local_imports(module, self.base_path, &mut typechecker);
        typechecker.check_module(module)?;
        Ok(typechecker)
    }

    /// Execute the full compiler pipeline: Lex -> Parse -> TypeCheck -> CodeGen
    pub fn execute(&self, source: &str) -> Result<CompilationResult, String> {
        let tokens = self.tokenize(source)?;
        let module = self.parse(tokens)?;
        self.typecheck(&module)?;

        if let Some(ref backend) = self.backend_strategy {
            let options = backend::BackendOptions {
                base_path: self.base_path,
                target_triple: None,
                dts_inputs: &self.dts_inputs,
                sourcemap: self.sourcemap_info.is_some(),
                js_filename: self.sourcemap_info.map(|(j, _)| j),
                aura_filename: self.sourcemap_info.map(|(_, a)| a),
            };
            backend.compile_module(&module, &options)
        } else if let Some((js_filename, aura_filename)) = self.sourcemap_info {
            let mut codegen = CodeGen::new();
            let (js_code, dts_code, source_map) =
                codegen.generate_with_sourcemap(&module, js_filename, aura_filename, source);
            Ok(CompilationResult {
                js_code,
                dts_code,
                source_map: Some(source_map),
            })
        } else {
            let mut codegen = CodeGen::with_base_path(self.base_path);
            let (js_code, dts_code) = codegen.generate(&module);
            Ok(CompilationResult {
                js_code,
                dts_code,
                source_map: None,
            })
        }
    }
}

/// Compiles an Aura source string into Golang source code.
pub fn compile_to_go(source: &str) -> Result<String, String> {
    compile_to_go_with_base_path(source, None)
}

/// Compiles an Aura source string into Golang source code, resolving local imports relative to base_path.
pub fn compile_to_go_with_base_path(
    source: &str,
    base_path: Option<&Path>,
) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module()?;
    let bundled = bundle_module(&module, base_path)?;
    backend::validate_backend_module(&bundled)?;
    let mut go_codegen = codegen_go::GoCodeGen::new();
    Ok(go_codegen.generate(&bundled))
}

/// Compiles an Aura source code string into backend executable representation.
/// Optionally ingests external TypeScript declaration (.d.ts) files for npm interop.
pub fn compile(source: &str, dts_inputs: &[(&str, &str)]) -> Result<CompilationResult, String> {
    CompilerPipeline::builder()
        .with_dts_inputs(dts_inputs)
        .execute(source)
}

/// Compiles an Aura source code string with Source Maps V3 generation.
pub fn compile_with_sourcemap(
    source: &str,
    js_filename: &str,
    aura_filename: &str,
    base_path: Option<&Path>,
    dts_inputs: &[(&str, &str)],
) -> Result<CompilationResult, String> {
    CompilerPipeline::builder()
        .with_base_path(base_path)
        .with_sourcemap(js_filename, aura_filename)
        .with_dts_inputs(dts_inputs)
        .execute(source)
}

pub fn resolve_local_imports(
    module: &ast::Module,
    base_path: Option<&Path>,
    typechecker: &mut TypeChecker,
) {
    let mut imported_sources = std::collections::HashSet::new();
    resolve_local_imports_recursive(module, base_path, typechecker, &mut imported_sources);
}

fn resolve_local_imports_recursive(
    module: &ast::Module,
    base_path: Option<&Path>,
    typechecker: &mut TypeChecker,
    imported_sources: &mut std::collections::HashSet<PathBuf>,
) {
    for item in &module.items {
        if let ast::Item::Import(imp) = item {
            let target_file = if imp.source.starts_with("./") || imp.source.starts_with("../") {
                let dep_file = if let Some(base) = base_path {
                    base.join(&imp.source)
                } else {
                    PathBuf::from(&imp.source)
                };
                let candidates = [dep_file.clone(), dep_file.with_extension("aura")];
                candidates.into_iter().find(|c| c.exists())
            } else {
                package::resolve_package_import(&imp.source, base_path)
            };

            if let Some(cand) = target_file {
                if cand.exists() && !imported_sources.contains(&cand) {
                    imported_sources.insert(cand.clone());
                    let cand_base = cand.parent();
                    if let Ok(dep_content) = std::fs::read_to_string(&cand) {
                        let mut dep_lexer = Lexer::new(&dep_content);
                        if let Ok(dep_tokens) = dep_lexer.tokenize() {
                            let mut dep_parser = Parser::new(dep_tokens);
                            if let Ok(dep_module) = dep_parser.parse_module() {
                                resolve_local_imports_recursive(
                                    &dep_module,
                                    cand_base,
                                    typechecker,
                                    imported_sources,
                                );
                                let _ = typechecker.check_module(&dep_module);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Bundles a root module and all its recursively imported local .aura dependencies into a single AST Module.
pub fn bundle_module(
    module: &ast::Module,
    base_path: Option<&Path>,
) -> Result<ast::Module, String> {
    let mut visited = std::collections::HashSet::new();
    let mut bundled_items = Vec::new();

    bundle_module_recursive(module, base_path, &mut visited, &mut bundled_items)?;

    for item in &module.items {
        if !matches!(item, ast::Item::Import(_)) {
            bundled_items.push(item.clone());
        }
    }

    Ok(ast::Module {
        name: module.name.clone(),
        items: bundled_items,
    })
}

fn bundle_module_recursive(
    module: &ast::Module,
    base_path: Option<&Path>,
    visited: &mut std::collections::HashSet<PathBuf>,
    bundled_items: &mut Vec<ast::Item>,
) -> Result<(), String> {
    for item in &module.items {
        if let ast::Item::Import(imp) = item {
            let target_file = if imp.source.starts_with("./") || imp.source.starts_with("../") {
                let dep_file = if let Some(base) = base_path {
                    base.join(&imp.source)
                } else {
                    PathBuf::from(&imp.source)
                };
                let candidates = [dep_file.clone(), dep_file.with_extension("aura")];
                candidates.into_iter().find(|c| c.exists() && c.is_file())
            } else {
                package::resolve_package_import(&imp.source, base_path)
            };

            if let Some(dep_path) = target_file {
                let canonical = dep_path.canonicalize().unwrap_or_else(|_| dep_path.clone());
                if !visited.contains(&canonical) {
                    visited.insert(canonical.clone());
                    let dep_base = dep_path.parent();
                    if let Ok(dep_content) = std::fs::read_to_string(&dep_path) {
                        let mut dep_lexer = Lexer::new(&dep_content);
                        if let Ok(dep_tokens) = dep_lexer.tokenize() {
                            let mut dep_parser = Parser::new(dep_tokens);
                            if let Ok(dep_module) = dep_parser.parse_module() {
                                bundle_module_recursive(
                                    &dep_module,
                                    dep_base,
                                    visited,
                                    bundled_items,
                                )?;

                                for dep_item in dep_module.items {
                                    match &dep_item {
                                        ast::Item::Import(_) => {}
                                        ast::Item::Function(f) if f.name == "main" => {}
                                        _ => {
                                            bundled_items.push(dep_item);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn compile_local_deps_to_mjs_from_source(
    source: &str,
    base_path: Option<&Path>,
) -> Vec<PathBuf> {
    let mut compiled_files = Vec::new();
    let mut lexer = Lexer::new(source);
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        if let Ok(module) = parser.parse_module() {
            let mut visited = std::collections::HashSet::new();
            compile_local_deps_recursive(&module, base_path, &mut visited, &mut compiled_files);
        }
    }
    compiled_files
}

fn compile_local_deps_recursive(
    module: &ast::Module,
    base_path: Option<&Path>,
    visited: &mut std::collections::HashSet<PathBuf>,
    compiled_files: &mut Vec<PathBuf>,
) {
    for item in &module.items {
        if let ast::Item::Import(imp) = item {
            let target_file = if imp.source.starts_with("./") || imp.source.starts_with("../") {
                let dep_file = if let Some(base) = base_path {
                    base.join(&imp.source)
                } else {
                    PathBuf::from(&imp.source)
                };
                let candidates = [dep_file.clone(), dep_file.with_extension("aura")];
                candidates.into_iter().find(|c| c.exists() && c.is_file())
            } else {
                package::resolve_package_import(&imp.source, base_path)
            };

            if let Some(cand) = target_file {
                if cand.exists() && cand.is_file() && !visited.contains(&cand) {
                    visited.insert(cand.clone());
                    let cand_base = cand.parent();
                    if let Ok(content) = std::fs::read_to_string(&cand) {
                        if let Ok(res) = compile_backend(&content, cand_base, &[]) {
                            let target_mjs =
                                if cand.extension().and_then(|s| s.to_str()) == Some("aura") {
                                    cand.with_extension("mjs")
                                } else {
                                    PathBuf::from(format!("{}.mjs", cand.display()))
                                };
                            if std::fs::write(&target_mjs, &res.js_code).is_ok() {
                                compiled_files.push(target_mjs);
                            }
                            let mut dep_lexer = Lexer::new(&content);
                            if let Ok(dep_tokens) = dep_lexer.tokenize() {
                                let mut dep_parser = Parser::new(dep_tokens);
                                if let Ok(dep_module) = dep_parser.parse_module() {
                                    compile_local_deps_recursive(
                                        &dep_module,
                                        cand_base,
                                        visited,
                                        compiled_files,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Compiles an Aura source code string resolving local .aura module imports relative to base_path.

pub fn compile_with_base_path(
    source: &str,
    base_path: Option<&Path>,
    dts_inputs: &[(&str, &str)],
) -> Result<CompilationResult, String> {
    CompilerPipeline::builder()
        .with_base_path(base_path)
        .with_dts_inputs(dts_inputs)
        .execute(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_basic_module() {
        let src = r#"
            export fn greet(name: String): String => `Hello, ${name}!`;
        "#;
        let res = compile(src, &[]).expect("Compilation failed");
        assert!(res.js_code.contains("export function greet(name)"));
        assert!(
            res.dts_code
                .contains("export declare function greet(name: string): string;")
        );
    }

    #[test]
    fn test_compile_with_dts_ingestion() {
        let dts = r#"
            export interface ApiConfig {
                endpoint: string;
            }
            export declare function initApi(cfg: ApiConfig): boolean;
        "#;
        let src = r#"
            export fn setup(url: String): Bool {
                return initApi({ endpoint: url });
            }
        "#;
        let res = compile(src, &[("my-api", dts)]).expect("Compilation with DTS failed");
        assert!(res.js_code.contains("initApi("));
        assert!(res.js_code.contains("endpoint"));
    }

    #[test]
    fn test_compile_error_propagation() {
        // Lexer error
        assert!(compile("let @invalid = 1;", &[]).is_err());

        // Parser error
        assert!(compile("fn unclosed() {", &[]).is_err());

        // Typechecker error
        assert!(compile("fn wrong(): Int => \"not an int\";", &[]).is_err());
    }

    #[test]
    fn test_compiler_pipeline_builder() {
        let src = r#"
            export fn add(x: Int, y: Int): Int => x + y;
        "#;
        let pipeline =
            CompilerPipeline::builder().with_backend(backend::BackendFactory::create("js"));
        let res = pipeline.execute(src).expect("Pipeline execution failed");
        assert!(res.js_code.contains("export function add(x, y)"));
    }
}
