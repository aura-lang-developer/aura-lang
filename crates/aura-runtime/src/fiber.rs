//! Fiber (Green Thread) control block and state machine.

use crate::context::{DEFAULT_STACK_SIZE, FiberStack, init_stack};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIBER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberState {
    Ready,
    Running,
    Blocked,
    Dead,
}

pub struct Fiber {
    pub id: u64,
    pub stack: FiberStack,
    pub sp: *mut u8,
    pub state: FiberState,
    pub entry_fn: Option<extern "C" fn(*mut ())>,
    pub entry_arg: *mut (),
}

impl Fiber {
    pub fn new(
        entry_fn: extern "C" fn(*mut ()),
        arg: *mut (),
        trampoline: extern "C" fn() -> !,
    ) -> Self {
        let id = NEXT_FIBER_ID.fetch_add(1, Ordering::Relaxed);
        let stack = FiberStack::new(DEFAULT_STACK_SIZE);
        let sp = unsafe { init_stack(&stack, trampoline) };

        Fiber {
            id,
            stack,
            sp,
            state: FiberState::Ready,
            entry_fn: Some(entry_fn),
            entry_arg: arg,
        }
    }
}

unsafe impl Send for Fiber {}
unsafe impl Sync for Fiber {}
