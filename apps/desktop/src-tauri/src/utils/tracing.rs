use anyhow::Result;
use tauri::AppHandle;
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub trait TracingExt {
    fn init_tracing(&self) -> Result<()>;
}

impl TracingExt for AppHandle {
    fn init_tracing(&self) -> Result<()> {
        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| {
                EnvFilter::try_from(if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "info"
                })
            })
            .unwrap();

        let timer = fmt::time::ChronoLocal::rfc_3339();

        #[cfg(debug_assertions)]
        {
            let fmt_layer = fmt::layer()
                .with_timer(timer)
                .with_target(true)
                .with_level(true)
                .with_line_number(true)
                .with_ansi(true);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .ok();
        }

        #[cfg(not(debug_assertions))]
        {
            use std::fs;
            use tauri::Manager;
            use tracing_appender::rolling;

            let log_dir = self.path().app_log_dir().unwrap_or_default();
            fs::create_dir_all(&log_dir).ok();

            let file_appender = rolling::daily(&log_dir, "skopio.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            self.manage(guard);

            let fmt_layer = fmt::layer()
                .with_timer(timer)
                .with_target(true)
                .with_level(true)
                .with_line_number(true)
                .with_ansi(false)
                .with_writer(non_blocking);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .ok();
        }

        Ok(())
    }
}
