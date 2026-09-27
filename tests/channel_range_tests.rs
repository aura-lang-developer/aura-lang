use aura_lang::compile;
use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use aura_lang::typechecker::TypeChecker;
use std::process::Command;

#[test]
fn test_directional_channels_typechecking_and_subtyping() {
    let valid_source = r#"
export fn produce(outCh: SendChannel<Int>) => {
    outCh <- 100;
    Channel.close(outCh);
};

export fn consume(inCh: RecvChannel<Int>): Task<Option<Int>, String> => {
    let val = <-inCh;
    return val;
};

export fn testPipeline(): Task<Unit, String> => {
    let ch = Channel.make<Int>(5);
    produce(ch); // Subtyping: Channel<Int> satisfies SendChannel<Int>
    let res = await consume(ch); // Subtyping: Channel<Int> satisfies RecvChannel<Int>
    println("Pipeline done");
};
"#;

    let result = compile(valid_source, &[]).expect("Directional channel subtyping should compile");
    assert!(result.dts_code.contains("SendChannel<number>"));
    assert!(result.dts_code.contains("RecvChannel<number>"));
}

#[test]
fn test_directional_channels_rejection_of_invalid_operations() {
    // 1. Cannot send to a RecvChannel
    let invalid_send = r#"
fn invalidSend(inCh: RecvChannel<Int>) => {
    inCh <- 42;
};
"#;
    let mut lexer = Lexer::new(invalid_send);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();
    let mut checker = TypeChecker::new();
    let err = checker.check_module(&module).unwrap_err();
    assert!(err.contains("Cannot send to a receive-only channel"));

    // 2. Cannot receive from a SendChannel
    let invalid_recv = r#"
fn invalidRecv(outCh: SendChannel<Int>) => {
    let x = <-outCh;
};
"#;
    let mut lexer = Lexer::new(invalid_recv);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();
    let mut checker = TypeChecker::new();
    let err = checker.check_module(&module).unwrap_err();
    assert!(err.contains("Cannot receive from a send-only channel"));
}

#[test]
fn test_e2e_channel_for_in_range_iteration_with_node() {
    let source = r#"
export fn runChannelRangeTest(): Task<List<Int>, String> => {
    let ch = Channel.make<Int>(10);
    let mut collected: List<Int> = [];

    spawn {
        ch <- 10;
        ch <- 20;
        ch <- 30;
        ch <- 40;
        Channel.close(ch);
    };

    for item in ch {
        collected.push(item);
    };

    return collected;
};
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    assert!(result.js_code.contains("Symbol.asyncIterator"));

    let temp_js_path = "dist/test_channel_range_e2e.mjs";
    std::fs::create_dir_all("dist").unwrap();
    std::fs::write(temp_js_path, &result.js_code).unwrap();

    let runner_script = format!(
        r#"
import {{ runChannelRangeTest }} from './test_channel_range_e2e.mjs';

runChannelRangeTest().then(result => {{
    console.log("Channel range consumed items:", JSON.stringify(result));
    if (!result || result.length !== 4) {{
        console.error("Expected 4 items from channel, got:", result);
        process.exit(1);
    }}
    if (result[0] !== 10 || result[1] !== 20 || result[2] !== 30 || result[3] !== 40) {{
        console.error("Item values mismatch:", result);
        process.exit(2);
    }}
    process.exit(0);
}}).catch(err => {{
    console.error("Error running channel range test:", err);
    process.exit(3);
}});
"#
    );

    let script_path = "dist/run_channel_range_check.mjs";
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
