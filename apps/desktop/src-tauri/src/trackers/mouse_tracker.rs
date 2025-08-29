#![cfg(target_os = "macos")]

use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
};
use core_graphics::geometry::CGPoint;
use log::{error, info};
use objc2_foundation::NSAutoreleasePool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// #[derive(Debug, Clone)]
// pub struct CursorPosition {
//     pub x: f64,
//     pub y: f64,
// }

#[derive(Debug, Clone, PartialEq)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub other: bool,
}

pub struct MouseTracker {
    last_position: Arc<Mutex<CGPoint>>,
    last_movement: Arc<Mutex<Instant>>,
    pressed_buttons: Arc<Mutex<MouseButtons>>,
    mouse_moved: Arc<AtomicBool>,
    runloop: Arc<Mutex<Option<CFRunLoop>>>,
    // pub tx: watch::Sender<Option<CursorPosition>>,
    // pub rx: watch::Receiver<Option<CursorPosition>>,
}

impl MouseTracker {
    pub fn new() -> Self {
        // let (tx, rx) = watch::channel(None);
        Self {
            last_position: Arc::new(Mutex::new(CGPoint::new(0.0, 0.0))),
            last_movement: Arc::new(Mutex::new(Instant::now())),
            pressed_buttons: Arc::new(Mutex::new(MouseButtons {
                left: false,
                right: false,
                middle: false,
                other: false,
            })),
            mouse_moved: Arc::new(AtomicBool::new(false)),
            runloop: Arc::new(Mutex::new(None)),
            // tx,
            // rx,
        }
    }

    pub fn start_tracking(&self) {
        let last_position = Arc::clone(&self.last_position);
        let last_movement = Arc::clone(&self.last_movement);
        let pressed_buttons = Arc::clone(&self.pressed_buttons);
        let mouse_moved = Arc::clone(&self.mouse_moved);
        let runloop_ref = Arc::clone(&self.runloop);
        // let tx = self.tx.clone();

        tokio::task::spawn_blocking(move || unsafe {
            let pool = NSAutoreleasePool::new();
            match CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
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
                            let dx = (position.x - last_pos.x).abs();
                            let dy = (position.y - last_pos.y).abs();
                            let movement_threshold = 100.0;
                            let debounce_duration = Duration::from_millis(50);
                            let now = Instant::now();

                            if (dx > movement_threshold || dy > movement_threshold)
                                && now.duration_since(*last_move_time) > debounce_duration
                            {
                                *last_pos = position;
                                *last_move_time = now;
                                mouse_moved.store(true, Ordering::Relaxed);

                                // let activity = CursorPosition {
                                //     x: position.x,
                                //     y: position.y,
                                // };

                                // if tx.send(Some(activity)).is_err() {
                                //     warn!("No subscribers to receive mouse updates");
                                // };
                            }
                        }

                        CGEventType::LeftMouseDown => buttons.left = true,
                        CGEventType::LeftMouseUp => buttons.left = false,
                        CGEventType::RightMouseDown => buttons.right = true,
                        CGEventType::RightMouseUp => buttons.right = false,
                        CGEventType::OtherMouseDown => buttons.other = true,
                        CGEventType::OtherMouseUp => buttons.other = false,
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
                    let current = CFRunLoop::get_current();
                    let current_clone = current.clone();
                    *runloop_ref.lock().unwrap() = Some(current_clone);
                    current.add_source(&loop_source, kCFRunLoopCommonModes);
                    tap.enable();
                    CFRunLoop::run_current();
                }
                Err(_) => {
                    error!("Failed to create cursor event tap!");
                }
            }
            drop(pool);
        });
    }

    pub fn get_pressed_mouse_buttons(&self) -> MouseButtons {
        self.pressed_buttons.lock().unwrap().clone()
    }

    pub fn has_mouse_moved(&self) -> bool {
        self.mouse_moved.swap(false, Ordering::Relaxed)
    }

    // pub fn subscribe(&self) -> watch::Receiver<Option<CursorPosition>> {
    //     self.rx.clone()
    // }

    pub fn stop_tracking(&self) {
        if let Some(ref rl) = *self.runloop.lock().unwrap() {
            CFRunLoop::stop(rl);
            info!("Mouse tracker stopped");
        }
    }
}
