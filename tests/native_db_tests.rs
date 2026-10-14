//! Tests for Native Standalone Database Subsystem in Aura.
//! Covers TypeChecking, Cranelift native binary execution, SQLite CRUD, and Transactions.

use aura_lang::typechecker::TypeChecker;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_native_sqlite_typecheck_and_compilation() {
    let source = r#"
        export fn main(): Unit => {
            let db: DB = sqlite.open(":memory:", 5);
            let ddl = "CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price REAL);";
            let res = db.exec(ddl, []);
            let insertSql = "INSERT INTO products (name, price) VALUES (?, ?);";
            db.exec(insertSql, ["Keyboard", 49.99]);
            db.exec(insertSql, ["Mouse", 24.50]);

            let rows: Rows = db.query("SELECT id, name, price FROM products ORDER BY id ASC;", []);
            while (rows.next()) {
                let id = rows.getInt(0);
                let name = rows.getString(1);
                let price = rows.getFloat(2);
                let rec = rows.toRecord();
                println(name);
            }
            rows.close();
            db.close();
        }
    "#;

    let mut lexer = aura_lang::lexer::Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = aura_lang::parser::Parser::new(tokens);
    let module = parser.parse_module().expect("Parsing failed");

    let mut tc = TypeChecker::new();
    let res = tc.check_module(&module);
    assert!(res.is_ok(), "Type checking failed: {:?}", res.err());
}

#[test]
fn test_native_sql_generic_module_typecheck() {
    let source = r#"
        export fn main(): Unit => {
            let db: DB = sql.open("sqlite", ":memory:", 10);
            let pingRes = db.ping();
            println(pingRes);

            let tx: Tx = db.begin();
            tx.exec("CREATE TABLE items (id INTEGER, val TEXT);", []);
            tx.exec("INSERT INTO items VALUES (1, 'Test');", []);
            let commitRes = tx.commit();
            println(commitRes);

            db.close();
        }
    "#;

    let mut lexer = aura_lang::lexer::Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = aura_lang::parser::Parser::new(tokens);
    let module = parser.parse_module().expect("Parsing failed");

    let mut tc = TypeChecker::new();
    let res = tc.check_module(&module);
    assert!(res.is_ok(), "Type checking failed: {:?}", res.err());
}

#[test]
fn test_native_cranelift_sqlite_execution() {
    let source = r#"
        export fn main(): Unit => {
            println("--- Starting Native SQLite Test ---");
            let db = sqlite.open(":memory:", 5);

            let ddl = "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, balance REAL);";
            db.exec(ddl, []);

            let insertSql = "INSERT INTO users (name, balance) VALUES (?, ?);";
            db.exec(insertSql, ["Alice", 1200.50]);
            db.exec(insertSql, ["Bob", 850.00]);

            let rows = db.query("SELECT id, name, balance FROM users ORDER BY id ASC;", []);
            while (rows.next()) {
                let id = rows.getInt(0);
                let name = rows.getString(1);
                let balance = rows.getFloat(2);
                println(name);
            }
            rows.close();

            println("--- Database Native Test Passed ---");
            db.close();
        }
    "#;

    let out_bin = Path::new("test_cranelift_sqlite_bin");
    let res = aura_lang::build_native_binary(source, out_bin);
    assert!(res.is_ok(), "Native compilation failed: {:?}", res.err());
    assert!(out_bin.exists());

    let output = Command::new("./test_cranelift_sqlite_bin")
        .output()
        .expect("Failed to run native binary");

    let _ = fs::remove_file(out_bin);

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Native DB Output:\n{}", stdout);
    assert!(stdout.contains("--- Starting Native SQLite Test ---"));
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Bob"));
    assert!(stdout.contains("--- Database Native Test Passed ---"));
}

#[test]
fn test_native_cranelift_sqlite_transactions() {
    let source = r#"
        export fn main(): Unit => {
            println("--- Testing Native Transactions ---");
            let db = sqlite.open(":memory:", 5);
            db.exec("CREATE TABLE accounts (id INTEGER PRIMARY KEY, name TEXT, balance INT);", []);
            db.exec("INSERT INTO accounts VALUES (1, 'Alice', 1000);", []);
            db.exec("INSERT INTO accounts VALUES (2, 'Bob', 500);", []);

            // Successful Transaction: Alice transfers 200 to Bob
            let tx = db.begin();
            tx.exec("UPDATE accounts SET balance = balance - 200 WHERE id = 1;", []);
            tx.exec("UPDATE accounts SET balance = balance + 200 WHERE id = 2;", []);
            tx.commit();

            let rows = db.query("SELECT name, balance FROM accounts WHERE id = 1;", []);
            while (rows.next()) {
                let name = rows.getString(0);
                let bal = rows.getInt(1);
                println(`Alice balance after commit: ${bal}`);
            }
            rows.close();

            // Rollback Transaction: Failed transfer
            let tx2 = db.begin();
            tx2.exec("UPDATE accounts SET balance = 0 WHERE id = 1;", []);
            tx2.rollback();

            let rows2 = db.query("SELECT name, balance FROM accounts WHERE id = 1;", []);
            while (rows2.next()) {
                let name = rows2.getString(0);
                let bal = rows2.getInt(1);
                println(`Alice balance after rollback: ${bal}`);
            }
            rows2.close();

            db.close();
            println("--- Transaction Tests Passed ---");
        }
    "#;

    let out_bin = Path::new("test_cranelift_tx_bin");
    let res = aura_lang::build_native_binary(source, out_bin);
    assert!(res.is_ok(), "Native compilation failed: {:?}", res.err());
    assert!(out_bin.exists());

    let output = Command::new("./test_cranelift_tx_bin")
        .output()
        .expect("Failed to run native binary");

    let _ = fs::remove_file(out_bin);

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Native Tx Output:\n{}", stdout);
    assert!(stdout.contains("Alice balance after commit: 800"));
    assert!(stdout.contains("Alice balance after rollback: 800"));
    assert!(stdout.contains("--- Transaction Tests Passed ---"));
}
