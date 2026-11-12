//! Aura Language Compiler Core Library.

pub mod ast;
pub mod backend;
pub mod bindgen;
pub mod build_tags;
pub mod cli;
pub mod codegen;
pub mod codegen_cranelift;
pub mod codegen_go;
pub mod desugar;
pub mod dts_parser;
pub mod formatter;
pub mod fs_util;
pub mod lexer;
pub mod linter;
pub mod lsp;
pub mod package;
pub mod parser;
pub mod playground;
pub mod sourcemap;
pub mod testing;
pub mod typechecker;
pub mod vuln;
pub mod watcher;

/// The version of the Aura Language compiler and toolchain.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the current version of the Aura Language compiler.
pub fn version() -> &'static str {
    VERSION
}

pub use ast::{
    AstMetrics, AstMetricsCollector, AstVisitor, walk_expr, walk_item, walk_module, walk_pattern,
    walk_statement,
};
pub use backend::{
    BackendFactory, BackendKind, BackendOptions, BackendStrategy, ExecutableBuildResult,
    build_backend_executable, build_backend_executable_with_base_path,
    build_backend_executable_with_options, build_backend_executable_with_target, compile_backend,
    compile_backend_with_sourcemap, validate_backend_module,
};
pub use bindgen::{AuraBindgen, BindgenConfig};
use codegen::CodeGen;
pub use codegen_cranelift::{
    RuntimeProfile, build_native_binary, build_native_binary_with_base_path,
    build_native_binary_with_options, build_native_binary_with_target, detect_runtime_profile,
    find_or_build_runtime_lib, find_or_build_runtime_lib_for_profile,
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
pub use typechecker::{TypeChecker, TypeError};

/// Represents a resolved dependency module within the dependency graph.
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Canonical filesystem path of the module.
    pub path: PathBuf,
    /// Parsed AST representation of the module.
    pub module: ast::Module,
}

/// Dependency graph that collects and topologically sorts dependent modules
/// (dependencies/leaves first, dependents later) to be reused across
/// typechecking, bundling, and compilation.
#[derive(Debug, Default, Clone)]
pub struct DependencyGraph {
    /// Ordered list of modules in topological dependency order (post-order).
    pub modules: Vec<DependencyNode>,
}

impl DependencyGraph {
    /// Resolves and topologically sorts all dependencies imported by `root`.
    pub fn resolve(root: &ast::Module, base_path: Option<&Path>) -> Result<Self, String> {
        let mut visited = std::collections::HashSet::new();
        let mut in_progress = std::collections::HashSet::new();
        let mut modules = Vec::new();

        Self::resolve_recursive(
            root,
            base_path,
            &mut visited,
            &mut in_progress,
            &mut modules,
        )?;

        Ok(Self { modules })
    }

    fn resolve_import_path(imp_source: &str, base_path: Option<&Path>) -> Option<PathBuf> {
        fs_util::resolve_import_path(imp_source, base_path)
    }

