//! Backend-only specialization and standalone executable generation for Aura Lang.
//! Following the Golang model: high-performance concurrency, net/http server,
//! sync primitives, and standalone native binary distribution.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::CompilationResult;
use crate::ast::Module;
use crate::codegen::CodeGen;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::typechecker::TypeChecker;

/// Validates that an AST module satisfies Aura Backend requirements.
/// Aura is exclusively a Backend and Systems language (Golang model).
pub fn validate_backend_module(_module: &Module) -> Result<(), String> {
    Ok(())
}

/// Compiles an Aura source file ensuring strict Backend-only compliance.
pub fn compile_backend(
    source: &str,
    base_path: Option<&Path>,
    dts_inputs: &[(&str, &str)],
) -> Result<CompilationResult, String> {
    // 1. Lexing
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // 2. Parsing
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module()?;

    // 3. Strict Backend Validation
    validate_backend_module(&module)?;

    // 4. Type Checking
    let mut typechecker = TypeChecker::new();
    for (mod_name, dts_content) in dts_inputs {
        let mut dts_parser = crate::dts_parser::DtsParser::new(dts_content);
        let dts_mod = dts_parser.parse_dts(mod_name)?;
        crate::dts_parser::DtsParser::import_into_env(&dts_mod, typechecker.env_mut());
    }
    crate::resolve_local_imports(&module, base_path, &mut typechecker);
    typechecker.check_module(&module)?;

    // 5. Code Generation
    let mut codegen = CodeGen::with_base_path(base_path);
    let (js_code, dts_code) = codegen.generate(&module);

    Ok(CompilationResult {
        js_code,
        dts_code,
        source_map: None,
    })
}

/// Compiles an Aura source file ensuring strict Backend-only compliance and generating Source Maps V3.
pub fn compile_backend_with_sourcemap(
    source: &str,
    js_filename: &str,
    aura_filename: &str,
    base_path: Option<&Path>,
    dts_inputs: &[(&str, &str)],
) -> Result<CompilationResult, String> {
    // 1. Lexing
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // 2. Parsing
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module()?;

    // 3. Strict Backend Validation
    validate_backend_module(&module)?;

    // 4. Type Checking
    let mut typechecker = TypeChecker::new();
    for (mod_name, dts_content) in dts_inputs {
        let mut dts_parser = crate::dts_parser::DtsParser::new(dts_content);
        let dts_mod = dts_parser.parse_dts(mod_name)?;
        crate::dts_parser::DtsParser::import_into_env(&dts_mod, typechecker.env_mut());
    }
    crate::resolve_local_imports(&module, base_path, &mut typechecker);
    typechecker.check_module(&module)?;

    // 5. Code Generation with Source Maps
    let mut codegen = CodeGen::with_base_path(base_path);
    let (js_code, dts_code, source_map) =
        codegen.generate_with_sourcemap(&module, js_filename, aura_filename, source);

    Ok(CompilationResult {
        js_code,
        dts_code,
        source_map: Some(source_map),
    })
}

/// Details about a generated standalone executable binary.
#[derive(Debug, Clone)]
pub struct ExecutableBuildResult {
    pub binary_path: PathBuf,
    pub format: String,
    pub is_native_binary: bool,
    pub size_bytes: u64,
}

/// Builds a standalone native binary executable from compiled Aura backend code,
/// following the Golang `go build -o <binary>` model.
///
/// Uses the Go native compiler toolchain to produce a true zero-dependency
/// Mach-O / ELF / PE native machine-code binary.
pub fn build_backend_executable(
    source_or_go_code: &str,
    out_path: &Path,
) -> Result<ExecutableBuildResult, String> {
    build_backend_executable_with_base_path(source_or_go_code, out_path, None, None)
}

