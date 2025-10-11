#![cfg(target_os = "macos")]

use std::{ffi::c_void, ptr};

use core_foundation::{
    array::CFArrayRef,
    base::{CFGetTypeID, CFRelease, CFRetain, CFTypeRef, TCFType},
    number::{
        kCFNumberFloatType, CFBooleanGetTypeID, CFBooleanGetValue, CFBooleanRef, CFNumberGetTypeID,
        CFNumberGetValue, CFNumberRef,
    },
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
    /// Creates an accessibility application element for a given process id.
    ///
    /// # Safety
    /// - Caller must ensure `pid` refers to a running process on this system.
    /// - The returned element retains a CoreFoundation object and **must** be dropped.
    pub unsafe fn app(pid: i32) -> Option<Self> {
        let p = AXUIElementCreateApplication(pid);
        if p.is_null() {
            None
        } else {
            Some(Self(p))
        }
    }

    /// Copy a named AX attribute from this element.
    ///
    /// # Safety
    /// - `name` must be a valid AX attribute for this element type.
    /// - Returned `CFTypeRef` is retained; the caller is responsible for eventual release.
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

    /// Return the element's AX children as `AXElement`s.
    ///
    /// # Safety
    /// - Traversing the AX tree is inherently racy; nodes can disappear between calls.
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

    /// Read the `AXRole` as a Rust `String`
    ///
    /// # Safety
    /// - The returned string is created from a retained CF object whuch we wrap in
    ///   `CFString::wrap_under_create_rule` to manage release.
    pub unsafe fn role(&self) -> Option<String> {
        self.copy_attr("AXRole").map(|t| {
            let s = CFString::wrap_under_create_rule(t as _);
            s.to_string()
        })
    }

    /// Read the `AXTitle` as a Rust `String`
    ///
    /// # Safety
    /// - See notes in [`role`]; we wrap under create rule to ensure CFRelease is called.
    pub unsafe fn title(&self) -> Option<String> {
        self.copy_attr("AXTitle").map(|t| {
            let s = CFString::wrap_under_create_rule(t as _);
            s.to_string()
        })
    }

    /// Read the `AXURL` as a Rust `String`, normalizing CFURL to its absolute string.
    ///
    /// # Safety
    /// - If the attribute is CFURL, we wrap it and ask for `absolute().get_string()`;
    ///   both wrappers observe CoreFoundation ownership correctly.
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

    /// Get the currently focused window of this application.
    ///
    /// Returns an `AXElement` that owns a retained reference to the AX window object.
    ///
    /// # Safety
    /// - The returned element is retained and must be dropped to avoid leaks (handled by `Drop`).
    /// - The focused window may change asynchronously; this may be stale by the time it's used.
    pub unsafe fn focused_window(&self) -> Option<Self> {
        self.copy_attr("AXFocusedWindow").map(|p| Self(p as _))
    }

    /// Depth-first search for the first descendant with the given AX role.
    ///
    /// # Safety
    /// - AX trees are dynamic; elements may disappear while traversing.
    /// - Every child we return is retained and dropped safely via `AxElement::drop`.
    pub unsafe fn find_descendants(&self, role: &str, max_depth: usize) -> Option<Self> {
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

    /// Read the `AXDocument` attribute as a `String`; supports CFURL or CFString.
    ///
    /// # Safety
    /// - Same ownership rules as `url()`: we wrap the returned CF object under create rule.
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

    /// Read an arbitrary string-valued AX attribute.
    ///
    /// # Safety
    /// - Assumes the attribute is a `CFString`. Wrapping under create rule ensures release.
    pub unsafe fn string_attr(&self, name: &str) -> Option<String> {
        self.copy_attr(name).map(|t| {
            let s = CFString::wrap_under_create_rule(t as _);
            s.to_string()
        })
    }

    /// Read an arbitrary boolean-valued AX attribute
    ///
    /// # Safety
    /// - The raw `CFTypeRef` is retained; we **must** `CFRelease` it after we use since we
    ///   don't wrap it in a RAII type here.
    pub unsafe fn bool_attr(&self, name: &str) -> Option<bool> {
        self.copy_attr(name).and_then(|t| {
            if CFGetTypeID(t) == CFBooleanGetTypeID() {
                let b = t as CFBooleanRef;
                Some(CFBooleanGetValue(b))
            } else {
                None
            }
        })
    }

    /// Read a numeric AX attribute as `f64`.
    ///
    /// # Safety
    /// - The raw `CFTypeRef` is retained; we explicitly `CFRelease` after extraction.
    pub unsafe fn number_attr_f64(&self, name: &str) -> Option<f64> {
        self.copy_attr(name).and_then(|t| {
            if CFGetTypeID(t) == CFNumberGetTypeID() {
                let n = t as CFNumberRef;
                let mut out: f64 = 0.0;
                let ok = CFNumberGetValue(n, kCFNumberFloatType, &mut out as *mut _ as *mut _);
                if ok {
                    Some(out)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// Convenience accessor for `AXIdentifier` (if present).
    ///
    /// # Safety
    /// - Delegates to `string_attr`, which handles ownership correctly.
    pub unsafe fn identifier(&self) -> Option<String> {
        self.string_attr("AXIdentifier")
    }

    /// Convenience accessor for `AXEnabled` (if present).
    ///
    /// # Safety
    /// - Delegates to `bool_attr`, which balances ownership via `CFRelease`.
    pub unsafe fn enabled(&self) -> Option<bool> {
        self.bool_attr("AXEnabled")
    }
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
