use tauri::{
    menu::{IconMenuItemBuilder, Menu, MenuItemKind, PredefinedMenuItem, Submenu, SubmenuBuilder},
    AppHandle, Manager, Runtime,
};
use tracing::error;

use crate::ui::window::{WindowExt, WindowKind};

const APP_CHECK_UPDATES_ID: &str = "app.check_updates";
const APP_SETTINGS_ID: &str = "app.settings";
const VIEW_RELOAD_ID: &str = "view.reload";
const VIEW_FORCE_RELOAD_ID: &str = "view.force_reload";
const VIEW_TOGGLE_DEVTOOLS_ID: &str = "view.toggle_devtools";
const VIEW_TOGGLE_SIDEBAR_ID: &str = "view.toggle_sidebar";
const HISTORY_BACK_ID: &str = "history.back";
const HISTORY_FORWARD_ID: &str = "history.forward";

pub trait MenuExt<R: Runtime> {
    fn init_menu(&self) -> tauri::Result<()>;
}

impl<R: Runtime> MenuExt<R> for AppHandle<R> {
    fn init_menu(&self) -> tauri::Result<()> {
        let menu = Menu::default(self)?;
        let app_menu = get_app_menu(self, &menu)?;
        let default_app_items = take_menu_items(&app_menu)?;
        let view_menu = get_view_menu(self, &menu)?;
        let default_view_items = take_menu_items(&view_menu)?;

        let check_updates =
            IconMenuItemBuilder::with_id(APP_CHECK_UPDATES_ID, "Check for Updates...")
                .build(self)?;
        let settings = IconMenuItemBuilder::with_id(APP_SETTINGS_ID, "Preferences...")
            .accelerator("CmdOrCtrl+,")
            .build(self)?;

        if let Some((about_item, remaining_items)) = default_app_items.split_first() {
            app_menu.append(about_item)?;
            app_menu.append(&check_updates)?;
            app_menu.append(&PredefinedMenuItem::separator(self)?)?;
            app_menu.append(&settings)?;
            for item in remaining_items {
                app_menu.append(item)?;
            }
        } else {
            app_menu.append(&check_updates)?;
            app_menu.append(&PredefinedMenuItem::separator(self)?)?;
            app_menu.append(&settings)?;
        }

        let reload = IconMenuItemBuilder::with_id(VIEW_RELOAD_ID, "Reload")
            .accelerator("CmdOrCtrl+R")
            .build(self)?;
        let force_reload = IconMenuItemBuilder::with_id(VIEW_FORCE_RELOAD_ID, "Force Reload")
            .accelerator("CmdOrCtrl+Shift+R")
            .build(self)?;
        let toggle_devtools =
            IconMenuItemBuilder::with_id(VIEW_TOGGLE_DEVTOOLS_ID, "Toggle Developer Tools")
                .accelerator("Alt+CmdOrCtrl+I")
                .build(self)?;
        let toggle_sidebar =
            IconMenuItemBuilder::with_id(VIEW_TOGGLE_SIDEBAR_ID, "Show/Hide Sidebar")
                .accelerator("CmdOrCtrl+B")
                .build(self)?;

        view_menu.append(&reload)?;
        view_menu.append(&force_reload)?;
        view_menu.append(&toggle_devtools)?;
        view_menu.append(&PredefinedMenuItem::separator(self)?)?;
        view_menu.append(&toggle_sidebar)?;
        if !default_view_items.is_empty() {
            view_menu.append(&PredefinedMenuItem::separator(self)?)?;
            for item in default_view_items {
                view_menu.append(&item)?;
            }
        }

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
            let result = match event.id().as_ref() {
                APP_SETTINGS_ID => app.show_window(WindowKind::Settings).map(|_| ()),
                APP_CHECK_UPDATES_ID => app
                    .get_webview_window("main")
                    .map(|window| {
                        window.eval("window.dispatchEvent(new Event('skopio:check-for-updates'));")
                    })
                    .unwrap_or(Ok(())),
                id => {
                    app.get_webview_window("main")
                        .map(|window| match id {
                            VIEW_RELOAD_ID => window.reload(),
                            VIEW_FORCE_RELOAD_ID => window.eval("window.location.reload(true);"),
                            VIEW_TOGGLE_DEVTOOLS_ID => {
                                if window.is_devtools_open() {
                                    window.close_devtools();
                                } else {
                                    window.open_devtools();
                                }
                                Ok(())
                            }
                            VIEW_TOGGLE_SIDEBAR_ID => window
                                .eval("window.dispatchEvent(new Event('skopio:toggle-sidebar'));"),
                            HISTORY_BACK_ID => window.eval("window.history.back();"),
                            HISTORY_FORWARD_ID => window.eval("window.history.forward();"),
                            _ => Ok(()),
                        })
                        .unwrap_or(Ok(()))
                }
            };

            if let Err(err) = result {
                error!(%err, "Failed to run menu action");
            }
        });

        Ok(())
    }
}

fn get_app_menu<R: Runtime>(app: &AppHandle<R>, menu: &Menu<R>) -> tauri::Result<Submenu<R>> {
    let app_name = &app.package_info().name;

    get_or_create_menu(app, menu, "app", app_name, |menu, submenu| {
        #[cfg(target_os = "macos")]
        menu.prepend(submenu)?;

        #[cfg(not(target_os = "macos"))]
        menu.append(submenu)?;

        Ok(())
    })
}

fn get_view_menu<R: Runtime>(app: &AppHandle<R>, menu: &Menu<R>) -> tauri::Result<Submenu<R>> {
    get_or_create_menu(app, menu, "view", "View", |menu, submenu| {
        #[cfg(target_os = "macos")]
        menu.insert(submenu, 3)?;

        #[cfg(not(target_os = "macos"))]
        menu.append(submenu)?;

        Ok(())
    })
}

fn get_or_create_menu<R, F>(
    app: &AppHandle<R>,
    menu: &Menu<R>,
    id: &'static str,
    label: &str,
    insert: F,
) -> tauri::Result<Submenu<R>>
where
    R: Runtime,
    F: FnOnce(&Menu<R>, &Submenu<R>) -> tauri::Result<()>,
{
    if let Some(submenu) = menu
        .items()?
        .into_iter()
        .filter_map(|item| match item {
            MenuItemKind::Submenu(submenu) => Some(submenu),
            _ => None,
        })
        .find(|submenu| submenu.text().is_ok_and(|text| text == label))
    {
        return Ok(submenu);
    }

    let submenu = SubmenuBuilder::with_id(app, id, label).build()?;
    insert(menu, &submenu)?;
    Ok(submenu)
}

fn take_menu_items<R: Runtime>(menu: &Submenu<R>) -> tauri::Result<Vec<MenuItemKind<R>>> {
    let item_count = menu.items()?.len();
    let mut items = Vec::with_capacity(item_count);

    for _ in 0..item_count {
        if let Some(item) = menu.remove_at(0)? {
            items.push(item);
        }
    }

    Ok(items)
}
