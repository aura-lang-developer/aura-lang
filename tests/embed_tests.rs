use aura_lang::compile;
use std::fs;
use std::process::Command;

#[test]
fn test_embed_static_text_and_binary_assets() {
    fs::create_dir_all("target/test_assets").unwrap();
    let text_path = "target/test_assets/hello.txt";
    let bin_path = "target/test_assets/data.bin";

    fs::write(
        text_path,
        "Hello from embedded file! \nWith special \"quotes\" and Unicode 🚀",
    )
    .unwrap();
    fs::write(bin_path, &[0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03]).unwrap();

    let source = format!(
        r#"
export fn getEmbeddedText(): String => {{
    return embed("{}");
}};

export fn getEmbeddedBytes(): List<Int> => {{
    return embedBytes("{}");
}};
"#,
        text_path, bin_path
    );

    let result = compile(&source, &[]).expect("Embedding assets should compile");
    assert!(result.js_code.contains("Hello from embedded file!"));
    assert!(
        result
            .js_code
            .contains("new Uint8Array([222, 173, 190, 239, 1, 2, 3])")
    );

    let temp_js_path = "dist/test_embed_e2e.mjs";
    fs::create_dir_all("dist").unwrap();
    fs::write(temp_js_path, &result.js_code).unwrap();

    let runner_script = format!(
        r#"
import {{ getEmbeddedText, getEmbeddedBytes }} from './test_embed_e2e.mjs';

const text = getEmbeddedText();
console.log("Embedded text:", text);
if (!text.includes("Hello from embedded file!") || !text.includes("🚀")) {{
    console.error("Embedded text content mismatch");
    process.exit(1);
}}

const bytes = getEmbeddedBytes();
console.log("Embedded bytes length:", bytes.length, "content:", bytes);
if (bytes.length !== 7 || bytes[0] !== 222 || bytes[1] !== 173 || bytes[2] !== 190 || bytes[3] !== 239) {{
    console.error("Embedded bytes content mismatch");
    process.exit(2);
}}

process.exit(0);
"#
    );

    let script_path = "dist/run_embed_check.mjs";
    fs::write(script_path, runner_script).unwrap();

    let output = Command::new("node")
        .arg(script_path)
        .output()
        .expect("Failed to run node");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!(
            "Node execution failed:\nSTDOUT: {}\nSTDERR: {}",
            stdout, stderr
        );
    }

    assert!(output.status.success());
}
