//! Native synchronization primitives (Mutex, WaitGroup) for Aura Runtime.

use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct AuraMutex {
    locked: AtomicBool,
}

impl AuraMutex {
    pub fn new() -> Self {
        AuraMutex {
            locked: AtomicBool::new(false),
        }
    }

    /// Acquires the lock cooperatively.
    /// Spins briefly for low contention, then yields execution to the fiber scheduler.
    pub fn lock(&self) {
        let mut spins = 0;
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spins += 1;
            if spins < 32 {
                std::hint::spin_loop();
            } else {
                crate::scheduler::yield_now();
                spins = 0;
            }
        }
    }

    /// Releases the lock.
    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mutex_new() -> *mut AuraMutex {
    let m = Box::new(AuraMutex::new());
    Box::into_raw(m)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mutex_lock(m: *mut AuraMutex) {
    if !m.is_null() {
        unsafe {
            (*m).lock();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_mutex_unlock(m: *mut AuraMutex) {
    if !m.is_null() {
        unsafe {
            (*m).unlock();
        }
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
