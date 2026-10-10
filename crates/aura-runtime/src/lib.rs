//! Native Standalone Runtime for Aura Language.
//! Provides M:N Fiber Scheduler, Garbage Collector (GC), CSP Channels, and C ABI.

// The C ABI functions exported by this runtime receive raw pointers across FFI from Cranelift native binaries.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod c_api;
pub mod channel;
pub mod context;
pub mod db;
pub mod fiber;
pub mod gc;
pub mod http;
pub mod json;
pub mod list;
pub mod mysql_client;
pub mod os;
pub mod postgres_client;
pub mod record;
pub mod scheduler;
pub mod string;
pub mod sync;
pub mod variant;

pub use c_api::*;
pub use channel::{AuraChannel, channel_metrics};
pub use db::{
    AuraDbConn, AuraDbPool, AuraDbRows, AuraDbTx, AuraExecResult, aura_db_begin, aura_db_close_any,
    aura_db_exec, aura_db_exec_free, aura_db_exec_last_insert_id, aura_db_exec_rows_affected,
    aura_db_open, aura_db_ping, aura_db_pool_close, aura_db_query, aura_db_rows_close,
    aura_db_rows_col_count, aura_db_rows_col_name, aura_db_rows_get_bool, aura_db_rows_get_f64,
    aura_db_rows_get_i64, aura_db_rows_get_str, aura_db_rows_get_val, aura_db_rows_next,
    aura_db_rows_to_list, aura_db_rows_to_record, aura_db_tx_close, aura_db_tx_commit,
    aura_db_tx_rollback,
};
pub use gc::{collect_garbage, gc_alloc, gc_safepoint};
pub use http::{AuraHttpRequest, AuraHttpResponse, AuraServeMux};
pub use json::AuraJsonDoc;
pub use list::AuraList;
pub use record::{AuraRecord, AuraVal};
pub use scheduler::{get_scheduler, init_scheduler, yield_now};
pub use string::AuraString;
pub use sync::AuraMutex;
pub use variant::AuraVariant;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, Ordering};

    #[test]
    fn test_gc_alloc_and_collect() {
        let ptr1 = gc_alloc(64, 1);
        assert!(!ptr1.is_null());
        unsafe {
            std::ptr::write(ptr1 as *mut i64, 42);
            assert_eq!(*(ptr1 as *const i64), 42);
        }

        let ptr2 = gc_alloc(128, 2);
        assert!(!ptr2.is_null());

        collect_garbage();
    }

    static TEST_SCHEDULER_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_fiber_execution_and_yielding() {
        let _guard = TEST_SCHEDULER_MUTEX.lock().unwrap();
        init_scheduler();
        let sched = get_scheduler();

        static COUNTER: AtomicI64 = AtomicI64::new(0);

        extern "C" fn worker_fiber(_arg: *mut ()) {
            for _ in 0..5 {
                COUNTER.fetch_add(1, Ordering::SeqCst);
                yield_now();
            }
        }

        sched.spawn(worker_fiber, std::ptr::null_mut());
        sched.spawn(worker_fiber, std::ptr::null_mut());

        sched.run_workers(1);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_csp_channel_handoff_between_fibers() {
        let _guard = TEST_SCHEDULER_MUTEX.lock().unwrap();
        init_scheduler();
        let sched = get_scheduler();

        let chan = Arc::new(AuraChannel::new(0)); // Unbuffered rendezvous channel
        let chan_clone = Arc::clone(&chan);

        static RESULT: AtomicI64 = AtomicI64::new(0);

        let raw_chan1 = Arc::into_raw(chan) as *mut ();
        let raw_chan2 = Arc::into_raw(chan_clone) as *mut ();

        extern "C" fn sender(arg: *mut ()) {
            let chan = unsafe { &*(arg as *const AuraChannel) };
            chan.send(42 as *mut ());
        }

        extern "C" fn receiver(arg: *mut ()) {
            let chan = unsafe { &*(arg as *const AuraChannel) };
            let val = chan.recv();
            RESULT.store(val as i64, Ordering::SeqCst);
        }

        sched.spawn(sender, raw_chan1);
        sched.spawn(receiver, raw_chan2);

        sched.run_workers(1);

        assert_eq!(RESULT.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_mutex_lock_and_unlock() {
        let m = sync::aura_mutex_new();
        sync::aura_mutex_lock(m);
        sync::aura_mutex_unlock(m);
        sync::aura_mutex_free(m);
    }

    #[test]
    fn test_list_operations() {
        let list = list::aura_list_new(2);
        assert_eq!(list::aura_list_len(list), 0);

        list::aura_list_push(list, 100);
        list::aura_list_push(list, 200);
        list::aura_list_push(list, 300);

        assert_eq!(list::aura_list_len(list), 3);
        assert_eq!(list::aura_list_get(list, 0), 100);
        assert_eq!(list::aura_list_get(list, 1), 200);
        assert_eq!(list::aura_list_get(list, 2), 300);

        list::aura_list_set(list, 1, 999);
        assert_eq!(list::aura_list_get(list, 1), 999);

        list::aura_list_free(list);
    }

    #[test]
    fn test_json_parsing_and_stringify() {
        let json_text = r#"{"id":"ord-101","total":227.29,"active":true}"#;
        let doc = json::aura_json_parse(json_text.as_ptr(), json_text.len());

        let id_str = json::aura_json_get_str(doc, "id".as_ptr(), 2);
        assert_eq!(unsafe { (*id_str).as_str() }, "ord-101");

        let serialized = json::aura_json_stringify(doc);
        let s = unsafe { (*serialized).as_str() };
        assert!(s.contains("ord-101"));
    }

    #[test]
    fn test_http_serve_mux_registration() {
        let mux = http::aura_http_new_serve_mux();

        extern "C" fn dummy_handler(_req: *mut AuraHttpRequest, _res: *mut AuraHttpResponse) {}

        let path = "/api/orders/:id";
        http::aura_http_mux_handle(
            mux,
            "GET".as_ptr(),
            3,
            path.as_ptr(),
            path.len(),
            dummy_handler,
        );

        let mux_ref = unsafe { &*mux };
        assert_eq!(mux_ref.routes.len(), 1);
        assert_eq!(mux_ref.routes[0].method, "GET");
        assert_eq!(mux_ref.routes[0].pattern, "/api/orders/:id");
    }

    #[test]
    fn test_record_json_serialization() {
        let rec = record::aura_record_new();
        let key_id = "id";
        let key_total = "total";
        let id_val = string::aura_string_from_rust_str("ord-202");
        record::aura_record_set_str(rec, key_id.as_ptr(), key_id.len(), id_val);
        record::aura_record_set_f64(rec, key_total.as_ptr(), key_total.len(), 99.95);

        let json_str_ptr = record::aura_record_to_json(rec);
        let s = unsafe { (*json_str_ptr).as_str() };
        assert!(s.contains("\"id\":\"ord-202\""));
        assert!(s.contains("\"total\":99.95"));
    }

    #[test]
    fn test_variant_option_and_result() {
        let ok_res = variant::aura_result_ok(42);
        assert_eq!(variant::aura_variant_tag(ok_res), 1);
        assert_eq!(variant::aura_variant_val(ok_res), 42);

        let err_res = variant::aura_result_err(99);
        assert_eq!(variant::aura_variant_tag(err_res), 2);
        assert_eq!(variant::aura_variant_val(err_res), 99);
    }

    #[test]
    fn test_security_arbitrary_int_not_dereferenced() {
        let test_vals = [
            0x10008i64,
            0x20000i64,
            1_048_576i64,
            1_726_000_000i64,
            0x7fff_0000_0008i64,
        ];

        for &val in &test_vals {
            let aval = record::infer_val_from_i64(val);
            match aval {
                record::AuraVal::Int(i) => assert_eq!(i, val),
                other => panic!("Expected AuraVal::Int for {}, got {:?}", val, other),
            }
        }
    }

    #[test]
    fn test_security_http_error_json_escaping() {
        let res = Box::into_raw(Box::new(http::AuraHttpResponse {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: Vec::new(),
        }));

        let malicious_msg = "test\", \"admin\": true, \"inject\": \"";
        http::aura_http_error(res, malicious_msg.as_ptr(), malicious_msg.len(), 400);

        let res_ref = unsafe { &*res };
        let body_str = std::str::from_utf8(&res_ref.body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(body_str).expect("Must be valid JSON");
        assert_eq!(parsed["error"], malicious_msg);
        assert!(parsed.get("admin").is_none());

        unsafe {
            drop(Box::from_raw(res));
        }
    }

    #[test]
    fn test_security_crlf_header_injection_prevention() {
        let res = Box::into_raw(Box::new(http::AuraHttpResponse {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: Vec::new(),
        }));

        let k = "X-Injected\r\nSet-Cookie: evil=true";
        let v = "safe_val\r\nEvil-Header: true";
        http::aura_http_set_header(res, k.as_ptr(), k.len(), v.as_ptr(), v.len());

        let res_ref = unsafe { &*res };
        for (header_k, header_v) in &res_ref.headers {
            assert!(!header_k.contains('\r') && !header_k.contains('\n'));
            assert!(!header_v.contains('\r') && !header_v.contains('\n'));
        }

        unsafe {
            drop(Box::from_raw(res));
        }
    }

    #[test]
    fn test_security_channel_send_closed_no_panic() {
        let chan = channel::AuraChannel::new(1);
        chan.close();
        chan.send(123 as *mut ());
        assert!(chan.recv().is_null());
    }

    #[test]
    fn test_native_sqlite_db_lifecycle_and_crud() {
        let driver = "sqlite";
        let dsn = ":memory:";
        let pool = aura_db_open(driver.as_ptr(), driver.len(), dsn.as_ptr(), dsn.len(), 5);
        assert!(!pool.is_null());

        // Ping test
        assert_eq!(aura_db_ping(pool), 0);

        // CREATE TABLE
        let ddl = "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, balance REAL NOT NULL, active INTEGER DEFAULT 1);";
        let exec_res = aura_db_exec(pool as *mut (), ddl.as_ptr(), ddl.len(), std::ptr::null_mut());
        assert!(!exec_res.is_null());
        assert_eq!(aura_db_exec_rows_affected(exec_res), 0);
        aura_db_exec_free(exec_res);

        // INSERT Alice
        let insert_sql = "INSERT INTO users (name, balance, active) VALUES (?, ?, ?);";
        let params = list::aura_list_new(3);
        let alice_str = string::aura_string_from_rust_str("Alice");
        list::aura_list_push(params, alice_str as i64);
        list::aura_list_push(params, 1500 as i64); // balance
        list::aura_list_push(params, 1 as i64);    // active = true

        let insert_res = aura_db_exec(pool as *mut (), insert_sql.as_ptr(), insert_sql.len(), params);
        assert!(!insert_res.is_null());
        assert_eq!(aura_db_exec_rows_affected(insert_res), 1);
        assert_eq!(aura_db_exec_last_insert_id(insert_res), 1);
        aura_db_exec_free(insert_res);

        // INSERT Bob
        let params_bob = list::aura_list_new(3);
        let bob_str = string::aura_string_from_rust_str("Bob");
        list::aura_list_push(params_bob, bob_str as i64);
        list::aura_list_push(params_bob, 2500 as i64);
        list::aura_list_push(params_bob, 1 as i64);

        let insert_bob_res = aura_db_exec(pool as *mut (), insert_sql.as_ptr(), insert_sql.len(), params_bob);
        assert_eq!(aura_db_exec_rows_affected(insert_bob_res), 1);
        assert_eq!(aura_db_exec_last_insert_id(insert_bob_res), 2);
        aura_db_exec_free(insert_bob_res);

        // SELECT Query
        let select_sql = "SELECT id, name, balance, active FROM users ORDER BY id ASC;";
        let rows = aura_db_query(pool as *mut (), select_sql.as_ptr(), select_sql.len(), std::ptr::null_mut());
        assert!(!rows.is_null());
        assert_eq!(aura_db_rows_col_count(rows), 4);

        // Row 1 (Alice)
        assert_eq!(aura_db_rows_next(rows), 1);
        let id1 = aura_db_rows_get_i64(rows, 0);
        assert_eq!(id1, 1);
        let name1 = aura_db_rows_get_str(rows, 1);
        assert_eq!(unsafe { (*name1).as_str() }, "Alice");

        // Convert Row 1 to AuraRecord
        let rec1 = aura_db_rows_to_record(rows);
        assert!(!rec1.is_null());
        let rec_name = record::aura_record_get(rec1, "name".as_ptr(), 4);
        assert_eq!(unsafe { (*(rec_name as *mut string::AuraString)).as_str() }, "Alice");

        // Row 2 (Bob)
        assert_eq!(aura_db_rows_next(rows), 1);
        let id2 = aura_db_rows_get_i64(rows, 0);
        assert_eq!(id2, 2);
        let name2 = aura_db_rows_get_str(rows, 1);
        assert_eq!(unsafe { (*name2).as_str() }, "Bob");

        // End of rows
        assert_eq!(aura_db_rows_next(rows), 0);
        aura_db_rows_close(rows);

        // Transaction Rollback test
        let tx = aura_db_begin(pool);
        assert!(!tx.is_null());

        let insert_temp = "INSERT INTO users (name, balance) VALUES ('TempUser', 100);";
        let tx_exec = aura_db_exec(tx as *mut (), insert_temp.as_ptr(), insert_temp.len(), std::ptr::null_mut());
        aura_db_exec_free(tx_exec);

        assert_eq!(aura_db_tx_rollback(tx), 0);

        aura_db_pool_close(pool);
    }
}
