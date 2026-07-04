use db::DBContext;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use sync_service::BufferedTrackingService;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tracing::{error, info};
use trackers::TrackerLifecycle;
use trackers::{
    afk_tracker::AFKTracker, event_tracker::EventTracker, keyboard_tracker::KeyboardTracker,
    mouse_tracker::MouseTracker, window_tracker::WindowTracker,
};
use tracking_service::{DBService, TrackingService};
use utils::{config::ConfigStore, db::get_db_path};

use crate::{
    goals_service::GoalService,
    server::{ServerManagerExt, ServerStatus},
    trackers::{input_activity::InputActivityBus, power_monitor::PowerMonitor},
    ui::{
        menu::MenuExt,
        tray::TrayExt,
        window::{NotificationPayload, WindowExt, WindowKind},
    },
    utils::{
        ax::{
            cache::{AxSnapshotCache, AxSnapshotCacheConfig},
            provider::SystemAxProvider,
        },
        tracing::TracingExt,
    },
};

mod goals_service;
pub mod monitored_app;
mod network;
mod server;
mod sync_service;
pub mod trackers;
mod tracking_service;
mod ui;
pub mod utils;

const APP_SHUTDOWN_STARTED_EVENT: &str = "app-shutdown-started";

#[derive(Clone, Copy)]
enum AppShutdownAction {
    Exit,
    Restart,
}

mod app_process {
    use super::*;

    #[tauri::command]
    #[specta::specta]
    pub fn relaunch<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
        request_app_shutdown_with_action(app_handle, AppShutdownAction::Restart);
        Ok(())
    }
}

#[tokio::main]
pub async fn run() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let input_activity_bus = Arc::new(InputActivityBus::new());

    let cursor_tracker = Arc::new(MouseTracker::new(Arc::clone(&input_activity_bus)));
    let keyboard_tracker = Arc::new(KeyboardTracker::new(Arc::clone(&input_activity_bus)));
    let window_tracker = Arc::new(WindowTracker::new());
    let shutdown = Arc::new(AppShutdown::default());

    let specta_builder = make_specta_builder();

    let app = tauri::Builder::default()
        .manage(Arc::clone(&cursor_tracker))
        .manage(Arc::clone(&keyboard_tracker))
        .manage(Arc::clone(&window_tracker))
        .manage(Arc::clone(&input_activity_bus))
        .manage(Arc::clone(&shutdown))
        .invoke_handler({
            let handler = specta_builder.invoke_handler();
            move |invoke| handler(invoke)
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            specta_builder.mount_events(&app_handle);
            app_handle.init_tracing()?;

            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup_trackers(&app_handle_clone).await {
                    error!("Failed async setup: {e}");
                }

                if !cfg!(debug_assertions)
                    && let Err(e) = app_handle_clone.ensure_server_ready().await
                {
                    error!("Server manager error: {e}")
                }
            });

            app_handle.init_menu()?;
            app_handle.show_window(WindowKind::Main)?;

            app.init_tray()?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.label() == "main"
            {
                let shutdown = window.state::<Arc<AppShutdown>>();
                if !shutdown.is_complete() {
                    api.prevent_close();
                    request_app_shutdown(window.app_handle().clone());
                }
            }
        })
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(tauri::generate_context!())
        .expect("Error while running Tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            let shutdown = app_handle.state::<Arc<AppShutdown>>();
            if !shutdown.is_complete() {
                api.prevent_exit();
                request_app_shutdown(app_handle.clone());
            }
        }
    });
}

