//! M:N Green Thread (Fiber) Scheduler with cooperative yielding and parking.

use crate::context::aura_swap_context;
use crate::fiber::{Fiber, FiberState};
use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};
use std::thread;

thread_local! {
    static CURRENT_FIBER: Cell<*mut Fiber> = const { Cell::new(std::ptr::null_mut()) };
    static SCHEDULER_SP: Cell<*mut *mut u8> = const { Cell::new(std::ptr::null_mut()) };
}

pub struct Scheduler {
    ready_queue: Mutex<VecDeque<Box<Fiber>>>,
    all_fibers: Mutex<Vec<usize>>,
    cond: Condvar,
    shutdown: AtomicBool,
    sleeping_workers: AtomicUsize,
    total_workers: AtomicUsize,
    deadlock_detection: AtomicBool,
}

use std::sync::OnceLock;

static GLOBAL_SCHEDULER: OnceLock<Scheduler> = OnceLock::new();

impl Default for Scheduler {
    fn default() -> Self {
        Scheduler {
            ready_queue: Mutex::new(VecDeque::new()),
            all_fibers: Mutex::new(Vec::new()),
            cond: Condvar::new(),
            shutdown: AtomicBool::new(false),
            sleeping_workers: AtomicUsize::new(0),
            total_workers: AtomicUsize::new(1),
            deadlock_detection: AtomicBool::new(true),
        }
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&self, entry: extern "C" fn(*mut ()), arg: *mut ()) -> u64 {
        let fiber = Box::new(Fiber::new(entry, arg, fiber_entry_trampoline));
        let id = fiber.id;
        let raw = Box::into_raw(fiber);

        {
            let mut all = self.all_fibers.lock().unwrap();
            all.push(raw as usize);
        }

        {
            let mut q = self.ready_queue.lock().unwrap();
            q.push_back(unsafe { Box::from_raw(raw) });
            self.cond.notify_one();
        }

        id
    }

    pub fn active_fibers_count(&self) -> usize {
        let all = match self.all_fibers.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        all.len()
    }

    pub fn ready_fibers_count(&self) -> usize {
        let q = match self.ready_queue.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        q.len()
    }

    pub fn parked_fibers_count(&self) -> usize {
        let active = self.active_fibers_count();
        let ready = self.ready_fibers_count();
        active.saturating_sub(ready)
    }

    pub fn set_deadlock_detection(&self, enabled: bool) {
        self.deadlock_detection.store(enabled, Ordering::Relaxed);
    }

    pub fn is_deadlock_detection_enabled(&self) -> bool {
        self.deadlock_detection.load(Ordering::Relaxed)
    }

    pub fn run_workers(&'static self, num_workers: usize) {
        self.total_workers.store(num_workers, Ordering::SeqCst);
        self.sleeping_workers.store(0, Ordering::SeqCst);

        let mut handles = Vec::new();
        for _ in 1..num_workers {
            handles.push(thread::spawn(move || {
                worker_loop(self);
            }));
        }

        // Main thread acts as worker 0
        worker_loop(self);

        for h in handles {
            let _ = h.join();
        }
    }
}

pub fn init_scheduler() {
    let _ = GLOBAL_SCHEDULER.get_or_init(Scheduler::new);
}

pub fn get_scheduler() -> &'static Scheduler {
    GLOBAL_SCHEDULER.get_or_init(Scheduler::new)
}

