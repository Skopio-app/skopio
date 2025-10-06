#![cfg(target_os = "macos")]

use std::{ffi::c_void, ptr};

use core_foundation::{
    array::CFArrayRef,
    base::{CFGetTypeID, CFTypeRef, TCFType},
    string::CFString,
    url::CFURL,
};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *const c_void;
    fn AXUIElementCopyAttributeValue(
        element: *const c_void,
        attribute: *const c_void,
        value: *mut *const c_void,
    ) -> i32;
    fn CFArrayGetCount(theArray: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: isize) -> *const c_void;
}

const ERR_SUCCESS: i32 = 0;

#[inline]
fn k(s: &str) -> CFString {
    CFString::new(s)
}

pub unsafe fn ax_app(pid: i32) -> *const c_void {
    AXUIElementCreateApplication(pid)
}

pub unsafe fn ax_copy_attr(element: *const c_void, name: &str) -> Option<CFTypeRef> {
    let mut out: *const c_void = ptr::null();
    let err = AXUIElementCopyAttributeValue(element, k(name).as_concrete_TypeRef() as _, &mut out);
    if err == ERR_SUCCESS && !out.is_null() {
        Some(out as CFTypeRef)
    } else {
        None
    }
}

pub unsafe fn ax_children(element: *const c_void) -> Vec<*const c_void> {
    let mut out = vec![];
    if let Some(arr) = ax_copy_attr(element, "AXChildren") {
        let cf_arr = arr as CFArrayRef;
        let count = CFArrayGetCount(cf_arr);
        if count > 0 {
            out.reserve(count as usize);
            for i in 0..count {
                let item = CFArrayGetValueAtIndex(cf_arr, i);
                if !item.is_null() {
                    out.push(item as CFTypeRef);
                }
            }
        }
    }
    out
}

pub unsafe fn ax_role(element: *const c_void) -> Option<String> {
    ax_copy_attr(element, "AXRole").map(|t| {
        let s = CFString::wrap_under_create_rule(t as _);
        s.to_string()
    })
}

pub unsafe fn ax_title(element: *const c_void) -> Option<String> {
    ax_copy_attr(element, "AXTitle").map(|t| {
        let s = CFString::wrap_under_create_rule(t as _);
        s.to_string()
    })
}

pub unsafe fn ax_url(element: *const c_void) -> Option<String> {
    ax_copy_attr(element, "AXURL").map(|t| {
        if CFURL::type_id() == CFGetTypeID(t) {
            let u = CFURL::wrap_under_create_rule(t as _);
            let abs = u.absolute();
            abs.get_string().to_string()
        } else {
            let s = CFString::wrap_under_create_rule(t as _);
            s.to_string()
        }
    })
}

pub unsafe fn ax_focused_window(app_el: *const c_void) -> Option<*const c_void> {
    ax_copy_attr(app_el, "AXFocusedWindow")
}

pub unsafe fn ax_find_descendant(
    root: *const c_void,
    role: &str,
    max_depth: usize,
) -> Option<*const c_void> {
    fn dfs(
        node: *const c_void,
        role: &str,
        depth: usize,
        max_depth: usize,
    ) -> Option<*const c_void> {
        if depth > max_depth {
            return None;
        }
        if let Some(r) = unsafe { ax_role(node) } {
            if r == role {
                return Some(node);
            }
        }
        for child in unsafe { ax_children(node) } {
            if let Some(found) = dfs(child, role, depth + 1, max_depth) {
                return Some(found);
            }
        }
        None
    }
    dfs(root, role, 0, max_depth)
}

pub unsafe fn ax_document(window: *const c_void) -> Option<String> {
    ax_copy_attr(window, "AXDocument").map(|t| {
        if CFURL::type_id() == CFGetTypeID(t) {
            let u = CFURL::wrap_under_create_rule(t as _);
            let abs = u.absolute();
            abs.get_string().to_string()
        } else {
            let s = CFString::wrap_under_create_rule(t as _);
            s.to_string()
        }
    })
}
