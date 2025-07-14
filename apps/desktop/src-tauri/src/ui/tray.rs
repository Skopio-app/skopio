use anyhow::anyhow;
use chrono::{Datelike, Duration, Local, TimeZone, Utc};
use common::models::inputs::SummaryQueryInput;
use ril::prelude::*;
use ril::text::Font;
use std::time::Duration as StdDuration;
use std::{
    io::Cursor,
    sync::{Arc, LazyLock},
};
use tauri::Error;
use tauri::{
    tray::{TrayIcon, TrayIconBuilder},
    App, AppHandle, Manager,
};
use tokio::sync::Mutex;

use crate::helpers::time::format_duration;
use crate::network::summaries::fetch_total_time;

static FONT: LazyLock<Font> = LazyLock::new(|| {
    let font_data = include_bytes!("../../fonts/RobotoMono-Regular.ttf");
    Font::from_bytes(font_data, 16.0).expect("Failed to load embedded font")
});

#[derive(Default)]
struct SharedTray {
    icon: Mutex<Option<TrayIcon>>,
}

fn generate_text_icon(app_handle: AppHandle, time_string: String) -> Result<Vec<u8>, String> {
    let scale_factor = app_handle
        .get_webview_window("main")
        .map(|w| w.scale_factor().unwrap_or(1.0))
        .unwrap_or(1.0);

    let font: &Font = &FONT;

    let base_width = 120.0;
    let base_height = 22.0;
    let base_font_size = 16.0;
    let base_padding = 4.0;

    let width = (base_width * scale_factor).round() as u32;
    let height = (base_height * scale_factor).round() as u32;
    let font_size = (base_font_size * scale_factor) as f32;
    let padding = (base_padding * scale_factor).round() as u32;

    let text_color = Rgba::new(255u8, 255u8, 255u8, 255u8);
    let box_color = Rgba::new(3u8, 7u8, 21u8, 0u8);

    let mut img = Image::new(width, height, Rgba::new(0, 0, 0, 0));

    let layout = TextLayout::new()
        .with_vertical_anchor(VerticalAnchor::Center)
        .with_position(padding, height / 2)
        .with_segment(&TextSegment::new(font, &time_string, text_color).with_size(font_size));

    let text_bg_rect = Rectangle::<Rgba>::at(0, 0)
        .with_size(width, height)
        .with_fill(box_color);
    img.draw(&text_bg_rect);

    img.draw(&layout);

    let mut png_bytes: Vec<u8> = Vec::new();
    match img.encode(ImageFormat::Png, &mut Cursor::new(&mut png_bytes)) {
        Ok(_) => Ok(png_bytes),
        Err(e) => Err(format!("Failed to encode PNG using ril: {}", e)),
    }
}

pub fn init_tray(app: &mut App) -> tauri::Result<()> {
    let app_handle = app.handle();
    let tray_state = Arc::new(SharedTray::default());

    let initial_icon_bytes = generate_text_icon(app_handle.clone(), "00.00".into())
        .map_err(|e| Error::from(anyhow!(e)))?;
    let initial_icon = tauri::image::Image::from_bytes(&initial_icon_bytes)?;
    let tray = TrayIconBuilder::new()
        .icon(initial_icon)
        .tooltip("Total active time for the day")
        .build(app)?;

    // Store tray in shared state
    {
        let tray_state = tray_state.clone();
        let tray_clone = tray.clone();
        tokio::spawn(async move {
            tray_state.icon.lock().await.replace(tray_clone);
        });
    }

    // Start tray update task
    {
        let app_handle = app_handle.clone();
        let tray_state = tray_state.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(StdDuration::from_secs(30));

            loop {
                interval.tick().await;

                let local_now = Local::now();
                let local_date = local_now.date_naive();

                let local_start = Local
                    .with_ymd_and_hms(
                        local_date.year(),
                        local_date.month(),
                        local_date.day(),
                        0,
                        0,
                        0,
                    )
                    .single()
                    .unwrap();

                let local_end = local_start + Duration::days(1) - Duration::nanoseconds(1);

                let utc_start = local_start.with_timezone(&Utc);
                let utc_end = local_end.with_timezone(&Utc);

                let query = SummaryQueryInput {
                    start: Some(utc_start),
                    end: Some(utc_end),
                    apps: None,
                    projects: None,
                    categories: None,
                    entities: None,
                    branches: None,
                    languages: None,
                    include_afk: false,
                };

                let time_secs = fetch_total_time(query).await.unwrap();

                let time = format_duration(time_secs as u64);

                if let Ok(icon_bytes) = generate_text_icon(app_handle.clone(), time) {
                    if let Ok(new_icon) = tauri::image::Image::from_bytes(&icon_bytes) {
                        let tray_lock = tray_state.icon.lock().await;
                        if let Some(ref tray) = *tray_lock {
                            let _ = tray.set_icon(Some(new_icon));
                        }
                    }
                }
            }
        });
    }
    Ok(())
}