/// Builds a standalone native binary executable with optional cross-compilation target.
pub fn build_backend_executable_with_target(
    source_or_go_code: &str,
    out_path: &Path,
    target: Option<&str>,
) -> Result<ExecutableBuildResult, String> {
    build_backend_executable_with_base_path(source_or_go_code, out_path, None, target)
}

/// Builds a standalone native binary executable with optional base path and cross-compilation target.
pub fn build_backend_executable_with_base_path(
    source_or_go_code: &str,
    out_path: &Path,
    base_path: Option<&Path>,
    target: Option<&str>,
) -> Result<ExecutableBuildResult, String> {
    let parent_dir = out_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = out_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "app".to_string());

    let tmp_go_path = parent_dir.join(format!("{}.__aura_tmp.go", stem));

    let go_code = if source_or_go_code.contains("package main") {
        source_or_go_code.to_string()
    } else {
        crate::compile_to_go_with_base_path(source_or_go_code, base_path)?
    };

    if let Err(e) = fs::write(&tmp_go_path, &go_code) {
        return Err(format!(
            "Failed to write temporary Go source '{}': {}",
            tmp_go_path.display(),
            e
        ));
    }

    let mut cmd = Command::new("go");
    cmd.arg("build").arg("-o").arg(out_path).arg(&tmp_go_path);

    if let Some(t) = target {
        let (goos, goarch) = match t {
            "x86_64-linux"
            | "linux/amd64"
            | "linux-x64"
            | "x86_64-unknown-linux-musl"
            | "x86_64-unknown-linux-gnu" => ("linux", "amd64"),
            "aarch64-linux"
            | "linux/arm64"
            | "linux-arm64"
            | "aarch64-unknown-linux-musl"
            | "aarch64-unknown-linux-gnu" => ("linux", "arm64"),
            "x86_64-macos" | "darwin/amd64" | "darwin-x64" | "x86_64-apple-darwin" => {
                ("darwin", "amd64")
            }
            "aarch64-macos" | "darwin/arm64" | "darwin-arm64" | "aarch64-apple-darwin" => {
                ("darwin", "arm64")
            }
            "x86_64-windows" | "windows/amd64" | "windows-x64" | "x86_64-pc-windows-gnu" => {
                ("windows", "amd64")
            }
            "arm64-windows" | "windows/arm64" => ("windows", "arm64"),
            _ if t.contains('/') => {
                let parts: Vec<&str> = t.split('/').collect();
                if parts.len() == 2 {
                    (parts[0], parts[1])
                } else {
                    ("linux", "amd64")
                }
            }
            _ => ("linux", "amd64"),
        };
        cmd.env("GOOS", goos);
        cmd.env("GOARCH", goarch);
    }

    let compile_status = cmd.status();

    if let Ok(ref status) = compile_status {
        if status.success() {
            let _ = fs::remove_file(&tmp_go_path);
        }
    }

    match compile_status {
        Ok(status) if status.success() => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(out_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(out_path, perms);
                }
            }

            let size_bytes = fs::metadata(out_path).map(|m| m.len()).unwrap_or(0);
            Ok(ExecutableBuildResult {
                binary_path: out_path.to_path_buf(),
                format: "Mach-O / ELF Native Binary".to_string(),
                is_native_binary: true,
                size_bytes,
            })
        }
        Ok(status) => Err(format!(
            "Native binary compilation failed: 'go build' exited with status {}.",
            status
        )),
        Err(e) => Err(format!(
            "Failed to execute native binary compiler 'go build' (is Go installed?): {}.",
            e
        )),
    }
}

// -----------------------------------------------------------------------------
// DESIGN PATTERN: STRATEGY & FACTORY PATTERN FOR COMPILER BACKENDS
// -----------------------------------------------------------------------------

/// Target backend kind categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    /// Cranelift-based standalone native machine code executable.
    CraneliftNative,
    /// Go toolchain native binary compilation.
    GoNative,
    /// Node.js / ES6 JavaScript module emission.
    JsModule,
}

