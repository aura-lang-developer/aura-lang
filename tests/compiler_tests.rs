use aura_lang::compile;
use aura_lang::dts_parser::DtsParser;
use std::fs;
use std::process::Command;

#[test]
fn test_compile_pipeline_and_records() {
    let source = r#"
type User = {
    id: String,
    name: String,
    age: Int
}

fn celebrateBirthday(u: User): User => {
    ...u,
    age: u.age + 1
}

fn formatGreeting(u: User): String =>
    `Hello, ${u.name}! You are now ${u.age} years old.`

export fn processUser(raw: User): String =>
    raw
    |> celebrateBirthday
    |> formatGreeting
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    assert!(result.js_code.contains("export function processUser"));
    assert!(
        result
            .js_code
            .contains("formatGreeting(celebrateBirthday(raw))")
    );
    assert!(
        result
            .dts_code
            .contains("export declare function processUser")
    );
}

#[test]
fn test_compile_sum_types_and_pattern_matching() {
    let source = r#"
export type PaymentMethod =
    | CreditCard { number: String, cvv: String }
    | BankTransfer { iban: String }
    | Cash

export fn getPaymentInfo(method: PaymentMethod): String =>
    match method {
        CreditCard { number, .. } => `Credit card ending in ${number}`,
        BankTransfer { iban } => `Bank transfer to ${iban}`,
        Cash => "Paid in cash"
    }
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    assert!(result.js_code.contains("export const PaymentMethod"));
    assert!(result.js_code.contains("CreditCard:"));
    assert!(result.js_code.contains("BankTransfer:"));
    assert!(result.js_code.contains("Cash:"));
    assert!(result.js_code.contains("Non-exhaustive pattern match"));
}

#[test]
fn test_reject_frontend_jsx() {
    let source = r#"
export fn ProductCard(title: String): Any =>
    <div className="product-card">{title}</div>
"#;

    let result = compile(source, &[]);
    assert!(
        result.is_err(),
        "JSX elements must be rejected in backend-only Aura"
    );
}

#[test]
fn test_tail_call_optimization() {
    let source = r#"
export fn sumList(list: List<Int>, acc: Int = 0): Int =>
    match list {
        [] => acc,
        [head, ...tail] => sumList(tail, acc + head)
    }
"#;

    let result = compile(source, &[]).expect("TCO compilation should succeed");
    assert!(result.js_code.contains("// Tail-Call Optimized (TCO) Loop"));
    assert!(result.js_code.contains("while (true)"));
    assert!(result.js_code.contains("continue;"));
}

#[test]
fn test_npm_dts_direct_ingestion() {
    let dts_content = r#"
export interface UserPayload {
    id: string;
    username: string;
    isActive: boolean;
}

export function fetchUserProfile(userId: string): Promise<UserPayload>;
export function calculateTax(amount: number): number;
"#;

    let mut dts_parser = DtsParser::new(dts_content);
    let dts_mod = dts_parser
        .parse_dts("my-npm-lib")
        .expect("DTS parsing should succeed");

    assert_eq!(dts_mod.types.len(), 1);
    assert_eq!(dts_mod.functions.len(), 2);
    assert!(dts_mod.types.contains_key("UserPayload"));
    assert!(dts_mod.functions.contains_key("fetchUserProfile"));
    assert!(dts_mod.functions.contains_key("calculateTax"));

    let source = r#"
fn runCalculation(total: Float): Float =>
    calculateTax(total)
"#;

    let result = compile(source, &[("my-npm-lib", dts_content)])
        .expect("Compilation with npm .d.ts should succeed");
    assert!(result.js_code.contains("calculateTax(total)"));
}

