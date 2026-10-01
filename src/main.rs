//! Aura Language CLI Compiler (aurac)

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn print_usage() {
    println!(
        "Aura Language Compiler (aurac) - Standalone Native Binary Compiler (Golang Model) v0.1.0"
    );
    println!("Usage:");
    println!(
        "  aurac <file.aura> [-o <binary>] [--target <native|go|<triple>>] [--release] [-O3] [--tags <tags>]"
    );
    println!(
        "  aurac build <file.aura> [-o <binary>] [--target <native|go|<triple>>] [--release] [-O3] [--tags <tags>]"
    );
    println!("  aurac run <file.aura> [--tags <tags>]");
    println!("  aurac debug <file.aura> [--step] [--port <port>] [--no-brk] [--tags <tags>]");
    println!("  aurac step <file.aura> [--port <port>] [--tags <tags>]");
    println!("  aurac check <file.aura> [--tags <tags>]");
    println!("  aurac emit-go <file.aura> [-o <output.go>]");
    println!("  aurac watch <file.aura> [-o <binary>] [--run]");
    println!("  aurac test [flags] [paths...]");
    println!("  aurac fmt [-w] [-l] [-d] [-c] [--tabs] [--indent <N>] [paths...]");
    println!("  aurac mod <init|tidy|get|vendor|verify|list> [args...]");
    println!("  aurac pkg <init|tidy|get|vendor|verify|list> [args...]");
    println!("  aurac dts-import <file.d.ts>");
    println!("  aurac bindgen <header.h> [-o <output.aura>] [--name <module>]");
    println!("  aurac lsp");
    println!("  aurac playground [--port <port>]");
}

