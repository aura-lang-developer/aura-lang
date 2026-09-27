//! Native synchronization primitives (Mutex, WaitGroup) for Aura Runtime.

use std::sync::{Mutex, MutexGuard};

pub struct AuraMutex {
    inner: Mutex<()>,
    guard: Option<MutexGuard<'static, ()>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mutex_new() -> *mut AuraMutex {
    let m = Box::new(AuraMutex {
        inner: Mutex::new(()),
        guard: None,
    });
    Box::into_raw(m)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mutex_lock(m: *mut AuraMutex) {
    if m.is_null() {
        return;
    }
    unsafe {
        let mutex_ref = &*m;
        // Acquire lock
        let guard = mutex_ref.inner.lock().unwrap();
        // Store guard in self
        let guard_static: MutexGuard<'static, ()> = std::mem::transmute(guard);
        let m_mut = &mut *m;
        m_mut.guard = Some(guard_static);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mutex_unlock(m: *mut AuraMutex) {
    if m.is_null() {
        return;
    }
    unsafe {
        let m_mut = &mut *m;
        m_mut.guard = None; // Drops guard, unlocking mutex
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mutex_free(m: *mut AuraMutex) {
    if !m.is_null() {
        unsafe {
            drop(Box::from_raw(m));
        }
    }
}
