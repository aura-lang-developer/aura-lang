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
    let cap = if initial_cap == 0 { 4 } else { initial_cap };
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
pub extern "C" fn aura_list_push(list: *mut AuraList, item: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let l = &mut *list;
        if l.len >= l.cap {
            let new_cap = l.cap * 2;
            let old_layout = Layout::array::<i64>(l.cap).unwrap();
            let new_size = Layout::array::<i64>(new_cap).unwrap().size();
            let new_items = realloc(l.items as *mut u8, old_layout, new_size) as *mut i64;
            l.items = new_items;
            l.cap = new_cap;
        }

        ptr::write(l.items.add(l.len), item);
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
        for i in 0..slice_len {
            aura_list_push(res, *l.items.add(s + i));
        }
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
    let res = aura_list_new(unsafe { (*list).len });
    let len = aura_list_len(list);
    for i in 0..len {
        let item = aura_list_get(list, i);
        let keep = pred(item);
        if keep != 0 {
            aura_list_push(res, item);
        }
    }
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_map(
    list: *mut AuraList,
    mapper: extern "C" fn(i64) -> i64,
) -> *mut AuraList {
    if list.is_null() {
        return aura_list_new(0);
    }
    let len = aura_list_len(list);
    let res = aura_list_new(len);
    for i in 0..len {
        let item = aura_list_get(list, i);
        let transformed = mapper(item);
        aura_list_push(res, transformed);
    }
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_to_json(list: *mut AuraList) -> *mut crate::string::AuraString {
    let json_val = crate::record::list_to_json_value(list);
    let s = serde_json::to_string(&json_val).unwrap_or_else(|_| "[]".to_string());
    crate::string::aura_string_from_rust_str(&s)
}

#[unsafe(no_mangle)]
pub extern "C" fn aura_list_free(list: *mut AuraList) {
    if !list.is_null() {
        crate::gc::unregister_valid_ptr(list as usize);
        unsafe {
            let l = Box::from_raw(list);
            if !l.items.is_null() && l.cap > 0 {
                let layout = Layout::array::<i64>(l.cap).unwrap();
                dealloc(l.items as *mut u8, layout);
            }
        }
    }
}
