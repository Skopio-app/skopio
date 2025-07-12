use log::{debug, error};
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow,
    WebviewWindowBuilder,
};
use url::{ParseError, Url};

const NOTIFICATION_WINDOW_LABEL: &str = "notification";

fn get_notification_url(payload: Option<String>) -> Result<Url, ParseError> {
    let base_url = if cfg!(dev) {
        "http://localhost:5173/notification.html"
    } else {
        "tauri://localhost/notification.html"
    };

    let mut url = Url::parse(base_url)?;
    if let Some(payload_data) = payload {
        url.query_pairs_mut().append_pair("payload", &payload_data);
    }

    Ok(url)
}

fn get_main_screen_safe_frame<R: Runtime>(
    app: &AppHandle<R>,
) -> Option<(PhysicalPosition<f64>, PhysicalSize<f64>, f64)> {
    let primary_monitor = app.primary_monitor().ok()??;

    let monitor_size = primary_monitor
        .size()
        .to_logical::<f64>(1.0)
        .to_physical(1.0);
    let monitor_position = primary_monitor
        .position()
        .to_logical::<f64>(1.0)
        .to_physical(1.0);
    let scale_factor = primary_monitor.scale_factor();

    Some((monitor_position, monitor_size, scale_factor))
}

fn show_notification_window<R: Runtime>(
    notification_window: WebviewWindow<R>,
    payload: Option<String>,
) -> tauri::Result<()> {
    let Ok(notification_url) = get_notification_url(payload) else {
        error!("Failed to get notification URl");
        return Err(tauri::Error::WindowNotFound);
    };

    let Ok(_) = notification_window.navigate(notification_url) else {
        error!("Failed to navigate to notification URL");
        return Err(tauri::Error::WindowNotFound);
    };

    notification_window.show()?;
    return Ok(());
}

fn get_notification_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<WebviewWindow<R>> {
    let app_handle = app.app_handle();
    let Some(notification_window) = app_handle.get_webview_window(NOTIFICATION_WINDOW_LABEL) else {
        error!("No notification window found");
        return Err(tauri::Error::WindowNotFound);
    };

    return Ok(notification_window);
}

pub fn create_notification_window<R: Runtime>(
    app: &AppHandle<R>,
    payload: Option<String>,
) -> tauri::Result<()> {
    let app_handle = app.app_handle();

    if let Ok(notification_window) = get_notification_window(app) {
        show_notification_window(notification_window, payload.clone())?;

        return Ok(());
    }

    let mut params = String::new();
    if let Some(ref payload_data) = payload {
        let encoded_payload = payload_data
            .replace("&", "%26")
            .replace("=", "%3D")
            .replace(" ", "%20");
        params.push_str(&format!("&payload={}", encoded_payload));
    }

    let notification_url = if cfg!(dev) {
        tauri::WebviewUrl::External(
            format!("http://localhost:5713/notification.html{}", params)
                .parse()
                .unwrap(),
        )
    } else {
        tauri::WebviewUrl::External(
            format!("tauri://localhost/notification.html{}", params)
                .parse()
                .unwrap(),
        )
    };

    let win_width_logical = 450.0;
    let win_height_logical = 100.0;

    let screen_info = get_main_screen_safe_frame(app);

    if let Some((monitor_position, monitor_size, scale_factor)) = screen_info {
        let win_width_physical = win_width_logical * scale_factor;
        // let win_height_physical = win_height_logical * scale_factor;

        let x_physical = monitor_position.x + (monitor_size.width - win_width_physical) / 2.0;

        let top_margin_physical = 40.0 * scale_factor;
        let y_physical = monitor_position.y + top_margin_physical;

        let x_logical = x_physical / scale_factor;
        let y_logical = y_physical / scale_factor;

        let notification_window =
            WebviewWindowBuilder::new(app, NOTIFICATION_WINDOW_LABEL, notification_url)
                .title("Notification")
                .inner_size(win_width_logical, win_height_logical)
                .position(x_logical, y_logical)
                .transparent(true)
                .decorations(false)
                .shadow(false)
                .resizable(false)
                .visible(false)
                .always_on_top(true)
                .build();

        match notification_window {
            Ok(window) => {
                debug!(
                    "Successfully created notification window: {}",
                    window.label()
                );
            }
            Err(err) => {
                error!("Failed to create notification window: {}", err);
                return Err(err);
            }
        }

        let notification_window = app_handle
            .get_webview_window(NOTIFICATION_WINDOW_LABEL)
            .unwrap();

        show_notification_window(notification_window, payload)?;
    }

    #[allow(dead_code)]
    pub fn dismiss_notification_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
        let Ok(window) = get_notification_window(app) else {
            error!("Failed to get notification window");
            return Err(tauri::Error::WindowNotFound);
        };

        window.close()
    }

    Ok(())
}