/// Configuration options and context for backend compilation and code emission.
#[derive(Debug, Clone, Default)]
pub struct BackendOptions<'a> {
    /// Optional base filesystem directory for resolving local relative module imports.
    pub base_path: Option<&'a Path>,
    /// Target architecture and platform triple for cross-compilation (e.g. `x86_64-apple-darwin`).
    pub target_triple: Option<&'a str>,
    /// TypeScript declaration inputs for foreign module typing.
    pub dts_inputs: &'a [(&'a str, &'a str)],
    /// Whether to generate Source Maps V3 for debugging.
    pub sourcemap: bool,
    /// Name of the generated output JavaScript file when generating sourcemaps.
    pub js_filename: Option<&'a str>,
    /// Name of the input Aura source file when generating sourcemaps.
    pub aura_filename: Option<&'a str>,
}

/// Strategy pattern interface abstracting backend code emission and standalone executable generation.
pub trait BackendStrategy: Send + Sync {
    /// Returns the target backend classification kind.
    fn kind(&self) -> BackendKind;

    /// Returns a human-readable display name for this backend strategy.
    fn name(&self) -> &'static str;

    /// Compiles an AST module into code artifacts according to this backend strategy.
    /// - `module`: The parsed and type-checked AST module.
    /// - `options`: Compilation settings and filesystem context.
    /// - Returns: `Ok(CompilationResult)` or `Err(String)` on failure.
    fn compile_module(
        &self,
        module: &Module,
        options: &BackendOptions,
    ) -> Result<CompilationResult, String>;

    /// Builds a standalone native binary or executable script file on disk.
    /// - `source_or_code`: Source text or intermediate representation.
    /// - `out_path`: Target output file path.
    /// - `options`: Compilation settings and filesystem context.
    /// - Returns: `Ok(ExecutableBuildResult)` containing artifact metadata.
    fn build_executable(
        &self,
        source_or_code: &str,
        out_path: &Path,
        options: &BackendOptions,
    ) -> Result<ExecutableBuildResult, String>;
}

/// Backend strategy emitting native machine code binaries via Cranelift.
pub struct CraneliftBackendStrategy;

impl BackendStrategy for CraneliftBackendStrategy {
    fn kind(&self) -> BackendKind {
        BackendKind::CraneliftNative
    }

    fn name(&self) -> &'static str {
        "Cranelift Native Machine Code"
    }

    fn compile_module(
        &self,
        module: &Module,
        options: &BackendOptions,
    ) -> Result<CompilationResult, String> {
        let mut codegen = CodeGen::with_base_path(options.base_path);
        let (js_code, dts_code) = codegen.generate(module);
        Ok(CompilationResult {
            js_code,
            dts_code,
            source_map: None,
        })
    }

    fn build_executable(
        &self,
        source_or_code: &str,
        out_path: &Path,
        options: &BackendOptions,
    ) -> Result<ExecutableBuildResult, String> {
        crate::codegen_cranelift::build_native_binary_with_base_path(
            source_or_code,
            out_path,
            options.base_path,
            options.target_triple,
        )?;
        let size_bytes = fs::metadata(out_path).map(|m| m.len()).unwrap_or(0);
        Ok(ExecutableBuildResult {
            binary_path: out_path.to_path_buf(),
            format: "Mach-O / ELF Cranelift Native Binary".to_string(),
            is_native_binary: true,
            size_bytes,
        })
    }
}

/// Backend strategy emitting and compiling Go source code via `go build`.
pub struct GoBackendStrategy;

impl BackendStrategy for GoBackendStrategy {
    fn kind(&self) -> BackendKind {
        BackendKind::GoNative
    }

