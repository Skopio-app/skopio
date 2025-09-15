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
            return Ok(InsightRange { start, end });
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

            return Ok(InsightRange { start, end });
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

            return Ok(InsightRange { start, end });
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
            return Ok(InsightRange { start, end });
        }

        Err(TimeError::InvalidDate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_day_ok() {
        let r = InsightRange::try_from("2025-03-15".to_string()).unwrap();
        assert_eq!(r.start, Utc.with_ymd_and_hms(2025, 3, 15, 0, 0, 0).unwrap());
        assert_eq!(r.end, Utc.with_ymd_and_hms(2025, 3, 16, 0, 0, 0).unwrap());
    }

    #[test]
    fn parse_month_ok() {
        let r = InsightRange::try_from("2025-02".to_string()).unwrap();
        assert_eq!(r.start, Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap());
        assert_eq!(r.end, Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn parse_year_ok() {
        let r = InsightRange::try_from("2025".to_string()).unwrap();
        assert_eq!(r.start, Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap());
        assert_eq!(r.end, Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn parse_month_december_rollover_ok() {
        let r = InsightRange::try_from("2024-12".to_string()).unwrap();
        assert_eq!(r.start, Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap());
        assert_eq!(r.end, Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn parse_iso_week_ok() {
        // ISO week 2020-W01 starts on 2019-12-30 (Monday), ends 7 days later.
        let r = InsightRange::try_from("2020-W01".to_string()).unwrap();
        assert_eq!(
            r.start,
            Utc.with_ymd_and_hms(2019, 12, 30, 0, 0, 0).unwrap()
        );
        assert_eq!(r.end, Utc.with_ymd_and_hms(2020, 1, 6, 0, 0, 0).unwrap());
    }

    #[test]
    fn parse_iso_week_within_year() {
        let r = InsightRange::try_from("2023-W25".to_string()).unwrap();
        assert!(r.end - r.start == Duration::days(7));
        assert!(r.start < r.end);
    }

    #[test]
    fn reject_bad_day_format() {
        let err = InsightRange::try_from("2025/03/15".to_string()).unwrap_err();
        matches!(err, TimeError::InvalidDate);
    }

    #[test]
    fn reject_nonexistent_day() {
        let err = InsightRange::try_from("2025-02-30".to_string()).unwrap_err();
        matches!(err, TimeError::InvalidDate);
    }

    #[test]
    fn reject_out_of_range_week() {
        let err = InsightRange::try_from("2025-W60".to_string()).unwrap_err();
        matches!(err, TimeError::InvalidDate);
    }
}
