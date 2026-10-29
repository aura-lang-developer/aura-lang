//! Unified Native Database Subsystem for Aura Runtime.
//! Supports SQLite (bundled embedded engine), PostgreSQL, and MySQL.

use crate::list::{AuraList, aura_list_get, aura_list_len, aura_list_new, aura_list_push};
use crate::mysql_client::{MySqlConfig, MySqlConnection};
use crate::postgres_client::{PgConfig, PgConnection};
use crate::record::{
    AuraRecord, AuraVal, aura_record_new, aura_record_set_val, infer_val_from_i64,
};
use crate::string::{AuraString, aura_string_from_rust_str};
#[cfg(feature = "sqlite")]
use rusqlite::types::{Value, ValueRef};
#[cfg(feature = "sqlite")]
use rusqlite::{Connection as SqliteConnection, params_from_iter};

#[cfg(not(feature = "sqlite"))]
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}
use std::slice;
use std::str;
use std::sync::{Arc, Condvar, Mutex};

pub enum DbBackend {
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteConnection),
    Postgres(PgConnection),
    MySql(MySqlConnection),
}

pub struct AuraDbConn {
    pub backend: DbBackend,
}

pub struct AuraDbPool {
    pub driver: String,
    pub dsn: String,
    pub max_conns: usize,
    pub conns: Mutex<Vec<AuraDbConn>>,
    pub is_closed: Mutex<bool>,
    pub active_conns: Mutex<usize>,
    pub cond: Condvar,
}

pub struct AuraDbRows {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<AuraVal>>,
    pub current_idx: isize,
    pub last_error: Option<String>,
}

pub struct AuraDbTx {
    pub conn: Option<AuraDbConn>,
    pub pool: Arc<AuraDbPool>,
    pub is_active: bool,
    pub savepoint_level: usize,
}

pub struct AuraExecResult {
    pub rows_affected: i64,
    pub last_insert_id: i64,
    pub error: Option<String>,
}

impl AuraDbPool {
    pub fn new(driver: &str, dsn: &str, max_conns: usize) -> Result<Arc<Self>, String> {
        let pool = Arc::new(Self {
            driver: driver.to_string(),
            dsn: dsn.to_string(),
            max_conns: if max_conns == 0 { 10 } else { max_conns },
            conns: Mutex::new(Vec::new()),
            is_closed: Mutex::new(false),
            active_conns: Mutex::new(0),
            cond: Condvar::new(),
        });

        // Initialize at least 1 connection to verify configuration
        let initial_conn = pool.create_connection()?;
        pool.conns.lock().unwrap().push(initial_conn);

        Ok(pool)
    }

    fn create_connection(&self) -> Result<AuraDbConn, String> {
        match self.driver.to_lowercase().as_str() {
            #[cfg(feature = "sqlite")]
            "sqlite" | "sqlite3" => {
                let conn =
                    if self.dsn.is_empty() || self.dsn == ":memory:" || self.dsn == "memory://" {
                        SqliteConnection::open_in_memory().map_err(|e| {
                            format!("Failed to open in-memory SQLite database: {}", e)
                        })?
                    } else {
                        let path = self.dsn.strip_prefix("sqlite://").unwrap_or(&self.dsn);
                        SqliteConnection::open(path).map_err(|e| {
                            format!("Failed to open SQLite database at '{}': {}", path, e)
                        })?
                    };

                // Enable foreign keys and WAL mode for better concurrency
                let _ = conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;");
                Ok(AuraDbConn {
                    backend: DbBackend::Sqlite(conn),
                })
            }
            #[cfg(not(feature = "sqlite"))]
            "sqlite" | "sqlite3" => {
                Err("SQLite support is disabled in this runtime build profile. Use profile with SQLite enabled.".to_string())
            }
            "postgres" | "postgresql" | "pg" => {
                let config = PgConfig::from_conn_str(&self.dsn)?;
                let pg = PgConnection::connect(&config)?;
                Ok(AuraDbConn {
                    backend: DbBackend::Postgres(pg),
                })
            }
            "mysql" | "mariadb" => {
                let config = MySqlConfig::from_conn_str(&self.dsn)?;
                let mysql = MySqlConnection::connect(&config)?;
                Ok(AuraDbConn {
                    backend: DbBackend::MySql(mysql),
                })
            }
            other => Err(format!(
                "Unsupported database driver: '{}'. Supported drivers: 'sqlite', 'postgres', 'mysql'",
                other
            )),
        }
    }

