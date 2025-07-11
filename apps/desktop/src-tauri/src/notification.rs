use anyhow::Result;
use tauri::{AppHandle, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use url::Url;

fn get_main_screen_safe_frame(
    app: &AppHandle,
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

pub fn show_notification(app: &AppHandle, message: &str, duration_ms: Option<u64>) -> Result<()> {
    let mut url = Url::parse("http://localhost:5173/notification.html")?;
    url.query_pairs_mut().append_pair("message", message);

    if let Some(duration) = duration_ms {
        url.query_pairs_mut()
            .append_pair("duration", &duration.to_string());
    }

    let label = "notification";
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

        WebviewWindowBuilder::new(app, label, WebviewUrl::External(url.to_string().parse()?))
            .title("Notification")
            .inner_size(win_width_logical, win_height_logical)
            .position(x_logical, y_logical)
            .transparent(true)
            .decorations(false)
            .shadow(false)
            .resizable(false)
            .visible(true)
            .always_on_top(true)
            .build()?;
    }

    Ok(())
}