fn worker_loop(sched: &'static Scheduler) {
    let mut worker_sp: *mut u8 = std::ptr::null_mut();

    loop {
        let mut fiber = {
            let mut q = sched.ready_queue.lock().unwrap();
            while q.is_empty() {
                if sched.shutdown.load(Ordering::Relaxed) {
                    return;
                }

                let sleeping = sched.sleeping_workers.fetch_add(1, Ordering::SeqCst) + 1;
                let tot = sched.total_workers.load(Ordering::SeqCst);

                if sched.deadlock_detection.load(Ordering::Relaxed) && sleeping >= tot {
                    let all_count = {
                        let all = match sched.all_fibers.lock() {
                            Ok(g) => g,
                            Err(p) => p.into_inner(),
                        };
                        all.len()
                    };

                    if all_count > 0 && q.is_empty() {
                        sched.sleeping_workers.fetch_sub(1, Ordering::SeqCst);
                        eprintln!("\n=======================================================");
                        eprintln!("fatal error: all fibers are asleep - deadlock detected!");
                        eprintln!("  Active Fibers: {}", all_count);
                        eprintln!("  Ready Fibers:  0");
                        eprintln!(
                            "  Workers:       {} (all blocked waiting on CSP channels)",
                            tot
                        );
                        eprintln!("=======================================================\n");
                        panic!(
                            "Aura Runtime Deadlock: all fibers are parked on CSP channels with no senders or receivers ready"
                        );
                    }
                }

                q = sched.cond.wait(q).unwrap();
                sched.sleeping_workers.fetch_sub(1, Ordering::SeqCst);
            }
            q.pop_front().unwrap()
        };

        if fiber.state == FiberState::Dead {
            continue;
        }

        fiber.state = FiberState::Running;
        let raw_fiber = Box::into_raw(fiber);

        CURRENT_FIBER.set(raw_fiber);
        SCHEDULER_SP.set(&mut worker_sp);

        unsafe {
            let fiber_sp = (*raw_fiber).sp;
            aura_swap_context(&mut worker_sp, fiber_sp);
        }

        // Resumed back in worker thread stack!
        let fiber = unsafe { Box::from_raw(raw_fiber) };
        CURRENT_FIBER.set(std::ptr::null_mut());

        match fiber.state {
            FiberState::Ready => {
                let mut q = sched.ready_queue.lock().unwrap();
                q.push_back(fiber);
                sched.cond.notify_one();
            }
            FiberState::Blocked => {
                // Fiber parked on channel or I/O, keep leaked until unparked
                let _ = Box::into_raw(fiber);
            }
            FiberState::Dead => {
                // Fiber completed, clean up
                let mut all = sched.all_fibers.lock().unwrap();
                all.retain(|&f| f != raw_fiber as usize);
                drop(fiber);
            }
            FiberState::Running => unreachable!(),
        }

        let q = sched.ready_queue.lock().unwrap();
        if q.is_empty() {
            let all = sched.all_fibers.lock().unwrap();
            if all.is_empty() {
                sched.shutdown.store(true, Ordering::Relaxed);
                sched.cond.notify_all();
                return;
            }
        }
    }
}

extern "C" fn fiber_entry_trampoline() -> ! {
    let raw = CURRENT_FIBER.get();
    if !raw.is_null() {
        let (entry_fn, arg) = unsafe { ((*raw).entry_fn.take(), (*raw).entry_arg) };
        if let Some(f) = entry_fn {
            f(arg);
        }
    }

    yield_exit();
}

pub fn yield_now() {
    let raw = CURRENT_FIBER.get();
    if raw.is_null() {
        return;
    }

    unsafe {
        (*raw).state = FiberState::Ready;
        let worker_sp_ptr = SCHEDULER_SP.get();
        if !worker_sp_ptr.is_null() {
            aura_swap_context(&mut (*raw).sp, *worker_sp_ptr);
        }
    }
}

pub fn park_current() {
    let raw = CURRENT_FIBER.get();
    if raw.is_null() {
        return;
    }

    unsafe {
        (*raw).state = FiberState::Blocked;
        let worker_sp_ptr = SCHEDULER_SP.get();
        if !worker_sp_ptr.is_null() {
            aura_swap_context(&mut (*raw).sp, *worker_sp_ptr);
        }
    }
}

pub fn unpark_fiber(raw_fiber: *mut Fiber) {
    if raw_fiber.is_null() {
        return;
    }

    let fiber = unsafe {
        (*raw_fiber).state = FiberState::Ready;
        Box::from_raw(raw_fiber)
    };

    let sched = get_scheduler();
    let mut q = sched.ready_queue.lock().unwrap();
    q.push_back(fiber);
    sched.cond.notify_one();
}

pub fn current_fiber_ptr() -> *mut Fiber {
    CURRENT_FIBER.get()
}

pub fn for_each_active_fiber<F: FnMut(*const u8, *const u8)>(mut f: F) {
    let sched = get_scheduler();
    let all = match sched.all_fibers.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    for &fiber_addr in all.iter() {
        if fiber_addr != 0 {
            let fiber = unsafe { &*(fiber_addr as *const Fiber) };
            f(fiber.sp, fiber.stack.top());
        }
    }
}

fn yield_exit() -> ! {
    let raw = CURRENT_FIBER.get();
    unsafe {
        if !raw.is_null() {
            (*raw).state = FiberState::Dead;
            let worker_sp_ptr = SCHEDULER_SP.get();
            if !worker_sp_ptr.is_null() {
                aura_swap_context(&mut (*raw).sp, *worker_sp_ptr);
            }
        }
    }
    unreachable!("Fiber exited but resumed execution");
}