async fn setup_trackers(app_handle: &AppHandle) -> Result<(), anyhow::Error> {
    if app_shutdown_started(app_handle) {
        return Ok(());
    }

    let config_store = ConfigStore::new(app_handle).await?;
    app_handle.manage(config_store.clone());

    if app_shutdown_started(app_handle) {
        return Ok(());
    }

    let window_tracker = app_handle.state::<Arc<WindowTracker>>();
    <WindowTracker as TrackerLifecycle>::start_tracking(Arc::clone(&window_tracker), ());

    let ax_cache_rx = window_tracker.subscribe();

    let ax_provider = Arc::new(SystemAxProvider);
    let ax_cache = Arc::new(AxSnapshotCache::new(
        ax_provider,
        ax_cache_rx,
        AxSnapshotCacheConfig {
            max_age: Duration::from_millis(700),
        },
    ));

    let db_path = get_db_path(app_handle);
    let db_url = format!("sqlite://{}", db_path.to_str().unwrap());

    let db_result = tokio::spawn(async move { DBContext::new(&db_url).await })
        .await
        .expect("DB task panicked");

    if app_shutdown_started(app_handle) {
        return Ok(());
    }

    let db = match db_result {
        Ok(db) => Arc::new(db),
        Err(err) => {
            error!("Failed to connect to database: {}", err);
            std::process::exit(1);
        }
    };

    app_handle.manage::<Arc<DBContext>>(db.clone());

    let raw_service = Arc::new(DBService::new(Arc::clone(&db)));
    let sync_interval_rx = config_store.subscribe_sync_interval();
    let flush_interval_rx = config_store.subscribe_flush_interval();
    let buffered_service = Arc::new(BufferedTrackingService::new(
        raw_service,
        Arc::clone(&db),
        flush_interval_rx,
        sync_interval_rx,
    ));
    app_handle.manage(Arc::clone(&buffered_service));

    let goal_service = Arc::new(GoalService::new(Arc::clone(&db)));
    app_handle.manage(Arc::clone(&goal_service));
    goal_service.start(app_handle);

    let service_trait: Arc<dyn TrackingService> = buffered_service.clone();

    let input_activity_bus = app_handle.state::<Arc<InputActivityBus>>();
    let mouse_tracker = app_handle.state::<Arc<MouseTracker>>();
    let keyboard_tracker = app_handle.state::<Arc<KeyboardTracker>>();
    let afk_timeout_rx = config_store.subscribe_afk_timeout();
    let afk_tracker = Arc::new(AFKTracker::new(afk_timeout_rx, Arc::clone(&service_trait)));
    app_handle.manage(Arc::clone(&afk_tracker));

    let tracked_apps_rx = config_store.subscribe_tracked_apps();
    let afk_state_rx = afk_tracker.subscribe_state();

    let event_tracker = Arc::new(EventTracker::new(
        Arc::clone(&service_trait),
        tracked_apps_rx,
        ax_cache.clone(),
    ));
    app_handle.manage(Arc::clone(&event_tracker));

    let event_window_rx = window_tracker.subscribe();

    <MouseTracker as TrackerLifecycle>::start_tracking(Arc::clone(&mouse_tracker), ());

    let power_monitor = Arc::new(PowerMonitor::new());
    <PowerMonitor as TrackerLifecycle>::start_tracking(Arc::clone(&power_monitor), ());
    app_handle.manage(Arc::clone(&power_monitor));

    let power_rx_afk = power_monitor.subscribe();
    let power_rx_events = power_monitor.subscribe();

    let input_activity_rx = input_activity_bus.subscribe();

    <AFKTracker as TrackerLifecycle>::start_tracking(
        Arc::clone(&afk_tracker),
        (power_rx_afk, input_activity_rx),
    );

    <KeyboardTracker as TrackerLifecycle>::start_tracking(Arc::clone(&keyboard_tracker), ());

    <EventTracker as TrackerLifecycle>::start_tracking(
        Arc::clone(&event_tracker),
        (event_window_rx, afk_state_rx, power_rx_events),
    );

    Ok(())
}

#[derive(Default)]
struct AppShutdown {
    started: AtomicBool,
    complete: AtomicBool,
}

