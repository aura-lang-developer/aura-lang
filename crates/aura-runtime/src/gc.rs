//! Tracing Garbage Collector (GC) with TLAB allocation and Fiber Stack Scanning.

use std::alloc::{Layout, alloc, dealloc};
use std::mem::size_of;
use std::ptr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[repr(C)]
pub struct GcHeader {
    pub size: usize,
    pub marked: bool,
    pub type_id: u32,
}

pub const TYPE_UNKNOWN: u32 = 0;
pub const TYPE_STRING: u32 = 1;
pub const TYPE_RECORD: u32 = 2;
pub const TYPE_LIST: u32 = 3;
pub const TYPE_MUTEX: u32 = 4;
pub const TYPE_CHANNEL: u32 = 5;
pub const TYPE_VARIANT: u32 = 6;
pub const TYPE_DB_POOL: u32 = 7;
pub const TYPE_DB_ROWS: u32 = 8;
pub const TYPE_DB_TX: u32 = 9;
pub const TYPE_DB_EXEC_RESULT: u32 = 10;
pub const TYPE_REDIS_CLIENT: u32 = 11;
pub const TYPE_MONGO_CLIENT: u32 = 12;
pub const TYPE_MONGO_DATABASE: u32 = 13;
pub const TYPE_MONGO_COLLECTION: u32 = 14;

use rustc_hash::FxHashMap;
use std::sync::{LazyLock, RwLock};

const SHARD_COUNT: usize = 64;

#[repr(align(64))]
struct PointerShard {
    map: RwLock<FxHashMap<usize, u32>>,
}

static POINTER_SHARDS: LazyLock<Vec<PointerShard>> = LazyLock::new(|| {
    let mut shards = Vec::with_capacity(SHARD_COUNT);
    for _ in 0..SHARD_COUNT {
        shards.push(PointerShard {
            map: RwLock::new(FxHashMap::with_capacity_and_hasher(256, Default::default())),
        });
    }
    shards
});

#[inline(always)]
fn get_shard(ptr: usize) -> &'static PointerShard {
    let hash = (ptr >> 4) ^ (ptr >> 10);
    &POINTER_SHARDS[hash % SHARD_COUNT]
}

pub fn register_valid_ptr(ptr: usize, type_id: u32) {
    if ptr >= 0x1000 {
        let shard = get_shard(ptr);
        if let Ok(mut map) = shard.map.write() {
            map.insert(ptr, type_id);
        }
    }
}

pub fn unregister_valid_ptr(ptr: usize) {
    if ptr >= 0x1000 {
        let shard = get_shard(ptr);
        if let Ok(mut map) = shard.map.write() {
            map.remove(&ptr);
        }
    }
}

