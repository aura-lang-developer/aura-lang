use aura_lang::ast::Expr;
use aura_lang::compile;
use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use std::fs;
use std::process::Command;

#[test]
fn test_routine_lexing_and_parsing() {
    let source = r#"
        export fn main(): Unit => {
            let ch = Channel.make<Int>(5);
            routine {
                ch <- 42;
            };
            routine worker(ch);
        };
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing routine should succeed");
    let mut parser = Parser::new(tokens);
    let module = parser
        .parse_module()
        .expect("Parsing routine should succeed");

    // Verify AST contains Expr::Spawn for routine expressions
    let mut routine_count = 0;
    for item in &module.items {
        if let aura_lang::ast::Item::Function(func) = item {
            if let Expr::Block(stmts) = &func.body {
                for stmt in stmts {
                    if let aura_lang::ast::Statement::Expr(Expr::Spawn(_)) = stmt {
                        routine_count += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        routine_count, 2,
        "Expected 2 Expr::Spawn from routine expressions"
    );
}

#[test]
fn test_routine_typecheck_and_compilation() {
    let source = r#"
        export fn producer(ch: SendChannel<String>) => {
            routine {
                ch <- "Hello from Aura Routine!";
                Channel.close(ch);
            };
        };

        export fn consumer(ch: RecvChannel<String>): Task<Option<String>, String> => {
            let msg = <-ch;
            return msg;
        };
    "#;

    let res = compile(source, &[]).expect("Compilation of routine should succeed");
    assert!(res.js_code.contains("spawn(async () => {"));
    assert!(res.dts_code.contains("export declare function producer"));
}

#[test]
fn test_e2e_aura_routines_execution_with_node() {
    let source = r#"
        export fn runRoutinesDemo(): Task<Int, String> => {
            let ch = Channel.make<Int>(10);
            let wg = WaitGroup.new();
            wg.add(3);

            // Routine 1: Producer A
            routine {
                ch <- 10;
                wg.done();
            };

            // Routine 2: Producer B
            routine {
                ch <- 20;
                wg.done();
            };

            // Routine 3: Producer C
            routine {
                ch <- 30;
                wg.done();
            };

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

    let res = compile(source, &[]).expect("Compilation of e2e routine test must succeed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str(
        r#"
runRoutinesDemo().then(sum => {
    if (sum !== 60) {
        console.error("Expected sum 60 from routines, got:", sum);
        process.exit(1);
    }
    console.log("Aura routines executed successfully with sum:", sum);
    process.exit(0);
}).catch(err => {
    console.error("Error running routines demo:", err);
    process.exit(2);
});
"#,
    );

    let tmp_file = "test_routine_exec.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(status.success(), "Node execution of Aura routines failed");
}
