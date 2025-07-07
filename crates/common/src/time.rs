use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};
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
        let local_now = Local::now();
        let local_today = local_now.date_naive();

        let today_start_local = Local
            .with_ymd_and_hms(
                local_today.year(),
                local_today.month(),
                local_today.day(),
                0,
                0,
                0,
            )
            .single()
            .unwrap();
        let today_start_utc = today_start_local.with_timezone(&Utc);

        match preset {
            TimeRangePreset::Today => {
                let start = today_start_utc;
                let end = start + Duration::days(1);
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Hour),
                }
            }
            TimeRangePreset::Yesterday => {
                let start = today_start_utc - Duration::days(1);
                let end = today_start_utc;
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Hour),
                }
            }
            TimeRangePreset::ThisWeek => {
                let weekday = local_today.weekday().num_days_from_monday() as i64;
                let week_start_local = today_start_local - Duration::days(weekday);
                let week_start_utc = week_start_local.with_timezone(&Utc);
                let end = week_start_utc + Duration::days(7);
                Self {
                    start: week_start_utc,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::LastWeek => {
                let weekday = local_today.weekday().num_days_from_monday() as i64;
                let last_week_start_local = today_start_local - Duration::days(weekday + 7);
                let start = last_week_start_local.with_timezone(&Utc);
                let end = start + Duration::days(7);
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::ThisMonth => {
                let start_local = Local
                    .with_ymd_and_hms(local_today.year(), local_today.month(), 1, 0, 0, 0)
                    .single()
                    .unwrap();
                let end_local = if local_today.month() == 12 {
                    Local
                        .with_ymd_and_hms(local_today.year() + 1, 1, 1, 0, 0, 0)
                        .single()
                        .unwrap()
                } else {
                    Local
                        .with_ymd_and_hms(local_today.year(), local_today.month() + 1, 1, 0, 0, 0)
                        .single()
                        .unwrap()
                };
                Self {
                    start: start_local.with_timezone(&Utc),
                    end: end_local.with_timezone(&Utc),
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::LastMonth => {
                let (year, month) = if local_today.month() == 1 {
                    (local_today.year() - 1, 12)
                } else {
                    (local_today.year(), local_today.month() - 1)
                };
                let start_local = Local
                    .with_ymd_and_hms(year, month, 1, 0, 0, 0)
                    .single()
                    .unwrap();
                let end_local = Local
                    .with_ymd_and_hms(local_today.year(), local_today.month(), 1, 0, 0, 0)
                    .single()
                    .unwrap();
                Self {
                    start: start_local.with_timezone(&Utc),
                    end: end_local.with_timezone(&Utc),
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::LastNDays(n) => {
                let clamped = n.clamp(1, 31);
                let start = today_start_utc - Duration::days(clamped);
                let end = today_start_utc;
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::LastNWeeks(n) => {
                let clamped = n.clamp(1, 9);
                let start = today_start_utc - Duration::weeks(clamped);
                let end = today_start_utc;
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Week),
                }
            }
            TimeRangePreset::LastNMonths(n) => {
                let clamped = n.clamp(1, 12);
                let mut start_year = local_today.year();
                let mut start_month = local_today.month() as i32 - clamped as i32;
                while start_month <= 0 {
                    start_month += 12;
                    start_year -= 1;
                }

                let start_local = Local
                    .with_ymd_and_hms(start_year, start_month as u32, 1, 0, 0, 0)
                    .single()
                    .unwrap();
                let end_local = Local
                    .with_ymd_and_hms(local_today.year(), local_today.month(), 1, 0, 0, 0)
                    .single()
                    .unwrap();

                Self {
                    start: start_local.with_timezone(&Utc),
                    end: end_local.with_timezone(&Utc),
                    bucket: Some(TimeBucket::Month),
                }
            }
            TimeRangePreset::LastNYears(n) => {
                let clamped = n.clamp(1, 12);
                let start_local = Local
                    .with_ymd_and_hms(local_today.year() - clamped as i32, 1, 1, 0, 0, 0)
                    .single()
                    .unwrap();
                let end_local = Local
                    .with_ymd_and_hms(local_today.year(), 1, 1, 0, 0, 0)
                    .single()
                    .unwrap();

                Self {
                    start: start_local.with_timezone(&Utc),
                    end: end_local.with_timezone(&Utc),
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
