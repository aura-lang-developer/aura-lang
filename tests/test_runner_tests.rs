use aura_lang::testing::{TestConfig, discover_test_files, run_tests};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_discover_test_files_and_categories() {
    let paths = vec![PathBuf::from("examples/tests")];
    let files = discover_test_files(&paths).expect("Discovery should succeed");

    assert!(
        !files.is_empty(),
        "Should discover test files in examples/tests"
    );

    let has_unit = files
        .iter()
        .any(|f| f.path.to_string_lossy().contains("math_unit_test.aura"));
    let has_integration = files.iter().any(|f| {
        f.path
            .to_string_lossy()
            .contains("order_integration_test.aura")
    });
    let has_e2e = files
        .iter()
        .any(|f| f.path.to_string_lossy().contains("api_e2e_test.aura"));

    assert!(has_unit, "Should discover math_unit_test.aura");
    assert!(
        has_integration,
        "Should discover order_integration_test.aura"
    );
    assert!(has_e2e, "Should discover api_e2e_test.aura");
}

#[test]
fn test_run_unit_tests_with_assertions() {
    let mut config = TestConfig::default();
    config.paths = vec![PathBuf::from("examples/tests/math_unit_test.aura")];
    config.verbose = true;

    let summary = run_tests(&config).expect("Test execution should succeed");
    assert!(summary.success, "All unit tests should pass");
    assert!(
        summary.passed >= 3,
        "At least 3 unit tests should pass (addition, factorial, global_assertions)"
    );
    assert_eq!(summary.failed, 0, "No unit tests should fail");
}

#[test]
fn test_run_integration_tests_filter() {
    let mut config = TestConfig::default();
    config.paths = vec![PathBuf::from("examples/tests/order_integration_test.aura")];
    config.integration_only = true;

    let summary = run_tests(&config).expect("Integration tests execution should succeed");
    assert!(summary.success, "Integration tests should pass");
    assert!(
        summary.passed >= 2,
        "Should pass order discount and pattern matching tests"
    );
}

#[test]
fn test_run_e2e_async_and_csp_tests() {
    let mut config = TestConfig::default();
    config.paths = vec![PathBuf::from("examples/tests/api_e2e_test.aura")];
    config.e2e_only = true;

    let summary = run_tests(&config).expect("E2E tests execution should succeed");
    assert!(summary.success, "E2E async and CSP tests should pass");
    assert!(
        summary.passed >= 2,
        "Should pass async fetch and CSP channel tests"
    );
}

#[test]
fn test_run_filter_pattern() {
    let mut config = TestConfig::default();
    config.paths = vec![PathBuf::from("examples/tests/math_unit_test.aura")];
    config.run_pattern = Some("test_addition".to_string());

    let summary = run_tests(&config).expect("Filtered test execution should succeed");
    assert!(summary.success);
    assert_eq!(summary.total, 1, "Only test_addition should run");
    assert_eq!(summary.passed, 1);
}

#[test]
fn test_run_benchmarks() {
    let mut config = TestConfig::default();
    config.paths = vec![PathBuf::from("examples/tests/math_unit_test.aura")];
    config.bench_pattern = Some(".".to_string());

    let summary = run_tests(&config).expect("Benchmark execution should succeed");
    assert!(
        !summary.benchmark_results.is_empty(),
        "Should collect benchmark results"
    );
    for bench in &summary.benchmark_results {
        assert!(bench.iterations > 0, "Iterations must be > 0");
        assert!(bench.ns_per_op > 0.0, "ns/op must be positive");
    }
}

#[test]
fn test_coverage_calculation_and_export() {
    let coverprofile_path = PathBuf::from("target/test_coverage.out");
    let html_path = PathBuf::from("target/test_coverage.html");

    let mut config = TestConfig::default();
    config.paths = vec![PathBuf::from("examples/tests/math_unit_test.aura")];
    config.coverage = true;
    config.coverprofile = Some(coverprofile_path.clone());
    config.coverage_html = Some(html_path.clone());

    let summary = run_tests(&config).expect("Coverage run should succeed");
    assert!(
        summary.coverage.is_some(),
        "Coverage report should be generated"
    );

    let cov = summary.coverage.as_ref().unwrap();
    assert!(
        cov.overall_percentage > 0.0,
        "Coverage percentage should be > 0"
    );
    assert!(cov.total_statements > 0, "Statements count should be > 0");

    assert!(
        coverprofile_path.exists(),
        "coverprofile file should be written"
    );
    let profile_content = fs::read_to_string(&coverprofile_path).unwrap();
    assert!(
        profile_content.starts_with("mode: set"),
        "Profile header should match Go test format"
    );

    assert!(html_path.exists(), "HTML coverage report should be written");
    let html_content = fs::read_to_string(&html_path).unwrap();
    assert!(
        html_content.contains("Aura Code Coverage Report"),
        "HTML should contain title"
    );

    let _ = fs::remove_file(coverprofile_path);
    let _ = fs::remove_file(html_path);
}

#[test]
fn test_test_failure_reporting() {
    let source = r#"
export fn test_failing_assertion(t: TestingT) => {
  t.assertEqual(10, 20, "10 should equal 20");
}
"#;
    let temp_file = PathBuf::from("target/failing_test.aura");
    fs::write(&temp_file, source).unwrap();

    let mut config = TestConfig::default();
    config.paths = vec![temp_file.clone()];

    let summary = run_tests(&config).expect("Runner should execute");
    assert!(!summary.success, "Suite should fail");
    assert_eq!(summary.failed, 1, "Should record 1 failure");

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_test_skip_reporting() {
    let source = r#"
export fn test_skipped_case(t: TestingT) => {
  t.skip("Feature not yet supported on this platform");
  t.assertEqual(1, 2, "Will not be reached");
}
"#;
    let temp_file = PathBuf::from("target/skipped_test.aura");
    fs::write(&temp_file, source).unwrap();

    let mut config = TestConfig::default();
    config.paths = vec![temp_file.clone()];

    let summary = run_tests(&config).expect("Runner should execute");
    assert!(
        summary.success,
        "Skipped test should not cause suite failure"
    );
    assert_eq!(summary.skipped, 1, "Should record 1 skip");

    let _ = fs::remove_file(temp_file);
}
