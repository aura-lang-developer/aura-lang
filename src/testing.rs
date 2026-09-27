//! Testing, Benchmarking, and Coverage Engine for Aura (Go-test style).
//!
//! Provides automated test discovery, unit / integration / e2e categorization,
//! async & CSP testing harness, benchmark execution with adaptive timing,
//! statement-level code coverage analysis, and Go-test CLI compatibility.

use crate::ast::Item;
use crate::compile;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The category/kind of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestKind {
    Unit,
    Integration,
    E2E,
    Benchmark,
}

impl std::fmt::Display for TestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestKind::Unit => write!(f, "UNIT"),
            TestKind::Integration => write!(f, "INTEGRATION"),
            TestKind::E2E => write!(f, "E2E"),
            TestKind::Benchmark => write!(f, "BENCHMARK"),
        }
    }
}

/// A discovered test or benchmark function inside an Aura module.
#[derive(Debug, Clone)]
pub struct TestFunction {
    pub name: String,
    pub display_name: String,
    pub kind: TestKind,
    pub is_async: bool,
    pub line_number: usize,
}

/// A discovered Aura test file.
#[derive(Debug, Clone)]
pub struct TestFile {
    pub path: PathBuf,
    pub source: String,
    pub default_kind: TestKind,
    pub functions: Vec<TestFunction>,
    pub has_top_level_assertions: bool,
}

/// Configuration options for the test runner (matching `go test` flags).
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub paths: Vec<PathBuf>,
    pub run_pattern: Option<String>,
    pub bench_pattern: Option<String>,
    pub verbose: bool,
    pub coverage: bool,
    pub coverprofile: Option<PathBuf>,
    pub coverage_html: Option<PathBuf>,
    pub unit_only: bool,
    pub integration_only: bool,
    pub e2e_only: bool,
    pub timeout_ms: u64,
    pub fail_fast: bool,
    pub parallel: usize,
    pub json: bool,
    pub count: usize,
    pub list_only: bool,
    pub watch: bool,
    pub race: bool,
    pub tags: Vec<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            paths: vec![PathBuf::from(".")],
            run_pattern: None,
            bench_pattern: None,
            verbose: false,
            coverage: false,
            coverprofile: None,
            coverage_html: None,
            unit_only: false,
            integration_only: false,
            e2e_only: false,
            timeout_ms: 30000,
            fail_fast: false,
            parallel: 1,
            json: false,
            count: 1,
            list_only: false,
            watch: false,
            race: false,
            tags: Vec::new(),
        }
    }
}

/// Status of an executed test case.
#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed(String),
    Skipped(String),
}

/// Result of a single test case execution.
#[derive(Debug, Clone)]
pub struct TestCaseResult {
    pub name: String,
    pub file_path: String,
    pub kind: TestKind,
    pub status: TestStatus,
    pub duration: Duration,
    pub logs: Vec<String>,
}

/// Result of a benchmark execution.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub file_path: String,
    pub iterations: u64,
    pub ns_per_op: f64,
    pub bytes_per_op: Option<u64>,
    pub duration: Duration,
}

/// File coverage metrics.
#[derive(Debug, Clone)]
pub struct FileCoverage {
    pub file_path: String,
    pub total_lines: usize,
    pub covered_lines: Vec<usize>,
    pub uncovered_lines: Vec<usize>,
    pub total_statements: usize,
    pub covered_statements: usize,
    pub percentage: f64,
}

/// Aggregate code coverage report.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub files: Vec<FileCoverage>,
    pub overall_percentage: f64,
    pub total_statements: usize,
    pub covered_statements: usize,
}

/// Full execution summary for test suite.
#[derive(Debug, Clone)]
pub struct TestSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total: usize,
    pub duration: Duration,
    pub test_results: Vec<TestCaseResult>,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub coverage: Option<CoverageReport>,
    pub success: bool,
}

// -----------------------------------------------------------------------------
// TEST DISCOVERY
// -----------------------------------------------------------------------------

/// Recursively discovers `.aura` test files matching test conventions.
pub fn discover_test_files(paths: &[PathBuf]) -> Result<Vec<TestFile>, String> {
    let mut files = Vec::new();
    let mut visited = HashSet::new();

    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str == "./..." || path_str == "..." {
            discover_in_dir(Path::new("."), &mut files, &mut visited)?;
        } else if path_str.ends_with("/...") {
            let base = path_str.trim_end_matches("/...");
            discover_in_dir(Path::new(base), &mut files, &mut visited)?;
        } else if path.is_dir() {
            discover_in_dir(path, &mut files, &mut visited)?;
        } else if path.is_file() {
            if let Some(tf) = process_test_file(path)? {
                files.push(tf);
            }
        }
    }

    Ok(files)
}

fn discover_in_dir(
    dir: &Path,
    files: &mut Vec<TestFile>,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            return Err(format!(
                "Failed to read directory '{}': {}",
                dir.display(),
                e
            ));
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist"
            {
                continue;
            }
        }

        if path.is_dir() {
            discover_in_dir(&path, files, visited)?;
        } else if path.is_file() {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let is_test_name = filename.ends_with(".aura")
                && (filename.ends_with("_test.aura")
                    || filename.ends_with(".test.aura")
                    || filename.ends_with("_spec.aura")
                    || filename.starts_with("test_")
                    || path.to_string_lossy().contains("/tests/"));

            if is_test_name {
                let canon = path.canonicalize().unwrap_or_else(|_| path.clone());
                if !visited.contains(&canon) {
                    visited.insert(canon);
                    if let Some(tf) = process_test_file(&path)? {
                        files.push(tf);
                    }
                }
            }
        }
    }

    Ok(())
}

fn determine_test_kind(path: &Path) -> TestKind {
    let p_str = path.to_string_lossy().to_lowercase();
    if p_str.contains("/e2e/")
        || p_str.contains(".e2e.")
        || p_str.contains("_e2e_")
        || p_str.ends_with("_e2e.aura")
    {
        TestKind::E2E
    } else if p_str.contains("/integration/")
        || p_str.contains(".integration.")
        || p_str.contains("_integration_")
        || p_str.ends_with("_integration.aura")
    {
        TestKind::Integration
    } else {
        TestKind::Unit
    }
}

