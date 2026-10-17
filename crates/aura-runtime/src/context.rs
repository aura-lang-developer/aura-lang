//! Low-level stack allocation and context switching for Aura green threads (Fibers).

#[cfg(not(unix))]
use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::ptr;

#[cfg(target_arch = "aarch64")]
pub const DEFAULT_STACK_SIZE: usize = 16 * 1024; // 16 KB stack per fiber (1 page on macOS arm64)
#[cfg(not(target_arch = "aarch64"))]
pub const DEFAULT_STACK_SIZE: usize = 8 * 1024; // 8 KB stack per fiber (2 pages on x86_64)

#[cfg(unix)]
static STACK_POOL: std::sync::Mutex<Vec<FiberStack>> = std::sync::Mutex::new(Vec::new());
#[cfg(unix)]
const MAX_STACK_POOL: usize = 8192;

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".globl _aura_swap_context",
    ".globl aura_swap_context",
    ".p2align 2",
    "_aura_swap_context:",
    "aura_swap_context:",
    "    sub sp, sp, #160",
    "    stp x19, x20, [sp, #0]",
    "    stp x21, x22, [sp, #16]",
    "    stp x23, x24, [sp, #32]",
    "    stp x25, x26, [sp, #48]",
    "    stp x27, x28, [sp, #64]",
    "    stp x29, x30, [sp, #80]",
    "    stp d8,  d9,  [sp, #96]",
    "    stp d10, d11, [sp, #112]",
    "    stp d12, d13, [sp, #128]",
    "    stp d14, d15, [sp, #144]",
    "    mov x2, sp",
    "    str x2, [x0]",
    "    mov sp, x1",
    "    ldp d14, d15, [sp, #144]",
    "    ldp d12, d13, [sp, #128]",
    "    ldp d10, d11, [sp, #112]",
    "    ldp d8,  d9,  [sp, #96]",
    "    ldp x29, x30, [sp, #80]",
    "    ldp x27, x28, [sp, #64]",
    "    ldp x25, x26, [sp, #48]",
    "    ldp x23, x24, [sp, #32]",
    "    ldp x21, x22, [sp, #16]",
    "    ldp x19, x20, [sp, #0]",
    "    add sp, sp, #160",
    "    ret"
);

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".globl _aura_swap_context",
    ".globl aura_swap_context",
    "_aura_swap_context:",
    "aura_swap_context:",
    "    pushq %rbx",
    "    pushq %rbp",
    "    pushq %r12",
    "    pushq %r13",
    "    pushq %r14",
    "    pushq %r15",
    "    movq %rsp, (%rdi)",
    "    movq %rsi, %rsp",
    "    popq %r15",
    "    popq %r14",
    "    popq %r13",
    "    popq %r12",
    "    popq %rbp",
    "    popq %rbx",
    "    ret",
    options(att_syntax)
);

unsafe extern "C" {
    pub fn aura_swap_context(old_sp: *mut *mut u8, new_sp: *const u8);
}

#[cfg(unix)]
pub struct FiberStack {
    mmap_ptr: *mut u8,
    total_mmap_size: usize,
    base: *mut u8,
    size: usize,
}

#[cfg(not(unix))]
pub struct FiberStack {
    base: *mut u8,
    size: usize,
    layout: Layout,
}

pub const STACK_CANARY_MAGIC: usize = 0xDEAD_BEEF_CAFE_BABE;

impl FiberStack {
    #[cfg(unix)]
    pub fn new(size: usize) -> Self {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }.max(4096);
        let guard_size = page_size;
        let aligned_user_size = size.div_ceil(page_size) * page_size;
        let total_mmap_size = aligned_user_size + guard_size;

        if size == DEFAULT_STACK_SIZE {
            if let Ok(mut pool) = STACK_POOL.lock() {
                if let Some(recycled) = pool.pop() {
                    if recycled.verify_canary() {
                        return recycled;
                    }
                    eprintln!(
                        "[Aura Runtime Security Alert] Corrupted stack canary detected in pool, discarding"
                    );
                }
            }
        }

        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                total_mmap_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED || ptr.is_null() {
            panic!("Failed to allocate fiber stack of size {}", size);
        }

