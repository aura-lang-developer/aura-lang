//! Dedicated CLI Entrypoint for `auratest` (Aura Language Test Runner).

use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Aura Test Runner (auratest) v0.1.0 (Go-test style)");
        println!("Usage:");
        println!("  auratest [flags] [packages/paths...]");
        println!();
        println!("Flags:");
        println!("  -v, --verbose           Verbose output: print all test names and logs");
        println!("  -run <pattern>          Run only tests matching the regular expression");
        println!("  -bench <pattern>        Run benchmarks matching the regular expression");
        println!("  -cover, --coverage      Enable code coverage analysis");
        println!("  -coverprofile <file>    Write coverage profile to file");
        println!("  --coverage-html <file>  Generate standalone interactive HTML coverage report");
        println!("  --unit                  Run only unit tests");
        println!("  --integration           Run only integration tests");
        println!("  --e2e                   Run only end-to-end (E2E) tests");
        println!("  -timeout <duration>     Test execution timeout (e.g. 30s, 5000ms)");
        println!("  -failfast               Stop execution on first test failure");
        println!("  -parallel <n>           Number of test files to run in parallel");
        println!("  -count <n>              Run each test and benchmark n times");
        println!("  -json                   Output test results in machine-readable JSON");
        println!("  -l, --list              List discovered tests without running them");
        println!("  -w, --watch             Watch mode: re-run tests on file changes");
        println!();
        println!("Examples:");
        println!("  auratest ./...");
        println!("  auratest -v ./examples/tests/math_unit_test.aura");
        println!("  auratest -run TestCalculateDiscount -cover");
        println!("  auratest -bench .");
        println!("  auratest --integration");
        return;
    }

    aura_lang::testing::run_cli(&args);
}
