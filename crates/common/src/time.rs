use chrono::{offset::LocalResult, DateTime, Datelike, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeError {
    #[error("Invalid UTC datetime")]
    InvalidDate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum TimeBucket {
    Day,
    Week,
    Month,
    Hour,
    Year,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub enum TimeRangePreset {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    // ThisYear,
    LastNDays(i64),
    LastNWeeks(i64),
    LastNMonths(i64),
    LastNYears(i64),
    Custom {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        bucket: TimeBucket,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub bucket: Option<TimeBucket>,
}

impl From<TimeRangePreset> for TimeRange {
    fn from(preset: TimeRangePreset) -> Self {
        let now = Utc::now();
        let today =
            safe_utc_datetime(now.year(), now.month(), now.day(), 0, 0, 0).unwrap_or_default();

        match preset {
            TimeRangePreset::Today => {
                let start = today;
                let end = start + Duration::days(1);
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Hour),
                }
            }
            TimeRangePreset::Yesterday => {
                let start = today - Duration::days(1);
                let end = today;
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Hour),
                }
            }
            TimeRangePreset::ThisWeek => {
                let start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
                let end = start + Duration::days(7);
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::LastWeek => {
                let start =
                    today - Duration::days(today.weekday().num_days_from_monday() as i64 + 7);
                let end = start + Duration::days(7);
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::ThisMonth => {
                let start =
                    safe_utc_datetime(now.year(), now.month(), 1, 0, 0, 0).unwrap_or_default();
                let end = if now.month() == 12 {
                    safe_utc_datetime(now.year() + 1, 1, 1, 0, 0, 0).unwrap_or_default()
                } else {
                    safe_utc_datetime(now.year(), now.month() + 1, 1, 0, 0, 0).unwrap_or_default()
                };
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::LastMonth => {
                let (year, month) = if now.month() == 1 {
                    (now.year() - 1, 12)
                } else {
                    (now.year(), now.month() - 1)
                };
                let start = safe_utc_datetime(year, month, 1, 0, 0, 0).unwrap_or_default();
                let end =
                    safe_utc_datetime(now.year(), now.month(), 1, 0, 0, 0).unwrap_or_default();
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            // TimeRangePreset::ThisYear => {

            // }
            TimeRangePreset::LastNDays(n) => {
                let clamped = n.clamp(1, 31);
                let start = today - Duration::days(clamped);
                let end = today;
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::LastNWeeks(n) => {
                let clamped = n.clamp(1, 9);
                let start = today - Duration::weeks(clamped);
                let end = today;
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Week),
                }
            }
            TimeRangePreset::LastNMonths(n) => {
                let clamped = n.clamp(1, 12);
                let naive_today = now.date_naive();
                let mut start_year = naive_today.year();
                let mut start_month = naive_today.month() as i32 - clamped as i32;

                while start_month <= 0 {
                    start_month += 12;
                    start_year -= 1;
                }
                let start = safe_utc_datetime(start_year, start_month as u32, 1, 0, 0, 0)
                    .unwrap_or_default();
                let end =
                    safe_utc_datetime(now.year(), now.month(), 1, 0, 0, 0).unwrap_or_default();
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Month),
                }
            }
            TimeRangePreset::LastNYears(n) => {
                let clamped = n.clamp(1, 12);
                let start = safe_utc_datetime(now.year() - clamped as i32, 1, 1, 0, 0, 0)
                    .unwrap_or_default();
                let end = safe_utc_datetime(now.year(), 1, 1, 0, 0, 0).unwrap_or_default();

                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Year),
                }
            }
            TimeRangePreset::Custom { start, end, bucket } => Self {
                start,
                end,
                bucket: Some(bucket),
            },
        }
    }
}

impl TimeRange {
    pub fn start(&self) -> DateTime<Utc> {
        self.start
    }
    pub fn end(&self) -> DateTime<Utc> {
        self.end
    }
    pub fn bucket(&self) -> Option<TimeBucket> {
        self.bucket
    }
    pub fn as_tuple(&self) -> (DateTime<Utc>, DateTime<Utc>, Option<TimeBucket>) {
        (self.start, self.end, self.bucket)
    }
}

fn safe_utc_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> Result<DateTime<Utc>, TimeError> {
    match Utc.with_ymd_and_hms(year, month, day, hour, min, sec) {
        LocalResult::Single(dt) => Ok(dt),
        _ => Err(TimeError::InvalidDate),
    }
}
