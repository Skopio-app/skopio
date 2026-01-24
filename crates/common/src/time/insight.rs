use chrono::{
    offset::LocalResult, DateTime, Datelike, Duration, Local, NaiveDate, TimeZone, Utc, Weekday,
};
use serde::{Deserialize, Serialize};

use crate::time::TimeError;

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct InsightRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

fn local_midnight_to_utc(date: NaiveDate) -> Result<DateTime<Utc>, TimeError> {
    let local_dt = match Local.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(dt1, _dt2) => dt1,
        LocalResult::None => return Err(TimeError::InvalidDate),
    };
    Ok(local_dt.with_timezone(&Utc))
}

impl TryFrom<String> for InsightRange {
    type Error = TimeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Handle "yyyy-mm-dd"
        if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
            let start = local_midnight_to_utc(date)?;
            let end = start + Duration::days(1);
            return Ok(InsightRange { start, end });
        }

        // Handle "yyyy-mm"
        if let Ok(first_of_month) = NaiveDate::parse_from_str(&format!("{}-01", value), "%Y-%m-%d")
        {
            let start = local_midnight_to_utc(first_of_month)?;
            let next_month = if first_of_month.month() == 12 {
                NaiveDate::from_ymd_opt(first_of_month.year() + 1, 1, 1)
                    .ok_or(TimeError::InvalidDate)?
            } else {
                NaiveDate::from_ymd_opt(first_of_month.year(), first_of_month.month() + 1, 1)
                    .ok_or(TimeError::InvalidDate)?
            };

            let end = local_midnight_to_utc(next_month)?;
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

            let start = local_midnight_to_utc(iso_week_start)?;
            let end = start + Duration::days(7);

            return Ok(InsightRange { start, end });
        }

        // Handle "yyyy"
        if let Ok(year) = value.parse::<i32>() {
            let start_date = NaiveDate::from_ymd_opt(year, 1, 1).ok_or(TimeError::InvalidDate)?;
            let end_date = NaiveDate::from_ymd_opt(year + 1, 1, 1).ok_or(TimeError::InvalidDate)?;

            let start = local_midnight_to_utc(start_date)?;
            let end = local_midnight_to_utc(end_date)?;
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
