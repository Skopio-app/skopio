#![cfg(target_os = "macos")]

use std::{ffi::c_void, ptr};

use core_foundation::{
    array::CFArrayRef,
    base::{CFGetTypeID, CFRelease, CFRetain, CFTypeRef, TCFType},
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

#[derive(Debug)]
pub struct AxElement(*const c_void);

impl AxElement {
    pub unsafe fn app(pid: i32) -> Option<Self> {
        let p = AXUIElementCreateApplication(pid);
        if p.is_null() {
            None
        } else {
            Some(Self(p))
        }
    }

    pub unsafe fn copy_attr(&self, name: &str) -> Option<CFTypeRef> {
        let mut out: *const c_void = ptr::null();
        let err =
            AXUIElementCopyAttributeValue(self.0, k(name).as_concrete_TypeRef() as _, &mut out);
        if err == ERR_SUCCESS && !out.is_null() {
            Some(out as CFTypeRef)
        } else {
            None
        }
    }

    pub unsafe fn children(&self) -> Vec<Self> {
        let mut out = vec![];
        if let Some(arr) = self.copy_attr("AXChildren") {
            let cf_arr = arr as CFArrayRef;
            let count = CFArrayGetCount(cf_arr);
            if count > 0 {
                out.reserve(count as usize);
                for i in 0..count {
                    let item = CFArrayGetValueAtIndex(cf_arr, i);
                    if !item.is_null() {
                        unsafe { CFRetain(item) };
                        out.push(Self(item));
                    }
                }
            }
            unsafe {
                CFRelease(arr as _);
            }
        }
        out
    }

    pub unsafe fn role(&self) -> Option<String> {
        self.copy_attr("AXRole").map(|t| {
            let s = CFString::wrap_under_create_rule(t as _);
            s.to_string()
        })
    }

    pub unsafe fn title(&self) -> Option<String> {
        self.copy_attr("AXTitle").map(|t| {
            let s = CFString::wrap_under_create_rule(t as _);
            s.to_string()
        })
    }

    pub unsafe fn url(&self) -> Option<String> {
        self.copy_attr("AXURL").map(|t| {
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

    pub unsafe fn focused_window(&self) -> Option<Self> {
        self.copy_attr("AXFocusedWindow").map(|p| Self(p as _))
    }

    pub unsafe fn find_descendant(&self, role: &str, max_depth: usize) -> Option<Self> {
        fn dfs(node: &AxElement, role: &str, depth: usize, max_depth: usize) -> Option<AxElement> {
            if depth > max_depth {
                return None;
            }
            if let Some(r) = unsafe { node.role() } {
                if r == role {
                    return Some(node.clone());
                }
            }
            for child in unsafe { node.children() } {
                if let Some(found) = dfs(&child, role, depth + 1, max_depth) {
                    return Some(found);
                }
            }
            None
        }
        dfs(self, role, 0, max_depth)
    }

    pub unsafe fn document(&self) -> Option<String> {
        self.copy_attr("AXDocument").map(|t| {
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

    // pub fn as_raw(&self) -> *const c_void {
    //     self.0
    // }
}

impl Clone for AxElement {
    fn clone(&self) -> Self {
        unsafe { CFRetain(self.0) };
        Self(self.0)
    }
}

impl Drop for AxElement {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.0);
        }
    }
}
