use aura_lang::compile;
use std::fs;
use std::process::Command;

#[test]
fn test_panic_recover_typechecking_and_codegen() {
    let source = r#"
        export fn safeDivide(a: Int, b: Int): Int {
            defer {
                let r = recover();
                if (r != None) {
                    println("Recovered from panic successfully");
                }
            };

            if (b == 0) {
                panic("division by zero");
            }
            return a / b;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of panic/recover failed");
    assert!(res.js_code.contains("panic("));
    assert!(res.js_code.contains("recover()"));
}

#[test]
fn test_e2e_panic_and_recover_with_node() {
    let source = r#"
        export fn testRecovery(): String {
            let mut result = "initial";
            let f = () => {
                defer {
                    let err = recover();
                    match err {
                        Some(msg) => {
                            result = `caught: ${msg}`;
                        },
                        None => ()
                    }
                };

                panic("critical database failure");
            };

            f();
            return result;
        }

        export fn main() {
            let status = testRecovery();
            assert(status == "caught: critical database failure", "Panic was not recovered");
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nmain();\n");

    let tmp_file = "test_panic_recover_exec.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(status.success(), "Node execution of panic/recover failed");
}
