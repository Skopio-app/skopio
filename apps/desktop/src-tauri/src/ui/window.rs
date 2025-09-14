use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, LogicalPosition, Manager, Position, Runtime, TitleBarStyle, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};
use tracing::error;
use url::{ParseError, Url};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum WindowKind {
    Main,
    Settings,
    Notification,
}

impl WindowKind {
    pub fn label(self) -> &'static str {
        match self {
            WindowKind::Main => "main",
            WindowKind::Settings => "settings",
            WindowKind::Notification => "notification",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            WindowKind::Main => "Skopio",
            WindowKind::Settings => "Settings",
            WindowKind::Notification => "Notification",
        }
    }

    /// Returns the base App route for this window (used when not using multiple HTML entries).
    pub fn default_route(self) -> &'static str {
        match self {
            WindowKind::Main => "/",
            WindowKind::Settings => "/settings",
            WindowKind::Notification => "/notification",
        }
    }

    pub fn get<R: Runtime>(self, app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
        app.get_webview_window(self.label())
    }

    fn base_builder<'a, R: Runtime>(
        self,
        app: &'a AppHandle<R>,
        url: WebviewUrl,
    ) -> WebviewWindowBuilder<'a, R, AppHandle<R>> {
        let builder = WebviewWindow::builder(app, self.label(), url)
            .title(self.title())
            .decorations(true)
            .hidden_title(true)
            .title_bar_style(TitleBarStyle::Overlay);

        builder
    }

    fn build<R: Runtime>(self, app: &AppHandle<R>) -> tauri::Result<WebviewWindow<R>> {
        let position = Position::from(LogicalPosition::new(20.0, 15.0));
        match self {
            WindowKind::Main => {
                let url = WebviewUrl::App(self.default_route().into());
                self.base_builder(app, url)
                    .inner_size(1600.0, 900.0)
                    .min_inner_size(800.0, 450.0)
                    .resizable(true)
                    .fullscreen(false)
                    .traffic_light_position(position)
                    .build()
            }
            WindowKind::Settings => {
                let url = WebviewUrl::App(self.default_route().into());
                self.base_builder(app, url)
                    .resizable(true)
                    .inner_size(800.0, 600.0)
                    .min_inner_size(800.0, 600.0)
                    .traffic_light_position(position)
                    .build()
            }
            WindowKind::Notification => {
                let url = notification_base_url();
                let builder = WebviewWindow::builder(app, self.label(), url)
                    .title(self.title())
                    .disable_drag_drop_handler()
                    .transparent(true)
                    .decorations(false)
                    .shadow(false)
                    .resizable(false)
                    .visible(false)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .focused(false)
                    .inner_size(450.0, 100.0);

                let win = builder.build()?;
                postion_notification(&win)?;
                Ok(win)
            }
        }
    }

    pub fn show<R: Runtime>(self, app: &AppHandle<R>) -> tauri::Result<WebviewWindow<R>> {
        if let Some(w) = self.get(app) {
            w.set_focus()?;
            w.show()?;
            return Ok(w);
        }
        let w = self.build(app)?;
        w.set_focus()?;
        w.show()?;
        Ok(w)
    }
}

pub trait WindowExt<R: Runtime> {
    fn show_window(&self, kind: WindowKind) -> tauri::Result<WebviewWindow<R>>;
    fn show_notification(&self, payload: NotificationPayload) -> tauri::Result<()>;
}

impl<R: Runtime> WindowExt<R> for AppHandle<R> {
    fn show_window(&self, kind: WindowKind) -> tauri::Result<WebviewWindow<R>> {
        kind.show(self)
    }

    fn show_notification(&self, payload: NotificationPayload) -> tauri::Result<()> {
        let win = if let Some(w) = WindowKind::Notification.get(self) {
            w
        } else {
            WindowKind::Notification.build(self)?
        };

        let url = get_notification_url(&payload).map_err(|e| {
            error!("Invalid notification url: {e}");
            tauri::Error::WindowNotFound
        })?;

        win.navigate(url)?;
        win.show()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPayload {
    pub title: String,
    pub duration_ms: i64,
    pub message: Option<String>,
    pub sound_file: Option<String>,
}

fn encode_payload(payload: &NotificationPayload) -> Result<String, tauri::Error> {
    let json = serde_json::to_string(payload).map_err(tauri::Error::Json)?;
    Ok(utf8_percent_encode(&json, NON_ALPHANUMERIC).to_string())
}

fn notification_base_url() -> WebviewUrl {
    if cfg!(debug_assertions) {
        WebviewUrl::External("http://localhost:5173/notification.html".parse().unwrap())
    } else {
        WebviewUrl::External("tauri://localhost/notification.html".parse().unwrap())
    }
}

fn get_notification_url(payload: &NotificationPayload) -> Result<Url, ParseError> {
    let base_url = if cfg!(dev) {
        "http://localhost:5173/notification.html"
    } else {
        "tauri://localhost/notification.html"
    };

    let mut url = Url::parse(base_url)?;
    if let Ok(payload_data) = encode_payload(&payload) {
        url.query_pairs_mut().append_pair("payload", &payload_data);
    }

    Ok(url)
}

fn postion_notification<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };

    let scale = monitor.scale_factor();
    let monitor_size = monitor.size().to_logical::<f64>(scale);
    let win_size = window.outer_size()?.to_logical::<f64>(scale);
    let x = (monitor_size.width - win_size.width) / 2.0;
    let y = 40.0;

    window.set_position(LogicalPosition { x, y })?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn dismiss_notification_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(win) = WindowKind::Notification.get(&app) {
        win.close()
            .map_err(|e| format!("Failed to close notification: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn show_window<R: Runtime>(app: AppHandle<R>, kind: WindowKind) -> Result<(), String> {
    app.show_window(kind)
        .map_err(|e| format!("Error showing settings window: {e}"))?;
    Ok(())
}

// #[tauri::command]
// #[specta::specta]
// pub fn open_devtools<R: Runtime>(window: tauri::WebviewWindow<R>) {
//     window.open_devtools();
// }
