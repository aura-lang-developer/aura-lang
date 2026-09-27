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

use std::collections::HashMap;
use std::sync::LazyLock;

const SHARD_COUNT: usize = 64;

#[repr(align(64))]
struct PointerShard {
    map: Mutex<HashMap<usize, u32>>,
}

static POINTER_SHARDS: LazyLock<Vec<PointerShard>> = LazyLock::new(|| {
    let mut shards = Vec::with_capacity(SHARD_COUNT);
    for _ in 0..SHARD_COUNT {
        shards.push(PointerShard {
            map: Mutex::new(HashMap::with_capacity(256)),
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
        if let Ok(mut map) = shard.map.lock() {
            map.insert(ptr, type_id);
        }
    }
}

pub fn unregister_valid_ptr(ptr: usize) {
    if ptr >= 0x1000 {
        let shard = get_shard(ptr);
        if let Ok(mut map) = shard.map.lock() {
            map.remove(&ptr);
        }
    }
}

pub fn lookup_valid_ptr(ptr: usize) -> Option<u32> {
    if ptr < 0x1000 {
        return None;
    }
    let shard = get_shard(ptr);
    if let Ok(map) = shard.map.lock() {
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

        let mut allocs = ALLOCATIONS.lock().unwrap();
        allocs.push(SendPtr(header));

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
    let mut allocs = ALLOCATIONS.lock().unwrap();
    GC_TRIGGERED.store(false, Ordering::Relaxed);

    // 1. Reset mark bits
    for hdr in allocs.iter() {
        unsafe {
            (*hdr.0).marked = false;
        }
    }

    // 2. Scan Global Roots
    {
        let roots = GLOBAL_ROOTS.lock().unwrap();
        for root_slot in roots.iter() {
            if !root_slot.0.is_null() {
                let ptr = unsafe { *root_slot.0 };
                mark_ptr(ptr, &allocs);
            }
        }
    }

    // 3. Scan All Active Fiber Stacks (Conservative stack scanning across ALL workers)
    crate::scheduler::for_each_active_fiber(|sp, top| {
        scan_stack_range(sp, top, &allocs);
    });

    // 4. Sweep Phase: reclaim unmarked
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

fn scan_stack_range(bottom: *const u8, top: *const u8, allocs: &[SendPtr<GcHeader>]) {
    let mut cur = bottom as usize;
    let limit = top as usize;

    // Align to 8 bytes
    cur = (cur + 7) & !7;

    while cur + 8 <= limit {
        let val = unsafe { *(cur as *const usize) };
        if val != 0 {
            mark_ptr(val as *mut u8, allocs);
        }
        cur += 8;
    }
}

fn mark_ptr(target: *mut u8, allocs: &[SendPtr<GcHeader>]) {
    if target.is_null() {
        return;
    }

    for hdr in allocs {
        let h = hdr.0;
        let obj_start = unsafe { (h as *mut u8).add(size_of::<GcHeader>()) };
        let obj_end = unsafe { obj_start.add((*h).size) };

        if target >= obj_start && target < obj_end {
            unsafe {
                if !(*h).marked {
                    (*h).marked = true;
                    // Scan internal fields of marked object for child pointers
                    scan_stack_range(obj_start, obj_end, allocs);
                }
            }
            break;
        }
    }
}

pub fn get_collections_count() -> usize {
    GC_COLLECTION_COUNT.load(Ordering::Relaxed)
}
