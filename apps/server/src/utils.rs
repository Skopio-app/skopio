use dirs::data_dir;
use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn get_db_name() -> String {
    if cfg!(debug_assertions) {
        String::from("skopio_server_test.db")
    } else {
        String::from("skopio_server.db")
    }
}

pub fn get_db_path() -> PathBuf {
    let data_dir = data_dir().unwrap_or_else(|| PathBuf::from("."));
    data_dir
        .join("com.samwahome.skopio")
        .join("server")
        .join(get_db_name())
}

pub fn init_tracing() -> anyhow::Result<()> {
    let level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    #[cfg(debug_assertions)]
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_line_number(true)
                    .with_timer(fmt::time::ChronoLocal::rfc_3339()),
            )
            .try_init()
            .ok();
    }

    #[cfg(not(debug_assertions))]
    {
        use std::fs;
        use tracing_appender::rolling::{RollingFileAppender, Rotation};

        let log_dir: PathBuf = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Library/Logs/com.samwahome.skopio/server");
        fs::create_dir_all(&log_dir).ok();

        let file_appender = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("server")
            .filename_suffix("log")
            .build(log_dir)?;
        let (file_nb, guard) = tracing_appender::non_blocking(file_appender);

        Box::leak(Box::new(guard));

        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_writer(file_nb)
                    .with_line_number(true)
                    .with_timer(fmt::time::ChronoLocal::rfc_3339()),
            )
            .try_init()
            .ok();
    }

    Ok(())
}
