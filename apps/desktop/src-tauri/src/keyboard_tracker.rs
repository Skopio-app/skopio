use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
};
use log::error;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct KeyboardTracker {
    last_keypress: Arc<Mutex<Instant>>,
    pressed_keys: Arc<Mutex<Vec<String>>>,
    runloop: Arc<Mutex<Option<CFRunLoop>>>,
}

impl KeyboardTracker {
    pub fn new() -> Self {
        Self {
            last_keypress: Arc::new(Mutex::new(Instant::now())),
            pressed_keys: Arc::new(Mutex::new(Vec::new())),
            runloop: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_tracking(self: Arc<Self>) {
        let pressed_keys = Arc::clone(&self.pressed_keys);
        let last_keypress = Arc::clone(&self.last_keypress);
        let runloop_ref = Arc::clone(&self.runloop);

        tokio::task::spawn_blocking(move || unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let current = CFRunLoop::get_current();
            match CGEventTap::new(
                CGEventTapLocation::HID,
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
            pool.drain();
        });
    }

    pub fn get_pressed_keys(&self) -> Vec<String> {
        let keys = self.pressed_keys.lock().unwrap();
        keys.clone()
    }

    pub fn stop_tracking(&self) {
        if let Some(ref rl) = *self.runloop.lock().unwrap() {
            CFRunLoop::stop(rl);
        }
    }
}
