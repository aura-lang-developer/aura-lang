use aura_lang::linter::lint_source;

#[test]
fn test_linter_sec003_insecure_random_detection() {
    let code = r#"
export fn generateSession(): String => {
    let sessionToken = Math.random();
    `${sessionToken}`
}
"#;
    let diagnostics = lint_source(code).expect("Should lint source cleanly");
    assert!(
        diagnostics.iter().any(|d| d.code == "SEC003"),
        "Expected SEC003 error for Math.random() assigned to sessionToken"
    );
}

#[test]
fn test_crypto_csprng_typecheck_and_compilation() {
    let code = r#"
export fn main(): Unit => {
    let token = crypto.randomBytes(32);
    let uuid = crypto.randomUUID();
    let matchCheck = crypto.timingSafeEqual(token, token);
    println(`Token: ${token}, UUID: ${uuid}, Match: ${matchCheck}`);
}
"#;
    let res = aura_lang::compile(code, &[]);
    assert!(res.is_ok(), "Expected crypto CSPRNG and timingSafeEqual to compile cleanly: {:?}", res.err());
}
