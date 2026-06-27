#![cfg(target_os = "macos")]

use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult, EventField,
};
use core_graphics::geometry::CGPoint;
use objc2_foundation::NSAutoreleasePool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{error, info};

use crate::trackers::input_activity::{InputActivityBus, InputActivityKind, MouseButton};

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
    runloop: Arc<Mutex<Option<CFRunLoop>>>,
    input_activity_bus: Arc<InputActivityBus>,
}

impl MouseTracker {
    pub fn new(input_activity_bus: Arc<InputActivityBus>) -> Self {
        Self {
            last_position: Arc::new(Mutex::new(CGPoint::new(0.0, 0.0))),
            last_movement: Arc::new(Mutex::new(Instant::now())),
            pressed_buttons: Arc::new(Mutex::new(MouseButtons {
                left: false,
                right: false,
                middle: false,
                other: false,
            })),
            runloop: Arc::new(Mutex::new(None)),
            input_activity_bus,
        }
    }

    pub fn start_tracking(&self) {
        let last_position = Arc::clone(&self.last_position);
        let last_movement = Arc::clone(&self.last_movement);
        let pressed_buttons = Arc::clone(&self.pressed_buttons);
        let runloop_ref = Arc::clone(&self.runloop);
        let input_activity_bus = Arc::clone(&self.input_activity_bus);

        tokio::task::spawn_blocking(move || unsafe {
            let pool = NSAutoreleasePool::new();
            match CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                vec![
                    CGEventType::MouseMoved,
                    CGEventType::ScrollWheel,
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
                                input_activity_bus.publish(InputActivityKind::MouseMoved {
                                    x: position.x,
                                    y: position.y,
                                });
                            }
                        }

                        CGEventType::LeftMouseDown => {
                            buttons.left = true;
                            input_activity_bus.publish(InputActivityKind::MouseButtonPressed {
                                button: MouseButton::Left,
                            });
                        }
                        CGEventType::LeftMouseUp => buttons.left = false,
                        CGEventType::RightMouseDown => {
                            buttons.right = true;
                            input_activity_bus.publish(InputActivityKind::MouseButtonPressed {
                                button: MouseButton::Right,
                            });
                        }
                        CGEventType::RightMouseUp => buttons.right = false,
                        CGEventType::OtherMouseDown => {
                            buttons.other = true;
                            input_activity_bus.publish(InputActivityKind::MouseButtonPressed {
                                button: MouseButton::Other,
                            });
                        }
                        CGEventType::OtherMouseUp => buttons.other = false,

                        CGEventType::ScrollWheel => {
                            let position = event.location();

                            let delta_y = event.get_integer_value_field(
                                EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_1,
                            );
                            let delta_x = event.get_integer_value_field(
                                EventField::SCROLL_WHEEL_EVENT_DELTA_AXIS_2,
                            );
                            let point_delta_y = event.get_integer_value_field(
                                EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1,
                            );
                            let point_delta_x = event.get_integer_value_field(
                                EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2,
                            );
                            let is_continuous = event.get_integer_value_field(
                                EventField::SCROLL_WHEEL_EVENT_IS_CONTINUOUS,
                            ) != 0;

                            input_activity_bus.publish(InputActivityKind::MouseScrolled {
                                x: position.x,
                                y: position.y,
                                delta_x,
                                delta_y,
                                point_delta_x,
                                point_delta_y,
                                is_continuous,
                            });
                        }
                        _ => {}
                    }

                    CallbackResult::Keep
                },
            ) {
                Ok(tap) => {
                    let loop_source = match tap.mach_port().create_runloop_source(0) {
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

    pub fn stop_tracking(&self) {
        if let Some(ref rl) = *self.runloop.lock().unwrap() {
            CFRunLoop::stop(rl);
            info!("Mouse tracker stopped");
        }
    }
}
