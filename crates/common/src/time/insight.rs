use chrono::{
    offset::LocalResult, DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday,
};
use serde::{Deserialize, Serialize};

use crate::time::TimeError;

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct InsightRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TryFrom<String> for InsightRange {
    type Error = TimeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Handle "yyyy-mm-dd"
        if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
            let start = match Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0) {
                LocalResult::Single(dt) => dt,
                _ => return Err(TimeError::InvalidDate),
            };
            let end = start + Duration::days(1);
            return Ok(InsightRange {
                start: start,
                end: end,
            });
        }

        // Handle "yyyy-mm"
        if let Ok(date) = NaiveDate::parse_from_str(&format!("{}-01", value), "%Y-%m-%d") {
            let start = match Utc.with_ymd_and_hms(date.year(), date.month(), 1, 0, 0, 0) {
                LocalResult::Single(dt) => dt,
                _ => return Err(TimeError::InvalidDate),
            };
            let end = match if date.month() == 12 {
                Utc.with_ymd_and_hms(date.year() + 1, 1, 1, 0, 0, 0)
            } else {
                Utc.with_ymd_and_hms(date.year(), date.month() + 1, 1, 0, 0, 0)
            } {
                LocalResult::Single(dt) => dt,
                _ => return Err(TimeError::InvalidDate),
            };

            return Ok(InsightRange {
                start: start,
                end: end,
            });
        }

        // Handle "yyyy-W##"
        if let Some((year, week_str)) = value.split_once("-W") {
            let year = year.parse::<i32>().map_err(|_| TimeError::InvalidDate)?;
            let week = week_str
                .parse::<u32>()
                .map_err(|_| TimeError::InvalidDate)?;

            let iso_week_start = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon)
                .ok_or(TimeError::InvalidDate)?;

            let start = match Utc.with_ymd_and_hms(
                iso_week_start.year(),
                iso_week_start.month(),
                iso_week_start.day(),
                0,
                0,
                0,
            ) {
                LocalResult::Single(dt) => dt,
                _ => return Err(TimeError::InvalidDate),
            };
            let end = start + Duration::days(7);

            return Ok(InsightRange {
                start: start,
                end: end,
            });
        }

        // Handle "yyyy"
        if let Ok(year) = value.parse::<i32>() {
            let start = match Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0) {
                LocalResult::Single(dt) => dt,
                _ => return Err(TimeError::InvalidDate),
            };
            let end = match Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0) {
                LocalResult::Single(dt) => dt,
                _ => return Err(TimeError::InvalidDate),
            };
            return Ok(InsightRange {
                start: start,
                end: end,
            });
        }

        Err(TimeError::InvalidDate)
    }
}