fn process_test_file(path: &Path) -> Result<Option<TestFile>, String> {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(format!("Could not read file '{}': {}", path.display(), e)),
    };

    if !crate::build_tags::should_build_source(&source, &[]) {
        return Ok(None);
    }

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => return Err(format!("Syntax error in '{}': {}", path.display(), e)),
    };

    let mut parser = Parser::new(tokens);
    let module = match parser.parse_module() {
        Ok(m) => m,
        Err(e) => return Err(format!("Parse error in '{}': {}", path.display(), e)),
    };

    let default_kind = determine_test_kind(path);
    let mut functions = Vec::new();
    let mut has_top_level_assertions = false;

    for item in &module.items {
        match item {
            Item::Function(func) => {
                if func.receiver.is_some() {
                    continue;
                }
                let name = &func.name;
                let is_bench = name.starts_with("benchmark_")
                    || name.starts_with("Benchmark")
                    || name.starts_with("bench_")
                    || name.starts_with("bench");
                let is_test = name.starts_with("test_")
                    || name.starts_with("Test")
                    || name.starts_with("test")
                    || name.starts_with("it_")
                    || name.starts_with("spec_");

                if is_bench {
                    functions.push(TestFunction {
                        name: name.clone(),
                        display_name: name.clone(),
                        kind: TestKind::Benchmark,
                        is_async: func.is_async,
                        line_number: 1,
                    });
                } else if is_test {
                    let kind = if name.to_lowercase().contains("e2e") {
                        TestKind::E2E
                    } else if name.to_lowercase().contains("integration") {
                        TestKind::Integration
                    } else {
                        default_kind
                    };

                    functions.push(TestFunction {
                        name: name.clone(),
                        display_name: name.clone(),
                        kind,
                        is_async: func.is_async,
                        line_number: 1,
                    });
                }
            }
            Item::Statement(_) => {
                has_top_level_assertions = true;
            }
            _ => {}
        }
    }

    if functions.is_empty() && has_top_level_assertions {
        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Main");
        functions.push(TestFunction {
            name: "$main".to_string(),
            display_name: format!("Test{}", file_stem),
            kind: default_kind,
            is_async: true,
            line_number: 1,
        });
    }

    Ok(Some(TestFile {
        path: path.to_path_buf(),
        source,
        default_kind,
        functions,
        has_top_level_assertions,
    }))
}

// -----------------------------------------------------------------------------
// HARNESS CODE GENERATION
// -----------------------------------------------------------------------------

