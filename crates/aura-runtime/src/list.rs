//! Native dynamic List<T> implementation for Aura Runtime.
//! Supports both standard 64-bit element lists (Int, Float, Pointer) and
//! compact 1-byte element lists (Bool, Byte) for maximum cache locality and minimal memory overhead.

use std::alloc::{Layout, alloc, dealloc, realloc};
use std::ptr;

pub const AURA_LIST_MAGIC: u64 = 0xA04C_4953_5400_0003;

#[repr(C)]
#[derive(Debug)]
pub struct AuraList {
    pub magic: u64,
    pub items: *mut u8,
    pub len: usize,
    pub cap: usize,
    pub elem_size: usize, // 1 for compact Byte/Bool, 8 for standard 64-bit values
}

impl AuraList {
    pub fn as_slice(&self) -> &[i64] {
        if self.items.is_null() || self.len == 0 || self.elem_size != 8 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.items as *const i64, self.len) }
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        if self.items.is_null() || self.len == 0 {
            &[]
        } else if self.elem_size == 1 {
            unsafe { std::slice::from_raw_parts(self.items, self.len) }
        } else {
            unsafe { std::slice::from_raw_parts(self.items, self.len * 8) }
        }
    }
}

#[inline]
fn compute_layout(cap: usize, elem_size: usize) -> Layout {
    let size = cap.max(1) * elem_size;
    let align = if elem_size == 1 { 1 } else { 8 };
    Layout::from_size_align(size, align).unwrap()
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_new_with_elem_size(
    initial_cap: usize,
    elem_size: usize,
) -> *mut AuraList {
    let es = if elem_size == 1 { 1 } else { 8 };
    let cap = if initial_cap == 0 { 8 } else { initial_cap };
    let layout = compute_layout(cap, es);
    let items = unsafe { alloc(layout) as *mut u8 };

    let list = Box::new(AuraList {
        magic: AURA_LIST_MAGIC,
        items,
        len: 0,
        cap,
        elem_size: es,
    });
    let raw = Box::into_raw(list);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_LIST);
    raw
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_new(initial_cap: usize) -> *mut AuraList {
    aura_list_new_with_elem_size(initial_cap, 8)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_new_byte(initial_cap: usize) -> *mut AuraList {
    aura_list_new_with_elem_size(initial_cap, 1)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_repeat_with_elem_size(
    item: i64,
    count: usize,
    elem_size: usize,
) -> *mut AuraList {
    let es = if elem_size == 1 { 1 } else { 8 };
    let cap = if count == 0 { 8 } else { count };
    let layout = compute_layout(cap, es);
    let items = unsafe { alloc(layout) as *mut u8 };
    if count > 0 && !items.is_null() {
        if es == 1 {
            let b = item as u8;
            unsafe {
                std::ptr::write_bytes(items, b, count);
            }
        } else {
            if item == 0 {
                unsafe {
                    std::ptr::write_bytes(items, 0, count * 8);
                }
            } else {
                unsafe {
                    std::slice::from_raw_parts_mut(items as *mut i64, count).fill(item);
                }
            }
        }
    }
    let list = Box::new(AuraList {
        magic: AURA_LIST_MAGIC,
        items,
        len: count,
        cap,
        elem_size: es,
    });
    let raw = Box::into_raw(list);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_LIST);
    raw
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_repeat(item: i64, count: usize) -> *mut AuraList {
    aura_list_repeat_with_elem_size(item, count, 8)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_repeat_byte(item: u8, count: usize) -> *mut AuraList {
    aura_list_repeat_with_elem_size(item as i64, count, 1)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_with_capacity(initial_cap: usize) -> *mut AuraList {
    aura_list_new(initial_cap)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_elem_size(list: *mut AuraList) -> usize {
    if list.is_null() {
        8
    } else {
        unsafe { (*list).elem_size }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_push(list: *mut AuraList, item: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let l = &mut *list;
        let es = if l.elem_size == 1 { 1 } else { 8 };
        if l.len >= l.cap {
            let new_cap = if l.cap == 0 { 8 } else { l.cap * 2 };
            let old_layout = compute_layout(l.cap, es);
            let new_layout = compute_layout(new_cap, es);
            let new_items = realloc(l.items, old_layout, new_layout.size());
            if new_items.is_null() {
                std::alloc::handle_alloc_error(new_layout);
            }
            l.items = new_items;
            l.cap = new_cap;
        }

        if es == 1 {
            *l.items.add(l.len) = item as u8;
        } else {
            *(l.items as *mut i64).add(l.len) = item;
        }
        l.len += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_get(list: *mut AuraList, idx: usize) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let l = &*list;
        if idx < l.len {
            if l.elem_size == 1 {
                *l.items.add(idx) as i64
            } else {
                *(l.items as *mut i64).add(idx)
            }
        } else {
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_set(list: *mut AuraList, idx: usize, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let l = &mut *list;
        if idx < l.len {
            if l.elem_size == 1 {
                *l.items.add(idx) = val as u8;
            } else {
                *(l.items as *mut i64).add(idx) = val;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_len(list: *mut AuraList) -> usize {
    if list.is_null() {
        0
    } else {
        unsafe { (*list).len }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_cap(list: *mut AuraList) -> usize {
    if list.is_null() {
        0
    } else {
        unsafe { (*list).cap }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_slice(list: *mut AuraList, start: i64, end: i64) -> *mut AuraList {
    if list.is_null() {
        return aura_list_new(0);
    }
    unsafe {
        let l = &*list;
        let es = if l.elem_size == 1 { 1 } else { 8 };
        let s = if start < 0 {
            0
        } else {
            (start as usize).min(l.len)
        };
        let e = if end < 0 {
            l.len
        } else {
            (end as usize).min(l.len)
        };
        let e = if e < s { s } else { e };

        let slice_len = e - s;
        let res = aura_list_new_with_elem_size(slice_len, es);
        let dst = (*res).items;
        let src = l.items;
        if es == 1 {
            std::ptr::copy_nonoverlapping(src.add(s), dst, slice_len);
        } else {
            std::ptr::copy_nonoverlapping((src as *mut i64).add(s), dst as *mut i64, slice_len);
        }
        (*res).len = slice_len;
        res
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_filter(
    list: *mut AuraList,
    pred: extern "C" fn(i64) -> i64,
) -> *mut AuraList {
    if list.is_null() {
        return aura_list_new(0);
    }
    unsafe {
        let src = &*list;
        let len = src.len;
        let es = if src.elem_size == 1 { 1 } else { 8 };
        let res = aura_list_new_with_elem_size(len, es);
        if len == 0 || src.items.is_null() {
            return res;
        }
        let mut count = 0;
        for i in 0..len {
            let item = aura_list_get(list, i);
            if pred(item) != 0 {
                aura_list_set(res, count, item);
                count += 1;
            }
        }
        (*res).len = count;
        res
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_map(
    list: *mut AuraList,
    mapper: extern "C" fn(i64) -> i64,
) -> *mut AuraList {
    if list.is_null() {
        return aura_list_new(0);
    }
    unsafe {
        let src = &*list;
        let len = src.len;
        let es = if src.elem_size == 1 { 1 } else { 8 };
        let res = aura_list_new_with_elem_size(len, es);
        if len == 0 || src.items.is_null() {
            return res;
        }
        for i in 0..len {
            let item = aura_list_get(list, i);
            aura_list_set(res, i, mapper(item));
        }
        (*res).len = len;
        res
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_sum(list: *mut AuraList) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let l = &*list;
        if l.len == 0 || l.items.is_null() {
            return 0;
        }
        let mut sum: i64 = 0;
        for i in 0..l.len {
            sum += aura_list_get(list, i);
        }
        sum
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_to_json(list: *mut AuraList) -> *mut crate::string::AuraString {
    let json_val = crate::record::list_to_json_value(list);
    let s = serde_json::to_string(&json_val).unwrap_or_else(|_| "[]".to_string());
    crate::string::aura_string_from_rust_str(&s)
}

impl Drop for AuraList {
    fn drop(&mut self) {
        if !self.items.is_null() && self.cap > 0 {
            let es = if self.elem_size == 1 { 1 } else { 8 };
            let layout = compute_layout(self.cap, es);
            unsafe {
                dealloc(self.items, layout);
            }
            self.items = ptr::null_mut();
            self.len = 0;
            self.cap = 0;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_free(list: *mut AuraList) {
    if !list.is_null() {
        crate::gc::unregister_valid_ptr(list as usize);
        unsafe {
            drop(Box::from_raw(list));
        }
    }
}
