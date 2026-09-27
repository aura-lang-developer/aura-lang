use aura_lang::compile;

#[test]
fn test_postgres_typecheck_and_codegen_basic() {
    let source = r#"
        let config: PgConfig = {
            host: "localhost",
            port: 5432,
            user: "postgres",
            password: "password123",
            database: "aura_test_db",
            uri: "",
            connectionLimit: 10
        }

        let db: PgPool = postgres.createPool(config)

        async fn runQueries(): Task<(), String> {
            let createRes = await db.execute("CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name VARCHAR(100))", [])
            match createRes {
                Ok(info) => println(`Affected: ${info.rowCount}`),
                Err(err) => println(`Error: ${err}`)
            }

            let selectRes = await db.query("SELECT * FROM users WHERE id = $1", [1])
            match selectRes {
                Ok(rows) => println("Found users"),
                Err(err) => println(`Query error: ${err}`)
            }
        }
    "#;

    let result =
        compile(source, &[]).expect("Compilation of PostgreSQL basic queries should succeed");
    assert!(result.js_code.contains("postgres.createPool"));
    assert!(result.js_code.contains("db.execute"));
    assert!(result.js_code.contains("db.query"));
    assert!(result.dts_code.contains("export interface PgPool"));
    assert!(result.dts_code.contains("export interface PgQueryResult"));
    assert!(result.dts_code.contains("export declare const postgres:"));
    assert!(
        result
            .dts_code
            .contains("export declare const pg: typeof postgres;")
    );
}

#[test]
fn test_postgres_transactions_and_query_row() {
    let source = r#"
        let db = postgres.open("postgresql://postgres:secret@localhost:5432/mydb")

        async fn transferMoney(fromId: Int, toId: Int, amount: Int): Task<Bool, String> {
            let txResult = await db.transaction(async fn(tx: PgTransaction): Task<(), String> {
                let r1 = await tx.execute("UPDATE accounts SET balance = balance - $1 WHERE id = $2", [amount, fromId])
                let r2 = await tx.execute("UPDATE accounts SET balance = balance + $1 WHERE id = $2", [amount, toId])
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
            let userOptRes = await db.queryRow("SELECT * FROM users WHERE id = $1", [id])
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
        .expect("Compilation of PostgreSQL transactions and queryRow should succeed");
    assert!(result.js_code.contains("postgres.open"));
    assert!(result.js_code.contains("db.transaction"));
    assert!(result.js_code.contains("db.queryRow"));
    assert!(result.dts_code.contains("export interface PgTransaction"));
}

#[test]
fn test_postgres_escape_and_formatting() {
    let source = r#"
        let ident = pg.escapeIdentifier("user_table")
        let literal = pg.escapeLiteral("Alice'; DROP TABLE accounts; --")
        let formatted = pg.format("SELECT * FROM items WHERE category = $1 AND price < $2", ["electronics", 50])
        println(ident)
        println(literal)
        println(formatted)
    "#;

    let result = compile(source, &[])
        .expect("Compilation of PostgreSQL escape and formatting should succeed");
    assert!(result.js_code.contains("pg.escapeIdentifier"));
    assert!(result.js_code.contains("pg.escapeLiteral"));
    assert!(result.js_code.contains("pg.format"));
}
