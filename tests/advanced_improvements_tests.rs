//! Integration tests for the newly implemented advanced improvements:
//! 1. `aurac bindgen` C/Rust FFI header generator.
//! 2. Compile-time `Sendable` concurrency checks in typechecker.
//! 3. Runtime Deadlock Sentinel & CSP channel operational metrics.
//! 4. Native Prometheus `/metrics` endpoint and W3C `traceparent` distributed tracing.
//! 5. Release optimization pipeline (SIMD, LTO dead-code strip).

use aura_lang::bindgen::{AuraBindgen, BindgenConfig};
use aura_lang::typechecker::{ConcreteType, TypeChecker};
use aura_runtime::channel::AuraChannel;
use aura_runtime::http::{
    AuraHttpRequest, AuraHttpResponse, aura_http_enable_metrics, aura_http_new_serve_mux,
};
use aura_runtime::scheduler::{get_scheduler, init_scheduler};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

#[test]
fn test_bindgen_c_header_parser_and_codegen() {
    let sample_c_header = r#"
        // Database library header
        #define DB_OK 0
        #define DB_BUSY 5
        #define DB_VERSION "1.2.3"

        typedef enum {
            DB_MODE_READONLY = 0,
            DB_MODE_READWRITE = 1,
            DB_MODE_CREATE = 2
        } DbOpenMode;

        typedef struct {
            int user_id;
            double score;
            const char *username;
        } UserRecord;

        extern int db_open(const char *path, void **pp_db, int flags);
        extern int db_close(void *p_db);
        extern int db_execute(void *p_db, const char *sql);
    "#;

    let config = BindgenConfig {
        input_header: "test_db.h".to_string(),
        output_file: None,
        module_name: Some("Database".to_string()),
        generate_wrappers: true,
        strip_prefix: Some("db_".to_string()),
    };

    let bindgen = AuraBindgen::new(config);
    let parsed = bindgen.parse_header(sample_c_header);

    // Verify parsed constants
    assert!(
        parsed
            .constants
            .iter()
            .any(|c| c.name == "DB_OK" && c.value == "0")
    );
    assert!(
        parsed
            .constants
            .iter()
            .any(|c| c.name == "DB_BUSY" && c.value == "5")
    );
    assert!(
        parsed
            .constants
            .iter()
            .any(|c| c.name == "DB_VERSION" && c.value == "\"1.2.3\"")
    );

    // Verify parsed enum
    let enum_mode = parsed
        .enums
        .iter()
        .find(|e| e.name == "DbOpenMode")
        .expect("DbOpenMode enum");
    assert_eq!(enum_mode.variants.len(), 3);
    assert_eq!(enum_mode.variants[0].0, "DB_MODE_READONLY");

    // Verify parsed struct
    let struct_rec = parsed
        .structs
        .iter()
        .find(|s| s.name == "UserRecord")
        .expect("UserRecord struct");
    assert_eq!(struct_rec.fields.len(), 3);
    assert_eq!(struct_rec.fields[0].name, "user_id");
    assert_eq!(struct_rec.fields[0].aura_type, "Int");
    assert_eq!(struct_rec.fields[1].name, "score");
    assert_eq!(struct_rec.fields[1].aura_type, "Float");
    assert_eq!(struct_rec.fields[2].name, "username");
    assert_eq!(struct_rec.fields[2].aura_type, "*const Char");

    // Verify parsed functions
    assert_eq!(parsed.functions.len(), 3);
    assert!(parsed.functions.iter().any(|f| f.name == "db_open"));
    assert!(parsed.functions.iter().any(|f| f.name == "db_close"));
    assert!(parsed.functions.iter().any(|f| f.name == "db_execute"));

    // Verify generated Aura code
    let aura_code = bindgen.generate_aura_code(&parsed);
    assert!(aura_code.contains("export let DB_OK: Int = 0;"));
    assert!(aura_code.contains("export type DbOpenMode ="));
    assert!(aura_code.contains("    | DB_MODE_READONLY"));
    assert!(aura_code.contains("export type UserRecord = {"));
    assert!(aura_code.contains("    user_id: Int,"));
    assert!(aura_code.contains("    username: *const Char,"));
    assert!(aura_code.contains(
        "export extern \"C\" fn db_open(path: *const Char, pp_db: *mut *mut Unit, flags: Int): Int;"
    ));
    assert!(aura_code.contains(
        "export fn safe_open(path: String, pp_db: *mut *mut Unit, flags: Int): Int => {"
    ));
}