    fn resolve_recursive(
        module: &ast::Module,
        base_path: Option<&Path>,
        visited: &mut std::collections::HashSet<PathBuf>,
        in_progress: &mut std::collections::HashSet<PathBuf>,
        modules: &mut Vec<DependencyNode>,
    ) -> Result<(), String> {
        for item in &module.items {
            if let ast::Item::Import(imp) = item {
                if let Some(target_file) = Self::resolve_import_path(&imp.source, base_path) {
                    let canonical = fs_util::canonicalize_path(&target_file);

                    if in_progress.contains(&canonical) {
                        return Err(format!(
                            "Circular dependency detected: '{}' is already being resolved",
                            canonical.display()
                        ));
                    }

                    if !visited.contains(&canonical) {
                        in_progress.insert(canonical.clone());
                        let cand_base = canonical.parent();

                        let dep_content = std::fs::read_to_string(&canonical).map_err(|e| {
                            format!("Failed to read dependency '{}': {}", canonical.display(), e)
                        })?;

                        let mut dep_lexer = Lexer::new(&dep_content);
                        let dep_tokens = dep_lexer.tokenize()?;
                        let mut dep_parser = Parser::new(dep_tokens);
                        let dep_module = dep_parser.parse_module()?;

                        Self::resolve_recursive(
                            &dep_module,
                            cand_base,
                            visited,
                            in_progress,
                            modules,
                        )?;

                        in_progress.remove(&canonical);
                        visited.insert(canonical.clone());

                        modules.push(DependencyNode {
                            path: canonical,
                            module: dep_module,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Populates a `TypeChecker` environment by typechecking all dependent modules in topological order.
    pub fn populate_typechecker(&self, typechecker: &mut TypeChecker) -> Result<(), TypeError> {
        for node in &self.modules {
            typechecker.check_module(&node.module)?;
        }
        Ok(())
    }

    /// Bundles all items from topologically sorted dependencies and the root module into a single AST `Module`.
    pub fn bundle(&self, root: &ast::Module) -> ast::Module {
        let mut bundled_items = Vec::new();

        for node in &self.modules {
            for item in &node.module.items {
                match item {
                    ast::Item::Import(_) => {}
                    ast::Item::Function(f) if f.name == "main" => {}
                    _ => bundled_items.push(item.clone()),
                }
            }
        }

        for item in &root.items {
            if !matches!(item, ast::Item::Import(_)) {
                bundled_items.push(item.clone());
            }
        }

        ast::Module {
            name: root.name.clone(),
            items: bundled_items,
        }
    }
}

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
#[derive(Default)]
pub struct CompilerPipeline<'a> {
    base_path: Option<&'a Path>,
    sourcemap_info: Option<(&'a str, &'a str)>,
    dts_inputs: Vec<(&'a str, &'a str)>,
    backend_strategy: Option<Box<dyn backend::BackendStrategy>>,
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

    /// Execute the full compiler pipeline: Lex -> Parse -> TypeCheck -> Desugar -> CodeGen
    pub fn execute(&self, source: &str) -> Result<CompilationResult, String> {
        let tokens = self.tokenize(source)?;
        let module = self.parse(tokens)?;
        self.typecheck(&module)?;
        let module = desugar::desugar_module(module);

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
            let mut codegen = CodeGen::with_base_path(self.base_path);
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
    let bundled = desugar::desugar_module(bundled);
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
    if let Ok(graph) = DependencyGraph::resolve(module, base_path) {
        let _ = graph.populate_typechecker(typechecker);
    }
}

/// Bundles a root module and all its recursively imported local .aura dependencies into a single AST Module.
pub fn bundle_module(
    module: &ast::Module,
    base_path: Option<&Path>,
) -> Result<ast::Module, String> {
    let graph = DependencyGraph::resolve(module, base_path)?;
    Ok(graph.bundle(module))
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
            if let Ok(graph) = DependencyGraph::resolve(&module, base_path) {
                for node in &graph.modules {
                    let cand = &node.path;
                    let cand_base = cand.parent();
                    if let Ok(content) = std::fs::read_to_string(cand)
                        && let Ok(res) = compile_backend(&content, cand_base, &[])
                    {
                        let target_mjs = if cand.extension().and_then(|s| s.to_str()) == Some("aura") {
                            cand.with_extension("mjs")
                        } else {
                            PathBuf::from(format!("{}.mjs", cand.display()))
                        };
                        if std::fs::write(&target_mjs, &res.js_code).is_ok() {
                            compiled_files.push(target_mjs);
                        }
                    }
                }
            }
        }
    }
    compiled_files
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

    #[test]
    fn test_dependency_graph_topological_sort_and_bundle() {
        use std::fs;

        let temp_dir = std::env::temp_dir().join(format!(
            "aura_depgraph_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        // c.aura (base dependency)
        let c_path = temp_dir.join("c.aura");
        fs::write(&c_path, "export fn base_c(): Int => 42;\n").unwrap();

        // b.aura (imports c)
        let b_path = temp_dir.join("b.aura");
        fs::write(
            &b_path,
            "import { base_c } from \"./c.aura\";\nexport fn helper_b(): Int => base_c() + 1;\n",
        )
        .unwrap();

        // a.aura (imports b and c)
        let a_path = temp_dir.join("a.aura");
        fs::write(
            &a_path,
            "import { helper_b } from \"./b.aura\";\nimport { base_c } from \"./c.aura\";\nexport fn main(): Int => helper_b() + base_c();\n",
        )
        .unwrap();

        let a_content = fs::read_to_string(&a_path).unwrap();
        let tokens = Lexer::new(&a_content).tokenize().unwrap();
        let root_module = Parser::new(tokens).parse_module().unwrap();

        let graph = DependencyGraph::resolve(&root_module, Some(&temp_dir))
            .expect("Resolution must succeed");

        // Topological order check: c.aura must precede b.aura
        assert_eq!(graph.modules.len(), 2);
        let c_canon = c_path.canonicalize().unwrap();
        let b_canon = b_path.canonicalize().unwrap();
        assert_eq!(graph.modules[0].path, c_canon);
        assert_eq!(graph.modules[1].path, b_canon);

        // Bundle check
        let bundled = graph.bundle(&root_module);
        let fn_names: Vec<&str> = bundled
            .items
            .iter()
            .filter_map(|it| match it {
                ast::Item::Function(f) => Some(f.name.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(fn_names, vec!["base_c", "helper_b", "main"]);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_dependency_graph_circular_detection() {
        use std::fs;

        let temp_dir = std::env::temp_dir().join(format!(
            "aura_depgraph_cycle_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let m1_path = temp_dir.join("m1.aura");
        let m2_path = temp_dir.join("m2.aura");

        fs::write(&m1_path, "import { f2 } from \"./m2.aura\";\nexport fn f1(): Int => f2();\n").unwrap();
        fs::write(&m2_path, "import { f1 } from \"./m1.aura\";\nexport fn f2(): Int => f1();\n").unwrap();

        let m1_content = fs::read_to_string(&m1_path).unwrap();
        let tokens = Lexer::new(&m1_content).tokenize().unwrap();
        let root_module = Parser::new(tokens).parse_module().unwrap();

        let res = DependencyGraph::resolve(&root_module, Some(&temp_dir));
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Circular dependency detected"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
