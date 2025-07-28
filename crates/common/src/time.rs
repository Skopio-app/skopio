use std::{fmt, str::FromStr};

use chrono::{DateTime, Datelike, Duration, Local, LocalResult, NaiveDate, TimeZone, Utc, Weekday};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::InsightBucket;

#[derive(Debug, Error)]
pub enum TimeError {
    #[error("Invalid UTC datetime")]
    InvalidDate,
}

/// A time granularity used for bucketing data in reports
///
/// - `Hour` groups events by each hour
/// - `Day` groups events by each day
/// - `Week` groups events by each week
/// - `Month` groups events by each month
/// - `Year` groups events by each year.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum TimeBucket {
    /// Bucket data by day
    Day,
    /// Bucket day by week
    Week,
    /// Bucket data by month
    Month,
    /// Bucket data by hour
    Hour,
    /// Bucket data by year
    Year,
}

/// A predefined range of time used to filter or summarize data.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum TimeRangePreset {
    /// Represents today (from midnight to now).
    Today,
    /// Represents yesterday (midnight to midnight)
    Yesterday,
    /// The current calendar week starting from Monday.
    ThisWeek,
    /// The current calendar week before this one.
    LastWeek,
    /// The current calendar month
    ThisMonth,
    /// The previous calendar month
    LastMonth,
    // ThisYear,
    /// The last N full days (excludes today by default).
    LastNDays(i64, bool),
    /// The last N full weeks (excludes this week by default).
    LastNWeeks(i64, bool),
    /// The last N full months (excludes this month by default).
    LastNMonths(i64, bool),
    /// The last N full years (excludes this year by default).
    LastNYears(i64, bool),
    /// A custom range of time with a specific bucket size.
    Custom {
        /// The start date (inclusive).
        start: DateTime<Utc>,
        /// The end date (exclusive).
        end: DateTime<Utc>,
        /// The desired bucket granularity.
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
            TimeRangePreset::LastNDays(n, include_this) => {
                let clamped = n.clamp(1, 31);
                let start =
                    today_start_utc - Duration::days(clamped - if include_this { 0 } else { 1 });
                let end = today_start_utc
                    + if include_this {
                        Duration::days(1)
                    } else {
                        Duration::zero()
                    };
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Day),
                }
            }
            TimeRangePreset::LastNWeeks(n, include_this) => {
                let clamped = n.clamp(1, 9);
                let weekday = local_today.weekday().num_days_from_monday() as i64;
                let this_week_start_local = today_start_local - Duration::days(weekday);
                let end = this_week_start_local.with_timezone(&Utc)
                    + if include_this {
                        Duration::days(7)
                    } else {
                        Duration::zero()
                    };
                let start = end - Duration::weeks(clamped);
                Self {
                    start,
                    end,
                    bucket: Some(TimeBucket::Week),
                }
            }
            TimeRangePreset::LastNMonths(n, include_this) => {
                let clamped = n.clamp(1, 12);
                let mut year = local_today.year();
                let mut month = local_today.month() as i32;

                if !include_this {
                    month -= 1;
                    if month <= 0 {
                        month += 12;
                        year -= 1;
                    }
                }

                let mut start_month = month - clamped as i32 + 1;
                let mut start_year = year;
                while start_month <= 0 {
                    start_month += 12;
                    start_year -= 1;
                }

                let start_local = Local
                    .with_ymd_and_hms(start_year, start_month as u32, 1, 0, 0, 0)
                    .single()
                    .unwrap();
                let end_local = Local
                    .with_ymd_and_hms(year, month as u32 + 1, 1, 0, 0, 0)
                    .single()
                    .unwrap_or_else(|| {
                        Local
                            .with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0)
                            .single()
                            .unwrap()
                    });

                Self {
                    start: start_local.with_timezone(&Utc),
                    end: end_local.with_timezone(&Utc),
                    bucket: Some(TimeBucket::Month),
                }
            }
            TimeRangePreset::LastNYears(n, include_this) => {
                let clamped = n.clamp(1, 12);
                let end_year = local_today.year() + if include_this { 1 } else { 0 };
                let start_year = end_year - clamped as i32;
                let start_local = Local
                    .with_ymd_and_hms(start_year, 1, 1, 0, 0, 0)
                    .single()
                    .unwrap();
                let end_local = Local
                    .with_ymd_and_hms(end_year, 1, 1, 0, 0, 0)
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

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct InsightRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub bucket: Option<InsightBucket>,
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
                bucket: Some(InsightBucket::Day),
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
                bucket: Some(InsightBucket::Month),
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
                bucket: Some(InsightBucket::Week),
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
                bucket: Some(InsightBucket::Day),
            });
        }

        Err(TimeError::InvalidDate)
    }
}

impl fmt::Display for InsightRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Customize to match frontend input format
        match self.bucket {
            Some(InsightBucket::Year) => write!(f, "{}", self.start.year()),
            Some(InsightBucket::Month) => write!(f, "{}", self.start.format("%Y-%m")),
            Some(InsightBucket::Day) => write!(f, "{}", self.start.format("%Y-%m-%d")),
            Some(InsightBucket::Week) => {
                let iso = self.start.date_naive().iso_week();
                write!(f, "{}-W{:02}", iso.year(), iso.week())
            }
            _ => Err(fmt::Error),
        }
    }
}

impl FromStr for InsightRange {
    type Err = TimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InsightRange::try_from(s.to_string())
    }
}