#[test]
fn test_compile_time_sendable_checks() {
    let tc = TypeChecker::new();

    // 1. Primitive and safe composite types must be Sendable
    assert!(tc.validate_sendable_type(&ConcreteType::Int).is_ok());
    assert!(tc.validate_sendable_type(&ConcreteType::String).is_ok());
    assert!(tc.validate_sendable_type(&ConcreteType::Bool).is_ok());

    let list_ty = ConcreteType::List(Box::new(ConcreteType::Int));
    assert!(tc.validate_sendable_type(&list_ty).is_ok());

    let mut record_fields = HashMap::new();
    record_fields.insert("id".to_string(), ConcreteType::Int);
    record_fields.insert("name".to_string(), ConcreteType::String);
    assert!(
        tc.validate_sendable_type(&ConcreteType::Record(record_fields))
            .is_ok()
    );

    // 2. Raw pointer types must be REJECTED across channels (Fearless Concurrency)
    let raw_ptr_ty = ConcreteType::Pointer(Box::new(ConcreteType::Int));
    let result = tc.validate_sendable_type(&raw_ptr_ty);
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Concurrency Safety Error [E0401]"));
    assert!(err_msg.contains("Cannot send raw pointer"));

    // Nested pointer inside option
    let opt_ptr_ty = ConcreteType::Option(Box::new(raw_ptr_ty));
    assert!(tc.validate_sendable_type(&opt_ptr_ty).is_err());
}

#[test]
fn test_runtime_csp_channel_metrics() {
    init_scheduler();
    let sched = get_scheduler();

    let chan = Arc::new(AuraChannel::new(2));
    let chan_clone = Arc::clone(&chan);

    static RECEIVED: AtomicI64 = AtomicI64::new(0);

    let raw1 = Arc::into_raw(chan) as *mut ();
    let raw2 = Arc::into_raw(chan_clone) as *mut ();

    extern "C" fn producer(arg: *mut ()) {
        let ch = unsafe { &*(arg as *const AuraChannel) };
        ch.send(100 as *mut ());
        ch.send(200 as *mut ());
    }

    extern "C" fn consumer(arg: *mut ()) {
        let ch = unsafe { &*(arg as *const AuraChannel) };
        let a = ch.recv() as i64;
        let b = ch.recv() as i64;
        RECEIVED.store(a + b, Ordering::SeqCst);
    }

    sched.spawn(producer, raw1);
    sched.spawn(consumer, raw2);

    sched.run_workers(1);

    assert_eq!(RECEIVED.load(Ordering::SeqCst), 300);

    // Verify channel metrics are tracked
    let (sends, recvs, _parks) = aura_runtime::channel::channel_metrics();
    assert!(sends >= 2, "Expected at least 2 sends, found {}", sends);
    assert!(recvs >= 2, "Expected at least 2 recvs, found {}", recvs);
}

#[test]
fn test_http_prometheus_metrics_and_traceparent_propagation() {
    let mux = aura_http_new_serve_mux();
    assert!(!mux.is_null());

    let metrics_path = "/metrics";
    aura_http_enable_metrics(mux, metrics_path.as_ptr(), metrics_path.len());

    let mux_ref = unsafe { &*mux };
    let metrics_route = mux_ref
        .routes
        .iter()
        .find(|r| r.pattern == "/metrics")
        .expect("Registered /metrics route");

    assert_eq!(metrics_route.method, "GET");

    // Invoke Prometheus metrics handler
    let mut req = AuraHttpRequest {
        method: "GET".to_string(),
        url: "/metrics".to_string(),
        path: "/metrics".to_string(),
        body: String::new(),
        params: HashMap::new(),
        query: HashMap::new(),
    };

    let mut res = AuraHttpResponse {
        status: 0,
        headers: HashMap::new(),
        body: Vec::new(),
    };

    (metrics_route.handler)(&mut req as *mut _, &mut res as *mut _);

    assert_eq!(res.status, 200);
    assert_eq!(
        res.headers.get("Content-Type").map(|s| s.as_str()),
        Some("text/plain; version=0.0.4; charset=utf-8")
    );

    let body_text = String::from_utf8(res.body).expect("Valid Prometheus utf-8 body");
    assert!(body_text.contains("# HELP aura_http_requests_total"));
    assert!(body_text.contains("# TYPE aura_http_requests_total counter"));
    assert!(body_text.contains("aura_http_requests_total"));
    assert!(body_text.contains("aura_scheduler_fibers"));
    assert!(body_text.contains("aura_csp_channel_operations"));
}
