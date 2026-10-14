//! Native dynamic List<T> implementation for Aura Runtime.

use std::alloc::{Layout, alloc, dealloc, realloc};
use std::ptr;

pub const AURA_LIST_MAGIC: u64 = 0xA04C_4953_5400_0003;

#[repr(C)]
#[derive(Debug)]
pub struct AuraList {
    pub magic: u64,
    pub items: *mut i64,
    pub len: usize,
    pub cap: usize,
}

impl AuraList {
    pub fn as_slice(&self) -> &[i64] {
        if self.items.is_null() || self.len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.items, self.len) }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_new(initial_cap: usize) -> *mut AuraList {
    let cap = if initial_cap == 0 { 8 } else { initial_cap };
    let layout = Layout::array::<i64>(cap).unwrap();
    let items = unsafe { alloc(layout) as *mut i64 };

    let list = Box::new(AuraList {
        magic: AURA_LIST_MAGIC,
        items,
        len: 0,
        cap,
    });
    let raw = Box::into_raw(list);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_LIST);
    raw
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_repeat(item: i64, count: usize) -> *mut AuraList {
    let cap = if count == 0 { 8 } else { count };
    let layout = Layout::array::<i64>(cap).unwrap();
    let items = unsafe { alloc(layout) as *mut i64 };
    if count > 0 && !items.is_null() {
        if item == 0 {
            unsafe {
                std::ptr::write_bytes(items as *mut u8, 0, count * std::mem::size_of::<i64>());
            }
        } else {
            unsafe {
                for i in 0..count {
                    *items.add(i) = item;
                }
            }
        }
    }
    let list = Box::new(AuraList {
        magic: AURA_LIST_MAGIC,
        items,
        len: count,
        cap,
    });
    let raw = Box::into_raw(list);
    crate::gc::register_valid_ptr(raw as usize, crate::gc::TYPE_LIST);
    raw
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_with_capacity(initial_cap: usize) -> *mut AuraList {
    aura_list_new(initial_cap)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_push(list: *mut AuraList, item: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let l = &mut *list;
        if l.len >= l.cap {
            let new_cap = if l.cap == 0 { 8 } else { l.cap * 2 };
            let old_layout = Layout::array::<i64>(l.cap).unwrap();
            let new_layout = Layout::array::<i64>(new_cap).unwrap();
            let new_items = realloc(l.items as *mut u8, old_layout, new_layout.size()) as *mut i64;
            if new_items.is_null() {
                std::alloc::handle_alloc_error(new_layout);
            }
            l.items = new_items;
            l.cap = new_cap;
        }

        *l.items.add(l.len) = item;
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
        if idx < l.len { *l.items.add(idx) } else { 0 }
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
            *l.items.add(idx) = val;
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
        let res = aura_list_new(slice_len);
        let dst = (*res).items;
        let src = l.items;
        for i in 0..slice_len {
            *dst.add(i) = *src.add(s + i);
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
        let res = aura_list_new(len);
        if len == 0 || src.items.is_null() {
            return res;
        }
        let src_items = src.items;
        let dst_items = (*res).items;
        let mut count = 0;
        for i in 0..len {
            let item = *src_items.add(i);
            if pred(item) != 0 {
                *dst_items.add(count) = item;
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
        let res = aura_list_new(len);
        if len == 0 || src.items.is_null() {
            return res;
        }
        let src_items = src.items;
        let dst_items = (*res).items;
        for i in 0..len {
            let item = *src_items.add(i);
            *dst_items.add(i) = mapper(item);
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
        let items = l.items;
        for i in 0..l.len {
            sum += *items.add(i);
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
            let layout = Layout::array::<i64>(self.cap).unwrap();
            unsafe {
                dealloc(self.items as *mut u8, layout);
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
