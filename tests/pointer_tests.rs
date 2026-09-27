use aura_lang::compile;
use std::fs;
use std::process::Command;

#[test]
fn test_pointer_typecheck_and_codegen() {
    let source = r#"
        fn increment(p: *Int) {
            *p = *p + 1;
        }

        export fn run(): Int {
            let mut x = 10;
            increment(&x);
            return x;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of pointer program failed");
    assert!(res.js_code.contains("new Pointer("));
    assert!(res.js_code.contains(".value"));
}

#[test]
fn test_e2e_pointer_mutation_with_node() {
    let source = r#"
        fn swap(a: *Int, b: *Int) {
            let temp = *a;
            *a = *b;
            *b = temp;
        }

        export fn main(): Int {
            let mut x = 42;
            let mut y = 99;
            swap(&x, &y);
            assert(x == 99, "x should be 99");
            assert(y == 42, "y should be 42");
            return x + y;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nconst sum = main();\nif (sum !== 141) process.exit(1);\n");

    let tmp_file = "test_pointer_exec.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(status.success(), "Node execution of pointer swap failed");
}
