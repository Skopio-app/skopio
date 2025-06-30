use objc2::runtime::AnyObject;
use objc2::{msg_send, runtime::Bool};
use objc2_app_kit::NSWindow;
use objc2_foundation::NSRect;

#[cfg(target_os = "macos")]
pub unsafe fn customize_toolbar(window: &NSWindow) {
    let _: () = msg_send![window, setTitlebarAppearsTransparent: Bool::YES];
    let _: () = msg_send![window, setMovableByWindowBackground: Bool::NO];
}

#[cfg(target_os = "macos")]
pub unsafe fn adjust_traffic_light_position(window: &NSWindow) {
    let button_types = [
        (0, 20.0), /* NSWindowCloseButton */
        (1, 40.0), /* NSWindowMiniaturizeButton */
        (2, 60.0), /* NSWindowZoomButton */
    ];

    for &(btn_type, x_offset) in &button_types {
        let button: *mut AnyObject = msg_send![window, standardWindowButton: btn_type];
        if button.is_null() {
            continue;
        }

        let mut frame: NSRect = msg_send![button, frame];
        frame.origin.x = x_offset;
        frame.origin.y = 4.0;

        let _: () = msg_send![button, setFrame: frame];
    }
}