        // Apply hardware guard page at the bottom (stack grows downwards towards base)
        let protect_res = unsafe { libc::mprotect(ptr, guard_size, libc::PROT_NONE) };
        if protect_res != 0 {
            unsafe {
                libc::munmap(ptr, total_mmap_size);
            }
            panic!("Failed to protect fiber stack guard page");
        }

        let base = unsafe { (ptr as *mut u8).add(guard_size) };
        // Plant stack canary at the bottom of the stack to detect frame buffer overflows
        unsafe {
            *(base as *mut usize) = STACK_CANARY_MAGIC;
        }

        FiberStack {
            mmap_ptr: ptr as *mut u8,
            total_mmap_size,
            base,
            size: aligned_user_size,
        }
    }

    #[cfg(not(unix))]
    pub fn new(size: usize) -> Self {
        let size = (size + 15) & !15; // 16-byte align
        let layout = Layout::from_size_align(size, 16).expect("Valid stack layout");
        let base = unsafe { alloc_zeroed(layout) };
        if base.is_null() {
            panic!("Failed to allocate fiber stack of size {}", size);
        }
        FiberStack { base, size, layout }
    }

    pub fn top(&self) -> *mut u8 {
        unsafe { self.base.add(self.size) }
    }

    pub fn bottom(&self) -> *mut u8 {
        self.base
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn verify_canary(&self) -> bool {
        if self.base.is_null() {
            return true;
        }
        unsafe { *(self.base as *const usize) == STACK_CANARY_MAGIC }
    }
}

impl Drop for FiberStack {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            if !self.mmap_ptr.is_null() {
                let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }.max(4096);
                let default_aligned =
                    ((DEFAULT_STACK_SIZE + page_size - 1) / page_size) * page_size;
                if self.size == default_aligned && self.verify_canary() {
                    if let Ok(mut pool) = STACK_POOL.lock() {
                        if pool.len() < MAX_STACK_POOL {
                            let mmap_ptr = self.mmap_ptr;
                            let total_mmap_size = self.total_mmap_size;
                            let base = self.base;
                            let size = self.size;
                            self.mmap_ptr = ptr::null_mut();
                            self.base = ptr::null_mut();
                            pool.push(FiberStack {
                                mmap_ptr,
                                total_mmap_size,
                                base,
                                size,
                            });
                            return;
                        }
                    }
                }
                unsafe {
                    libc::munmap(self.mmap_ptr as *mut libc::c_void, self.total_mmap_size);
                }
                self.mmap_ptr = ptr::null_mut();
                self.base = ptr::null_mut();
            }
        }
        #[cfg(not(unix))]
        {
            if !self.base.is_null() {
                unsafe {
                    dealloc(self.base, self.layout);
                }
                self.base = ptr::null_mut();
            }
        }
    }
}

unsafe impl Send for FiberStack {}
unsafe impl Sync for FiberStack {}

/// Prepares a new fiber stack so that the first context switch into it
/// jumps directly to `trampoline`.
///
/// # Safety
///
/// Caller must ensure that `stack` contains valid memory allocated for a fiber execution context.
pub unsafe fn init_stack(stack: &FiberStack, trampoline: extern "C" fn() -> !) -> *mut u8 {
    let top = stack.top();

    #[cfg(target_arch = "aarch64")]
    unsafe {
        // ARM64 context size = 160 bytes
        let sp = top.sub(160);
        // At sp + 80 we store x29 (FP) and x30 (LR)
        let lr_ptr = sp.add(88) as *mut usize;
        let fp_ptr = sp.add(80) as *mut usize;
        ptr::write(lr_ptr, trampoline as usize);
        ptr::write(fp_ptr, 0);
        sp
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        // x86_64: 1 ret address + 6 callee-saved registers = 7 * 8 = 56 bytes
        let sp = top.sub(64);
        let ret_addr_ptr = sp.add(48) as *mut usize;
        ptr::write(ret_addr_ptr, trampoline as usize);
        sp
    }
}
