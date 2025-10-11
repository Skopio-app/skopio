#![cfg(target_os = "macos")]

use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField,
};
use objc2_foundation::NSAutoreleasePool;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{error, info};

pub struct KeyboardTracker {
    last_keypress: Arc<Mutex<Instant>>,
    pressed_keys: Arc<Mutex<HashSet<i64>>>,
    runloop: Arc<Mutex<Option<CFRunLoop>>>,
}

impl KeyboardTracker {
    pub fn new() -> Self {
        Self {
            last_keypress: Arc::new(Mutex::new(Instant::now())),
            pressed_keys: Arc::new(Mutex::new(HashSet::new())),
            runloop: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_tracking(self: Arc<Self>) {
        let pressed_keys = Arc::clone(&self.pressed_keys);
        let last_keypress = Arc::clone(&self.last_keypress);
        let runloop_ref = Arc::clone(&self.runloop);

        tokio::task::spawn_blocking(move || unsafe {
            let pool = NSAutoreleasePool::new();
            let current = CFRunLoop::get_current();
            match CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                vec![
                    CGEventType::KeyDown,
                    CGEventType::KeyUp,
                    CGEventType::FlagsChanged,
                ],
                |_proxy, event_type, event| {
                    let key_event = event.clone();
                    let now = Instant::now();

                    *last_keypress.lock().unwrap() = now;

                    let key_code =
                        key_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);

                    let mut keys = pressed_keys.lock().unwrap();

                    match event_type {
                        CGEventType::KeyDown => {
                            let repeat = key_event
                                .get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT)
                                != 0;
                            if !repeat {
                                keys.insert(key_code);
                            }
                        }
                        CGEventType::KeyUp => {
                            keys.remove(&key_code);
                        }
                        CGEventType::FlagsChanged => {
                            let flags = event.get_flags();
                            if let Some(flag) = flag_for_modifier(key_code) {
                                if flags.contains(flag) {
                                    keys.insert(key_code);
                                } else {
                                    keys.remove(&key_code);
                                }
                            } else {
                                keys.remove(&key_code);
                            }
                        }
                        _ => {}
                    }
                    None
                },
            ) {
                Ok(tap) => {
                    let loop_source = match tap.mach_port.create_runloop_source(0) {
                        Ok(source) => source,
                        Err(_) => {
                            error!("Failed to create runloop source!");
                            return;
                        }
                    };
                    let current_clone = current.clone();
                    *runloop_ref.lock().unwrap() = Some(current_clone);
                    current.add_source(&loop_source, kCFRunLoopCommonModes);
                    tap.enable();
                    CFRunLoop::run_current();
                }
                Err(_) => {
                    error!("Failed to create keyboard event tap!");
                }
            }
            drop(pool);
        });
    }

    pub fn get_pressed_keys(&self) -> HashSet<i64> {
        let keys = self.pressed_keys.lock().unwrap();
        keys.clone()
    }

    pub fn stop_tracking(&self) {
        if let Some(ref rl) = *self.runloop.lock().unwrap() {
            CFRunLoop::stop(rl);
            info!("Keyboard tracker stopped");
        }
    }
}

fn flag_for_modifier(key_code: i64) -> Option<CGEventFlags> {
    match key_code {
        56 | 60 => Some(CGEventFlags::CGEventFlagShift), // Left/Right Shift
        59 | 62 => Some(CGEventFlags::CGEventFlagControl), // Left/Right Control
        58 | 61 => Some(CGEventFlags::CGEventFlagAlternate), // Left/Right Option (Alt)
        55 | 54 => Some(CGEventFlags::CGEventFlagCommand), // Left/Right Command
        57 => Some(CGEventFlags::CGEventFlagAlphaShift), // Caps Lock
        _ => None,
    }
}

impl Default for KeyboardTracker {
    fn default() -> Self {
        Self::new()
    }
}