    pub fn acquire(&self) -> Result<AuraDbConn, String> {
        let timeout = std::time::Duration::from_secs(15);
        let start = std::time::Instant::now();

        loop {
            if *self.is_closed.lock().unwrap() {
                return Err("Database pool is closed".to_string());
            }

            // 1. Try taking an idle connection from the pool
            {
                let mut conns = self.conns.lock().unwrap();
                if let Some(conn) = conns.pop() {
                    let mut active = self.active_conns.lock().unwrap();
                    *active += 1;
                    return Ok(conn);
                }
            }

            // 2. If no idle connection, check if under max_conns limit to create a new one
            {
                let mut active = self.active_conns.lock().unwrap();
                if *active < self.max_conns {
                    *active += 1;
                    drop(active);
                    match self.create_connection() {
                        Ok(conn) => return Ok(conn),
                        Err(e) => {
                            let mut active = self.active_conns.lock().unwrap();
                            *active = active.saturating_sub(1);
                            self.cond.notify_one();
                            return Err(e);
                        }
                    }
                }
            }

            // 3. Otherwise wait on Condvar until a connection is returned
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Err(format!(
                    "Database pool timeout ({}s) waiting for available connection (max_conns={})",
                    timeout.as_secs(),
                    self.max_conns
                ));
            }

            let remaining = timeout - elapsed;
            let conns = self.conns.lock().unwrap();
            let (guard, timeout_res) = self.cond.wait_timeout(conns, remaining).unwrap();
            drop(guard);
            if timeout_res.timed_out() {
                return Err(format!(
                    "Database pool timeout ({}s) waiting for available connection (max_conns={})",
                    timeout.as_secs(),
                    self.max_conns
                ));
            }
        }
    }

    pub fn release(&self, conn: AuraDbConn) {
        let mut conns = self.conns.lock().unwrap();
        let mut active = self.active_conns.lock().unwrap();
        *active = active.saturating_sub(1);
        if conns.len() < self.max_conns {
            conns.push(conn);
        }
        drop(conns);
        drop(active);
        self.cond.notify_one();
    }
}

// Convert AuraList params to SQLite Values
fn extract_aura_params(params: *mut AuraList) -> Vec<Value> {
    let mut out = Vec::new();
    if params.is_null() {
        return out;
    }
    let count = aura_list_len(params);
    for i in 0..count {
        let raw_val = aura_list_get(params, i);
        let aval = infer_val_from_i64(raw_val);
        match aval {
            AuraVal::Int(n) => out.push(Value::Integer(n)),
            AuraVal::Float(f) => out.push(Value::Real(f)),
            AuraVal::String(s) => {
                let rust_str = if !s.is_null() {
                    unsafe { (*s).as_str().to_string() }
                } else {
                    String::new()
                };
                out.push(Value::Text(rust_str));
            }
            AuraVal::Bool(b) => out.push(Value::Integer(if b { 1 } else { 0 })),
            AuraVal::Null => out.push(Value::Null),
            _ => {
                let jval = crate::record::val_to_json_value(&aval);
                let text = serde_json::to_string(&jval).unwrap_or_default();
                out.push(Value::Text(text));
            }
        }
    }
    out
}

// -----------------------------------------------------------------------------
// C ABI EXPORTS FOR CRANELIFT AND NATIVE EXECUTABLE DISTRIBUTION
// -----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_open(
    driver_ptr: *const u8,
    driver_len: usize,
    dsn_ptr: *const u8,
    dsn_len: usize,
    max_conns: i32,
) -> *mut AuraDbPool {
    let driver = if !driver_ptr.is_null() && driver_len > 0 {
        let bytes = unsafe { slice::from_raw_parts(driver_ptr, driver_len) };
        str::from_utf8(bytes).unwrap_or("sqlite")
    } else {
        "sqlite"
    };

    let dsn = if !dsn_ptr.is_null() && dsn_len > 0 {
        let bytes = unsafe { slice::from_raw_parts(dsn_ptr, dsn_len) };
        str::from_utf8(bytes).unwrap_or(":memory:")
    } else {
        ":memory:"
    };

    match AuraDbPool::new(driver, dsn, max_conns.max(1) as usize) {
        Ok(pool) => {
            let raw = Arc::into_raw(pool) as *mut AuraDbPool;
            crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_DB_POOL);
            raw
        }
        Err(err) => {
            eprintln!("[Aura DB Error] Failed to open database pool: {}", err);
            std::ptr::null_mut()
        }
    }
}