impl AppShutdown {
    fn begin(&self) -> bool {
        self.started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    fn is_started(&self) -> bool {
        self.started.load(Ordering::SeqCst)
    }

    fn is_complete(&self) -> bool {
        self.complete.load(Ordering::SeqCst)
    }

    fn mark_complete(&self) {
        self.complete.store(true, Ordering::SeqCst);
    }
}

fn app_shutdown_started<R: Runtime>(app_handle: &AppHandle<R>) -> bool {
    app_handle
        .try_state::<Arc<AppShutdown>>()
        .is_some_and(|shutdown| shutdown.is_started())
}

pub(crate) fn request_app_shutdown<R: Runtime>(app_handle: AppHandle<R>) {
    request_app_shutdown_with_action(app_handle, AppShutdownAction::Exit);
}

fn request_app_shutdown_with_action<R: Runtime>(
    app_handle: AppHandle<R>,
    action: AppShutdownAction,
) {
    let Some(shutdown) = app_handle
        .try_state::<Arc<AppShutdown>>()
        .map(|shutdown| Arc::clone(shutdown.inner()))
    else {
        match action {
            AppShutdownAction::Exit => app_handle.exit(0),
            AppShutdownAction::Restart => app_handle.restart(),
        }
        return;
    };

    if !shutdown.begin() {
        return;
    }

    if let Err(err) = app_handle.show_window(WindowKind::Main) {
        error!(%err, "Failed to show main window before shutdown");
    }

    if let Err(err) = app_handle.emit(APP_SHUTDOWN_STARTED_EVENT, ()) {
        error!(%err, "Failed to emit app shutdown started event");
    }

    tokio::spawn(async move {
        run_app_shutdown(&app_handle).await;
        shutdown.mark_complete();
        match action {
            AppShutdownAction::Exit => app_handle.exit(0),
            AppShutdownAction::Restart => app_handle.restart(),
        }
    });
}

async fn run_app_shutdown<R: Runtime>(app_handle: &AppHandle<R>) {
    info!("App shutdown started");

    // tokio::time::sleep(Duration::from_secs(12)).await;

    if let Some(mouse_tracker) = managed_state::<MouseTracker, R>(app_handle) {
        mouse_tracker.shutdown().await;
    }

    if let Some(keyboard_tracker) = managed_state::<KeyboardTracker, R>(app_handle) {
        keyboard_tracker.shutdown().await;
    }

    if let Some(window_tracker) = managed_state::<WindowTracker, R>(app_handle) {
        window_tracker.shutdown().await;
    }

    if let Some(power_monitor) = managed_state::<PowerMonitor, R>(app_handle) {
        power_monitor.shutdown().await;
    }

    if let Some(goal_service) = managed_state::<GoalService, R>(app_handle) {
        goal_service.shutdown();
    }

    let event_tracker = managed_state::<EventTracker, R>(app_handle);
    let afk_tracker = managed_state::<AFKTracker, R>(app_handle);

    if let Some(event_tracker) = &event_tracker {
        event_tracker.shutdown().await;
    }

    if let Some(afk_tracker) = &afk_tracker {
        afk_tracker.shutdown().await;
    }

    if let Some(event_tracker) = &event_tracker {
        event_tracker.flush().await;
    }

    if let Some(afk_tracker) = &afk_tracker {
        afk_tracker.flush().await;
    }

    if let Some(buffered_service) = managed_state::<BufferedTrackingService, R>(app_handle) {
        buffered_service.shutdown().await;
    }

    info!("App shutdown complete");
}

fn managed_state<T, R>(app_handle: &AppHandle<R>) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
    R: Runtime,
{
    app_handle
        .try_state::<Arc<T>>()
        .map(|state| Arc::clone(state.inner()))
}

fn make_specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    let builder = tauri_specta::Builder::<R>::new()
        .commands(tauri_specta::collect_commands![
            crate::utils::config::get_config::<tauri::Wry>,
            crate::utils::config::set_theme::<tauri::Wry>,
            crate::utils::config::set_afk_timeout::<tauri::Wry>,
            crate::utils::config::set_tracked_apps::<tauri::Wry>,
            crate::utils::config::set_global_shortcut::<tauri::Wry>,
            crate::utils::permissions::get_permissions,
            crate::utils::permissions::request_accessibility_permission,
            crate::utils::permissions::request_input_monitoring_permission,
            crate::utils::permissions::open_permission_settings,
            crate::network::summaries::fetch_bucketed_summary,
            crate::network::summaries::fetch_total_time,
            crate::goals_service::add_goal,
            crate::goals_service::get_goals,
            crate::goals_service::update_goal,
            crate::goals_service::remove_goal,
            crate::network::data::fetch_apps,
            crate::network::data::fetch_categories,
            crate::network::data::fetch_projects,
            crate::network::data::fetch_project,
            crate::network::insights::fetch_insights,
            crate::network::events::fetch_events,
            crate::network::afk_events::fetch_afk_events,
            crate::ui::window::dismiss_notification_window::<tauri::Wry>,
            crate::ui::window::show_window::<tauri::Wry>,
            crate::monitored_app::get_open_apps,
            crate::server::get_server_status::<tauri::Wry>,
            crate::app_process::relaunch::<tauri::Wry>,
        ])
        .events(tauri_specta::collect_events![ServerStatus])
        .error_handling(tauri_specta::ErrorHandlingMode::Throw)
        .typ::<NotificationPayload>();

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .formatter(specta_typescript::formatter::prettier)
                .bigint(specta_typescript::BigIntExportBehavior::Number)
                .header("/* eslint-disable */\n// @ts-nocheck\n\n"),
            "../src/types/tauri.gen.ts",
        )
        .expect("Failed to export typescript bindings");

    builder
}
