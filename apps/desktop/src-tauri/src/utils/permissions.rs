#![cfg(target_os = "macos")]

use core_foundation::{
    base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
};
use serde::Serialize;
use specta::Type;
use std::ffi::c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
}

/// Normalized status for a permission check.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Type)]
pub enum PermissionStatus {
    Granted,
    DeniedOrNotDetermined,
}

impl PermissionStatus {
    #[inline]
    fn from_bool(ok: bool) -> Self {
        if ok {
            PermissionStatus::Granted
        } else {
            PermissionStatus::DeniedOrNotDetermined
        }
    }
}

fn check_accessibility(prompt: bool) -> PermissionStatus {
    let granted = if prompt {
        let key_str = "AXTrustedCheckOptionPrompt";
        let key = CFString::new(key_str);
        let val = CFBoolean::true_value();

        let pairs: Vec<(CFString, CFBoolean)> = vec![(key, val)];
        let dict = CFDictionary::from_CFType_pairs(&pairs);

        unsafe { AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as *const c_void) }
    } else {
        unsafe { AXIsProcessTrustedWithOptions(std::ptr::null()) }
    };

    PermissionStatus::from_bool(granted)
}

fn check_input_monitoring() -> PermissionStatus {
    let granted = unsafe { CGPreflightListenEventAccess() };
    PermissionStatus::from_bool(granted)
}

fn request_input_monitoring() -> bool {
    unsafe { CGRequestListenEventAccess() }
}

#[derive(Debug, Serialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct PermissionSummary {
    pub accessibility: PermissionStatus,
    pub input_monitoring: PermissionStatus,
}

#[tauri::command]
#[specta::specta]
pub async fn get_permissions() -> PermissionSummary {
    PermissionSummary {
        accessibility: check_accessibility(false),
        input_monitoring: check_input_monitoring(),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn request_accessibility_permission() -> PermissionStatus {
    check_accessibility(true)
}

#[tauri::command]
#[specta::specta]
pub async fn request_input_monitoring_permission() -> PermissionStatus {
    let ok = request_input_monitoring();
    PermissionStatus::from_bool(ok)
}

#[tauri::command]
#[specta::specta]
pub async fn open_permission_settings(kind: String) -> Result<(), String> {
    let url = match kind.as_str() {
        "accessibility" => {
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
        }
        "inputMonitoring" => {
            "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
        }
        _ => return Err("Unsupported permission kind".into()),
    };
    std::process::Command::new("open")
        .arg(url)
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}
