//! Watch Mode and Incremental Dev Server for the Aura Compiler.
//!
//! Watches `.aura` source files for file modifications and triggers
//! sub-millisecond recompilation directly to standalone native binaries,
//! with optional live process reloading.

use crate::backend::{build_backend_executable, compile_backend};
use std::fs;

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime};

/// Configuration options for the file watcher.
pub struct WatchConfig<'a> {
    pub input_file: String,
    pub out_file: Option<String>,
    pub run_on_change: bool,
    pub dts_inputs: Vec<(&'a str, &'a str)>,
    pub poll_interval_ms: u64,
}

/// Watches an Aura file and recompiles incrementally on changes.
pub fn start_watch(config: WatchConfig) -> Result<(), String> {
    let input_path = PathBuf::from(&config.input_file);
    if !input_path.exists() {
        return Err(format!("File '{}' does not exist.", config.input_file));
    }

    let target_binary = config.out_file.clone().unwrap_or_else(|| {
        let p = Path::new(&config.input_file);
        p.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "app".to_string())
    });

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 👁️  Aura Compiler Watch Mode 👁️               ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  ➜ Watching:        {:<41}║", config.input_file);
    println!("║  ➜ Target Binary:   {:<41}║", target_binary);
    println!(
        "║  ➜ Auto-Run:        {:<41}║",
        if config.run_on_change {
            "Enabled (Native Executable)"
        } else {
            "Disabled"
        }
    );
    println!("║  ➜ Press Ctrl+C to exit                                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut last_modified = get_mtime(&input_path);
    let mut running_child: Option<Child> = None;

    // Initial compile
    if let Err(e) = compile_and_emit(
        &config.input_file,
        &target_binary,
        &config.dts_inputs,
        config.run_on_change,
        &mut running_child,
    ) {
        eprintln!("✕ Initial compilation failed:\n{}", e);
    }

    let poll_duration = Duration::from_millis(config.poll_interval_ms.max(50));

    loop {
        sleep(poll_duration);

        let current_mtime = get_mtime(&input_path);
        if current_mtime > last_modified {
            last_modified = current_mtime;

            if let Err(e) = compile_and_emit(
                &config.input_file,
                &target_binary,
                &config.dts_inputs,
                config.run_on_change,
                &mut running_child,
            ) {
                eprintln!("✕ Compilation error:\n{}", e);
            }
        }
    }
}

fn compile_and_emit(
    input_file: &str,
    target_binary: &str,
    dts_inputs: &[(&str, &str)],
    run_on_change: bool,
    child_process: &mut Option<Child>,
) -> Result<(), String> {
    let start_time = Instant::now();
    let source = fs::read_to_string(input_file)
        .map_err(|e| format!("Failed to read '{}': {}", input_file, e))?;
    let base_path = Path::new(input_file).parent();

    let _ = compile_backend(&source, base_path, dts_inputs)?;

    let target_path = Path::new(target_binary);
    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }

    build_backend_executable(&source, target_path)?;

    let elapsed = start_time.elapsed().as_millis();
    let timestamp = format_current_time();

    println!(
        "[{}] ⚡ [{}ms] Recompiled '{}' ➜ Standalone Native Binary '{}'",
        timestamp, elapsed, input_file, target_binary
    );

    if run_on_change {
        // Kill existing child process if still running
        if let Some(mut child) = child_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        let run_cmd = if target_binary.starts_with('/') || target_binary.starts_with('.') {
            target_binary.to_string()
        } else {
            format!("./{}", target_binary)
        };

        println!("[{}] 🚀 Running binary '{}'...", timestamp, run_cmd);
        match Command::new(&run_cmd).spawn() {
            Ok(new_child) => {
                *child_process = Some(new_child);
            }
            Err(e) => {
                eprintln!("✕ Failed to spawn binary '{}': {}", run_cmd, e);
            }
        }
    }

    Ok(())
}

fn get_mtime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn format_current_time() -> String {
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = duration.as_secs();
    let hours = (total_secs / 3600) % 24;
    let mins = (total_secs / 60) % 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}