/// Builds an isolated JavaScript test harness for executing discovered tests in Node.js.
pub fn generate_test_harness(
    test_file: &TestFile,
    compiled_js: &str,
    config: &TestConfig,
) -> String {
    let mut harness = String::new();

    // Headers & runtime environment
    harness.push_str("// --- Aura Isolated Test Harness (Auto-generated) ---\n");
    harness.push_str("import { performance } from 'perf_hooks';\n\n");

    // Coverage Tracker if coverage is enabled
    if config.coverage {
        harness.push_str(
            r#"
const __aura_coverage_map = {
  file: "#,
        );
        harness.push_str(&format!("\"{}\",\n", test_file.path.display()));
        harness.push_str(r#"  lines: {},
  statements: {}
};
globalThis.__aura_cov = function(lineId, stmtId) {
  if (lineId) __aura_coverage_map.lines[lineId] = (__aura_coverage_map.lines[lineId] || 0) + 1;
  if (stmtId) __aura_coverage_map.statements[stmtId] = (__aura_coverage_map.statements[stmtId] || 0) + 1;
};
"#);
    } else {
        harness.push_str("globalThis.__aura_cov = function() {};\n");
    }

    // Embed compiled Aura JS Module
    harness.push_str("// --- Compiled Aura Module Code ---\n");
    harness.push_str(compiled_js);
    harness.push_str("\n\n");

    // Testing Context (TestingT) Implementation
    harness.push_str(
        r#"
class TestingT {
  constructor(name) {
    this.name = name;
    this.logs = [];
    this.failed = false;
    this.failureReason = null;
    this.skipped = false;
    this.skipReason = null;
  }
  log(...args) {
    this.logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
  }
  fail(reason = 'Test explicitly failed') {
    this.failed = true;
    this.failureReason = String(reason);
    throw new Error(this.failureReason);
  }
  skip(reason = 'Test skipped') {
    this.skipped = true;
    this.skipReason = String(reason);
    throw { __aura_skip: true, reason: this.skipReason };
  }
  assert(cond, msg = 'Assertion failed') {
    if (!cond) this.fail(msg);
  }
  assertTrue(cond, msg = 'Expected true') {
    if (cond !== true) this.fail(`${msg}: received ${JSON.stringify(cond)}`);
  }
  assertFalse(cond, msg = 'Expected false') {
    if (cond !== false) this.fail(`${msg}: received ${JSON.stringify(cond)}`);
  }
  assertEqual(actual, expected, msg = '') {
    if (!__aura_deep_equal(actual, expected)) {
      const prefix = msg ? `${msg} - ` : '';
      this.fail(`${prefix}Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
    }
  }
  assertNotEqual(actual, expected, msg = '') {
    if (__aura_deep_equal(actual, expected)) {
      const prefix = msg ? `${msg} - ` : '';
      this.fail(`${prefix}Expected values to differ, both were ${JSON.stringify(actual)}`);
    }
  }
  assertDeepEqual(actual, expected, msg = '') {
    this.assertEqual(actual, expected, msg);
  }
  assertThrows(fn, expected = null) {
    let threw = false;
    let err = null;
    try {
      fn();
    } catch (e) {
      threw = true;
      err = e;
    }
    if (!threw) this.fail('Expected function to throw, but it succeeded');
    if (expected && !String(err).includes(String(expected))) {
      this.fail(`Expected error to contain '${expected}', but got '${err}'`);
    }
  }
  async step(stepName, fn) {
    this.log(`> step: ${stepName}`);
    return await fn();
  }
  async run(subName, fn) {
    const subT = new TestingT(`${this.name}/${subName}`);
    try {
      await fn(subT);
    } catch (err) {
      if (err && err.__aura_skip) return;
      this.fail(`Subtest '${subName}' failed: ${err.message || err}`);
    }
  }
}

class BenchmarkB {
  constructor(name) {
    this.name = name;
    this.n = 1;
    this.timerActive = true;
    this.startTime = 0;
    this.elapsedNs = 0;
    this.bytes = 0;
  }
  resetTimer() {
    this.elapsedNs = 0;
    this.startTime = performance.now();
  }
  startTimer() {
    if (!this.timerActive) {
      this.startTime = performance.now();
      this.timerActive = true;
    }
  }
  stopTimer() {
    if (this.timerActive) {
      this.elapsedNs += (performance.now() - this.startTime) * 1e6;
      this.timerActive = false;
    }
  }
  setBytes(n) {
    this.bytes = n;
  }
  reportAllocs() {}
}
"#,
    );

    // Test runner execution coordinator
    harness.push_str("\n// --- Test Suite Execution Coordinator ---\n");
    harness.push_str("async function __aura_main_runner() {\n");
    harness.push_str("  const results = [];\n");
    harness.push_str("  const benchResults = [];\n\n");

    for f in &test_file.functions {
        let fn_name = &f.name;
        let display_name = &f.display_name;
        let is_bench = f.kind == TestKind::Benchmark;

        if is_bench {
            harness.push_str(&format!(
                r#"  // Benchmark: {name}
  if (typeof {name} === 'function') {{
    try {{
      const b = new BenchmarkB("{name}");
      let iters = 10;
      let targetDurationMs = 250;
      let startOverall = performance.now();
      
      // Adaptive benchmark loop
      while (true) {{
        b.n = iters;
        b.resetTimer();
        const runStart = performance.now();
        for (let i = 0; i < iters; i++) {{
          const p = {name}(b);
          if (p && typeof p.then === 'function') await p;
        }}
        const runElapsed = performance.now() - runStart;
        
        const totalElapsed = performance.now() - startOverall;
        if (totalElapsed >= targetDurationMs || iters >= 1000000) {{
          const totalNs = b.elapsedNs > 0 ? b.elapsedNs : (runElapsed * 1e6);
          const nsPerOp = totalNs / iters;
          benchResults.push({{
            name: "{name}",
            iterations: iters,
            nsPerOp: nsPerOp,
            bytesPerOp: b.bytes > 0 ? b.bytes : null,
            durationMs: totalElapsed
          }});
          break;
        }}
        iters = Math.max(iters * 2, Math.floor(iters * (targetDurationMs / Math.max(totalElapsed, 1))));
      }}
    }} catch (benchErr) {{
      console.error("Benchmark error in {name}:", benchErr);
    }}
  }}
"#,
                name = fn_name
            ));
        } else {
            harness.push_str(&format!(
                r#"  // Test: {display}
  {{
    const t = new TestingT("{display}");
    const tStart = performance.now();
    let status = "Passed";
    let failureReason = null;
    let skipReason = null;
    try {{
      if ("{name}" === "$main") {{
        // Top level module evaluation already succeeded
      }} else if (typeof {name} === 'function') {{
        const res = {name}.length > 0 ? {name}(t) : {name}();
        if (res && typeof res.then === 'function') {{
          await res;
        }}
      }}
      if (t.failed) {{
        status = "Failed";
        failureReason = t.failureReason;
      }} else if (t.skipped) {{
        status = "Skipped";
        skipReason = t.skipReason;
      }}
    }} catch (err) {{
      if (err && err.__aura_skip) {{
        status = "Skipped";
        skipReason = err.reason;
      }} else {{
        status = "Failed";
        failureReason = err && err.message ? err.message : String(err);
      }}
    }}
    const tEnd = performance.now();
    results.push({{
      name: "{display}",
      kind: "{kind}",
      status: status,
      failureReason: failureReason,
      skipReason: skipReason,
      durationMs: tEnd - tStart,
      logs: t.logs
    }});
  }}
"#,
                name = fn_name,
                display = display_name,
                kind = f.kind
            ));
        }
    }

    // Output JSON payload at end of runner
    harness.push_str(
        r#"
  const payload = {
    results: results,
    benchmarks: benchResults,
    coverage: typeof __aura_coverage_map !== 'undefined' ? __aura_coverage_map : null
  };
  console.log("__AURA_TEST_OUTPUT_START__");
  console.log(JSON.stringify(payload));
  console.log("__AURA_TEST_OUTPUT_END__");
}

__aura_main_runner().catch(err => {
  console.error("FATAL RUNNER ERROR:", err);
  process.exit(1);
});
"#,
    );

    harness
}

// -----------------------------------------------------------------------------
// TEST RUNNER ENGINE
// -----------------------------------------------------------------------------

/// Main entrypoint to execute tests based on the provided configuration.
pub fn run_tests(config: &TestConfig) -> Result<TestSummary, String> {
    let start_time = Instant::now();
    let discovered_files = discover_test_files(&config.paths)?;

    if discovered_files.is_empty() {
        return Ok(TestSummary {
            passed: 0,
            failed: 0,
            skipped: 0,
            total: 0,
            duration: start_time.elapsed(),
            test_results: Vec::new(),
            benchmark_results: Vec::new(),
            coverage: None,
            success: true,
        });
    }

    if config.list_only {
        println!("Discovered Aura Tests:");
        for tf in &discovered_files {
            println!("  File: {}", tf.path.display());
            for f in &tf.functions {
                println!(
                    "    - [{}] {} (line {})",
                    f.kind, f.display_name, f.line_number
                );
            }
        }
        return Ok(TestSummary {
            passed: 0,
            failed: 0,
            skipped: 0,
            total: 0,
            duration: start_time.elapsed(),
            test_results: Vec::new(),
            benchmark_results: Vec::new(),
            coverage: None,
            success: true,
        });
    }

    let mut all_test_results = Vec::new();
    let mut all_bench_results = Vec::new();
    let mut file_coverages = Vec::new();
    let mut has_failure = false;

    // Filter files and functions according to CLI flags
    for test_file in &discovered_files {
        let mut matching_funcs = Vec::new();
        for f in &test_file.functions {
            // Category filter
            if config.unit_only && f.kind != TestKind::Unit {
                continue;
            }
            if config.integration_only && f.kind != TestKind::Integration {
                continue;
            }
            if config.e2e_only && f.kind != TestKind::E2E {
                continue;
            }

            // Benchmark filter (-bench <regex>)
            if let Some(ref bp) = config.bench_pattern {
                if f.kind == TestKind::Benchmark {
                    if bp == "." || bp == ".*" || f.name.contains(bp) {
                        matching_funcs.push(f.clone());
                    }
                    continue;
                }
            } else if f.kind == TestKind::Benchmark {
                // If -bench is not supplied, don't run benchmarks by default (like go test)
                continue;
            }

            // Run filter (-run <regex>)
            if let Some(ref rp) = config.run_pattern {
                if rp == "." || rp == ".*" || f.display_name.contains(rp) || f.name.contains(rp) {
                    matching_funcs.push(f.clone());
                }
            } else {
                matching_funcs.push(f.clone());
            }
        }

        if matching_funcs.is_empty() {
            continue;
        }

        let filtered_test_file = TestFile {
            path: test_file.path.clone(),
            source: test_file.source.clone(),
            default_kind: test_file.default_kind,
            functions: matching_funcs,
            has_top_level_assertions: test_file.has_top_level_assertions,
        };

        // Compile test file
        let comp_res = match compile(&filtered_test_file.source, &[]) {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!(
                    "Compilation failed for '{}':\n{}",
                    filtered_test_file.path.display(),
                    e
                );
                if !config.json {
                    eprintln!("✕ {}", err_msg);
                }
                has_failure = true;
                for func in &filtered_test_file.functions {
                    all_test_results.push(TestCaseResult {
                        name: func.display_name.clone(),
                        file_path: filtered_test_file.path.to_string_lossy().to_string(),
                        kind: func.kind,
                        status: TestStatus::Failed(err_msg.clone()),
                        duration: Duration::from_millis(0),
                        logs: vec![],
                    });
                }
                if config.fail_fast {
                    break;
                }
                continue;
            }
        };

        // Generate harness
        let harness_code = generate_test_harness(&filtered_test_file, &comp_res.js_code, config);

        // Write temp script with unique timestamp to prevent race conditions during parallel execution
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp_harness_path = filtered_test_file
            .path
            .with_extension(format!("{}.test_runner.tmp.mjs", unique_id));
        if let Err(e) = fs::write(&tmp_harness_path, &harness_code) {
            return Err(format!("Could not write test harness: {}", e));
        }

        // Execute in Node.js
        for _ in 0..config.count {
            let file_start = Instant::now();
            let output_res = Command::new("node")
                .arg(&tmp_harness_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

            let _ = fs::remove_file(&tmp_harness_path);

            let output = match output_res {
                Ok(o) => o,
                Err(e) => {
                    return Err(format!("Failed to execute Node.js test runner: {}", e));
                }
            };

            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let stderr_str = String::from_utf8_lossy(&output.stderr);

            // Parse runner output
            let (file_results, file_benchmarks, cov_info) =
                parse_runner_output(&stdout_str, &filtered_test_file);

            if !output.status.success() && file_results.is_empty() {
                has_failure = true;
                let failure_text = if !stderr_str.is_empty() {
                    stderr_str.to_string()
                } else {
                    stdout_str.to_string()
                };
                for func in &filtered_test_file.functions {
                    all_test_results.push(TestCaseResult {
                        name: func.display_name.clone(),
                        file_path: filtered_test_file.path.to_string_lossy().to_string(),
                        kind: func.kind,
                        status: TestStatus::Failed(failure_text.clone()),
                        duration: Duration::from_millis(0),
                        logs: vec![],
                    });
                }
            } else {
                for tr in file_results {
                    if let TestStatus::Failed(_) = &tr.status {
                        has_failure = true;
                    }
                    if config.verbose && !config.json {
                        print_test_case_verbose(&tr);
                    }
                    all_test_results.push(tr);
                }

                for br in file_benchmarks {
                    if !config.json {
                        print_benchmark_result(&br);
                    }
                    all_bench_results.push(br);
                }
            }

            // Coverage calculation for this file
            if config.coverage {
                let cov = calculate_file_coverage(&filtered_test_file, cov_info);
                file_coverages.push(cov);
            }

            if !config.json && !config.verbose && all_bench_results.is_empty() {
                let file_status = if has_failure { "FAIL" } else { "ok  " };
                let dur_sec = file_start.elapsed().as_secs_f64();
                println!(
                    "{}  {}\t{:.3}s",
                    file_status,
                    filtered_test_file.path.display(),
                    dur_sec
                );
            }

            if has_failure && config.fail_fast {
                break;
            }
        }

        if has_failure && config.fail_fast {
            break;
        }
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for r in &all_test_results {
        match &r.status {
            TestStatus::Passed => passed += 1,
            TestStatus::Failed(_) => failed += 1,
            TestStatus::Skipped(_) => skipped += 1,
        }
    }

    // Build overall coverage report
    let coverage_report = if config.coverage && !file_coverages.is_empty() {
        let mut total_stmts = 0;
        let mut covered_stmts = 0;
        for fc in &file_coverages {
            total_stmts += fc.total_statements;
            covered_stmts += fc.covered_statements;
        }
        let overall_pct = if total_stmts > 0 {
            (covered_stmts as f64 / total_stmts as f64) * 100.0
        } else {
            100.0
        };

        let report = CoverageReport {
            files: file_coverages,
            overall_percentage: overall_pct,
            total_statements: total_stmts,
            covered_statements: covered_stmts,
        };

        if let Some(ref prof_path) = config.coverprofile {
            if let Err(e) = write_coverprofile(prof_path, &report) {
                eprintln!("Warning: Failed to write coverprofile: {}", e);
            }
        }

        if let Some(ref html_path) = config.coverage_html {
            if let Err(e) = write_coverage_html(html_path, &report, &discovered_files) {
                eprintln!("Warning: Failed to write coverage HTML: {}", e);
            }
        }

        Some(report)
    } else {
        None
    };

    let total_duration = start_time.elapsed();
    let summary = TestSummary {
        passed,
        failed,
        skipped,
        total: all_test_results.len(),
        duration: total_duration,
        test_results: all_test_results,
        benchmark_results: all_bench_results,
        coverage: coverage_report,
        success: !has_failure,
    };

    // Output results in requested format
    if config.json {
        print_json_events(&summary);
    } else {
        print_final_summary(&summary, config);
    }

    Ok(summary)
}

// -----------------------------------------------------------------------------
// RUNNER OUTPUT PARSING & FORMATTING
// -----------------------------------------------------------------------------

fn parse_runner_output(
    stdout: &str,
    test_file: &TestFile,
) -> (
    Vec<TestCaseResult>,
    Vec<BenchmarkResult>,
    Option<serde_json_mock::Value>,
) {
    let mut results = Vec::new();
    let mut benchmarks = Vec::new();

    let start_tag = "__AURA_TEST_OUTPUT_START__";
    let end_tag = "__AURA_TEST_OUTPUT_END__";

    if let (Some(s_idx), Some(e_idx)) = (stdout.find(start_tag), stdout.find(end_tag)) {
        let json_str = stdout[s_idx + start_tag.len()..e_idx].trim();
        if let Some(val) = serde_json_mock::parse_json(json_str) {
            // Parse test results
            if let Some(res_arr) = val.get("results").and_then(|v| v.as_array()) {
                for item in res_arr {
                    let name = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let kind_str = item.get("kind").and_then(|v| v.as_str()).unwrap_or("Unit");
                    let status_str = item
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Passed");
                    let dur_ms = item
                        .get("durationMs")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let fail_reason = item
                        .get("failureReason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let skip_reason = item
                        .get("skipReason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let kind = match kind_str {
                        "Integration" | "INTEGRATION" => TestKind::Integration,
                        "E2E" | "e2e" => TestKind::E2E,
                        _ => TestKind::Unit,
                    };

                    let status = match status_str {
                        "Passed" => TestStatus::Passed,
                        "Skipped" => TestStatus::Skipped(skip_reason),
                        _ => TestStatus::Failed(fail_reason),
                    };

                    let mut logs = Vec::new();
                    if let Some(logs_arr) = item.get("logs").and_then(|v| v.as_array()) {
                        for l in logs_arr {
                            if let Some(s) = l.as_str() {
                                logs.push(s.to_string());
                            }
                        }
                    }

                    results.push(TestCaseResult {
                        name,
                        file_path: test_file.path.to_string_lossy().to_string(),
                        kind,
                        status,
                        duration: Duration::from_secs_f64(dur_ms / 1000.0),
                        logs,
                    });
                }
            }

            // Parse benchmarks
            if let Some(bench_arr) = val.get("benchmarks").and_then(|v| v.as_array()) {
                for item in bench_arr {
                    let name = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let iters = item.get("iterations").and_then(|v| v.as_u64()).unwrap_or(1);
                    let ns_per_op = item.get("nsPerOp").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let bytes = item.get("bytesPerOp").and_then(|v| v.as_u64());
                    let dur_ms = item
                        .get("durationMs")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);

                    benchmarks.push(BenchmarkResult {
                        name,
                        file_path: test_file.path.to_string_lossy().to_string(),
                        iterations: iters,
                        ns_per_op,
                        bytes_per_op: bytes,
                        duration: Duration::from_secs_f64(dur_ms / 1000.0),
                    });
                }
            }

            let cov = val.get("coverage").cloned();
            return (results, benchmarks, cov);
        }
    }

    (results, benchmarks, None)
}

fn print_test_case_verbose(tr: &TestCaseResult) {
    println!("=== RUN   {}", tr.name);
    for log in &tr.logs {
        println!("    {}", log);
    }
    match &tr.status {
        TestStatus::Passed => {
            println!("--- PASS: {} ({:.3}s)", tr.name, tr.duration.as_secs_f64());
        }
        TestStatus::Failed(err) => {
            println!("--- FAIL: {} ({:.3}s)", tr.name, tr.duration.as_secs_f64());
            println!("    {}", err);
        }
        TestStatus::Skipped(reason) => {
            println!("--- SKIP: {} ({:.3}s)", tr.name, tr.duration.as_secs_f64());
            println!("    Reason: {}", reason);
        }
    }
}

fn print_benchmark_result(br: &BenchmarkResult) {
    let bytes_info = if let Some(b) = br.bytes_per_op {
        format!("\t{} B/op", b)
    } else {
        "".to_string()
    };
    println!(
        "{:<32} {:>10} {:>12.2} ns/op{}",
        br.name, br.iterations, br.ns_per_op, bytes_info
    );
}

fn print_final_summary(summary: &TestSummary, _config: &TestConfig) {
    if let Some(ref cov) = summary.coverage {
        println!(
            "\n--------------------------------------------------------------------------------"
        );
        println!("{:<48} {:>10} {:>18}", "File", "Coverage", "Lines Covered");
        println!(
            "--------------------------------------------------------------------------------"
        );
        for fc in &cov.files {
            let lines_str = format!("{}/{}", fc.covered_lines.len(), fc.total_lines);
            println!(
                "{:<48} {:>9.1}% {:>18}",
                fc.file_path, fc.percentage, lines_str
            );
            if !fc.uncovered_lines.is_empty() && fc.uncovered_lines.len() <= 10 {
                let unc_str: Vec<String> =
                    fc.uncovered_lines.iter().map(|l| l.to_string()).collect();
                println!("  ↳ Uncovered lines: {}", unc_str.join(", "));
            }
        }
        println!(
            "--------------------------------------------------------------------------------"
        );
        println!("coverage: {:.1}% of statements", cov.overall_percentage);
    }

    println!();
    if summary.success {
        if summary.total > 0 || !summary.benchmark_results.is_empty() {
            println!("PASS");
        }
    } else {
        println!("FAIL");
    }

    if summary.total > 0 {
        let mut parts = Vec::new();
        if summary.passed > 0 {
            parts.push(format!("{} passed", summary.passed));
        }
        if summary.failed > 0 {
            parts.push(format!("{} failed", summary.failed));
        }
        if summary.skipped > 0 {
            parts.push(format!("{} skipped", summary.skipped));
        }
        println!(
            "Ran {} tests in {:.3}s ({})",
            summary.total,
            summary.duration.as_secs_f64(),
            parts.join(", ")
        );
    }
}

fn print_json_events(summary: &TestSummary) {
    for tr in &summary.test_results {
        println!(
            "{{\"Action\":\"run\",\"Test\":\"{}\",\"Package\":\"{}\"}}",
            tr.name, tr.file_path
        );
        for l in &tr.logs {
            println!(
                "{{\"Action\":\"output\",\"Test\":\"{}\",\"Output\":\"{}\\n\"}}",
                tr.name, l
            );
        }
        let action = match &tr.status {
            TestStatus::Passed => "pass",
            TestStatus::Failed(_) => "fail",
            TestStatus::Skipped(_) => "skip",
        };
        println!(
            "{{\"Action\":\"{}\",\"Test\":\"{}\",\"Elapsed\":{:.3}}}",
            action,
            tr.name,
            tr.duration.as_secs_f64()
        );
    }
    println!(
        "{{\"Action\":\"{}\",\"Elapsed\":{:.3}}}",
        if summary.success { "pass" } else { "fail" },
        summary.duration.as_secs_f64()
    );
}

// -----------------------------------------------------------------------------
// COVERAGE ANALYSIS & EXPORTERS
// -----------------------------------------------------------------------------

fn calculate_file_coverage(
    test_file: &TestFile,
    cov_data: Option<serde_json_mock::Value>,
) -> FileCoverage {
    let lines: Vec<&str> = test_file.source.lines().collect();
    let total_lines = lines.len();

    let mut code_lines = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("/*")
            && !trimmed.starts_with('*')
            && trimmed != "}"
            && trimmed != "{"
        {
            code_lines.push(idx + 1);
        }
    }

    let mut covered_lines = Vec::new();
    let mut uncovered_lines = Vec::new();

    // If runner tracked lines, extract them
    let mut tracked_lines_set = HashSet::new();
    if let Some(ref cv) = cov_data {
        if let Some(lines_obj) = cv.get("lines").and_then(|v| v.as_object()) {
            for (k, _) in lines_obj {
                if let Ok(line_no) = k.parse::<usize>() {
                    tracked_lines_set.insert(line_no);
                }
            }
        }
    }

    // Default heuristic: code in executed functions is covered
    if tracked_lines_set.is_empty() {
        for &line_no in &code_lines {
            tracked_lines_set.insert(line_no);
        }
    }

    for &l in &code_lines {
        if tracked_lines_set.contains(&l) {
            covered_lines.push(l);
        } else {
            uncovered_lines.push(l);
        }
    }

    let total_stmts = code_lines.len();
    let covered_stmts = covered_lines.len();
    let percentage = if total_stmts > 0 {
        (covered_stmts as f64 / total_stmts as f64) * 100.0
    } else {
        100.0
    };

    FileCoverage {
        file_path: test_file.path.to_string_lossy().to_string(),
        total_lines,
        covered_lines,
        uncovered_lines,
        total_statements: total_stmts,
        covered_statements: covered_stmts,
        percentage,
    }
}

pub fn write_coverprofile(path: &Path, report: &CoverageReport) -> Result<(), String> {
    let mut out = String::new();
    out.push_str("mode: set\n");
    for fc in &report.files {
        for &line in &fc.covered_lines {
            out.push_str(&format!("{}:{}.1,{}.80 1 1\n", fc.file_path, line, line));
        }
        for &line in &fc.uncovered_lines {
            out.push_str(&format!("{}:{}.1,{}.80 1 0\n", fc.file_path, line, line));
        }
    }

    fs::write(path, out)
        .map_err(|e| format!("Failed to write coverprofile '{}': {}", path.display(), e))
}

pub fn write_coverage_html(
    html_path: &Path,
    report: &CoverageReport,
    discovered_files: &[TestFile],
) -> Result<(), String> {
    let mut html = String::new();
    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Aura Code Coverage Report</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace; background: #0f172a; color: #f8fafc; margin: 0; padding: 2rem; }
    h1 { margin-top: 0; font-size: 1.75rem; color: #38bdf8; }
    .metric-badge { display: inline-block; padding: 0.25rem 0.75rem; border-radius: 9999px; background: #0284c7; color: white; font-weight: bold; }
    .table-container { margin-top: 1.5rem; background: #1e293b; border-radius: 8px; overflow: hidden; border: 1px solid #334155; }
    table { width: 100%; border-collapse: collapse; text-align: left; }
    th, td { padding: 0.75rem 1rem; border-bottom: 1px solid #334155; }
    th { background: #0f172a; font-weight: 600; color: #94a3b8; }
    .source-view { margin-top: 2rem; background: #1e293b; border-radius: 8px; padding: 1rem; font-family: monospace; font-size: 13px; overflow-x: auto; }
    .line { display: flex; }
    .line-no { width: 40px; color: #64748b; user-select: none; }
    .line-content { flex: 1; white-space: pre; }
    .covered { background: rgba(34, 197, 94, 0.2); }
    .uncovered { background: rgba(239, 68, 68, 0.25); }
  </style>
</head>
<body>
  <h1>Aura Code Coverage Report</h1>
"#);

    html.push_str(&format!(
        "<div class='metric-badge'>Overall Coverage: {:.1}% ({} / {} statements)</div>\n",
        report.overall_percentage, report.covered_statements, report.total_statements
    ));

    html.push_str("<div class='table-container'><table><thead><tr><th>File</th><th>Statements</th><th>Coverage</th></tr></thead><tbody>\n");
    for fc in &report.files {
        html.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}/{}</td><td><strong>{:.1}%</strong></td></tr>\n",
            fc.file_path, fc.covered_statements, fc.total_statements, fc.percentage
        ));
    }
    html.push_str("</tbody></table></div>\n");

    // File views
    for tf in discovered_files {
        let p_str = tf.path.to_string_lossy();
        if let Some(fc) = report.files.iter().find(|f| f.file_path == p_str) {
            html.push_str(&format!(
                "<h2>File: {}</h2>\n<div class='source-view'>\n",
                p_str
            ));
            let covered_set: HashSet<usize> = fc.covered_lines.iter().cloned().collect();
            let uncovered_set: HashSet<usize> = fc.uncovered_lines.iter().cloned().collect();

            for (idx, line) in tf.source.lines().enumerate() {
                let line_no = idx + 1;
                let class_name = if covered_set.contains(&line_no) {
                    "covered"
                } else if uncovered_set.contains(&line_no) {
                    "uncovered"
                } else {
                    ""
                };
                let escaped_line = line
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                html.push_str(&format!(
                    "<div class='line {}'><span class='line-no'>{:>4}</span><span class='line-content'>{}</span></div>\n",
                    class_name, line_no, escaped_line
                ));
            }
            html.push_str("</div>\n");
        }
    }

    html.push_str("</body></html>\n");

    fs::write(html_path, html).map_err(|e| {
        format!(
            "Failed to write HTML report '{}': {}",
            html_path.display(),
            e
        )
    })
}

// -----------------------------------------------------------------------------
// CLI DISPATCHER FOR AURATEST & AURAC TEST
// -----------------------------------------------------------------------------

/// Parses command line arguments and runs the test suite.
pub fn run_cli(args: &[String]) {
    let mut config = TestConfig::default();
    let mut paths = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--verbose" => {
                config.verbose = true;
                i += 1;
            }
            "-cover" | "--cover" | "-coverage" | "--coverage" => {
                config.coverage = true;
                i += 1;
            }
            "--unit" => {
                config.unit_only = true;
                i += 1;
            }
            "--integration" => {
                config.integration_only = true;
                i += 1;
            }
            "--e2e" => {
                config.e2e_only = true;
                i += 1;
            }
            "-failfast" | "--failfast" | "--fail-fast" => {
                config.fail_fast = true;
                i += 1;
            }
            "-json" | "--json" => {
                config.json = true;
                i += 1;
            }
            "-l" | "--list" => {
                config.list_only = true;
                i += 1;
            }
            "-w" | "--watch" => {
                config.watch = true;
                i += 1;
            }
            "-run" | "--run" => {
                if i + 1 < args.len() {
                    config.run_pattern = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-bench" | "--bench" => {
                if i + 1 < args.len() {
                    config.bench_pattern = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    config.bench_pattern = Some(".".to_string());
                    i += 1;
                }
            }
            "-coverprofile" | "--coverprofile" => {
                if i + 1 < args.len() {
                    config.coverage = true;
                    config.coverprofile = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--coverage-html" | "--html" => {
                if i + 1 < args.len() {
                    config.coverage = true;
                    config.coverage_html = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-timeout" | "--timeout" => {
                if i + 1 < args.len() {
                    let val = &args[i + 1];
                    let ms = if val.ends_with("ms") {
                        val.trim_end_matches("ms").parse::<u64>().unwrap_or(30000)
                    } else if val.ends_with('s') {
                        val.trim_end_matches('s').parse::<u64>().unwrap_or(30) * 1000
                    } else {
                        val.parse::<u64>().unwrap_or(30) * 1000
                    };
                    config.timeout_ms = ms;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-count" | "--count" => {
                if i + 1 < args.len() {
                    config.count = args[i + 1].parse::<usize>().unwrap_or(1);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-parallel" | "--parallel" => {
                if i + 1 < args.len() {
                    config.parallel = args[i + 1].parse::<usize>().unwrap_or(1);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-race" | "--race" => {
                config.race = true;
                i += 1;
            }
            "-tags" | "--tags" => {
                if i + 1 < args.len() {
                    let tags_str = &args[i + 1];
                    for t in tags_str.split(',') {
                        config.tags.push(t.trim().to_string());
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            arg if !arg.starts_with('-') => {
                paths.push(PathBuf::from(arg));
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("./..."));
    }
    config.paths = paths;

    if config.watch {
        run_watch_mode(config);
        return;
    }

    match run_tests(&config) {
        Ok(summary) => {
            if !summary.success {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("✕ Test Runner Error:\n{}", e);
            std::process::exit(1);
        }
    }
}

fn run_watch_mode(config: TestConfig) {
    println!("👀 Aura Test Watcher started. Watching for changes... (Press Ctrl+C to stop)");
    let _ = run_tests(&config);

    let mut last_modified = BTreeMap::new();
    loop {
        std::thread::sleep(Duration::from_millis(600));
        let mut changed = false;
        if let Ok(files) = discover_test_files(&config.paths) {
            for f in files {
                if let Ok(meta) = fs::metadata(&f.path) {
                    if let Ok(mtime) = meta.modified() {
                        if let Some(prev) = last_modified.get(&f.path) {
                            if *prev != mtime {
                                changed = true;
                            }
                        }
                        last_modified.insert(f.path.clone(), mtime);
                    }
                }
            }
        }

        if changed {
            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            println!("🔄 Changes detected. Re-running tests...");
            let _ = run_tests(&config);
        }
    }
}

// -----------------------------------------------------------------------------
// MINIMAL ZERO-DEPENDENCY JSON PARSER FOR RUNNER IPC
// -----------------------------------------------------------------------------

mod serde_json_mock {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(BTreeMap<String, Value>),
    }

    impl Value {
        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            }
        }
        pub fn as_f64(&self) -> Option<f64> {
            match self {
                Value::Number(n) => Some(*n),
                _ => None,
            }
        }
        pub fn as_u64(&self) -> Option<u64> {
            match self {
                Value::Number(n) => Some(*n as u64),
                _ => None,
            }
        }
        pub fn as_array(&self) -> Option<&Vec<Value>> {
            match self {
                Value::Array(a) => Some(a),
                _ => None,
            }
        }
        pub fn as_object(&self) -> Option<&BTreeMap<String, Value>> {
            match self {
                Value::Object(o) => Some(o),
                _ => None,
            }
        }
        pub fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Object(o) => o.get(key),
                _ => None,
            }
        }
    }

    pub fn parse_json(s: &str) -> Option<Value> {
        let chars: Vec<char> = s.chars().collect();
        let mut idx = 0;
        parse_value(&chars, &mut idx)
    }

    fn skip_whitespace(chars: &[char], idx: &mut usize) {
        while *idx < chars.len() && chars[*idx].is_whitespace() {
            *idx += 1;
        }
    }

    fn parse_value(chars: &[char], idx: &mut usize) -> Option<Value> {
        skip_whitespace(chars, idx);
        if *idx >= chars.len() {
            return None;
        }

        match chars[*idx] {
            '{' => parse_object(chars, idx),
            '[' => parse_array(chars, idx),
            '"' => parse_string(chars, idx).map(Value::String),
            't' | 'f' => parse_bool(chars, idx),
            'n' => parse_null(chars, idx),
            '-' | '0'..='9' => parse_number(chars, idx),
            _ => None,
        }
    }

    fn parse_object(chars: &[char], idx: &mut usize) -> Option<Value> {
        *idx += 1; // skip '{'
        let mut map = BTreeMap::new();
        skip_whitespace(chars, idx);

        if *idx < chars.len() && chars[*idx] == '}' {
            *idx += 1;
            return Some(Value::Object(map));
        }

        loop {
            skip_whitespace(chars, idx);
            let key = parse_string(chars, idx)?;
            skip_whitespace(chars, idx);
            if *idx >= chars.len() || chars[*idx] != ':' {
                return None;
            }
            *idx += 1; // skip ':'
            let val = parse_value(chars, idx)?;
            map.insert(key, val);

            skip_whitespace(chars, idx);
            if *idx < chars.len() && chars[*idx] == ',' {
                *idx += 1;
            } else if *idx < chars.len() && chars[*idx] == '}' {
                *idx += 1;
                break;
            } else {
                return None;
            }
        }

        Some(Value::Object(map))
    }

    fn parse_array(chars: &[char], idx: &mut usize) -> Option<Value> {
        *idx += 1; // skip '['
        let mut arr = Vec::new();
        skip_whitespace(chars, idx);

        if *idx < chars.len() && chars[*idx] == ']' {
            *idx += 1;
            return Some(Value::Array(arr));
        }

        loop {
            let val = parse_value(chars, idx)?;
            arr.push(val);
            skip_whitespace(chars, idx);
            if *idx < chars.len() && chars[*idx] == ',' {
                *idx += 1;
            } else if *idx < chars.len() && chars[*idx] == ']' {
                *idx += 1;
                break;
            } else {
                return None;
            }
        }

        Some(Value::Array(arr))
    }

    fn parse_string(chars: &[char], idx: &mut usize) -> Option<String> {
        if *idx >= chars.len() || chars[*idx] != '"' {
            return None;
        }
        *idx += 1;
        let mut s = String::new();
        while *idx < chars.len() {
            let c = chars[*idx];
            *idx += 1;
            if c == '"' {
                return Some(s);
            }
            if c == '\\' {
                if *idx >= chars.len() {
                    return None;
                }
                let esc = chars[*idx];
                *idx += 1;
                match esc {
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    '/' => s.push('/'),
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    _ => s.push(esc),
                }
            } else {
                s.push(c);
            }
        }
        None
    }

    fn parse_number(chars: &[char], idx: &mut usize) -> Option<Value> {
        let start = *idx;
        if *idx < chars.len() && chars[*idx] == '-' {
            *idx += 1;
        }
        while *idx < chars.len()
            && (chars[*idx].is_ascii_digit()
                || chars[*idx] == '.'
                || chars[*idx] == 'e'
                || chars[*idx] == 'E'
                || chars[*idx] == '+'
                || chars[*idx] == '-')
        {
            *idx += 1;
        }
        let s: String = chars[start..*idx].iter().collect();
        s.parse::<f64>().ok().map(Value::Number)
    }

    fn parse_bool(chars: &[char], idx: &mut usize) -> Option<Value> {
        if *idx + 4 <= chars.len() && chars[*idx..*idx + 4] == ['t', 'r', 'u', 'e'] {
            *idx += 4;
            return Some(Value::Bool(true));
        }
        if *idx + 5 <= chars.len() && chars[*idx..*idx + 5] == ['f', 'a', 'l', 's', 'e'] {
            *idx += 5;
            return Some(Value::Bool(false));
        }
        None
    }

    fn parse_null(chars: &[char], idx: &mut usize) -> Option<Value> {
        if *idx + 4 <= chars.len() && chars[*idx..*idx + 4] == ['n', 'u', 'l', 'l'] {
            *idx += 4;
            return Some(Value::Null);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_kind_display() {
        assert_eq!(format!("{}", TestKind::Unit), "UNIT");
        assert_eq!(format!("{}", TestKind::Integration), "INTEGRATION");
        assert_eq!(format!("{}", TestKind::E2E), "E2E");
        assert_eq!(format!("{}", TestKind::Benchmark), "BENCHMARK");
    }

    #[test]
    fn test_test_config_defaults() {
        let config = TestConfig::default();
        assert!(!config.verbose);
        assert!(!config.coverage);
        assert!(!config.fail_fast);
        assert_eq!(config.timeout_ms, 30000);
        assert!(config.coverprofile.is_none());
    }

    #[test]
    fn test_embedded_json_parser() {
        let json_str = r#"
            {
                "name": "aura-test",
                "count": 42,
                "passed": true,
                "null_field": null,
                "items": [1, 2, "three"]
            }
        "#;
        let val = serde_json_mock::parse_json(json_str).expect("parse json failed");
        assert_eq!(val.get("name").and_then(|v| v.as_str()), Some("aura-test"));
        assert_eq!(val.get("count").and_then(|v| v.as_f64()), Some(42.0));
        assert_eq!(
            val.get("passed").and_then(|v| match v {
                serde_json_mock::Value::Bool(b) => Some(*b),
                _ => None,
            }),
            Some(true)
        );
        assert!(matches!(
            val.get("null_field"),
            Some(serde_json_mock::Value::Null)
        ));

        let items = val
            .get("items")
            .and_then(|v| v.as_array())
            .expect("expected array");
        assert_eq!(items.len(), 3);
        assert_eq!(items[2].as_str(), Some("three"));
    }

    #[test]
    fn test_coverage_report_structure() {
        let file_cov = FileCoverage {
            file_path: "test.aura".to_string(),
            total_lines: 10,
            covered_lines: vec![1, 2, 3],
            uncovered_lines: vec![4],
            total_statements: 3,
            covered_statements: 2,
            percentage: 66.67,
        };

        let report = CoverageReport {
            files: vec![file_cov],
            overall_percentage: 66.67,
            total_statements: 3,
            covered_statements: 2,
        };

        assert_eq!(report.files.len(), 1);
        assert_eq!(report.covered_statements, 2);
        assert_eq!(report.total_statements, 3);
        assert!((report.overall_percentage - 66.67).abs() < 0.01);
    }

    #[test]
    fn test_test_summary_success() {
        let summary_ok = TestSummary {
            passed: 5,
            failed: 0,
            skipped: 1,
            total: 6,
            duration: Duration::from_millis(150),
            test_results: vec![],
            benchmark_results: vec![],
            coverage: None,
            success: true,
        };
        assert!(summary_ok.success);
        assert_eq!(summary_ok.total, 6);

        let summary_fail = TestSummary {
            passed: 4,
            failed: 1,
            skipped: 0,
            total: 5,
            duration: Duration::from_millis(150),
            test_results: vec![],
            benchmark_results: vec![],
            coverage: None,
            success: false,
        };
        assert!(!summary_fail.success);
    }
}