pub fn lookup_valid_ptr(ptr: usize) -> Option<u32> {
    if ptr < 0x1000 {
        return None;
    }
    let shard = get_shard(ptr);
    if let Ok(map) = shard.map.read() {
        map.get(&ptr).copied()
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SendPtr<T>(pub *mut T);
unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

static ALLOCATIONS: Mutex<Vec<SendPtr<GcHeader>>> = Mutex::new(Vec::new());
static GLOBAL_ROOTS: Mutex<Vec<SendPtr<*mut u8>>> = Mutex::new(Vec::new());
static GC_TRIGGERED: AtomicBool = AtomicBool::new(false);
static TOTAL_ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
static GC_COLLECTION_COUNT: AtomicUsize = AtomicUsize::new(0);

const GC_THRESHOLD_BYTES: usize = 8 * 1024 * 1024; // 8 MB default GC threshold
const TLAB_BATCH_SIZE: usize = 64;

thread_local! {
    static TLAB_BUFFER: std::cell::RefCell<Vec<SendPtr<GcHeader>>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Flushes the calling thread's local allocation buffer into the global allocation registry.
pub fn flush_tlab() {
    TLAB_BUFFER.with(|buf| {
        let mut local = buf.borrow_mut();
        if !local.is_empty() {
            let mut allocs = ALLOCATIONS.lock().unwrap();
            allocs.extend(local.drain(..));
        }
    });
}

pub fn get_gc_type(ptr: *const u8) -> Option<u32> {
    if ptr.is_null() || (ptr as usize) < 0x1000 {
        return None;
    }
    lookup_valid_ptr(ptr as usize)
}

pub fn gc_alloc(user_size: usize, type_id: u32) -> *mut u8 {
    let header_size = size_of::<GcHeader>();
    let total_size = header_size + user_size;
    let aligned_total = (total_size + 15) & !15;

    // Check if we should trigger GC
    let current_bytes = TOTAL_ALLOCATED_BYTES.fetch_add(aligned_total, Ordering::Relaxed);
    if current_bytes > GC_THRESHOLD_BYTES {
        GC_TRIGGERED.store(true, Ordering::Relaxed);
    }

    let layout = Layout::from_size_align(aligned_total, 16).unwrap();
    let raw = unsafe { alloc(layout) };

    if raw.is_null() {
        panic!("Out of memory allocating {} bytes", user_size);
    }

    let header = raw as *mut GcHeader;
    unsafe {
        ptr::write(
            header,
            GcHeader {
                size: user_size,
                marked: false,
                type_id,
            },
        );
        let user_ptr = raw.add(header_size);
        ptr::write_bytes(user_ptr, 0, user_size);

        // Amortized TLAB lock-free allocation: batch additions to reduce Mutex contention
        let should_flush = TLAB_BUFFER.with(|buf| {
            let mut local = buf.borrow_mut();
            local.push(SendPtr(header));
            local.len() >= TLAB_BATCH_SIZE
        });

        if should_flush {
            flush_tlab();
        }

        register_valid_ptr(user_ptr as usize, type_id);

        user_ptr
    }
}

pub fn register_global_root(root: *mut *mut u8) {
    let mut roots = GLOBAL_ROOTS.lock().unwrap();
    roots.push(SendPtr(root));
}

pub fn gc_safepoint() {
    if GC_TRIGGERED.load(Ordering::Relaxed) {
        collect_garbage();
    }
}

pub fn collect_garbage() {
    let _world_guard = crate::scheduler::gc_enter_safepoint();
    flush_tlab();
    let mut allocs = ALLOCATIONS.lock().unwrap();
    GC_TRIGGERED.store(false, Ordering::Relaxed);

    // 1. Reset mark bits and sort by memory address for O(log N) binary search marking
    allocs.sort_unstable_by_key(|hdr| hdr.0 as usize);
    for hdr in allocs.iter() {
        unsafe {
            (*hdr.0).marked = false;
        }
    }

    // 2. Scan Global Roots and Fiber Stacks into an iterative Worklist
    let mut worklist = Vec::with_capacity(1024);
    {
        let roots = GLOBAL_ROOTS.lock().unwrap();
        for root_slot in roots.iter() {
            if !root_slot.0.is_null() {
                let ptr = unsafe { *root_slot.0 };
                if !ptr.is_null() && (ptr as usize) >= 0x1000 {
                    worklist.push(ptr as usize);
                }
            }
        }
    }

    // 3. Scan All Active Fiber Stacks into the Worklist
    crate::scheduler::for_each_active_fiber(|sp, top| {
        scan_stack_range(sp, top, &mut worklist);
    });

    // 4. Iteratively process all reachable objects (zero-recursion, immune to Stack Overflow)
    process_mark_worklist(&mut worklist, &allocs);

    // 5. Sweep Phase: reclaim unmarked
    let mut i = 0;
    while i < allocs.len() {
        let hdr = allocs[i].0;
        if unsafe { !(*hdr).marked } {
            let total_size = size_of::<GcHeader>() + unsafe { (*hdr).size };
            let aligned = (total_size + 15) & !15;
            let layout = Layout::from_size_align(aligned, 16).unwrap();
            let obj_start = unsafe { (hdr as *mut u8).add(size_of::<GcHeader>()) };
            unregister_valid_ptr(obj_start as usize);

            unsafe {
                dealloc(hdr as *mut u8, layout);
            }

            allocs.swap_remove(i);
        } else {
            i += 1;
        }
    }

    GC_COLLECTION_COUNT.fetch_add(1, Ordering::Relaxed);
}

fn scan_stack_range(bottom: *const u8, top: *const u8, worklist: &mut Vec<usize>) {
    let mut cur = bottom as usize;
    let limit = top as usize;

    // Align to 8 bytes
    cur = (cur + 7) & !7;

    while cur + 8 <= limit {
        let val = unsafe { *(cur as *const usize) };
        if val >= 0x1000 {
            worklist.push(val);
        }
        cur += 8;
    }
}

fn process_mark_worklist(worklist: &mut Vec<usize>, allocs: &[SendPtr<GcHeader>]) {
    while let Some(target_addr) = worklist.pop() {
        if target_addr < 0x1000 || allocs.is_empty() {
            continue;
        }

        // O(log N) binary search over sorted allocations
        let idx = match allocs.binary_search_by(|hdr| {
            let h = hdr.0;
            let obj_start = unsafe { (h as *mut u8).add(size_of::<GcHeader>()) } as usize;
            let obj_end = obj_start + unsafe { (*h).size };
            if target_addr < obj_start {
                std::cmp::Ordering::Greater
            } else if target_addr >= obj_end {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        }) {
            Ok(i) => i,
            Err(_) => continue,
        };

        let h = allocs[idx].0;
        unsafe {
            if !(*h).marked {
                (*h).marked = true;
                let obj_start = (h as *mut u8).add(size_of::<GcHeader>()) as usize;
                let obj_end = obj_start + (*h).size;

                let mut cur = (obj_start + 7) & !7;
                while cur + 8 <= obj_end {
                    let val = *(cur as *const usize);
                    if val >= 0x1000 {
                        worklist.push(val);
                    }
                    cur += 8;
                }
            }
        }
    }
}

#[allow(dead_code)]
#[inline]
fn mark_ptr(target: *mut u8, allocs: &[SendPtr<GcHeader>]) {
    if !target.is_null() && (target as usize) >= 0x1000 {
        let mut worklist = vec![target as usize];
        process_mark_worklist(&mut worklist, allocs);
    }
}

pub fn get_collections_count() -> usize {
    GC_COLLECTION_COUNT.load(Ordering::Relaxed)
}
