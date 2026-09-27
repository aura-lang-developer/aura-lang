use aura_lang::compile;
use std::process::Command;

#[test]
fn test_labeled_loops_compilation() {
    let source = r#"
export fn findMatrixTarget(): Option<Int> => {
    let matrix = [[1, 2, 3], [4, 99, 6], [7, 8, 9]];
    let mut found = None;

    outer: for (r, row) in matrix {
        for (c, val) in row {
            if val == 99 {
                found = Some(val);
                break outer;
            }
        }
    }

    return found;
};
"#;

    let result = compile(source, &[]).expect("Labeled loops should compile");
    assert!(result.js_code.contains("outer: for") || result.js_code.contains("outer: for await"));
    assert!(result.js_code.contains("break outer;"));
}

#[test]
fn test_e2e_labeled_loops_execution_with_node() {
    let source = r#"
export fn runLabeledBreakTest(): Int => {
    let mut sum = 0;
    outer: for i in [1, 2, 3, 4, 5] {
        for j in [10, 20, 30] {
            if i == 3 && j == 20 {
                break outer;
            }
            sum = sum + 1;
        }
    }
    return sum;
};

export fn runLabeledContinueTest(): Int => {
    let mut iterations = 0;
    outer: for i in [1, 2, 3] {
        for j in [1, 2, 3] {
            if j == 2 {
                continue outer;
            }
            iterations = iterations + 1;
        }
    }
    return iterations;
};

export fn runLabeledWhileTest(): Int => {
    let mut i = 0;
    let mut count = 0;
    outerLoop: while i < 5 {
        i = i + 1;
        let mut j = 0;
        while j < 5 {
            j = j + 1;
            if i == 2 && j == 2 {
                break outerLoop;
            }
            count = count + 1;
        }
    }
    return count;
};
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    let temp_js_path = "dist/test_labeled_loops_e2e.mjs";
    std::fs::create_dir_all("dist").unwrap();
    std::fs::write(temp_js_path, &result.js_code).unwrap();

    let runner_script = format!(
        r#"
import {{ runLabeledBreakTest, runLabeledContinueTest, runLabeledWhileTest }} from './test_labeled_loops_e2e.mjs';

// 1. Labeled break test:
// i=1 (j=10, 20, 30 -> 3 steps)
// i=2 (j=10, 20, 30 -> 3 steps)
// i=3 (j=10 -> 1 step; at j=20 break outer)
// Total = 3 + 3 + 1 = 7 steps
const breakRes = runLabeledBreakTest();
console.log("Labeled break result:", breakRes);
if (breakRes !== 7) {{
    console.error("Expected 7 from labeled break test, got:", breakRes);
    process.exit(1);
}}

// 2. Labeled continue test:
// i=1 (j=1 -> 1 step; at j=2 continue outer)
// i=2 (j=1 -> 1 step; at j=2 continue outer)
// i=3 (j=1 -> 1 step; at j=2 continue outer)
// Total = 3 steps
const continueRes = runLabeledContinueTest();
console.log("Labeled continue result:", continueRes);
if (continueRes !== 3) {{
    console.error("Expected 3 from labeled continue test, got:", continueRes);
    process.exit(2);
}}

// 3. Labeled while test:
// i=1 (j=1..5 -> 5 steps)
// i=2 (j=1 -> 1 step; at j=2 break outerLoop)
// Total = 5 + 1 = 6 steps
const whileRes = runLabeledWhileTest();
console.log("Labeled while result:", whileRes);
if (whileRes !== 6) {{
    console.error("Expected 6 from labeled while test, got:", whileRes);
    process.exit(3);
}}

process.exit(0);
"#
    );

    let script_path = "dist/run_labeled_loops_check.mjs";
    std::fs::write(script_path, runner_script).unwrap();

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
