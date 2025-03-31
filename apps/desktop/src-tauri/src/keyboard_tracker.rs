use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
};
use log::error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub struct KeyboardTracker {
    last_keypress: Arc<Mutex<Instant>>,
    pressed_keys: Arc<Mutex<Vec<String>>>,
}

impl KeyboardTracker {
    pub fn new() -> Self {
        Self {
            last_keypress: Arc::new(Mutex::new(Instant::now())),
            pressed_keys: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start_tracking(self: Arc<Self>) {
        let pressed_keys = Arc::clone(&self.pressed_keys);

        thread::spawn(move || {
            let current = CFRunLoop::get_current();
            match CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![
                    CGEventType::KeyDown,
                    CGEventType::KeyUp,
                    CGEventType::FlagsChanged,
                ],
                |_proxy, event_type, event| {
                    let key_event = event.clone();
                    let now = Instant::now();
                    let last_keypress = Arc::clone(&self.last_keypress);

                    *last_keypress.lock().unwrap() = now;

                    let key_code = key_event.get_integer_value_field(0);
                    let key_str = format!("{:?}", key_code);

                    let mut keys = pressed_keys.lock().unwrap();

                    match event_type {
                        CGEventType::KeyDown | CGEventType::FlagsChanged => {
                            if !keys.contains(&key_str) {
                                keys.push(key_str.clone());
                            }
                        }
                        CGEventType::KeyUp => {
                            keys.retain(|k| k != &key_str);
                        }
                        _ => {}
                    }
                    None
                },
            ) {
                Ok(tap) => unsafe {
                    let loop_source = match tap.mach_port.create_runloop_source(0) {
                        Ok(source) => source,
                        Err(_) => {
                            error!("Failed to create runloop source!");
                            return;
                        }
                    };
                    current.add_source(&loop_source, kCFRunLoopCommonModes);
                    tap.enable();
                    CFRunLoop::run_current();
                },
                Err(_) => {
                    error!("Failed to create keyboard event tap!");
                }
            }
        });
    }

    pub fn get_pressed_keys(&self) -> Vec<String> {
        let keys = self.pressed_keys.lock().unwrap();
        keys.clone()
    }
}
