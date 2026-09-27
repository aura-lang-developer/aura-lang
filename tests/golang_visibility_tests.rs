use aura_lang::{compile, compile_to_go};
use std::fs;
use std::process::Command;

// ==============================================================================
// 1. HAPPY PATH TESTS (Casos Exitosos)
// ==============================================================================

#[test]
fn test_happy_path_public_methods_value_and_pointer() {
    let source = r#"
        struct Counter {
            name: String,
            count: Int
        };

        // Public value receiver (PascalCase): exported externally
        fn (c: Counter) GetCount(): Int => c.count;

        fn (c: Counter) FormatLabel(): String => `[${c.name}] = ${c.count}`;

        // Public pointer receiver (PascalCase): in-place mutation
        fn (c: *Counter) Increment(delta: Int): Unit => {
            c.count = c.count + delta;
        }

        export fn run(): String {
            let mut c: Counter = { name: "Hits", count: 10 };
            let p = &c;
            p.Increment(5);
            return p.FormatLabel();
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    assert!(res.dts_code.contains("GetCount"));
    assert!(res.dts_code.contains("Increment"));
    assert!(res.dts_code.contains("FormatLabel"));

    let go_code = compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("func (c Counter) GetCount() int64 {"));
    assert!(go_code.contains("func (c *Counter) Increment(delta int64) {"));

    let tmp = "tmp_test_hp1.mjs";
    fs::write(
        tmp,
        format!(
            "{}\nif (run() !== '[Hits] = 15') process.exit(1);\n",
            res.js_code
        ),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_happy_path_public_calling_private_helper() {
    let source = r#"
        struct Authenticator {
            salt: String
        };

        // Public method (PascalCase)
        fn (a: Authenticator) CheckAccess(token: String): Bool => {
            a.isValidLength(token) && a.matchesSecret(token)
        }

        // Private method (camelCase): unexported helper
        fn (a: Authenticator) isValidLength(s: String): Bool => len(s) >= 4;

        // Private method (camelCase): unexported helper
        fn (a: Authenticator) matchesSecret(s: String): Bool => s == "secret";

        export fn run(): Bool {
            let auth: Authenticator = { salt: "xyz" };
            return auth.CheckAccess("secret") && !auth.CheckAccess("no");
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    assert!(res.dts_code.contains("CheckAccess"));
    assert!(!res.dts_code.contains("isValidLength"));
    assert!(!res.dts_code.contains("matchesSecret"));

    let go_code = compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("func (a Authenticator) CheckAccess(token string) bool {"));
    assert!(go_code.contains("func (a Authenticator) isValidLength(s string) bool {"));
    assert!(go_code.contains("func (a Authenticator) matchesSecret(s string) bool {"));
    assert!(go_code.contains("a.isValidLength(token)"));
    assert!(go_code.contains("a.matchesSecret(token)"));

    let tmp = "tmp_test_hp2.mjs";
    fs::write(
        tmp,
        format!("{}\nif (!run()) process.exit(1);\n", res.js_code),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_happy_path_polymorphic_receivers() {
    let source = r#"
        struct Square {
            side: Float
        };

        struct Circle {
            radius: Float
        };

        // Public method on Square
        fn (s: Square) Area(): Float => s.side * s.side;
        fn (s: Square) describe(): String => "Square";

        // Public method on Circle with identical name
        fn (c: Circle) Area(): Float => 3.14159 * c.radius * c.radius;
        fn (c: Circle) describe(): String => "Circle";

        export fn run(): Float {
            let sq: Square = { side: 4.0 };
            let ci: Circle = { radius: 2.0 };
            return sq.Area() + ci.Area();
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    assert!(res.dts_code.contains("Area"));
    assert!(!res.dts_code.contains("describe"));

    let go_code = compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("func (s Square) Area() float64 {"));
    assert!(go_code.contains("func (c Circle) Area() float64 {"));
    assert!(go_code.contains("func (s Square) describe() string {"));
    assert!(go_code.contains("func (c Circle) describe() string {"));

    let tmp = "tmp_test_hp3.mjs";
    fs::write(
        tmp,
        format!(
            "{}\nconst total = run(); if (Math.abs(total - 28.56636) > 0.01) process.exit(1);\n",
            res.js_code
        ),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_happy_path_interface_satisfaction_with_public_methods() {
    let source = r#"
        interface AreaCalculator {
            Area(): Float;
        }

        struct Box {
            w: Float,
            h: Float
        };

        fn (b: Box) Area(): Float => b.w * b.h;

        fn printArea(c: AreaCalculator): Float => c.Area();

        export fn run(): Float {
            let bx: Box = { w: 5.0, h: 6.0 };
            return printArea(bx);
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let go_code = compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("type AreaCalculator interface {"));
    assert!(go_code.contains("Area() float64"));
    assert!(go_code.contains("func (b Box) Area() float64 {"));

    let tmp = "tmp_test_hp4.mjs";
    fs::write(
        tmp,
        format!("{}\nif (run() !== 30.0) process.exit(1);\n", res.js_code),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_happy_path_fluent_builder_chaining() {
    let source = r#"
        struct RequestBuilder {
            url: String,
            timeout: Int,
            isSecure: Bool
        };

        fn newBuilder(): RequestBuilder => {
            { url: "", timeout: 1000, isSecure: false }
        };

        fn (b: RequestBuilder) SetUrl(u: String): RequestBuilder => {
            { ...b, url: u }
        }

        fn (b: RequestBuilder) SetTimeout(t: Int): RequestBuilder => {
            { ...b, timeout: t }
        }

        fn (b: RequestBuilder) WithSecurity(s: Bool): RequestBuilder => {
            { ...b, isSecure: s }
        }

        fn (b: RequestBuilder) BuildSummary(): String => {
            `Req: ${b.url}, Timeout: ${b.timeout}, Sec: ${b.isSecure}`
        }

        export fn run(): String {
            let summary = newBuilder()
                .SetUrl("https://api.aura.dev")
                .SetTimeout(5000)
                .WithSecurity(true)
                .BuildSummary();
            return summary;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let tmp = "tmp_test_hp5.mjs";
    fs::write(
        tmp,
        format!(
            "{}\nif (run() !== 'Req: https://api.aura.dev, Timeout: 5000, Sec: true') process.exit(1);\n",
            res.js_code
        ),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

// ==============================================================================
// 2. SAD PATH TESTS (Casos de Error y Fallo)
// ==============================================================================

#[test]
fn test_sad_path_missing_interface_method() {
    let source = r#"
        interface Serializer {
            Serialize(): String;
        }

        struct Document {
            title: String
        };

        fn serializeDoc(s: Serializer): String => s.Serialize();

        export fn run() {
            let doc: Document = { title: "Draft" };
            serializeDoc(doc);
        }
    "#;

    let err = compile(source, &[]).expect_err("Must fail when missing interface method");
    assert!(
        err.contains("missing required interface method 'Serialize'")
            || err.contains("Missing required interface method")
            || err.contains("Serialize")
    );
}

#[test]
fn test_sad_path_mismatched_return_type() {
    let source = r#"
        interface Formatter {
            Format(): String;
        }

        struct Data {
            num: Int
        };

        // Returns Int instead of String!
        fn (d: Data) Format(): Int => d.num;

        fn execute(f: Formatter): String => f.Format();

        export fn run() {
            let d: Data = { num: 42 };
            execute(d);
        }
    "#;

    let err = compile(source, &[]).expect_err("Must fail when return type doesn't match interface");
    assert!(err.contains("Format") || err.contains("Type mismatch"));
}

#[test]
fn test_sad_path_mismatched_parameter_types() {
    let source = r#"
        interface Validator {
            Validate(code: Int): Bool;
        }

        struct SecurityCheck {};

        // Parameter is String instead of Int!
        fn (s: SecurityCheck) Validate(code: String): Bool => len(code) > 0;

        fn runValidation(v: Validator): Bool => v.Validate(123);

        export fn run() {
            let s: SecurityCheck = {};
            runValidation(s);
        }
    "#;

    let err = compile(source, &[]).expect_err("Must fail with mismatched parameter types");
    assert!(err.contains("Validate") || err.contains("Type mismatch"));
}

#[test]
fn test_sad_path_nonexistent_method_call() {
    let source = r#"
        struct User {
            name: String
        };

        export fn run() {
            let u: User = { name: "Alice" };
            u.NonExistentMethod();
        }
    "#;

    let err = compile(source, &[]).expect_err("Must fail when calling nonexistent method");
    assert!(err.contains("not found") || err.contains("NonExistentMethod"));
}

#[test]
fn test_sad_path_invalid_argument_type_to_receiver_method() {
    let source = r#"
        struct MathTool {};

        fn (m: MathTool) Factorial(n: Int): Int => n;

        export fn run() {
            let m: MathTool = {};
            m.Factorial("not_an_int");
        }
    "#;

    let err = compile(source, &[]).expect_err("Must fail when passing string to int parameter");
    assert!(err.contains("Type mismatch") || err.contains("String") || err.contains("Int"));
}

// ==============================================================================
// 3. EDGE CASES TESTS (Casos Borde)
// ==============================================================================

#[test]
fn test_edge_case_receiver_method_with_default_parameters() {
    let source = r#"
        struct Greeter {
            prefix: String
        };

        fn (g: Greeter) Greet(target: String = "World", loud: Bool = false): String => {
            let msg = `${g.prefix} ${target}`;
            if (loud) {
                `${msg}!`
            } else {
                msg
            }
        }

        export fn run(): String {
            let g: Greeter = { prefix: "Hello" };
            let r1 = g.Greet();
            let r2 = g.Greet("Aura");
            let r3 = g.Greet("Developer", true);
            return `${r1} | ${r2} | ${r3}`;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation with default parameters failed");
    let tmp = "tmp_test_ec1.mjs";
    fs::write(
        tmp,
        format!(
            "{}\nif (run() !== 'Hello World | Hello Aura | Hello Developer!') process.exit(1);\n",
            res.js_code
        ),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_edge_case_recursive_receiver_method() {
    let source = r#"
        struct Calculator {};

        fn (c: Calculator) Factorial(n: Int): Int => {
            if (n <= 1) {
                1
            } else {
                n * c.Factorial(n - 1)
            }
        }

        export fn run(): Int {
            let calc: Calculator = {};
            return calc.Factorial(5);
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of recursive receiver method failed");
    let tmp = "tmp_test_ec2.mjs";
    fs::write(
        tmp,
        format!("{}\nif (run() !== 120) process.exit(1);\n", res.js_code),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_edge_case_defer_and_errdefer_in_receiver_methods() {
    let source = r#"
        struct Session {
            isOpen: Bool,
            events: []String
        };

        fn (s: *Session) Execute(fail: Bool): Result<String, String> => {
            s.isOpen = true;
            defer {
                s.isOpen = false;
            };
            errdefer {
                s.events = append(s.events, "FAILED");
            };

            if (fail) {
                return Err("Simulated crash");
            }
            s.events = append(s.events, "SUCCESS");
            Ok("Done")
        }

        export fn run(): Bool {
            let mut s1: Session = { isOpen: false, events: [] };
            let p1 = &s1;
            let _ = p1.Execute(false);
            let s1Closed = !s1.isOpen;
            let s1Success = len(s1.events) == 1 && s1.events[0] == "SUCCESS";

            let mut s2: Session = { isOpen: false, events: [] };
            let p2 = &s2;
            let _ = p2.Execute(true);
            let s2Closed = !s2.isOpen;
            let s2Failed = len(s2.events) == 1 && s2.events[0] == "FAILED";

            return s1Closed && s1Success && s2Closed && s2Failed;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let tmp = "tmp_test_ec3.mjs";
    fs::write(
        tmp,
        format!("{}\nif (!run()) process.exit(1);\n", res.js_code),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_edge_case_empty_struct_receiver_methods() {
    let source = r#"
        struct PingService {};

        // Public method
        fn (p: PingService) Ping(): String => "pong";

        // Private method
        fn (p: PingService) secret(): Int => 42;

        fn (p: PingService) GetSecret(): Int => p.secret();

        export fn run(): String {
            let svc: PingService = {};
            return `${svc.Ping()}-${svc.GetSecret()}`;
        }
    "#;

    let res = compile(source, &[]).expect("Empty struct with receiver methods failed");
    let tmp = "tmp_test_ec4.mjs";
    fs::write(
        tmp,
        format!(
            "{}\nif (run() !== 'pong-42') process.exit(1);\n",
            res.js_code
        ),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_edge_case_result_and_try_operator_in_receiver_methods() {
    let source = r#"
        struct AccountService {
            balance: Float
        };

        fn (a: *AccountService) debit(amt: Float): Result<Float, String> => {
            if (amt > a.balance) {
                return Err("Insufficient funds");
            }
            a.balance = a.balance - amt;
            Ok(a.balance)
        }

        fn (a: *AccountService) Transfer(targetAmt: Float): Result<Float, String> => {
            let newBal = a.debit(targetAmt)?;
            Ok(newBal)
        }

        export fn run(): Bool {
            let mut acc: AccountService = { balance: 100.0 };
            let p = &acc;
            let okRes = p.Transfer(40.0);
            let failRes = p.Transfer(100.0);

            let okMatch = match okRes {
                Ok(b) => b == 60.0,
                Err(_) => false,
            };

            let failMatch = match failRes {
                Ok(_) => false,
                Err(_) => true,
            };

            return okMatch && failMatch;
        }
    "#;

    let res = compile(source, &[]).expect("Result ? in receiver failed");
    let tmp = "tmp_test_ec5.mjs";
    fs::write(
        tmp,
        format!("{}\nif (!run()) process.exit(1);\n", res.js_code),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_edge_case_go_native_compilation_and_execution() {
    let source = r#"
        struct Server {
            port: Int
        };

        fn (s: Server) Serve(): String => {
            `Listening on :${s.port}`
        }

        fn (s: Server) validatePort(): Bool => s.port > 0;

        export fn main(): Unit => {
            let s: Server = { port: 8080 };
            if (s.validatePort()) {
                println(s.Serve());
            }
        }
    "#;

    let go_code = compile_to_go(source).expect("Go compilation failed");
    // Public method is exported (capitalized)
    assert!(go_code.contains("func (s Server) Serve() string"));
    // Private method is unexported (lowercase)
    assert!(go_code.contains("func (s Server) validatePort() bool"));
    assert!(go_code.contains("s.validatePort()"));

    let bin_path = std::path::Path::new("test_bin_go_visibility");
    let build_res = aura_lang::build_backend_executable(source, bin_path)
        .expect("Building native binary via Go toolchain failed");
    assert!(build_res.size_bytes > 0);

    let output = Command::new("./test_bin_go_visibility")
        .output()
        .expect("Failed to execute native binary");
    let _ = fs::remove_file("test_bin_go_visibility");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Listening on :8080"));
}

// ==============================================================================
// 4. STANDALONE FUNCTIONS & FIRST-ORDER FUNCTIONS (Funciones Libres y de Primer Orden)
// ==============================================================================

#[test]
fn test_happy_path_standalone_capitalized_functions_and_first_order() {
    let source = r#"
        // Public standalone function (PascalCase)
        fn Add(a: Int, b: Int): Int => a + b;

        // Private standalone helper (camelCase)
        fn multiplyInternal(a: Int, b: Int): Int => a * b;

        // Higher-order function receiving a first-class function
        fn ApplyBinary(op: (Int, Int) -> Int, x: Int, y: Int): Int => {
            op(x, y)
        }

        // Higher-order function returning a closure
        fn MakeMultiplier(factor: Int): (Int) -> Int => {
            fn Mul(val: Int): Int => multiplyInternal(val, factor);
            Mul
        }

        export fn run(): String {
            // Direct invocation of capitalized standalone function
            let sum = Add(10, 20);

            // First-class function passed as argument
            let appliedSum = ApplyBinary(Add, 30, 40);

            // Stored in a capitalized variable (PascalCase variable invocation)
            let MyOperation = Add;
            let varSum = MyOperation(50, 60);

            // Factory returning a function
            let triple = MakeMultiplier(3);
            let triVal = triple(10);

            return `${sum}-${appliedSum}-${varSum}-${triVal}`;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of standalone capitalized functions failed");
    // Public standalone functions are exported in DTS
    assert!(
        res.dts_code
            .contains("export declare function Add(a: number, b: number): number;")
    );
    assert!(res.dts_code.contains("ApplyBinary"));
    assert!(res.dts_code.contains("MakeMultiplier"));
    // Private standalone function is unexported
    assert!(!res.dts_code.contains("multiplyInternal"));

    let go_code = compile_to_go(source).expect("Go compilation failed");
    // Public standalone function is exported in Go
    assert!(go_code.contains("func Add(a int64, b int64) int64 {"));
    assert!(
        go_code.contains("func ApplyBinary(op func(int64, int64) int64, x int64, y int64) int64 {")
    );
    assert!(go_code.contains("func MakeMultiplier(factor int64) func(int64) int64 {"));
    // Private standalone function is unexported in Go
    assert!(go_code.contains("func multiplyInternal(a int64, b int64) int64 {"));

    let tmp = "tmp_test_hp_standalone.mjs";
    fs::write(
        tmp,
        format!(
            "{}\nif (run() !== '30-70-110-30') process.exit(1);\n",
            res.js_code
        ),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_happy_path_capitalized_functions_with_collections_and_pipeline() {
    let source = r#"
        fn Double(x: Int): Int => x * 2;
        fn AddTen(x: Int): Int => x + 10;
        fn IsPositive(x: Int): Bool => x > 0;

        export fn run(): String {
            // Pipeline chaining standalone functions
            let pipeVal = 5 |> Double |> AddTen;

            // First-order function passed to list callbacks
            let nums: []Int = [1, 2, 3, 4];
            let doubledList = nums.map(Double);
            let filteredList = nums.filter(IsPositive);

            let firstDoubled = doubledList[0];
            let lastDoubled = doubledList[3];

            return `Pipe:${pipeVal}, First:${firstDoubled}, Last:${lastDoubled}, Len:${len(filteredList)}`;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of pipeline and callbacks failed");
    let tmp = "tmp_test_hp_pipeline.mjs";
    fs::write(
        tmp,
        format!(
            "{}\nif (run() !== 'Pipe:20, First:2, Last:8, Len:4') process.exit(1);\n",
            res.js_code
        ),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_sad_path_invalid_argument_type_to_standalone_capitalized_function() {
    let source = r#"
        fn Calculate(x: Int, y: Int): Int => x + y;

        export fn run() {
            Calculate("not_an_int", 42);
        }
    "#;

    let err = compile(source, &[])
        .expect_err("Must fail when passing String to Int argument in capitalized function");
    assert!(err.contains("Type mismatch") || err.contains("String") || err.contains("Int"));
}

#[test]
fn test_sad_path_nonexistent_capitalized_function_call() {
    let source = r#"
        export fn run() {
            UndefinedFunc(1, 2);
        }
    "#;

    let err = compile(source, &[]).expect_err("Must fail on undefined capitalized function");
    assert!(
        err.contains("Undefined") || err.contains("UndefinedFunc") || err.contains("not found")
    );
}

#[test]
fn test_sad_path_first_order_function_signature_mismatch() {
    let source = r#"
        fn Apply(f: (Int) -> String, val: Int): String => f(val);

        // Takes Int and returns Int, but Apply expects Int -> String!
        fn Incompatible(x: Int): Int => x * 2;

        export fn run() {
            Apply(Incompatible, 10);
        }
    "#;

    let err = compile(source, &[])
        .expect_err("Must fail when first-order function signature does not match");
    assert!(err.contains("Type mismatch") || err.contains("String") || err.contains("Int"));
}

#[test]
fn test_edge_case_recursive_capitalized_standalone_function() {
    let source = r#"
        fn Fib(n: Int): Int => {
            if (n <= 1) {
                n
            } else {
                Fib(n - 1) + Fib(n - 2)
            }
        }

        export fn run(): Int => Fib(10);
    "#;

    let res = compile(source, &[]).expect("Compilation of recursive standalone function failed");
    let tmp = "tmp_test_ec_fib.mjs";
    fs::write(
        tmp,
        format!("{}\nif (run() !== 55) process.exit(1);\n", res.js_code),
    )
    .unwrap();
    let status = Command::new("node").arg(tmp).status().unwrap();
    let _ = fs::remove_file(tmp);
    assert!(status.success());
}

#[test]
fn test_edge_case_go_native_compilation_with_first_order_functions() {
    let source = r#"
        fn Add(a: Int, b: Int): Int => a + b;

        fn ExecuteOp(op: (Int, Int) -> Int, x: Int, y: Int): Int => {
            op(x, y)
        }

        export fn main(): Unit => {
            let res = ExecuteOp(Add, 25, 17);
            println(`Result of Add via first-order func: ${res}`);
        }
    "#;

    let go_code = compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("func Add(a int64, b int64) int64 {"));
    assert!(
        go_code.contains("func ExecuteOp(op func(int64, int64) int64, x int64, y int64) int64 {")
    );
    assert!(go_code.contains("ExecuteOp(Add, 25, 17)"));

    let bin_path = std::path::Path::new("test_bin_go_first_order");
    let build_res = aura_lang::build_backend_executable(source, bin_path)
        .expect("Building native binary via Go toolchain failed");
    assert!(build_res.size_bytes > 0);

    let output = Command::new("./test_bin_go_first_order")
        .output()
        .expect("Failed to execute native binary");
    let _ = fs::remove_file("test_bin_go_first_order");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Result of Add via first-order func: 42"));
}