fn execute_build(
    input_file: &str,
    out_file: Option<String>,
    _target_backend: &str,
    tags: &[String],
    dts_inputs: &[(String, String)],
    release: bool,
) {
    let source = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", input_file, e);
            std::process::exit(1);
        }
    };

    let tags_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
    if !aura_lang::build_tags::should_build_source(&source, &tags_refs) {
        println!("ℹ Skipping '{}': build tags do not match.", input_file);
        return;
    }

    let out_path = out_file.unwrap_or_else(|| {
        let p = Path::new(input_file);
        p.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "app".to_string())
    });

    let dts_refs: Vec<(&str, &str)> = dts_inputs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // If an explicit .mjs extension is requested, output module (used for compiler self-hosting)
    if out_path.ends_with(".mjs") {
        let base_path = Path::new(input_file).parent();
        match aura_lang::compile_with_base_path(&source, base_path, &dts_refs) {
            Ok(result) => {
                if let Some(parent) = Path::new(&out_path).parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let mut js_code = result.js_code;
                if !js_code.contains("__aura_invoked_main")
                    && (js_code.contains("function main(") || js_code.contains("const main ="))
                {
                    js_code.push_str("\n\n// Auto-invoke main entrypoint (Golang model)\nconst __aura_invoked_main = true;\nif (typeof main === 'function') {\n  const __r = main();\n  if (__r && typeof __r.then === 'function') {\n    __r.catch(_e => { console.error(_e); if (typeof process !== 'undefined') process.exit(1); });\n  }\n}\n");
                }
                if let Err(e) = fs::write(&out_path, &js_code) {
                    eprintln!("Error writing module output '{}': {}", out_path, e);
                    return;
                }
                println!("✨ Compiled module '{}' successfully!", out_path);
            }
            Err(e) => {
                eprintln!("✕ Compilation failed:\n{}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    let explicit_target = if _target_backend != "native"
        && _target_backend != "cranelift"
        && _target_backend != "go"
        && !_target_backend.is_empty()
    {
        Some(_target_backend)
    } else {
        None
    };

    let base_path = Path::new(input_file).parent();

    if _target_backend == "native" || _target_backend == "cranelift" || explicit_target.is_some() {
        println!(
            "⚡ Compiling standalone native binary executable via Cranelift{}{}...",
            explicit_target
                .map(|t| format!(" (target: {})", t))
                .unwrap_or_default(),
            if release {
                " [release: -O3, SIMD enabled, dead-code strip]"
            } else {
                ""
            }
        );
        match aura_lang::build_native_binary_with_options(
            &source,
            Path::new(&out_path),
            base_path,
            explicit_target,
            release,
        ) {
            Ok(_) => {
                let size_bytes = fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
                let mb = (size_bytes as f64) / (1024.0 * 1024.0);
                println!(
                    "📦 Built standalone native binary executable '{}' successfully!",
                    out_path
                );
                println!("   ➜ Backend: Cranelift Native Machine Code");
                if release {
                    println!(
                        "   ➜ Mode: Release (-O3 speed, SIMD autovectorization, LTO dead-strip)"
                    );
                }
                if let Some(t) = explicit_target {
                    println!("   ➜ Target Triple: {}", t);
                }
                println!("   ➜ Runtime: Aura M:N Fiber Scheduler + GC (libaura_runtime.a)");
                println!("   ➜ Size: {:.2} MB", mb);
                println!("   ➜ Zero external runtime dependencies");
                println!("   ➜ Execute: ./{}", out_path);
                return;
            }
            Err(e) => {
                if _target_backend == "cranelift" {
                    eprintln!("✕ Cranelift native compilation error:\n{}", e);
                    std::process::exit(1);
                } else {
                    println!("ℹ Cranelift native backend fallback: {}", e);
                }
            }
        }
    }

    println!(
        "⚡ Compiling standalone native binary executable via Go toolchain{}...",
        explicit_target
            .map(|t| format!(" (target: {})", t))
            .unwrap_or_default()
    );
    match aura_lang::build_backend_executable_with_base_path(
        &source,
        Path::new(&out_path),
        base_path,
        explicit_target,
    ) {
        Ok(build_result) => {
            let mb = (build_result.size_bytes as f64) / (1024.0 * 1024.0);
            println!(
                "📦 Built standalone native binary executable '{}' successfully!",
                out_path
            );
            println!("   ➜ Format: {}", build_result.format);
            println!("   ➜ Size: {:.2} MB", mb);
            println!("   ➜ Golang model: Distributable single binary (zero external dependencies)");
            println!("   ➜ Execute: ./{}", out_path);
        }
        Err(e) => {
            eprintln!("✕ Build error:\n{}", e);
            std::process::exit(1);
        }
    }
}

// -----------------------------------------------------------------------------
// DESIGN PATTERN: COMMAND PATTERN FOR CLI DISPATCH
// -----------------------------------------------------------------------------

/// Command pattern trait abstracting individual CLI subcommands (`build`, `check`, `emit-go`).
pub trait CliCommand {
    /// Executes the encapsulated CLI operation.
    fn execute(&self);
}

/// Command encapsulating the `aurac build` execution with target backend and tags.
pub struct BuildCommand {
    /// Input Aura source file to compile.
    pub input_file: String,
    /// Optional output file path for the standalone binary or module.
    pub out_file: Option<String>,
    /// Target backend identifier (`"native"`, `"cranelift"`, `"go"`, or `"js"`).
    pub target_backend: String,
    /// Build tags passed for conditional compilation.
    pub tags: Vec<String>,
    /// TypeScript declaration inputs for foreign typing.
    pub dts_inputs: Vec<(String, String)>,
    /// Release mode optimization flag (-O3, SIMD enabled, LTO dead-code strip).
    pub release: bool,
}

impl CliCommand for BuildCommand {
    fn execute(&self) {
        execute_build(
            &self.input_file,
            self.out_file.clone(),
            &self.target_backend,
            &self.tags,
            &self.dts_inputs,
            self.release,
        );
    }
}

/// Command encapsulating the fast static type check (`aurac check <file.aura>`).
pub struct CheckCommand {
    /// Path to the Aura source file to validate.
    pub filename: String,
}

impl CliCommand for CheckCommand {
    fn execute(&self) {
        let filename = &self.filename;
        let base_path = Path::new(filename).parent();
        match fs::read_to_string(filename) {
            Ok(content) => match aura_lang::compile_backend(&content, base_path, &[]) {
                Ok(_) => println!(
                    "✓ Backend type check succeeded for '{}'. Ready for binary build.",
                    filename
                ),
                Err(e) => {
                    eprintln!("✕ Type error in '{}':\n{}", filename, e);
                    std::process::exit(1);
                }
            },
            Err(e) => eprintln!("Error reading '{}': {}", filename, e),
        }
    }
}

/// Command encapsulating direct Golang source code emission (`aurac emit-go <file.aura>`).
pub struct EmitGoCommand {
    /// Input Aura source file to transpile.
    pub input_file: String,
    /// Optional destination path to write the generated `.go` file.
    pub out_file: Option<String>,
}

impl CliCommand for EmitGoCommand {
    fn execute(&self) {
        let source = match fs::read_to_string(&self.input_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading '{}': {}", self.input_file, e);
                return;
            }
        };

        let base_path = Path::new(&self.input_file).parent();
        match aura_lang::compile_to_go_with_base_path(&source, base_path) {
            Ok(go_code) => {
                if let Some(ref out_path) = self.out_file {
                    if let Err(e) = fs::write(out_path, &go_code) {
                        eprintln!("Error writing Go output '{}': {}", out_path, e);
                        return;
                    }
                    println!(
                        "✨ Generated Golang source code '{}' successfully!",
                        out_path
                    );
                } else {
                    print!("{}", go_code);
                }
            }
            Err(e) => {
                eprintln!("✕ Go transpilation error:\n{}", e);
                std::process::exit(1);
            }
        }
    }
}

const DEBUGGER_RUNNER_SCRIPT: &str = include_str!("debugger.js");

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command_or_file = &args[1];

    match command_or_file.as_str() {
        "--help" | "-h" => {
            print_usage();
        }
        "watch" | "--watch" => {
            let mut input_file = String::new();
            let mut out_file = None;
            let mut run_on_change = false;
            let mut dts_inputs: Vec<(String, String)> = Vec::new();

            let mut i = 1;
            while i < args.len() {
                if args[i] == "watch" || args[i] == "--watch" {
                    i += 1;
                } else if args[i] == "--run" || args[i] == "-r" {
                    run_on_change = true;
                    i += 1;
                } else if args[i] == "-o" && i + 1 < args.len() {
                    out_file = Some(args[i + 1].clone());
                    i += 2;
                } else if args[i] == "--dts" && i + 1 < args.len() {
                    let dts_path = &args[i + 1];
                    if let Ok(dts_content) = fs::read_to_string(dts_path) {
                        dts_inputs.push((dts_path.clone(), dts_content));
                    }
                    i += 2;
                } else if input_file.is_empty() && !args[i].starts_with('-') {
                    input_file = args[i].clone();
                    i += 1;
                } else {
                    i += 1;
                }
            }

            if input_file.is_empty() {
                eprintln!("Error: Missing input file to watch.");
                return;
            }

            let dts_refs: Vec<(&str, &str)> = dts_inputs
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            let config = aura_lang::watcher::WatchConfig {
                input_file,
                out_file,
                run_on_change,
                dts_inputs: dts_refs,
                poll_interval_ms: 100,
            };

            if let Err(e) = aura_lang::watcher::start_watch(config) {
                eprintln!("Watch mode stopped: {}", e);
            }
        }
        "lsp" => {
            let mut server = aura_lang::lsp::LspServer::new();
            if let Err(e) = server.run_stdio() {
                eprintln!("Aura LSP Server encountered an error: {}", e);
                std::process::exit(1);
            }
        }
        "playground" => {
            let mut port = aura_lang::playground::DEFAULT_PORT;
            let mut i = 2;
            while i < args.len() {
                if (args[i] == "--port" || args[i] == "-p") && i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if let Err(e) = aura_lang::playground::start_server(port) {
                eprintln!("Error starting Aura Playground: {}", e);
                std::process::exit(1);
            }
        }
        "test" => {
            let test_args: Vec<String> = args.into_iter().skip(2).collect();
            aura_lang::testing::run_cli(&test_args);
        }
        "emit-go" => {
            let mut input_file = String::new();
            let mut out_file = None;

            let mut i = 2;
            while i < args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    out_file = Some(args[i + 1].clone());
                    i += 2;
                } else if input_file.is_empty() && !args[i].starts_with('-') {
                    input_file = args[i].clone();
                    i += 1;
                } else {
                    i += 1;
                }
            }

            if input_file.is_empty() {
                eprintln!("Error: Missing input file for 'emit-go'.");
                return;
            }

            let cmd = EmitGoCommand {
                input_file,
                out_file,
            };
            cmd.execute();
        }
        "build" => {
            let mut input_file = String::new();
            let mut out_file = None;
            let mut target_backend = "native".to_string(); // "native" or "go"
            let mut tags = Vec::new();
            let mut dts_inputs: Vec<(String, String)> = Vec::new();
            let mut release = false;

            let mut i = 2;
            while i < args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    out_file = Some(args[i + 1].clone());
                    i += 2;
                } else if (args[i] == "--target" || args[i] == "-target") && i + 1 < args.len() {
                    target_backend = args[i + 1].to_lowercase();
                    i += 2;
                } else if args[i] == "--go" {
                    target_backend = "go".to_string();
                    i += 1;
                } else if args[i] == "--standalone" {
                    // Always default in backend Golang model
                    i += 1;
                } else if args[i] == "--release"
                    || args[i] == "-O"
                    || args[i] == "-O3"
                    || args[i] == "--simd"
                {
                    release = true;
                    i += 1;
                } else if (args[i] == "--tags" || args[i] == "-tags") && i + 1 < args.len() {
                    tags.push(args[i + 1].clone());
                    i += 2;
                } else if args[i] == "--dts" && i + 1 < args.len() {
                    let dts_path = &args[i + 1];
                    if let Ok(dts_content) = fs::read_to_string(dts_path) {
                        dts_inputs.push((dts_path.clone(), dts_content));
                    }
                    i += 2;
                } else if input_file.is_empty() && !args[i].starts_with('-') {
                    input_file = args[i].clone();
                    i += 1;
                } else {
                    i += 1;
                }
            }

            if input_file.is_empty() {
                eprintln!("Error: Missing input file for 'build'.");
                return;
            }

            let cmd = BuildCommand {
                input_file,
                out_file,
                target_backend,
                tags,
                dts_inputs,
                release,
            };
            cmd.execute();
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("Error: Missing input file for 'check'.");
                return;
            }
            let filename = args[2].clone();
            let cmd = CheckCommand { filename };
            cmd.execute();
        }
        "run" => {
            let mut input_file = String::new();
            let mut tags = Vec::new();
            let mut trailing_args = Vec::new();
            let mut i = 2;
            while i < args.len() {
                if (args[i] == "--tags" || args[i] == "-tags") && i + 1 < args.len() {
                    tags.push(args[i + 1].clone());
                    i += 2;
                } else if input_file.is_empty() && !args[i].starts_with('-') {
                    input_file = args[i].clone();
                    i += 1;
                } else {
                    trailing_args.push(args[i].clone());
                    i += 1;
                }
            }

            if input_file.is_empty() {
                eprintln!("Error: Missing input file for 'run'.");
                return;
            }

            let filename = &input_file;
            let base_path = Path::new(filename).parent();
            match fs::read_to_string(filename) {
                Ok(content) => {
                    let tags_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
                    if !aura_lang::build_tags::should_build_source(&content, &tags_refs) {
                        println!("Skipping '{}' due to build tags mismatch.", filename);
                        return;
                    }

                    let tmp_js = format!("{}.tmp.mjs", filename);
                    let tmp_map = format!("{}.map", tmp_js);

                    match aura_lang::compile_backend_with_sourcemap(
                        &content,
                        &tmp_js,
                        filename,
                        base_path,
                        &[],
                    ) {
                        Ok(res) => {
                            let mut runner_code = res.js_code;
                            if !runner_code.contains("__aura_invoked_main")
                                && (runner_code.contains("function main(")
                                    || runner_code.contains("const main ="))
                            {
                                runner_code.push_str("\n\n// Auto-invoke main entrypoint (Golang model)\nconst __aura_invoked_main = true;\nif (typeof main === 'function') {\n  const __r = main();\n  if (__r && typeof __r.then === 'function') {\n    __r.catch(_e => { console.error(_e); if (typeof process !== 'undefined') process.exit(1); });\n  }\n}\n");
                            }
                            if let Err(e) = fs::write(&tmp_js, &runner_code) {
                                eprintln!("Error writing temp file: {}", e);
                                return;
                            }
                            if let Some(ref map_content) = res.source_map {
                                let _ = fs::write(&tmp_map, map_content);
                            }

                            let compiled_deps = aura_lang::compile_local_deps_to_mjs_from_source(
                                &content, base_path,
                            );

                            let mut cmd = Command::new("node");
                            cmd.arg("--enable-source-maps")
                                .arg(&tmp_js)
                                .args(&trailing_args);
                            let status = cmd.status();

                            let _ = fs::remove_file(&tmp_js);
                            let _ = fs::remove_file(&tmp_map);
                            for dep in compiled_deps {
                                let _ = fs::remove_file(dep);
                            }

                            if let Err(e) = status {
                                eprintln!("Failed to execute node: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("✕ Compilation error:\n{}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => eprintln!("Error reading '{}': {}", filename, e),
            }
        }
        "debug" | "step" => {
            let is_step_cmd = command_or_file == "step";
            let mut input_file = String::new();
            let mut port = 9229u16;
            let mut stop_on_entry = true;
            let mut step_mode = is_step_cmd;
            let mut tags = Vec::new();
            let mut i = 2;
            while i < args.len() {
                if (args[i] == "--port" || args[i] == "-p") && i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                    }
                    i += 2;
                } else if args[i] == "--step" || args[i] == "-s" {
                    step_mode = true;
                    i += 1;
                } else if args[i] == "--no-brk" {
                    stop_on_entry = false;
                    i += 1;
                } else if args[i] == "--brk" {
                    stop_on_entry = true;
                    i += 1;
                } else if (args[i] == "--tags" || args[i] == "-tags") && i + 1 < args.len() {
                    tags.push(args[i + 1].clone());
                    i += 2;
                } else if input_file.is_empty() && !args[i].starts_with('-') {
                    input_file = args[i].clone();
                    i += 1;
                } else {
                    i += 1;
                }
            }

            if input_file.is_empty() {
                eprintln!("Error: Missing input file for debugging.");
                eprintln!(
                    "Usage: aurac debug <file.aura> [--step] [--port <port>] [--no-brk] [--tags <tags>]"
                );
                eprintln!("       aurac step <file.aura> [--port <port>] [--tags <tags>]");
                return;
            }

            let filename = &input_file;
            let base_path = Path::new(filename).parent();
            match fs::read_to_string(filename) {
                Ok(content) => {
                    let tags_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
                    if !aura_lang::build_tags::should_build_source(&content, &tags_refs) {
                        println!("Skipping '{}' due to build tags mismatch.", filename);
                        return;
                    }

                    let debug_js = format!("{}.debug.mjs", filename);
                    let debug_map = format!("{}.map", debug_js);

                    match aura_lang::compile_backend_with_sourcemap(
                        &content,
                        &debug_js,
                        filename,
                        base_path,
                        &[],
                    ) {
                        Ok(res) => {
                            let mut runner_code = res.js_code;
                            if !runner_code.contains("__aura_invoked_main")
                                && (runner_code.contains("function main(")
                                    || runner_code.contains("const main ="))
                            {
                                runner_code.push_str("\n\n// Auto-invoke main entrypoint (Golang model)\nconst __aura_invoked_main = true;\nif (typeof main === 'function') {\n  const __r = main();\n  if (__r && typeof __r.then === 'function') {\n    __r.catch(_e => { console.error(_e); if (typeof process !== 'undefined') process.exit(1); });\n  }\n}\n");
                            }

                            if let Err(e) = fs::write(&debug_js, &runner_code) {
                                eprintln!("Error writing debug bundle '{}': {}", debug_js, e);
                                return;
                            }
                            if let Some(ref map_content) = res.source_map
                                && let Err(e) = fs::write(&debug_map, map_content)
                            {
                                eprintln!("Error writing sourcemap '{}': {}", debug_map, e);
                                return;
                            }

                            if step_mode {
                                let step_runner_js = format!("{}.step_runner.cjs", filename);
                                if let Err(e) = fs::write(&step_runner_js, DEBUGGER_RUNNER_SCRIPT) {
                                    eprintln!("Error writing debugger script: {}", e);
                                    let _ = fs::remove_file(&debug_js);
                                    let _ = fs::remove_file(&debug_map);
                                    return;
                                }

                                let status = Command::new("node")
                                    .arg(&step_runner_js)
                                    .arg(filename)
                                    .arg(&debug_js)
                                    .arg(&debug_map)
                                    .arg(port.to_string())
                                    .status();

                                let _ = fs::remove_file(&step_runner_js);
                                let _ = fs::remove_file(&debug_js);
                                let _ = fs::remove_file(&debug_map);

                                if let Err(e) = status {
                                    eprintln!("Failed to execute step debugger: {}", e);
                                }
                            } else {
                                println!("🐛 Aura Debugger initialized for '{}'", filename);
                                println!("   ➜ V8 Inspector endpoint: 127.0.0.1:{}", port);
                                println!(
                                    "   ➜ SourceMap active: {} (high-precision lines)",
                                    debug_map
                                );
                                if stop_on_entry {
                                    println!(
                                        "   ➜ Status: Waiting for debugger to attach (--inspect-brk)..."
                                    );
                                } else {
                                    println!(
                                        "   ➜ Status: Running with live inspector enabled (--inspect)..."
                                    );
                                }
                                println!(
                                    "   ➜ Supported editors: VS Code, Antigravity IDE, Zed, Chrome DevTools"
                                );

                                let inspect_flag = if stop_on_entry {
                                    format!("--inspect-brk=127.0.0.1:{}", port)
                                } else {
                                    format!("--inspect=127.0.0.1:{}", port)
                                };

                                let status = Command::new("node")
                                    .arg("--enable-source-maps")
                                    .arg(&inspect_flag)
                                    .arg(&debug_js)
                                    .status();

                                let _ = fs::remove_file(&debug_js);
                                let _ = fs::remove_file(&debug_map);

                                if let Err(e) = status {
                                    eprintln!("Failed to execute node for debugging: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("✕ Compilation error:\n{}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => eprintln!("Error reading '{}': {}", filename, e),
            }
        }
        "fmt" => {
            let fmt_args: Vec<String> = args.into_iter().skip(2).collect();
            aura_lang::formatter::run_cli(&fmt_args);
        }
        "mod" | "pkg" => {
            let mod_args: Vec<String> = args.into_iter().skip(2).collect();
            aura_lang::package::run_cli(&mod_args);
        }
        "dts-import" => {
            if args.len() < 3 {
                eprintln!("Error: Missing .d.ts file path.");
                return;
            }

            let filename = &args[2];
            match fs::read_to_string(filename) {
                Ok(content) => {
                    let mut parser = aura_lang::dts_parser::DtsParser::new(&content);
                    match parser.parse_dts(filename) {
                        Ok(module) => {
                            println!(
                                "✓ Successfully parsed TypeScript Definitions from '{}':",
                                filename
                            );
                            println!("\nTypes ({}):", module.types.len());
                            for (name, ty) in &module.types {
                                println!("  - type {}: {}", name, ty);
                            }
                            println!("\nFunctions ({}):", module.functions.len());
                            for (name, ty) in &module.functions {
                                println!("  - fn {}: {}", name, ty);
                            }
                        }
                        Err(e) => eprintln!("✕ DTS Parser error:\n{}", e),
                    }
                }
                Err(e) => eprintln!("Error reading '{}': {}", filename, e),
            }
        }
        "bindgen" => {
            let bindgen_args: Vec<String> = args.into_iter().skip(2).collect();
            aura_lang::bindgen::run_cli(&bindgen_args);
        }

        _ => {
            // Default action: Build standalone native binary (Golang model: 'aurac file.aura' or 'aurac compile file.aura')
            let mut input_file = command_or_file.clone();
            let mut out_file = None;
            let mut target_backend = "native".to_string();
            let mut tags = Vec::new();
            let mut dts_inputs: Vec<(String, String)> = Vec::new();
            let mut release = false;

            let mut i = 1;
            if (args[1] == "compile" || args[1] == "build") && args.len() > 2 {
                input_file = args[2].clone();
                i = 3;
            }

            while i < args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    out_file = Some(args[i + 1].clone());
                    i += 2;
                } else if (args[i] == "--target" || args[i] == "-target") && i + 1 < args.len() {
                    target_backend = args[i + 1].to_lowercase();
                    i += 2;
                } else if args[i] == "--go" {
                    target_backend = "go".to_string();
                    i += 1;
                } else if args[i] == "--standalone" {
                    i += 1;
                } else if args[i] == "--release"
                    || args[i] == "-O"
                    || args[i] == "-O3"
                    || args[i] == "--simd"
                {
                    release = true;
                    i += 1;
                } else if (args[i] == "--tags" || args[i] == "-tags") && i + 1 < args.len() {
                    tags.push(args[i + 1].clone());
                    i += 2;
                } else if args[i] == "--dts" && i + 1 < args.len() {
                    let dts_path = &args[i + 1];
                    if let Ok(dts_content) = fs::read_to_string(dts_path) {
                        dts_inputs.push((dts_path.clone(), dts_content));
                    }
                    i += 2;
                } else if input_file.is_empty() && !args[i].starts_with('-') {
                    input_file = args[i].clone();
                    i += 1;
                } else {
                    i += 1;
                }
            }

            if input_file.is_empty() {
                eprintln!("Error: Missing input file to compile.");
                return;
            }

            execute_build(
                &input_file,
                out_file,
                &target_backend,
                &tags,
                &dts_inputs,
                release,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_usage_execution() {
        print_usage();
    }
}
