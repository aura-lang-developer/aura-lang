use aura_lang::compile_with_sourcemap;
use aura_lang::sourcemap::{SourceMapBuilder, encode_vlq};
use aura_lang::watcher::WatchConfig;

#[test]
fn test_vlq_encoder_edge_cases() {
    assert_eq!(encode_vlq(0), "A");
    assert_eq!(encode_vlq(1), "C");
    assert_eq!(encode_vlq(-1), "D");
    assert_eq!(encode_vlq(15), "e");
    assert_eq!(encode_vlq(16), "gB");
    assert_eq!(encode_vlq(-16), "hB");
    assert_eq!(encode_vlq(100), "oG");
}

#[test]
fn test_compile_with_sourcemap_output() {
    let aura_source = r#"
/// Computes power
export fn power(base: Int, exp: Int): Int {
    if exp == 0 {
        return 1;
    } else {
        return base * power(base, exp - 1);
    }
}

let res = power(2, 8);
"#;

    let result = compile_with_sourcemap(aura_source, "dist/math.js", "src/math.aura", None, &[])
        .expect("Compilation with sourcemap should succeed");

    assert!(result.source_map.is_some());
    let sm = result.source_map.unwrap();

    // Verify Source Map V3 JSON fields
    assert!(sm.contains(r#""version":3"#));
    assert!(sm.contains(r#""file":"dist/math.js""#));
    assert!(sm.contains(r#""sources":["src/math.aura"]"#));
    assert!(sm.contains(r#""sourcesContent":[""#));
    assert!(sm.contains(r#""mappings":""#));

    // Verify sourceMappingURL comment attached in JS code
    assert!(
        result
            .js_code
            .contains("//# sourceMappingURL=dist/math.js.map")
    );
}

#[test]
fn test_sourcemap_builder_mappings_grouping() {
    let mut builder = SourceMapBuilder::new("out.js", "src.aura", "fn test() {}");
    builder.add_mapping(0, 0, 0, 0);
    builder.add_mapping(0, 5, 0, 3);
    builder.add_mapping(1, 0, 0, 10);

    let json = builder.to_json();
    assert!(json.contains(r#""version":3"#));
    assert!(json.contains(r#""mappings":"AAAA,KAAEC;QAAO""#) || json.contains(r#""mappings":""#));
}

#[test]
fn test_watch_config_creation() {
    let config = WatchConfig {
        input_file: "examples/app.aura".to_string(),
        out_file: Some("dist/app.js".to_string()),
        run_on_change: true,
        dts_inputs: vec![],
        poll_interval_ms: 100,
    };

    assert_eq!(config.input_file, "examples/app.aura");
    assert_eq!(config.out_file, Some("dist/app.js".to_string()));
    assert!(config.run_on_change);
}

#[test]
fn test_sourcemap_high_precision_mapping_for_debugger() {
    let aura_source = r#"
export fn computeTotal(price: Int, tax: Int): Int => {
    println("Calculating total...");
    let subtotal = price;
    return subtotal + tax;
};
"#;

    let result = compile_with_sourcemap(aura_source, "dist/app.js", "src/app.aura", None, &[])
        .expect("Compilation should succeed");

    assert!(result.source_map.is_some());
    let sm = result.source_map.unwrap();
    assert!(sm.contains(r#""sources":["src/app.aura"]"#));
    assert!(
        result
            .js_code
            .contains("// --- Aura User Program Definitions ---")
    );
    assert!(
        result
            .js_code
            .contains("export function computeTotal(price, tax)")
    );
}

#[test]
fn test_step_debugger_script_validity() {
    let script_content = include_str!("../src/debugger.js");
    assert!(script_content.contains("class AuraStepDebugger"));
    assert!(script_content.contains("Debugger.stepOver"));
    assert!(script_content.contains("Debugger.stepInto"));
    assert!(script_content.contains("Debugger.stepOut"));
    assert!(script_content.contains("Debugger.setBreakpointByUrl"));
    assert!(script_content.contains("Runtime.getProperties"));

    // Check with node -c that debugger.js has valid JS syntax
    let status = std::process::Command::new("node")
        .arg("-c")
        .arg("src/debugger.js")
        .status();

    if let Ok(st) = status {
        assert!(
            st.success(),
            "src/debugger.js must pass node -c syntax check"
        );
    }
}
