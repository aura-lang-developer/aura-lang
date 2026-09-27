//! C ABI exports for Aura Native Binaries (Cranelift and LLVM targets).

use crate::channel::AuraChannel;
use crate::gc::{collect_garbage, gc_alloc, gc_safepoint};
use crate::scheduler::{get_scheduler, init_scheduler, yield_now};
use std::io::{self, Write};
use std::slice;

#[unsafe(no_mangle)]
pub extern "C" fn aura_rt_init() {
    init_scheduler();
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_rt_start(main_fn: extern "C" fn()) {
    init_scheduler();
    let sched = get_scheduler();

    extern "C" fn main_trampoline(arg: *mut ()) {
        let f: extern "C" fn() = unsafe { std::mem::transmute(arg) };
        f();
    }

    sched.spawn(main_trampoline, main_fn as *mut ());
    let num_threads = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
        .max(4);
    sched.run_workers(num_threads);
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_fiber_spawn(entry: extern "C" fn(*mut ()), arg: *mut ()) -> u64 {
    get_scheduler().spawn(entry, arg)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_fiber_yield() {
    yield_now();
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_chan_new(capacity: usize) -> *mut AuraChannel {
    let chan = Box::new(AuraChannel::new(capacity));
    Box::into_raw(chan)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_chan_send(chan: *mut AuraChannel, val: *mut ()) {
    assert!(!chan.is_null());
    unsafe {
        (*chan).send(val);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_chan_recv(chan: *mut AuraChannel) -> *mut () {
    assert!(!chan.is_null());
    unsafe { (*chan).recv() }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_chan_close(chan: *mut AuraChannel) {
    assert!(!chan.is_null());
    unsafe {
        (*chan).close();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_gc_alloc(size: usize, type_id: u32) -> *mut u8 {
    gc_alloc(size, type_id)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_gc_safepoint() {
    gc_safepoint();
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_gc_collect() {
    collect_garbage();
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_print_str(ptr: *const u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        let bytes = unsafe { slice::from_raw_parts(ptr, len) };
        let _ = io::stdout().write_all(bytes);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_println_str(ptr: *const u8, len: usize) {
    aura_print_str(ptr, len);
    println!();
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_println_i64(val: i64) {
    println!("{}", val);
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_println_f64(val: f64) {
    println!("{}", val);
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_println_bool(val: bool) {
    println!("{}", val);
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_println_any(val: i64) {
    if val == 0 {
        println!("()");
        return;
    }
    let aval = crate::record::infer_val_from_i64(val);
    match aval {
        crate::record::AuraVal::String(s) => {
            if !s.is_null() {
                println!("{}", unsafe { (*s).as_str() });
            } else {
                println!();
            }
        }
        crate::record::AuraVal::Record(_) | crate::record::AuraVal::List(_) => {
            let jval = crate::record::val_to_json_value(&aval);
            println!(
                "{}",
                serde_json::to_string(&jval).unwrap_or_else(|_| "{}".to_string())
            );
        }
        crate::record::AuraVal::Int(i) => println!("{}", i),
        crate::record::AuraVal::Float(f) => println!("{}", f),
        crate::record::AuraVal::Bool(b) => println!("{}", b),
        crate::record::AuraVal::Null => println!("null"),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_val_eq(v1: i64, v2: i64) -> i64 {
    if v1 == v2 {
        return 1;
    }
    if v1 == 0 || v2 == 0 {
        return 0;
    }
    if crate::gc::lookup_valid_ptr(v1 as usize) == Some(crate::gc::TYPE_STRING)
        && crate::gc::lookup_valid_ptr(v2 as usize) == Some(crate::gc::TYPE_STRING)
    {
        let s1 = v1 as *mut crate::string::AuraString;
        let s2 = v2 as *mut crate::string::AuraString;
        if crate::string::aura_string_eq(s1, s2) {
            return 1;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_val_to_str(val: i64) -> *mut crate::string::AuraString {
    if val == 0 {
        return crate::string::aura_string_from_rust_str("");
    }
    let aval = crate::record::infer_val_from_i64(val);
    match aval {
        crate::record::AuraVal::String(s) => s,
        crate::record::AuraVal::Int(i) => crate::string::aura_string_from_rust_str(&i.to_string()),
        crate::record::AuraVal::Float(f) => {
            crate::string::aura_string_from_rust_str(&f.to_string())
        }
        crate::record::AuraVal::Bool(b) => {
            crate::string::aura_string_from_rust_str(if b { "true" } else { "false" })
        }
        crate::record::AuraVal::Record(_) | crate::record::AuraVal::List(_) => {
            let jval = crate::record::val_to_json_value(&aval);
            let s = serde_json::to_string(&jval).unwrap_or_else(|_| "{}".to_string());
            crate::string::aura_string_from_rust_str(&s)
        }
        crate::record::AuraVal::Null => crate::string::aura_string_from_rust_str("null"),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_val_add(a: i64, b: i64) -> i64 {
    let t1 = crate::gc::lookup_valid_ptr(a as usize);
    let t2 = crate::gc::lookup_valid_ptr(b as usize);
    if t1 == Some(crate::gc::TYPE_STRING) || t2 == Some(crate::gc::TYPE_STRING) {
        let s1 = aura_val_to_str(a);
        let s2 = aura_val_to_str(b);
        return crate::string::aura_string_concat(s1, s2) as i64;
    }
    a.wrapping_add(b)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_val_sub(a: i64, b: i64) -> i64 {
    a.wrapping_sub(b)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_val_mul(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_val_div(a: i64, b: i64) -> i64 {
    if b == 0 { 0 } else { a.wrapping_div(b) }
}