fn format_postgres_literal(val: &Value) -> String {
    match val {
        Value::Null => "NULL".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => f.to_string(),
        Value::Text(s) => {
            let escaped = s.replace('\'', "''");
            format!("'{}'", escaped)
        }
        Value::Blob(b) => {
            let mut hex = String::with_capacity(4 + b.len() * 2);
            hex.push_str("'\\x");
            for byte in b {
                use std::fmt::Write as _;
                let _ = write!(hex, "{:02x}", byte);
            }
            hex.push('\'');
            hex
        }
    }
}

fn bind_postgres_sql(sql: &str, params: &[Value]) -> String {
    if params.is_empty() {
        return sql.to_string();
    }
    let mut result = sql.to_string();
    for (i, param) in params.iter().enumerate() {
        let placeholder = format!("${}", i + 1);
        let literal = format_postgres_literal(param);
        result = result.replace(&placeholder, &literal);
    }
    result
}

fn format_mysql_literal(val: &Value) -> String {
    match val {
        Value::Null => "NULL".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => f.to_string(),
        Value::Text(s) => {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('\'');
            for c in s.chars() {
                match c {
                    '\'' => out.push_str("''"),
                    '\\' => out.push_str("\\\\"),
                    '\0' => out.push_str("\\0"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    _ => out.push(c),
                }
            }
            out.push('\'');
            out
        }
        Value::Blob(b) => {
            let mut hex = String::with_capacity(3 + b.len() * 2);
            hex.push_str("X'");
            for byte in b {
                use std::fmt::Write as _;
                let _ = write!(hex, "{:02x}", byte);
            }
            hex.push('\'');
            hex
        }
    }
}

fn bind_mysql_sql(sql: &str, params: &[Value]) -> String {
    if params.is_empty() {
        return sql.to_string();
    }
    let mut out = String::with_capacity(sql.len() + params.len() * 16);
    let mut param_idx = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            out.push(c);
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            out.push(c);
        } else if c == '?' && !in_single_quote && !in_double_quote {
            if param_idx < params.len() {
                out.push_str(&format_mysql_literal(&params[param_idx]));
                param_idx += 1;
            } else {
                out.push('?');
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

#[unsafe(no_mangle)]
fn execute_query_on_conn(
    conn: &mut AuraDbConn,
    sql: &str,
    params: *mut AuraList,
) -> Result<AuraDbRows, String> {
    match &mut conn.backend {
        #[cfg(feature = "sqlite")]
        DbBackend::Sqlite(sqlite) => {
            let sql_params = extract_aura_params(params);
            let stmt_res = sqlite.prepare(sql);
            match stmt_res {
                Ok(mut stmt) => {
                    let col_names: Vec<String> = stmt
                        .column_names()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect();
                    let mut rows_data = Vec::new();

                    let query_res = stmt.query_map(params_from_iter(sql_params.iter()), |row| {
                        let mut row_vals = Vec::with_capacity(col_names.len());
                        for i in 0..col_names.len() {
                            let val_ref = row.get_ref(i)?;
                            let aval = match val_ref {
                                ValueRef::Null => AuraVal::Null,
                                ValueRef::Integer(n) => AuraVal::Int(n),
                                ValueRef::Real(f) => AuraVal::Float(f),
                                ValueRef::Text(t) => {
                                    let s = str::from_utf8(t).unwrap_or("");
                                    AuraVal::String(aura_string_from_rust_str(s))
                                }
                                ValueRef::Blob(b) => {
                                    let s = String::from_utf8_lossy(b).to_string();
                                    AuraVal::String(aura_string_from_rust_str(&s))
                                }
                            };
                            row_vals.push(aval);
                        }
                        Ok(row_vals)
                    });

                    match query_res {
                        Ok(mapped_rows) => {
                            for r in mapped_rows {
                                if let Ok(val) = r {
                                    rows_data.push(val);
                                }
                            }
                            Ok(AuraDbRows {
                                columns: col_names,
                                rows: rows_data,
                                current_idx: -1,
                                last_error: None,
                            })
                        }
                        Err(e) => Err(format!("SQLite query execution error: {}", e)),
                    }
                }
                Err(e) => Err(format!("SQLite query prepare error: {}", e)),
            }
        }
        DbBackend::Postgres(pg) => {
            let sql_params = extract_aura_params(params);
            let final_sql = bind_postgres_sql(sql, &sql_params);
            match pg.query(&final_sql) {
                Ok(q_res) => {
                    let mut rows_data = Vec::new();
                    for pg_row in q_res.rows {
                        let mut row_vals = Vec::new();
                        for val_opt in pg_row.values {
                            let aval = match val_opt {
                                Some(s) => AuraVal::String(aura_string_from_rust_str(&s)),
                                None => AuraVal::Null,
                            };
                            row_vals.push(aval);
                        }
                        rows_data.push(row_vals);
                    }
                    Ok(AuraDbRows {
                        columns: q_res.columns,
                        rows: rows_data,
                        current_idx: -1,
                        last_error: None,
                    })
                }
                Err(e) => Err(e),
            }
        }
        DbBackend::MySql(mysql) => {
            let sql_params = extract_aura_params(params);
            let final_sql = bind_mysql_sql(sql, &sql_params);
            match mysql.query(&final_sql) {
                Ok(q_res) => {
                    let mut rows_data = Vec::new();
                    for my_row in q_res.rows {
                        let mut row_vals = Vec::new();
                        for val_opt in my_row.values {
                            let aval = match val_opt {
                                Some(s) => AuraVal::String(aura_string_from_rust_str(&s)),
                                None => AuraVal::Null,
                            };
                            row_vals.push(aval);
                        }
                        rows_data.push(row_vals);
                    }
                    Ok(AuraDbRows {
                        columns: q_res.columns,
                        rows: rows_data,
                        current_idx: -1,
                        last_error: None,
                    })
                }
                Err(e) => Err(e),
            }
        }
    }
}

fn execute_exec_on_conn(
    conn: &mut AuraDbConn,
    sql: &str,
    params: *mut AuraList,
) -> Result<(i64, i64), String> {
    match &mut conn.backend {
        #[cfg(feature = "sqlite")]
        DbBackend::Sqlite(sqlite) => {
            let sql_params = extract_aura_params(params);
            if sql_params.is_empty() {
                match sqlite.execute_batch(sql) {
                    Ok(()) => {
                        let affected = sqlite.changes() as i64;
                        let last_id = sqlite.last_insert_rowid();
                        Ok((affected, last_id))
                    }
                    Err(e) => Err(format!("SQLite exec error: {}", e)),
                }
            } else {
                match sqlite.execute(sql, params_from_iter(sql_params.iter())) {
                    Ok(affected) => {
                        let last_id = sqlite.last_insert_rowid();
                        Ok((affected as i64, last_id))
                    }
                    Err(e) => Err(format!("SQLite exec error: {}", e)),
                }
            }
        }
        DbBackend::Postgres(pg) => {
            let sql_params = extract_aura_params(params);
            let final_sql = bind_postgres_sql(sql, &sql_params);
            match pg.query(&final_sql) {
                Ok(res) => Ok((res.rows_affected, res.last_insert_id)),
                Err(e) => Err(e),
            }
        }
        DbBackend::MySql(mysql) => {
            let sql_params = extract_aura_params(params);
            let final_sql = bind_mysql_sql(sql, &sql_params);
            match mysql.query(&final_sql) {
                Ok(res) => Ok((res.rows_affected, res.last_insert_id)),
                Err(e) => Err(e),
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_query(
    handle: *mut (),
    sql_ptr: *const u8,
    sql_len: usize,
    params: *mut AuraList,
) -> *mut AuraDbRows {
    if handle.is_null() || sql_ptr.is_null() || sql_len == 0 {
        let rows = Box::new(AuraDbRows {
            columns: Vec::new(),
            rows: Vec::new(),
            current_idx: -1,
            last_error: Some("Invalid handle or SQL query".to_string()),
        });
        return Box::into_raw(rows);
    }

    let sql = unsafe {
        let bytes = slice::from_raw_parts(sql_ptr, sql_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    let type_id = crate::gc::lookup_valid_ptr(handle as usize);
    let result = match type_id {
        Some(crate::gc::TYPE_DB_TX) => {
            let tx = unsafe { &mut *(handle as *mut AuraDbTx) };
            if let Some(ref mut conn) = tx.conn {
                execute_query_on_conn(conn, sql, params)
            } else {
                Err("Transaction is inactive or closed".to_string())
            }
        }
        Some(crate::gc::TYPE_DB_POOL) => {
            let pool_arc = unsafe { Arc::from_raw(handle as *mut AuraDbPool) };
            let pool_clone = Arc::clone(&pool_arc);
            std::mem::forget(pool_arc);

            match pool_clone.acquire() {
                Ok(mut conn) => {
                    let res = execute_query_on_conn(&mut conn, sql, params);
                    pool_clone.release(conn);
                    res
                }
                Err(e) => Err(e),
            }
        }
        _ => Err("Invalid or unallocated database handle passed to query".to_string()),
    };

    let rows = match result {
        Ok(r) => Box::new(r),
        Err(err) => Box::new(AuraDbRows {
            columns: Vec::new(),
            rows: Vec::new(),
            current_idx: -1,
            last_error: Some(err),
        }),
    };

    let raw = Box::into_raw(rows);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_DB_ROWS);
    raw
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_exec(
    handle: *mut (),
    sql_ptr: *const u8,
    sql_len: usize,
    params: *mut AuraList,
) -> *mut AuraExecResult {
    if handle.is_null() || sql_ptr.is_null() || sql_len == 0 {
        let res = Box::new(AuraExecResult {
            rows_affected: 0,
            last_insert_id: 0,
            error: Some("Invalid handle or SQL command".to_string()),
        });
        return Box::into_raw(res);
    }

    let sql = unsafe {
        let bytes = slice::from_raw_parts(sql_ptr, sql_len);
        str::from_utf8(bytes).unwrap_or("")
    };

    let type_id = crate::gc::lookup_valid_ptr(handle as usize);
    let result = match type_id {
        Some(crate::gc::TYPE_DB_TX) => {
            let tx = unsafe { &mut *(handle as *mut AuraDbTx) };
            if let Some(ref mut conn) = tx.conn {
                execute_exec_on_conn(conn, sql, params)
            } else {
                Err("Transaction is inactive or closed".to_string())
            }
        }
        Some(crate::gc::TYPE_DB_POOL) => {
            let pool_arc = unsafe { Arc::from_raw(handle as *mut AuraDbPool) };
            let pool_clone = Arc::clone(&pool_arc);
            std::mem::forget(pool_arc);

            match pool_clone.acquire() {
                Ok(mut conn) => {
                    let res = execute_exec_on_conn(&mut conn, sql, params);
                    pool_clone.release(conn);
                    res
                }
                Err(e) => Err(e),
            }
        }
        _ => Err("Invalid or unallocated database handle passed to exec".to_string()),
    };

    let exec_res = match result {
        Ok((affected, insert_id)) => Box::new(AuraExecResult {
            rows_affected: affected,
            last_insert_id: insert_id,
            error: None,
        }),
        Err(err) => Box::new(AuraExecResult {
            rows_affected: 0,
            last_insert_id: 0,
            error: Some(err),
        }),
    };

    let raw = Box::into_raw(exec_res);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_DB_EXEC_RESULT);
    raw
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_begin(pool: *mut AuraDbPool) -> *mut AuraDbTx {
    if pool.is_null() {
        return std::ptr::null_mut();
    }

    let pool_arc = unsafe { Arc::from_raw(pool) };
    let pool_clone = Arc::clone(&pool_arc);
    std::mem::forget(pool_arc);

    let mut conn = match pool_clone.acquire() {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[Aura DB Error] Failed to acquire connection for transaction: {}",
                e
            );
            return std::ptr::null_mut();
        }
    };

    // Execute BEGIN
    let begin_res = match &mut conn.backend {
        #[cfg(feature = "sqlite")]
        DbBackend::Sqlite(sqlite) => sqlite
            .execute_batch("BEGIN TRANSACTION;")
            .map_err(|e| e.to_string()),
        DbBackend::Postgres(pg) => pg.query("BEGIN;").map(|_| ()),
        DbBackend::MySql(mysql) => mysql.query("START TRANSACTION;").map(|_| ()),
    };

    if let Err(err) = begin_res {
        eprintln!("[Aura DB Error] BEGIN TRANSACTION failed: {}", err);
        pool_clone.release(conn);
        return std::ptr::null_mut();
    }

    let tx = Box::new(AuraDbTx {
        conn: Some(conn),
        pool: pool_clone,
        is_active: true,
        savepoint_level: 0,
    });

    let raw = Box::into_raw(tx);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_DB_TX);
    raw
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_tx_commit(tx: *mut AuraDbTx) -> i32 {
    if tx.is_null() {
        return -1;
    }
    unsafe {
        let t = &mut *tx;
        if !t.is_active || t.conn.is_none() {
            return -1;
        }

        let mut conn = t.conn.take().unwrap();
        let res = match &mut conn.backend {
            #[cfg(feature = "sqlite")]
            DbBackend::Sqlite(sqlite) => sqlite.execute_batch("COMMIT;").map_err(|e| e.to_string()),
            DbBackend::Postgres(pg) => pg.query("COMMIT;").map(|_| ()),
            DbBackend::MySql(mysql) => mysql.query("COMMIT;").map(|_| ()),
        };

        t.is_active = false;
        t.pool.release(conn);

        if res.is_ok() { 0 } else { -1 }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_tx_rollback(tx: *mut AuraDbTx) -> i32 {
    if tx.is_null() {
        return -1;
    }
    unsafe {
        let t = &mut *tx;
        if !t.is_active || t.conn.is_none() {
            return -1;
        }

        let mut conn = t.conn.take().unwrap();
        let res = match &mut conn.backend {
            #[cfg(feature = "sqlite")]
            DbBackend::Sqlite(sqlite) => {
                sqlite.execute_batch("ROLLBACK;").map_err(|e| e.to_string())
            }
            DbBackend::Postgres(pg) => pg.query("ROLLBACK;").map(|_| ()),
            DbBackend::MySql(mysql) => mysql.query("ROLLBACK;").map(|_| ()),
        };

        t.is_active = false;
        t.pool.release(conn);

        if res.is_ok() { 0 } else { -1 }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_tx_close(tx: *mut AuraDbTx) {
    if !tx.is_null() && crate::gc::lookup_valid_ptr(tx as usize) == Some(crate::gc::TYPE_DB_TX) {
        crate::gc::unregister_valid_ptr(tx as usize);
        unsafe {
            let mut t = Box::from_raw(tx);
            if t.is_active {
                if let Some(conn) = t.conn.take() {
                    let mut c = conn;
                    let _ = match &mut c.backend {
                        #[cfg(feature = "sqlite")]
                        DbBackend::Sqlite(sqlite) => {
                            sqlite.execute_batch("ROLLBACK;").map_err(|e| e.to_string())
                        }
                        DbBackend::Postgres(pg) => pg.query("ROLLBACK;").map(|_| ()),
                        DbBackend::MySql(mysql) => mysql.query("ROLLBACK;").map(|_| ()),
                    };
                    t.is_active = false;
                    t.pool.release(c);
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_next(rows: *mut AuraDbRows) -> i32 {
    if rows.is_null() {
        return 0;
    }
    unsafe {
        let r = &mut *rows;
        r.current_idx += 1;
        if r.current_idx >= 0 && (r.current_idx as usize) < r.rows.len() {
            1
        } else {
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_col_count(rows: *mut AuraDbRows) -> i32 {
    if rows.is_null() {
        return 0;
    }
    unsafe { (*rows).columns.len() as i32 }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_col_name(rows: *mut AuraDbRows, idx: i32) -> *mut AuraString {
    if rows.is_null() || idx < 0 {
        return aura_string_from_rust_str("");
    }
    unsafe {
        let r = &*rows;
        if (idx as usize) < r.columns.len() {
            aura_string_from_rust_str(&r.columns[idx as usize])
        } else {
            aura_string_from_rust_str("")
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_get_val(rows: *mut AuraDbRows, idx: i32) -> i64 {
    if rows.is_null() || idx < 0 {
        return 0;
    }
    unsafe {
        let r = &*rows;
        if r.current_idx >= 0 && (r.current_idx as usize) < r.rows.len() {
            let row = &r.rows[r.current_idx as usize];
            if (idx as usize) < row.len() {
                return match &row[idx as usize] {
                    AuraVal::Int(n) => *n,
                    AuraVal::Float(f) => *f as i64,
                    AuraVal::Bool(b) => {
                        if *b {
                            1
                        } else {
                            0
                        }
                    }
                    AuraVal::String(s) => *s as i64,
                    AuraVal::Record(rec) => *rec as i64,
                    AuraVal::List(l) => *l as i64,
                    AuraVal::Null => 0,
                };
            }
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_get_str(rows: *mut AuraDbRows, idx: i32) -> *mut AuraString {
    if rows.is_null() || idx < 0 {
        return aura_string_from_rust_str("");
    }
    unsafe {
        let r = &*rows;
        if r.current_idx >= 0 && (r.current_idx as usize) < r.rows.len() {
            let row = &r.rows[r.current_idx as usize];
            if (idx as usize) < row.len() {
                return match &row[idx as usize] {
                    AuraVal::String(s) => *s,
                    AuraVal::Int(n) => aura_string_from_rust_str(&n.to_string()),
                    AuraVal::Float(f) => aura_string_from_rust_str(&f.to_string()),
                    AuraVal::Bool(b) => {
                        aura_string_from_rust_str(if *b { "true" } else { "false" })
                    }
                    _ => aura_string_from_rust_str(""),
                };
            }
        }
    }
    aura_string_from_rust_str("")
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_get_i64(rows: *mut AuraDbRows, idx: i32) -> i64 {
    if rows.is_null() || idx < 0 {
        return 0;
    }
    unsafe {
        let r = &*rows;
        if r.current_idx >= 0 && (r.current_idx as usize) < r.rows.len() {
            let row = &r.rows[r.current_idx as usize];
            if (idx as usize) < row.len() {
                return match &row[idx as usize] {
                    AuraVal::Int(n) => *n,
                    AuraVal::Float(f) => *f as i64,
                    AuraVal::Bool(b) => {
                        if *b {
                            1
                        } else {
                            0
                        }
                    }
                    AuraVal::String(s) => {
                        if !s.is_null() {
                            (**s).as_str().parse().unwrap_or(0)
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };
            }
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_get_f64(rows: *mut AuraDbRows, idx: i32) -> i64 {
    if rows.is_null() || idx < 0 {
        return 0f64.to_bits() as i64;
    }
    unsafe {
        let r = &*rows;
        if r.current_idx >= 0 && (r.current_idx as usize) < r.rows.len() {
            let row = &r.rows[r.current_idx as usize];
            if (idx as usize) < row.len() {
                let f = match &row[idx as usize] {
                    AuraVal::Float(f) => *f,
                    AuraVal::Int(n) => *n as f64,
                    AuraVal::String(s) => {
                        if !s.is_null() {
                            (**s).as_str().parse().unwrap_or(0.0)
                        } else {
                            0.0
                        }
                    }
                    _ => 0.0,
                };
                return f.to_bits() as i64;
            }
        }
    }
    0f64.to_bits() as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_get_bool(rows: *mut AuraDbRows, idx: i32) -> bool {
    if rows.is_null() || idx < 0 {
        return false;
    }
    unsafe {
        let r = &*rows;
        if r.current_idx >= 0 && (r.current_idx as usize) < r.rows.len() {
            let row = &r.rows[r.current_idx as usize];
            if (idx as usize) < row.len() {
                return match &row[idx as usize] {
                    AuraVal::Bool(b) => *b,
                    AuraVal::Int(n) => *n != 0,
                    AuraVal::String(s) => {
                        if !s.is_null() {
                            let str_val = (**s).as_str();
                            str_val == "true" || str_val == "1" || str_val == "t"
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_to_record(rows: *mut AuraDbRows) -> *mut AuraRecord {
    let rec = aura_record_new();
    if rows.is_null() {
        return rec;
    }
    unsafe {
        let r = &*rows;
        if r.current_idx >= 0 && (r.current_idx as usize) < r.rows.len() {
            let row = &r.rows[r.current_idx as usize];
            for (i, col_name) in r.columns.iter().enumerate() {
                if i < row.len() {
                    let val = row[i].clone();
                    aura_record_set_val(rec, col_name.as_ptr(), col_name.len(), val);
                }
            }
        }
    }
    rec
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_to_list(rows: *mut AuraDbRows) -> *mut AuraList {
    if rows.is_null() {
        return aura_list_new(0);
    }
    unsafe {
        let r = &*rows;
        let list = aura_list_new(r.rows.len());
        for row_idx in 0..r.rows.len() {
            let rec = aura_record_new();
            let row = &r.rows[row_idx];
            for (i, col_name) in r.columns.iter().enumerate() {
                if i < row.len() {
                    let val = row[i].clone();
                    aura_record_set_val(rec, col_name.as_ptr(), col_name.len(), val);
                }
            }
            aura_list_push(list, rec as i64);
        }
        list
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_error(rows: *mut AuraDbRows) -> *mut AuraString {
    if rows.is_null() {
        return aura_string_from_rust_str("");
    }
    unsafe {
        let r = &*rows;
        match &r.last_error {
            Some(err) => aura_string_from_rust_str(err),
            None => aura_string_from_rust_str(""),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_rows_close(rows: *mut AuraDbRows) {
    if !rows.is_null() {
        unsafe {
            crate::gc::unregister_valid_ptr(rows as usize);
            let _ = Box::from_raw(rows);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_exec_rows_affected(res: *mut AuraExecResult) -> i64 {
    if res.is_null() {
        return 0;
    }
    unsafe { (*res).rows_affected }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_exec_last_insert_id(res: *mut AuraExecResult) -> i64 {
    if res.is_null() {
        return 0;
    }
    unsafe { (*res).last_insert_id }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_exec_error(res: *mut AuraExecResult) -> *mut AuraString {
    if res.is_null() {
        return aura_string_from_rust_str("");
    }
    unsafe {
        let r = &*res;
        match &r.error {
            Some(err) => aura_string_from_rust_str(err),
            None => aura_string_from_rust_str(""),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_exec_free(res: *mut AuraExecResult) {
    if !res.is_null() {
        unsafe {
            crate::gc::unregister_valid_ptr(res as usize);
            let _ = Box::from_raw(res);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_pool_close(pool: *mut AuraDbPool) {
    if !pool.is_null()
        && crate::gc::lookup_valid_ptr(pool as usize) == Some(crate::gc::TYPE_DB_POOL)
    {
        crate::gc::unregister_valid_ptr(pool as usize);
        unsafe {
            let pool_arc = Arc::from_raw(pool);
            *pool_arc.is_closed.lock().unwrap() = true;
            pool_arc.conns.lock().unwrap().clear();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_close_any(ptr: *mut ()) {
    if ptr.is_null() {
        return;
    }
    let type_id = crate::gc::lookup_valid_ptr(ptr as usize);
    match type_id {
        Some(crate::gc::TYPE_DB_ROWS) => {
            aura_db_rows_close(ptr as *mut AuraDbRows);
        }
        Some(crate::gc::TYPE_DB_TX) => {
            aura_db_tx_close(ptr as *mut AuraDbTx);
        }
        Some(crate::gc::TYPE_DB_POOL) => {
            aura_db_pool_close(ptr as *mut AuraDbPool);
        }
        Some(crate::gc::TYPE_REDIS_CLIENT) => {
            crate::redis_client::aura_redis_close(ptr as *mut crate::redis_client::AuraRedisClient);
        }
        Some(crate::gc::TYPE_MONGO_CLIENT) => {
            crate::mongo_client::aura_mongo_close(ptr as *mut crate::mongo_client::AuraMongoClient);
        }
        _ => {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_db_ping(pool: *mut AuraDbPool) -> i64 {
    if pool.is_null() {
        return -1;
    }
    let type_id = crate::gc::lookup_valid_ptr(pool as usize);
    if type_id == Some(crate::gc::TYPE_REDIS_CLIENT) {
        let pong =
            crate::redis_client::aura_redis_ping(pool as *mut crate::redis_client::AuraRedisClient);
        return pong as i64;
    }
    if type_id == Some(crate::gc::TYPE_MONGO_CLIENT) {
        return crate::mongo_client::aura_mongo_ping(
            pool as *mut crate::mongo_client::AuraMongoClient,
        ) as i64;
    }
    unsafe {
        let pool_arc = Arc::from_raw(pool);
        let pool_clone = Arc::clone(&pool_arc);
        std::mem::forget(pool_arc);

        match pool_clone.acquire() {
            Ok(conn) => {
                pool_clone.release(conn);
                0
            }
            Err(_) => -1,
        }
    }
}
