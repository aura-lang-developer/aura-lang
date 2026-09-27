use aura_lang::compile;
use std::process::Command;

#[test]
fn test_context_typechecking_and_compilation() {
    let source = r#"
export fn runContextTypecheck(): Task<Unit, String> => {
    let root = Context.background();
    let ctxWithValue = Context.withValue(root, "traceId", "req-12345");
    let (ctx, cancel) = Context.withCancel(ctxWithValue);
    let (timeoutCtx, cancelTimeout) = Context.withTimeout(ctx, 100);

    let isFinished = timeoutCtx.isDone();
    let maybeErr = timeoutCtx.err();
    let traceId = timeoutCtx.value("traceId");

    cancel();
    cancelTimeout();
};
"#;

    let result = compile(source, &[]).expect("Context operations should typecheck cleanly");
    assert!(result.js_code.contains("export class Context"));
    assert!(result.dts_code.contains("export declare class Context"));
}

#[test]
fn test_e2e_context_cancellation_and_timeouts_with_node() {
    let source = r#"
export fn testContextCancelPropagation(): Task<String, String> => {
    let root = Context.background();
    let (ctx, cancel) = Context.withCancel(root);
    let (childCtx, childCancel) = Context.withCancel(ctx);

    let mut resultStatus = "running";

    spawn {
        <-childCtx.done();
        resultStatus = "child received cancellation";
    };

    // Cancel the parent context
    cancel();

    // Sleep briefly to let microtasks resolve
    await sleep(20);

    return resultStatus;
};

export fn testContextTimeout(): Task<Bool, String> => {
    let root = Context.background();
    let (ctx, cancel) = Context.withTimeout(root, 30);

    // Wait on ctx.done()
    <-ctx.done();

    let isDone = ctx.isDone();
    cancel();
    return isDone;
};

export fn testContextValueInheritance(): Task<Option<Any>, String> => {
    let root = Context.background();
    let ctx1 = Context.withValue(root, "userId", 42);
    let ctx2 = Context.withValue(ctx1, "role", "admin");

    let val = ctx2.value("userId");
    return val;
};
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    let temp_js_path = "dist/test_context_e2e.mjs";
    std::fs::create_dir_all("dist").unwrap();
    std::fs::write(temp_js_path, &result.js_code).unwrap();

    let runner_script = format!(
        r#"
import {{ testContextCancelPropagation, testContextTimeout, testContextValueInheritance }} from './test_context_e2e.mjs';

async function run() {{
    const cancelRes = await testContextCancelPropagation();
    console.log("Cancel propagation result:", cancelRes);
    if (cancelRes !== "child received cancellation") {{
        console.error("Cancellation did not propagate properly");
        process.exit(1);
    }}

    const timeoutRes = await testContextTimeout();
    console.log("Timeout result:", timeoutRes);
    if (!timeoutRes) {{
        console.error("Context did not time out properly");
        process.exit(2);
    }}

    const valRes = await testContextValueInheritance();
    console.log("Value inheritance result:", JSON.stringify(valRes));
    if (!valRes || valRes.$ !== "Some" || valRes._0 !== 42) {{
        console.error("Context value lookup failed, got:", valRes);
        process.exit(3);
    }}

    process.exit(0);
}}

run().catch(err => {{
    console.error("Error running context tests:", err);
    process.exit(4);
}});
"#
    );

    let script_path = "dist/run_context_check.mjs";
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