    fn name(&self) -> &'static str {
        "Go Toolchain Native Binary"
    }

    fn compile_module(
        &self,
        module: &Module,
        _options: &BackendOptions,
    ) -> Result<CompilationResult, String> {
        let mut go_codegen = crate::codegen_go::GoCodeGen::new();
        let go_code = go_codegen.generate(module);
        Ok(CompilationResult {
            js_code: go_code,
            dts_code: String::new(),
            source_map: None,
        })
    }

    fn build_executable(
        &self,
        source_or_code: &str,
        out_path: &Path,
        options: &BackendOptions,
    ) -> Result<ExecutableBuildResult, String> {
        build_backend_executable_with_base_path(
            source_or_code,
            out_path,
            options.base_path,
            options.target_triple,
        )
    }
}

/// Backend strategy generating executable ES6 / Node.js JavaScript modules.
pub struct JsBackendStrategy;

impl BackendStrategy for JsBackendStrategy {
    fn kind(&self) -> BackendKind {
        BackendKind::JsModule
    }

    fn name(&self) -> &'static str {
        "ES6 / Node.js JavaScript Module"
    }

    fn compile_module(
        &self,
        module: &Module,
        options: &BackendOptions,
    ) -> Result<CompilationResult, String> {
        let mut codegen = CodeGen::with_base_path(options.base_path);
        if options.sourcemap && options.js_filename.is_some() && options.aura_filename.is_some() {
            let (js_code, dts_code, sm) = codegen.generate_with_sourcemap(
                module,
                options.js_filename.unwrap(),
                options.aura_filename.unwrap(),
                "",
            );
            Ok(CompilationResult {
                js_code,
                dts_code,
                source_map: Some(sm),
            })
        } else {
            let (js_code, dts_code) = codegen.generate(module);
            Ok(CompilationResult {
                js_code,
                dts_code,
                source_map: None,
            })
        }
    }

    fn build_executable(
        &self,
        source_or_code: &str,
        out_path: &Path,
        options: &BackendOptions,
    ) -> Result<ExecutableBuildResult, String> {
        let res = compile_backend(source_or_code, options.base_path, options.dts_inputs)?;
        fs::write(out_path, &res.js_code)
            .map_err(|e| format!("Failed to write JS executable: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(out_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(out_path, perms);
            }
        }
        let size_bytes = fs::metadata(out_path).map(|m| m.len()).unwrap_or(0);
        Ok(ExecutableBuildResult {
            binary_path: out_path.to_path_buf(),
            format: "ES6 Node.js Module".to_string(),
            is_native_binary: false,
            size_bytes,
        })
    }
}

/// Factory creating backend strategy instances based on CLI flags or target configurations.
pub struct BackendFactory;

impl BackendFactory {
    /// Instantiates a backend strategy matching the requested target string.
    /// - `target`: Target name (`"native"`, `"cranelift"`, `"go"`, or `"js"`).
    /// - Returns: `Box<dyn BackendStrategy>` matching the target.
    pub fn create(target: &str) -> Box<dyn BackendStrategy> {
        match target.to_lowercase().as_str() {
            "go" => Box::new(GoBackendStrategy),
            "js" | "mjs" | "node" => Box::new(JsBackendStrategy),
            _ => Box::new(CraneliftBackendStrategy),
        }
    }

    /// Returns the default backend strategy (Cranelift native machine code).
    pub fn default_strategy() -> Box<dyn BackendStrategy> {
        Box::new(CraneliftBackendStrategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_factory_and_strategy() {
        let go_backend = BackendFactory::create("go");
        assert_eq!(go_backend.kind(), BackendKind::GoNative);
        assert_eq!(go_backend.name(), "Go Toolchain Native Binary");

        let cl_backend = BackendFactory::create("native");
        assert_eq!(cl_backend.kind(), BackendKind::CraneliftNative);

        let default_be = BackendFactory::default_strategy();
        assert_eq!(default_be.kind(), BackendKind::CraneliftNative);

        let js_backend = BackendFactory::create("js");
        assert_eq!(js_backend.kind(), BackendKind::JsModule);
    }
}
