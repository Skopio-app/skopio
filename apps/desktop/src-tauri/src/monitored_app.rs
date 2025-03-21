use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display)]
pub enum MonitoredApp {
    #[strum(serialize = "com.google.Chrome")]
    Chrome,
    #[strum(serialize = "org.mozilla.firefox")]
    Firefox,
    #[strum(serialize = "com.apple.Safari")]
    Safari,
    #[strum(serialize = "com.apple.Terminal")]
    Terminal,
    #[strum(serialize = "com.apple.dt.Xcode")]
    Xcode,
    #[strum(serialize = "notion.id")]
    Notion,
    #[strum(serialize = "comany.thebrowser.Browser")]
    ArcBrowser,
    #[strum(serialize = "unknown")]
    Unknown,
}
