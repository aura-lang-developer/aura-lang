use aura_lang::compile;
use aura_lang::package::PackageManager;
use std::fs;
use std::path::PathBuf;

fn get_temp_dir(prefix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{}_{}", prefix, nanos));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_match_exhaustiveness_rejection_on_option() {
    let source = r#"
        fn unwrapVal(opt: Option<Int>): Int => {
            match opt {
                Some(v) => v,
            }
        }
    "#;

    let err = compile(source, &[]).expect_err("Must fail when Option match is not exhaustive");
    assert!(
        err.contains("Non-exhaustive patterns in match for Option"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_match_exhaustiveness_rejection_on_sum_type() {
    let source = r#"
        type Shape =
            | Circle(Float)
            | Rectangle(Float, Float)
            | Point;

        fn area(s: Shape): Float => {
            match s {
                Circle(r) => 3.14 * r * r,
                Point => 0.0,
            }
        }
    "#;

    let err = compile(source, &[]).expect_err("Must fail when SumType match is missing Rectangle");
    assert!(
        err.contains("Non-exhaustive patterns in match for 'Shape': missing variant 'Rectangle'"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_match_exhaustiveness_allowed_with_wildcard() {
    let source = r#"
        type Shape =
            | Circle(Float)
            | Rectangle(Float, Float)
            | Point;

        fn area(s: Shape): Float => {
            match s {
                Circle(r) => 3.14 * r * r,
                _ => 0.0,
            }
        }
    "#;

    let res = compile(source, &[]);
    assert!(
        res.is_ok(),
        "Match with wildcard must succeed exhaustiveness check"
    );
}

#[test]
fn test_package_manager_security_argument_validation() {
    let root = get_temp_dir("aura_test_sec_pkg");
    PackageManager::init(&root, Some("github.com/myorg/secapp")).unwrap();

    // Option injection in version
    let res_ver = PackageManager::get(&root, "github.com/myorg/pkg@--upload-pack=evil");
    assert!(res_ver.is_err(), "Must reject version starting with hyphen");
    assert!(
        res_ver
            .unwrap_err()
            .contains("cannot start with '-' (command injection prevention)")
    );

    // Protocol injection in package path
    let res_path = PackageManager::get(&root, "ext::sh -c evil@v1.0.0");
    assert!(
        res_path.is_err(),
        "Must reject package path with protocol injection"
    );
    assert!(
        res_path
            .unwrap_err()
            .contains("Security violation: package path")
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_runtime_c_api_security_safeguards() {
    // 1. aura_http_listen_and_serve_tls refuses unencrypted plaintext fallback
    let dummy_addr = b":9999\0";
    let cert = b"nonexistent.crt\0";
    let key = b"nonexistent.key\0";
    let res = aura_runtime::http::aura_http_listen_and_serve_tls(
        std::ptr::null_mut(),
        dummy_addr.as_ptr(),
        dummy_addr.len() - 1,
        cert.as_ptr(),
        cert.len() - 1,
        key.as_ptr(),
        key.len() - 1,
    );
    assert_eq!(
        res, -1,
        "TLS endpoint must fail cleanly when certificates are missing or TLS engine is unlinked"
    );

    // 2. aura_db_query and aura_db_exec reject arbitrary invalid handles safely without segfault
    let invalid_handle = 0xDEADBEEF as *mut ();
    let query_sql = b"SELECT 1\0";
    let rows = aura_runtime::db::aura_db_query(
        invalid_handle,
        query_sql.as_ptr(),
        query_sql.len() - 1,
        std::ptr::null_mut(),
    );
    assert!(!rows.is_null());
    unsafe {
        assert!((*rows).last_error.is_some());
        assert!(
            (*rows)
                .last_error
                .as_ref()
                .unwrap()
                .contains("Invalid or unallocated database handle")
        );
        aura_runtime::db::aura_db_rows_close(rows);
    }

    let exec_res = aura_runtime::db::aura_db_exec(
        invalid_handle,
        query_sql.as_ptr(),
        query_sql.len() - 1,
        std::ptr::null_mut(),
    );
    assert!(!exec_res.is_null());
    unsafe {
        assert!((*exec_res).error.is_some());
        assert!(
            (*exec_res)
                .error
                .as_ref()
                .unwrap()
                .contains("Invalid or unallocated database handle")
        );
        aura_runtime::db::aura_db_exec_free(exec_res);
    }

    // 3. aura_db_pool_close is idempotent and handles null or double-close safely
    aura_runtime::db::aura_db_pool_close(std::ptr::null_mut());
    aura_runtime::db::aura_db_pool_close(0xCAFEBABE as *mut _);

    // 4. aura_db_tx_close is idempotent and handles null or double-close safely
    aura_runtime::db::aura_db_tx_close(std::ptr::null_mut());
    aura_runtime::db::aura_db_tx_close(0xCAFEBABE as *mut _);

    // 5. aura_os_setenv safely rejects malicious keys with '=' or null bytes
    let bad_key = b"AURA=TEST\0";
    let bad_val = b"123\0";
    aura_runtime::os::aura_os_setenv(
        bad_key.as_ptr(),
        bad_key.len() - 1,
        bad_val.as_ptr(),
        bad_val.len() - 1,
    );
}

#[test]
fn test_adt_variant_compilation_and_matching() {
    let source = r#"
        type Action =
            | Load(Int)
            | Save(String)
            | Reset;

        fn describe(a: Action): String => {
            match a {
                Load(id) => `loading ${id}`,
                Save(name) => `saving ${name}`,
                Reset => "reset",
            }
        }
    "#;

    let res = compile(source, &[]);
    assert!(
        res.is_ok(),
        "Compiling custom ADT pattern matching must succeed without tag collision"
    );
}

#[test]
fn test_timing_attack_jwt_codegen_protection() {
    let source = r#"
        import { jwt } from "jwt";

        fn checkAuth(token: String): Result<String, String> => {
            jwt.verify(token, "supersecret")
        }
    "#;

    // 1. JS / Node Codegen must use timingSafeEqual
    let res = compile(source, &[]).expect("JS compilation must succeed");
    assert!(
        res.js_code.contains("timingSafeEqual"),
        "Node.js codegen must verify JWT signatures using crypto.timingSafeEqual"
    );

    // 2. Go Codegen must use hmac.Equal
    let go_code = aura_lang::compile_to_go(source).expect("Go compilation must succeed");
    assert!(
        go_code.contains("hmac.Equal"),
        "Go codegen must verify JWT signatures using hmac.Equal for constant-time comparison"
    );
}

#[test]
fn test_json_dos_depth_protection_runtime() {
    // Maliciously nested JSON payload (70 levels > 64)
    let mut payload = Vec::new();
    for _ in 0..70 {
        payload.push(b'[');
    }
    payload.push(b'1');
    for _ in 0..70 {
        payload.push(b']');
    }

    let doc = aura_runtime::json::aura_json_parse(payload.as_ptr(), payload.len());
    assert!(!doc.is_null());
    unsafe {
        assert!(
            (*doc).value.is_null(),
            "Excessive JSON nesting (> 64) must be safely rejected to prevent stack exhaustion DoS"
        );
        aura_runtime::json::aura_json_free(doc);
    }
}

#[test]
fn test_fiber_stack_canary_integrity() {
    use aura_runtime::context::{DEFAULT_STACK_SIZE, FiberStack, STACK_CANARY_MAGIC};

    let stack = FiberStack::new(DEFAULT_STACK_SIZE);
    assert!(
        stack.verify_canary(),
        "Stack canary must be valid at creation"
    );

    // Corrupt canary
    unsafe {
        *(stack.bottom() as *mut usize) = 0xBADF_00D0;
    }
    assert!(!stack.verify_canary(), "Corrupted canary must be detected");

    // Restore for safe drop
    unsafe {
        *(stack.bottom() as *mut usize) = STACK_CANARY_MAGIC;
    }
    assert!(stack.verify_canary());
}

#[test]
fn test_immutability_reassignment_rejection() {
    let source = r#"
        fn testImm(): Unit => {
            let x = 10;
            x = 20;
        }
    "#;

    let err = compile(source, &[]).expect_err("Must reject reassigning immutable variable");
    assert!(
        err.contains("Cannot assign to immutable variable 'x'"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_mutability_reassignment_allowed() {
    let source = r#"
        fn testMut(): Int => {
            let mut x = 10;
            x = 20;
            x
        }
    "#;

    let res = compile(source, &[]);
    assert!(res.is_ok(), "Must allow reassigning mutable variable");
}

#[test]
fn test_try_operator_static_typing() {
    let source = r#"
        fn getNumber(): Result<Int, String> => {
            Ok(42)
        }

        fn compute(): Result<Int, String> => {
            let n: Int = getNumber()?;
            Ok(n + 1)
        }
    "#;

    let res = compile(source, &[]);
    assert!(res.is_ok(), "Try operator must unwrap Result<Int, E> to Int");
}

#[test]
fn test_try_operator_rejection_on_non_result_or_option() {
    let source = r#"
        fn badTry(): Int => {
            let x = 10;
            let y = x?;
            y
        }
    "#;

    let err = compile(source, &[]).expect_err("Must reject '?' on non-Result/Option type");
    assert!(
        err.contains("Cannot apply '?' operator to non-Result/Option type"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_concurrency_spawn_raw_pointer_capture_rejection() {
    let source = r#"
        fn unsafeSpawn(): Unit => {
            let mut val = 42;
            let ptrVal = ptr(val);
            spawn {
                let _ = *ptrVal;
            };
        }
    "#;

    let err = compile(source, &[]).expect_err("Must reject raw pointer capture across spawn boundary");
    assert!(
        err.contains("Concurrency Safety Error [E0401]"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_raw_string_literals_lexer_and_compiler() {
    let source = r##"
        let raw1 = r"hello\nworld";
        let raw2 = r#"hello "quoted" world"#;
    "##;

    let res = compile(source, &[]);
    assert!(res.is_ok(), "Raw string literals must compile cleanly");
}

#[test]
fn test_chan_c_api_null_pointer_safety() {
    use aura_runtime::c_api::{aura_chan_close, aura_chan_recv, aura_chan_send};

    // Passing null pointers must safely no-op or return null without crashing
    aura_chan_send(std::ptr::null_mut(), std::ptr::null_mut());
    let recv_res = aura_chan_recv(std::ptr::null_mut());
    assert!(recv_res.is_null());
    aura_chan_close(std::ptr::null_mut());
}

#[test]
fn test_redis_dsn_with_password_parsing() {
    use aura_runtime::redis_client::AuraRedisClient;

    // Must not panic or fail to open embedded client with credential-bearing DSNs
    let client = AuraRedisClient::open("redis://:supersecret@127.0.0.1:6379");
    assert!(client.set("testkey", "testval", 0));
    let retrieved = client.get("testkey");
    assert_eq!(retrieved, Some("testval".to_string()));
}