#[test]
fn test_e2e_tco_execution() {
    let source = r#"
export fn sumList(list: List<Int>, acc: Int = 0): Int =>
    match list {
        [] => acc,
        [head, ...tail] => sumList(tail, acc + head)
    }
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    let test_file = "target/test_tco.mjs";
    let script = format!(
        "{}\nif (sumList([1, 2, 3, 4, 5], 0) !== 15) throw new Error('Expected 15');",
        result.js_code
    );
    fs::write(test_file, script).expect("Writing test file");

    let output = Command::new("node")
        .arg(test_file)
        .output()
        .expect("Executing node");
    let _ = fs::remove_file(test_file);

    assert!(
        output.status.success(),
        "Node execution failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_e2e_block_control_flow_and_early_returns() {
    let source = r#"
export type Customer = {
    isVip: Bool,
    loyaltyYears: Int
}

export fn calculateDiscount(customer: Customer, subtotal: Float): Float => {
    if (customer.isVip) {
        return subtotal * 0.25;
    }
    if (customer.loyaltyYears >= 5) {
        return subtotal * 0.15;
    }
    0.0
}
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    let test_file = "target/test_discount.mjs";
    let script = format!(
        r#"{};
if (calculateDiscount({{ isVip: true, loyaltyYears: 0 }}, 100) !== 25) throw new Error('VIP discount failed');
if (calculateDiscount({{ isVip: false, loyaltyYears: 5 }}, 100) !== 15) throw new Error('Loyalty discount failed');
if (calculateDiscount({{ isVip: false, loyaltyYears: 1 }}, 100) !== 0) throw new Error('Default discount failed');
"#,
        result.js_code
    );
    fs::write(test_file, script).expect("Writing test file");

    let output = Command::new("node")
        .arg(test_file)
        .output()
        .expect("Executing node");
    let _ = fs::remove_file(test_file);

    assert!(
        output.status.success(),
        "Node execution failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_generated_dts_precision() {
    let source = r#"
export type User = {
    id: String,
    age: Int,
    isActive: Bool
}

export fn getUserAge(u: User): Int =>
    u.age
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    assert!(
        result
            .dts_code
            .contains("export type User = { id: string; age: number; isActive: boolean };")
    );
    assert!(
        result
            .dts_code
            .contains("export declare function getUserAge(u: User): number;")
    );
}

#[test]
fn test_arrow_functions_syntax_and_codegen() {
    let source = r#"
export fn testArrows(): Int => {
    let add = (x: Int, y: Int): Int => x + y;
    let double = x => x * 2;
    let getConstant = () => 42;
    let asyncMultiplier = async (a: Int, b: Int): Int => a * b;

    add(5, double(getConstant()))
}
"#;

    let result = compile(source, &[]).expect("Arrow functions compilation should succeed");
    assert!(result.js_code.contains("const add = ((x, y) => (x + y));"));
    assert!(result.js_code.contains("const double = ((x) => (x * 2));"));
    assert!(result.js_code.contains("const getConstant = (() => 42);"));
    assert!(
        result
            .js_code
            .contains("const asyncMultiplier = (async (a, b) => (a * b));")
    );
}

#[test]
fn test_e2e_arrow_functions_execution() {
    let source = r#"
export fn runCalculations(): Int => {
    let add = (x: Int, y: Int): Int => x + y;
    let square = n => n * n;
    let greet = name => `Hello, ${name}!`;

    let res = add(square(4), 10);
    res
}
"#;

    let result = compile(source, &[]).expect("Compilation should succeed");
    let test_file = "target/test_arrow_exec.mjs";
    let script = format!(
        r#"{};
const val = runCalculations();
if (val !== 26) throw new Error('Expected 26, got ' + val);
"#,
        result.js_code
    );
    fs::write(test_file, script).expect("Writing test file");

    let output = Command::new("node")
        .arg(test_file)
        .output()
        .expect("Executing node");
    let _ = fs::remove_file(test_file);

    assert!(
        output.status.success(),
        "Node execution failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_reject_frontend_components() {
    let source = r#"
export component Button(props: ButtonProps) => {
    return "Button";
}
"#;

    let result = compile(source, &[]);
    assert!(
        result.is_err(),
        "Frontend 'component' syntax must be rejected in backend-only Aura"
    );
}

#[test]
fn test_backend_record_destructuring() {
    let source = r#"
export type User = {
    name: String,
    email: String
}

export fn getUserInfo(user: User): String => {
    let { name, email } = user;
    `${name} <${email}>`
}
"#;

    let result = compile(source, &[]).expect("Backend record destructuring should succeed");
    assert!(result.js_code.contains("const { name, email } = user;"));
    assert!(
        result
            .dts_code
            .contains("export declare function getUserInfo(user: User): string;")
    );
}

#[test]
fn test_compile_csp_spawn_channels_and_select() {
    let source = r#"
export fn producerConsumerPipeline(): Task<Int, String> => {
    let ch = Channel.make<Int>(5);
    let wg = WaitGroup.new();
    wg.add(1);

    spawn {
        ch <- 10;
        ch <- 20;
        ch <- 30;
        Channel.close(ch);
        wg.done();
    };

    let mut sum = 0;
    let val1 = <-ch;
    match val1 {
        Some(v) => { sum = sum + v; },
        None => ()
    };

    let val2 = <-ch;
    match val2 {
        Some(v) => { sum = sum + v; },
        None => ()
    };

    let val3 = <-ch;
    match val3 {
        Some(v) => { sum = sum + v; },
        None => ()
    };

    await wg.wait();
    sum
}

export fn testSelectTimeout(): Task<String, String> => {
    let ch = Channel.make<String>();
    select {
        case msg = <-ch => `Received: ${msg}`,
        case timeout(50) => "Timed out as expected",
        default => "Fallback immediately"
    }
}
"#;

    let result = compile(source, &[]).expect("CSP concurrency compilation should succeed");
    assert!(result.js_code.contains("export class Channel"));
    assert!(result.js_code.contains("export class WaitGroup"));
    assert!(result.js_code.contains("await ch.send(10)"));
    assert!(result.js_code.contains("await ch.recv()"));
    assert!(result.js_code.contains("__aura_select"));
    assert!(result.dts_code.contains("export declare class Channel<T>"));
    assert!(result.dts_code.contains("export declare class WaitGroup"));
}

#[test]
fn test_e2e_csp_concurrency_execution() {
    let source = r#"
export fn runPipeline(): Task<Int, String> => {
    let ch = Channel.make<Int>(10);
    let wg = WaitGroup.new();
    wg.add(2);

    // Producer 1
    spawn {
        ch <- 100;
        ch <- 200;
        wg.done();
    };

    // Producer 2
    spawn {
        ch <- 300;
        wg.done();
    };

    // Consumer reads 3 messages
    let mut total = 0;
    let m1 = <-ch;
    match m1 {
        Some(x) => { total = total + x; },
        None => ()
    };
    let m2 = <-ch;
    match m2 {
        Some(x) => { total = total + x; },
        None => ()
    };
    let m3 = <-ch;
    match m3 {
        Some(x) => { total = total + x; },
        None => ()
    };

    await wg.wait();
    total
}
"#;

    let result = compile(source, &[]).expect("Compilation must succeed");
    let runner_code = format!(
        "{}\n\nasync function main() {{\n  const res = await runPipeline();\n  if (res !== 600) throw new Error(`Expected 600, got ${{res}}`);\n  console.log('CSP_SUCCESS_600');\n}}\nmain();",
        result.js_code
    );

    let test_file = "test_csp_runner.mjs";
    fs::write(test_file, runner_code).expect("Should write test runner file");

    let output = Command::new("node")
        .arg(test_file)
        .output()
        .expect("Failed to execute node command");

    let _ = fs::remove_file(test_file);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Node.js failed: stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("CSP_SUCCESS_600"),
        "Expected CSP_SUCCESS_600 output"
    );
}

#[test]
fn test_async_function_syntax_and_codegen() {
    let source = r#"
export fn authenticateUser(userId: String): Task<String, String> => async {
    let response = await sleep(10);
    `Authenticated user: ${userId}`
}

export fn fetchProfile(id: String): Task<String, String> => {
    let unused = await sleep(5);
    `Profile: ${id}`
}
"#;

    let result = compile(source, &[]).expect("Compilation must succeed");
    assert!(
        result
            .js_code
            .contains("export async function authenticateUser(userId) {")
    );
    assert!(
        result
            .js_code
            .contains("export async function fetchProfile(id) {")
    );
    assert!(
        result
            .js_code
            .contains("const response = (await sleep(10));")
    );
    assert!(!result.js_code.contains("return(async() => (() => {"));
    assert!(
        result
            .dts_code
            .contains("export declare function authenticateUser(userId: string): Promise<string>;")
    );
    assert!(
        result
            .dts_code
            .contains("export declare function fetchProfile(id: string): Promise<string>;")
    );
}

#[test]
fn test_e2e_async_function_node_execution() {
    let source = r#"
export fn fetchData(id: String): Task<String, String> => async {
    await sleep(20);
    `Data for ${id}`
}

export fn processAll(): Task<String, String> => async {
    let res = await fetchData("42");
    `Result: ${res}`
}
"#;

    let result = compile(source, &[]).expect("Compilation must succeed");
    let runner_code = format!(
        "{}\n\nasync function main() {{\n  const res = await processAll();\n  if (res !== 'Result: Data for 42') throw new Error(`Unexpected result: ${{res}}`);\n  console.log('ASYNC_SUCCESS');\n}}\nmain();",
        result.js_code
    );

    let test_file = "test_async_runner.mjs";
    fs::write(test_file, runner_code).expect("Should write test runner file");

    let output = Command::new("node")
        .arg(test_file)
        .output()
        .expect("Failed to execute node command");

    let _ = fs::remove_file(test_file);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Node.js failed: stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("ASYNC_SUCCESS"),
        "Expected ASYNC_SUCCESS output"
    );
}

#[test]
fn test_e2e_while_and_for_loops() {
    let source = r#"
export fn computeSum(n: Int): Int {
    let mut total = 0;
    let mut i = 1;
    while (i <= n) {
        total = total + i;
        i = i + 1;
    }
    return total;
}

export fn iterateList(items: List<String>): String {
    let mut acc = "";
    for (item in items) {
        acc = acc + item + ",";
    }
    return acc;
}
"#;

    let result = compile(source, &[]).expect("Compilation must succeed");
    let runner_code = format!(
        "{}\n\nfunction main() {{\n  const s = computeSum(10);\n  if (s !== 55) throw new Error(`Expected 55, got ${{s}}`);\n  const str = iterateList(['a', 'b', 'c']);\n  if (str !== 'a,b,c,') throw new Error(`Expected 'a,b,c,', got ${{str}}`);\n  console.log('LOOPS_SUCCESS');\n}}\nmain();",
        result.js_code
    );

    let test_file = "test_loops_runner.mjs";
    fs::write(test_file, runner_code).expect("Should write test runner file");

    let output = Command::new("node")
        .arg(test_file)
        .output()
        .expect("Failed to execute node command");

    let _ = fs::remove_file(test_file);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Node.js failed: stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("LOOPS_SUCCESS"),
        "Expected LOOPS_SUCCESS output"
    );
}

