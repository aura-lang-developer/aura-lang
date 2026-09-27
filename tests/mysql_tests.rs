use aura_lang::compile;

#[test]
fn test_mysql_typecheck_and_codegen_basic() {
    let source = r#"
        let config: MysqlConfig = {
            host: "localhost",
            port: 3306,
            user: "root",
            password: "password123",
            database: "test_db",
            uri: "",
            connectionLimit: 10
        }

        let db: MysqlPool = mysql.createPool(config)

        async fn runQueries(): Task<(), String> {
            let createRes = await db.execute("CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY, name VARCHAR(100))", [])
            match createRes {
                Ok(info) => println(`Affected: ${info.affectedRows}`),
                Err(err) => println(`Error: ${err}`)
            }

            let selectRes = await db.query("SELECT * FROM users WHERE id = ?", [1])
            match selectRes {
                Ok(rows) => println("Found users"),
                Err(err) => println(`Query error: ${err}`)
            }
        }
    "#;

    let result = compile(source, &[]).expect("Compilation of MySQL basic queries should succeed");
    assert!(result.js_code.contains("mysql.createPool"));
    assert!(result.js_code.contains("db.execute"));
    assert!(result.js_code.contains("db.query"));
    assert!(result.dts_code.contains("export interface MysqlPool"));
    assert!(
        result
            .dts_code
            .contains("export interface MysqlQueryResult")
    );
    assert!(result.dts_code.contains("export declare const mysql:"));
}

#[test]
fn test_mysql_transactions_and_query_row() {
    let source = r#"
        let db = mysql.open("mysql://root:secret@localhost:3306/mydb")

        async fn transferMoney(fromId: Int, toId: Int, amount: Int): Task<Bool, String> {
            let txResult = await db.transaction(async fn(tx: MysqlTransaction): Task<(), String> {
                let r1 = await tx.execute("UPDATE accounts SET balance = balance - ? WHERE id = ?", [amount, fromId])
                let r2 = await tx.execute("UPDATE accounts SET balance = balance + ? WHERE id = ?", [amount, toId])
            })

            match txResult {
                Ok(_) => true,
                Err(e) => {
                    println(`Transaction failed: ${e}`)
                    false
                }
            }
        }

        async fn findUser(id: Int): Task<(), String> {
            let userOptRes = await db.queryRow("SELECT * FROM users WHERE id = ?", [id])
            match userOptRes {
                Ok(opt) => {
                    match opt {
                        Some(u) => println("User exists"),
                        None => println("User not found")
                    }
                },
                Err(e) => println(`Error: ${e}`)
            }
        }
    "#;

    let result = compile(source, &[])
        .expect("Compilation of MySQL transactions and queryRow should succeed");
    assert!(result.js_code.contains("mysql.open"));
    assert!(result.js_code.contains("db.transaction"));
    assert!(result.js_code.contains("db.queryRow"));
    assert!(
        result
            .dts_code
            .contains("export interface MysqlTransaction")
    );
}

#[test]
fn test_mysql_escape_and_formatting() {
    let source = r#"
        let escaped = mysql.escape("hello'; DROP TABLE users; --")
        let formatted = mysql.format("SELECT * FROM items WHERE category = ? AND price < ?", ["electronics", 50])
        println(escaped)
        println(formatted)
    "#;

    let result =
        compile(source, &[]).expect("Compilation of MySQL escape and formatting should succeed");
    assert!(result.js_code.contains("mysql.escape"));
    assert!(result.js_code.contains("mysql.format"));
}

#[test]
fn test_bundler_safe_dynamic_imports() {
    let source = "let x = 42";
    let result = compile(source, &[]).expect("Compilation should succeed");
    assert!(result.js_code.contains("const __aura_import ="));
    assert!(
        result
            .js_code
            .contains("import(/* @vite-ignore */ /* webpackIgnore: true */ m)")
    );
    assert!(result.js_code.contains("__aura_import('mysql2/promise')"));
    assert!(result.js_code.contains("__aura_import('pg')"));
    assert!(result.js_code.contains("__aura_import('mongodb')"));
    assert!(result.js_code.contains("__aura_import('ioredis')"));
    assert!(!result.js_code.contains("await import('mysql2/promise')"));
    assert!(!result.js_code.contains("await import('pg')"));
}
