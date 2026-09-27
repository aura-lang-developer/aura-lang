//! Variant / ADT runtime representations for Option<T> and Result<T, E>.

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AuraVariant {
    pub tag: u32, // 0 = None, 1 = Some / Ok, 2 = Err
    pub val: i64,
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_variant_new(tag: u32, val: i64) -> *mut AuraVariant {
    let v = Box::new(AuraVariant { tag, val });
    Box::into_raw(v)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_variant_tag(v: *mut AuraVariant) -> u32 {
    if v.is_null() { 0 } else { unsafe { (*v).tag } }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_variant_val(v: *mut AuraVariant) -> i64 {
    if v.is_null() { 0 } else { unsafe { (*v).val } }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_option_some(val: i64) -> *mut AuraVariant {
    aura_variant_new(1, val)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_option_none() -> *mut AuraVariant {
    aura_variant_new(0, 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_result_ok(val: i64) -> *mut AuraVariant {
    aura_variant_new(1, val)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_result_err(err: i64) -> *mut AuraVariant {
    aura_variant_new(2, err)
}
