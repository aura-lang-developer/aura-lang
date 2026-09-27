use aura_lang::build_tags::should_build_with_exact_tags;
use std::collections::HashSet;

#[test]
fn test_build_tags_go_and_aura_syntax() {
    let source_aura = r#"//aura:build darwin || linux
let os = "unix";
"#;
    let darwin_tag: HashSet<String> = vec!["darwin".to_string()].into_iter().collect();
    let linux_tag: HashSet<String> = vec!["linux".to_string()].into_iter().collect();
    let windows_tag: HashSet<String> = vec!["windows".to_string()].into_iter().collect();

    assert!(should_build_with_exact_tags(source_aura, &darwin_tag));
    assert!(should_build_with_exact_tags(source_aura, &linux_tag));
    assert!(!should_build_with_exact_tags(source_aura, &windows_tag));

    let source_go = r#"//go:build (linux && amd64) || (darwin && arm64)
let arch = "supported";
"#;
    let mac_arm: HashSet<String> = vec!["darwin".to_string(), "arm64".to_string()]
        .into_iter()
        .collect();
    let mac_intel: HashSet<String> = vec!["darwin".to_string(), "amd64".to_string()]
        .into_iter()
        .collect();
    let linux_intel: HashSet<String> = vec!["linux".to_string(), "amd64".to_string()]
        .into_iter()
        .collect();

    assert!(should_build_with_exact_tags(source_go, &mac_arm));
    assert!(!should_build_with_exact_tags(source_go, &mac_intel));
    assert!(should_build_with_exact_tags(source_go, &linux_intel));
}

#[test]
fn test_build_tags_negation() {
    let source = r#"//aura:build !windows
let nonWindows = true;
"#;
    let mac_tag: HashSet<String> = vec!["darwin".to_string()].into_iter().collect();
    let win_tag: HashSet<String> = vec!["windows".to_string()].into_iter().collect();

    assert!(should_build_with_exact_tags(source, &mac_tag));
    assert!(!should_build_with_exact_tags(source, &win_tag));
}
