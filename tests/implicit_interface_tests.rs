use aura_lang::compile;
use std::fs;
use std::process::Command;

#[test]
fn test_implicit_interface_duck_typing() {
    let source = r#"
        interface Greeter {
            greet(name: String): String;
        }

        fn welcome(g: Greeter, user: String): String {
            return g.greet(user);
        }

        export fn run(): String {
            let robot = {
                greet: (n: String): String => `Hello, ${n}! Robot at your service.`
            };
            return welcome(robot, "Alice");
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of duck typing interface failed");
    assert!(res.dts_code.contains("interface Greeter"));
    assert!(res.dts_code.contains("greet(name: string): string;"));
}

#[test]
fn test_implicit_interface_missing_method_rejection() {
    let source = r#"
        interface Reader {
            read(): String;
        }

        fn readData(r: Reader): String {
            return r.read();
        }

        fn run() {
            let invalidObj = {
                write: (data: String) => ()
            };
            readData(invalidObj);
        }
    "#;

    let err = compile(source, &[]).expect_err("Should fail when object lacks interface method");
    assert!(err.contains("missing required interface method 'read'") || err.contains("Missing"));
}

#[test]
fn test_e2e_interface_dispatch_with_node() {
    let source = r#"
        interface Shape {
            area(): Float;
        }

        fn printArea(s: Shape): Float {
            return s.area();
        }

        export fn main(): Float {
            let circle = {
                radius: 5.0,
                area: (): Float => 3.14159 * 5.0 * 5.0
            };
            let rect = {
                w: 10.0,
                h: 4.0,
                area: (): Float => 10.0 * 4.0
            };

            let a1 = printArea(circle);
            let a2 = printArea(rect);
            return a1 + a2;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str(
        "\n\nconst total = main();\nif (Math.abs(total - 118.53975) > 0.01) process.exit(1);\n",
    );

    let tmp_file = "test_interface_exec.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(
        status.success(),
        "Node execution of implicit interfaces failed"
    );
}

#[test]
fn test_golang_style_struct_receiver_methods_satisfy_interface() {
    let source = r#"
        interface Greeter {
            greet(name: String): String;
        }

        struct Robot {
            model: String
        };

        fn (r: Robot) greet(name: String): String => `Beep boop, ${name}! I am ${r.model}.`;

        fn welcome(g: Greeter, user: String): String {
            return g.greet(user);
        }

        export fn run(): String {
            let bot: Robot = { model: "R2-D2" };
            return welcome(bot, "Luke");
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of Go-style interface method failed");
    assert!(res.dts_code.contains("interface Greeter"));

    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str(
        "\n\nconst msg = run();\nif (msg !== 'Beep boop, Luke! I am R2-D2.') process.exit(1);\n",
    );

    let tmp_file = "test_golang_interface_exec.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(
        status.success(),
        "Node execution of Go-style receiver method interface failed"
    );
}

#[test]
fn test_golang_codegen_interface_and_struct_receiver() {
    let source = r#"
        interface Greeter {
            greet(name: String): String;
        }

        struct User {
            name: String
        };

        fn (u: User) greet(target: String): String => `Hello ${target}, I am ${u.name}`;

        fn sayHello(g: Greeter): String => g.greet("World");

        export fn main(): Unit => {
            let user: User = { name: "Alice" };
            println(sayHello(user));
        }
    "#;

    let go_code = aura_lang::compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("type Greeter interface {"));
    assert!(go_code.contains("Greet(name string) string"));
    assert!(go_code.contains("type User struct {"));
    assert!(go_code.contains("func (u User) Greet(target string) string {"));
}

#[test]
fn test_golang_style_pointer_receiver_interface() {
    let source = r#"
        interface Incrementor {
            increment(): Int;
        }

        struct Counter {
            val: Int
        };

        fn (c: *Counter) increment(): Int => {
            c.val = c.val + 1;
            c.val
        }

        fn bump(inc: Incrementor): Int => {
            inc.increment()
        }

        export fn main(): Int => {
            let mut c: Counter = { val: 10 };
            let p = &c;
            bump(p)
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of pointer receiver interface failed");
    assert!(res.dts_code.contains("interface Incrementor"));

    let go_code = aura_lang::compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("type Incrementor interface {"));
    assert!(go_code.contains("func (c *Counter) Increment() int64 {"));
}

#[test]
fn test_golang_public_and_private_receiver_methods() {
    let source = r#"
        struct BankAccount {
            owner: String,
            balance: Float,
            pinHash: String
        };

        // Public method (PascalCase): exported, callable externally
        fn (b: *BankAccount) Deposit(amount: Float): Float => {
            b.balance = b.balance + amount;
            b.balance
        }

        // Public method (PascalCase): exported, calls internal private helper
        fn (b: *BankAccount) Withdraw(amount: Float, pin: String): Result<Float, String> => {
            if (!b.verifyPin(pin)) {
                return Err("PIN incorrecto");
            }
            if (amount > b.balance) {
                return Err("Fondos insuficientes");
            }
            b.balance = b.balance - amount;
            Ok(b.balance)
        }

        // Private method (camelCase): internal helper, unexported in Go
        fn (b: BankAccount) verifyPin(inputPin: String): Bool => {
            b.pinHash == inputPin
        }

        // Public method (PascalCase): query balance
        fn (b: BankAccount) GetBalance(): Float => b.balance;

        export fn main(): Unit => {
            let mut acc: BankAccount = {
                owner: "Alice",
                balance: 1000.0,
                pinHash: "1234"
            };
            let p = &acc;
            p.Deposit(500.0);
            let _ = p.Withdraw(200.0, "1234");
            println(`Final balance: ${acc.GetBalance()}`);
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of public/private receiver methods failed");
    assert!(res.dts_code.contains("Deposit"));
    assert!(res.dts_code.contains("Withdraw"));
    assert!(res.dts_code.contains("GetBalance"));
    assert!(!res.dts_code.contains("verifyPin"));

    let go_code = aura_lang::compile_to_go(source).expect("Go compilation failed");
    assert!(go_code.contains("func (b *BankAccount) Deposit(amount float64) float64 {"));
    assert!(go_code.contains(
        "func (b *BankAccount) Withdraw(amount float64, pin string) (__aura_ret auraResult) {"
    ));
    assert!(go_code.contains("func (b BankAccount) GetBalance() float64 {"));
    assert!(go_code.contains("func (b BankAccount) verifyPin(inputPin string) bool {"));
    assert!(go_code.contains("b.verifyPin(pin)"));

    let tmp_file = "test_bank_account_exec.tmp.mjs";
    let mut js_runner = res.js_code.clone();
    js_runner.push_str("\n\nmain();\n");
    std::fs::write(tmp_file, js_runner).expect("Failed to write runner");
    let status = std::process::Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to run node");
    let _ = std::fs::remove_file(tmp_file);
    assert!(status.success());
}
