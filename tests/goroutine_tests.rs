use aura_lang::ast::Expr;
use aura_lang::compile;
use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use std::fs;
use std::process::Command;

#[test]
fn test_goroutine_lexing_and_parsing() {
    let source = r#"
        export fn main(): Unit => {
            let ch = Channel.make<Int>(5);
            
            // 1. Direct block goroutine
            go {
                ch <- 42;
            };

            // 2. Direct function call goroutine
            go worker(ch);

            // 3. Anonymous function goroutine: exact Go 'go func() { ... }()' syntax
            go (fn() => {
                ch <- 100;
            })();
        };
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing goroutine should succeed");
    let mut parser = Parser::new(tokens);
    let module = parser
        .parse_module()
        .expect("Parsing goroutine should succeed");

    // Verify AST contains Expr::Spawn for all 3 goroutines
    let mut go_count = 0;
    for item in &module.items {
        if let aura_lang::ast::Item::Function(func) = item {
            if let Expr::Block(stmts) = &func.body {
                for stmt in stmts {
                    if let aura_lang::ast::Statement::Expr(Expr::Spawn(_)) = stmt {
                        go_count += 1;
                    }
                }
            }
        }
    }
    assert_eq!(go_count, 3, "Expected 3 Expr::Spawn from 'go' expressions");
}

#[test]
fn test_e2e_goroutines_execution_with_node() {
    let source = r#"
        export fn runGoroutinesDemo(): Task<Int, String> => {
            let ch = Channel.make<Int>(10);
            let wg = WaitGroup.new();
            wg.add(3);

            // Goroutine 1: go block
            go {
                ch <- 10;
                wg.done();
            };

            // Goroutine 2: go block
            go {
                ch <- 20;
                wg.done();
            };

            // Goroutine 3: exact Go-style 'go func() { ... }()'
            go (fn() => {
                ch <- 30;
                wg.done();
            })();

            let mut sum = 0;
            let m1 = <-ch;
            match m1 {
                Some(v) => { sum = sum + v; },
                None => ()
            };

            let m2 = <-ch;
            match m2 {
                Some(v) => { sum = sum + v; },
                None => ()
            };

            let m3 = <-ch;
            match m3 {
                Some(v) => { sum = sum + v; },
                None => ()
            };

            await wg.wait();
            return sum;
        };
    "#;

    let res = compile(source, &[]).expect("Compilation of goroutine e2e test must succeed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str(
        r#"
runGoroutinesDemo().then(sum => {
    if (sum !== 60) {
        console.error("Expected sum 60 from goroutines, got:", sum);
        process.exit(1);
    }
    console.log("Goroutines executed successfully with sum:", sum);
    process.exit(0);
}).catch(err => {
    console.error("Error running goroutines demo:", err);
    process.exit(2);
});
"#,
    );

    let tmp_file = "test_goroutine_exec.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(status.success(), "Node execution of Goroutines failed");
}
