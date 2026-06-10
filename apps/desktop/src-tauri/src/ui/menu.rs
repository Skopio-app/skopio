use tauri::{
    menu::{IconMenuItemBuilder, Menu, SubmenuBuilder},
    AppHandle, Manager, Runtime,
};
use tracing::error;

const HISTORY_BACK_ID: &str = "history.back";
const HISTORY_FORWARD_ID: &str = "history.forward";

pub trait MenuExt<R: Runtime> {
    fn init_menu(&self) -> tauri::Result<()>;
}

impl<R: Runtime> MenuExt<R> for AppHandle<R> {
    fn init_menu(&self) -> tauri::Result<()> {
        let menu = Menu::default(self)?;

        let back = IconMenuItemBuilder::with_id(HISTORY_BACK_ID, "Back")
            .accelerator("CmdOrCtrl+[")
            .build(self)?;
        let forward = IconMenuItemBuilder::with_id(HISTORY_FORWARD_ID, "Forward")
            .accelerator("CmdOrCtrl+]")
            .build(self)?;

        let history_menu = SubmenuBuilder::with_id(self, "history", "History")
            .item(&back)
            .item(&forward)
            .build()?;

        #[cfg(target_os = "macos")]
        menu.insert(&history_menu, 4)?;

        #[cfg(not(target_os = "macos"))]
        menu.append(&history_menu)?;

        self.set_menu(menu)?;
        self.on_menu_event(|app, event| {
            let script = match event.id().as_ref() {
                HISTORY_BACK_ID => "window.history.back();",
                HISTORY_FORWARD_ID => "window.history.forward();",
                _ => return,
            };

            if let Some(window) = app.get_webview_window("main") {
                if let Err(err) = window.eval(script) {
                    error!(%err, "Failed to run history menu action");
                }
            }
        });

        Ok(())
    }
}
