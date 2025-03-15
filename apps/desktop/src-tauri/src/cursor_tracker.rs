use crate::window_tracker::WindowTracker;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
};
use log::error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub other: bool,
}

pub struct CursorTracker {
    last_position: Arc<Mutex<(f64, f64)>>,
    last_movement: Arc<Mutex<Instant>>,
    pressed_buttons: Arc<Mutex<MouseButtons>>,
}

impl CursorTracker {
    pub fn new() -> Self {
        Self {
            last_position: Arc::new(Mutex::new((0.0, 0.0))),
            last_movement: Arc::new(Mutex::new(Instant::now())),
            pressed_buttons: Arc::new(Mutex::new(MouseButtons {
                left: false,
                right: false,
                middle: false,
                other: false,
            })),
        }
    }

    pub fn start_tracking<F>(self: Arc<Self>, heartbeat_callback: F)
    where
        F: Fn(&str, &str, f64, f64) + Send + Sync + 'static,
    {
        let last_position = Arc::clone(&self.last_position);
        let last_movement = Arc::clone(&self.last_movement);
        let pressed_buttons = Arc::clone(&self.pressed_buttons);
        let heartbeat_callback = Arc::new(heartbeat_callback);

        thread::spawn(move || {
            match CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![
                    CGEventType::MouseMoved,
                    CGEventType::LeftMouseDown,
                    CGEventType::LeftMouseUp,
                    CGEventType::RightMouseDown,
                    CGEventType::RightMouseUp,
                    CGEventType::OtherMouseDown,
                    CGEventType::OtherMouseUp,
                ],
                move |_proxy, event_type, event| {
                    let mut last_pos = last_position.lock().unwrap();
                    let mut last_move_time = last_movement.lock().unwrap();
                    let mut buttons = pressed_buttons.lock().unwrap();

                    match event_type {
                        CGEventType::MouseMoved => {
                            let position = event.location();
                            let dx = (position.x - last_pos.0).abs();
                            let dy = (position.y - last_pos.1).abs();
                            if dx > 0.5 || dy > 0.5 {
                                *last_pos = (position.x, position.y);
                                *last_move_time = Instant::now();

                                if let Some(window) = WindowTracker::get_active_window() {
                                    let app_name = window.app_name;
                                    let file = window.title;
                                    let callback = Arc::clone(&heartbeat_callback);
                                    callback(&app_name, &file, position.x, position.y);
                                }
                            }
                        }

                        CGEventType::LeftMouseDown => {
                            buttons.left = true;
                        }
                        CGEventType::LeftMouseUp => {
                            buttons.left = false;
                        }
                        CGEventType::RightMouseDown => {
                            buttons.right = true;
                        }
                        CGEventType::RightMouseUp => {
                            buttons.right = false;
                        }
                        CGEventType::OtherMouseDown => {
                            buttons.other = true;
                        }
                        CGEventType::OtherMouseUp => {
                            buttons.other = false;
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
                    let current = CFRunLoop::get_current();
                    current.add_source(&loop_source, kCFRunLoopCommonModes);
                    tap.enable();
                    CFRunLoop::run_current();
                },
                Err(_) => {
                    error!("Failed to create cursor event tap!");
                }
            }
        });
    }

    pub fn get_pressed_mouse_buttons(&self) -> MouseButtons {
        self.pressed_buttons.lock().unwrap().clone()
    }

    pub fn get_global_cursor_position(&self) -> (f64, f64) {
        *self.last_position.lock().unwrap()
    }
}