#[test]
fn test_e2e_try_operator_result_propagation() {
    let source = r#"
fn validateAge(age: Int): Result<Int, String> {
    if (age < 0) {
        return Err("Age cannot be negative");
    }
    return Ok(age);
}

export fn registerUser(name: String, age: Int): Result<String, String> {
    let validAge = validateAge(age)?;
    return Ok(`User ${name} registered with age ${validAge}`);
}
"#;

    let result = compile(source, &[]).expect("Compilation must succeed");
    let runner_code = format!(
        "{}\n\nfunction main() {{\n  const okRes = registerUser('Alice', 25);\n  if (okRes.$ !== 'Ok' || okRes.value !== 'User Alice registered with age 25') throw new Error('Expected Ok');\n  const errRes = registerUser('Bob', -5);\n  if (errRes.$ !== 'Err' || errRes.error !== 'Age cannot be negative') throw new Error('Expected Err');\n  console.log('TRY_SUCCESS');\n}}\nmain();",
        result.js_code
    );

    let test_file = "test_try_runner.mjs";
    fs::write(test_file, runner_code).expect("Should write test runner file");

    let output = Command::new("node")
        .arg(test_file)
        .output()
        .expect("Failed to execute node command");

    let _ = fs::remove_file(test_file);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Node.js failed: stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("TRY_SUCCESS"),
        "Expected TRY_SUCCESS output"
    );
}

#[test]
fn test_e2e_string_list_map_methods() {
    let source = r#"
export fn testStringMethods(s: String): List<String> {
    let trimmed = s.trim();
    let parts = trimmed.split(" ");
    return parts;
}

export fn testMapMethods(): Int {
    let m = Map.new();
    m.set("key1", 100);
    m.set("key2", 200);
    return m.size;
}
"#;

    let result = compile(source, &[]).expect("Compilation must succeed");
    let runner_code = format!(
        "{}\n\nfunction main() {{\n  const parts = testStringMethods('  hello world aura  ');\n  if (parts.length !== 3 || parts[0] !== 'hello') throw new Error('String methods failed');\n  const size = testMapMethods();\n  if (size !== 2) throw new Error('Map methods failed');\n  console.log('METHODS_SUCCESS');\n}}\nmain();",
        result.js_code
    );

    let test_file = "test_methods_runner.mjs";
    fs::write(test_file, runner_code).expect("Should write test runner file");

    let output = Command::new("node")
        .arg(test_file)
        .output()
        .expect("Failed to execute node command");

    let _ = fs::remove_file(test_file);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Node.js failed: stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("METHODS_SUCCESS"),
        "Expected METHODS_SUCCESS output"
    );
}
