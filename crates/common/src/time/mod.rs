pub mod insight;

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    /// The last N full minutes
    LastNMinutes(i64),
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
            TimeRangePreset::LastNMinutes(n) => {
                let end = Utc::now();
                let start = end - Duration::minutes(n);
                Self {
                    start,
                    end,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: local midnight (00:00:00) as Utc
    fn today_start_utc() -> DateTime<Utc> {
        let local_now = Local::now();
        let d = local_now.date_naive();
        let local_midnight = Local
            .with_ymd_and_hms(d.year(), d.month(), d.day(), 0, 0, 0)
            .single()
            .unwrap();
        local_midnight.with_timezone(&Utc)
    }

    /// Helper: start of current week (Monday 00:00:00 local) as Utc
    fn this_week_start_utc() -> DateTime<Utc> {
        let local_now = Local::now();
        let d = local_now.date_naive();
        let local_midnight = Local
            .with_ymd_and_hms(d.year(), d.month(), d.day(), 0, 0, 0)
            .single()
            .unwrap();
        let weekday = d.weekday().num_days_from_monday() as i64;
        (local_midnight - Duration::days(weekday)).with_timezone(&Utc)
    }

    /// Helper: first day (00:00:00 local) of (year, month) as Utc
    fn month_start_utc(year: i32, month: u32) -> DateTime<Utc> {
        Local
            .with_ymd_and_hms(year, month, 1, 0, 0, 0)
            .single()
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn preset_today() {
        let r = TimeRange::from(TimeRangePreset::Today);
        let start = today_start_utc();
        assert_eq!(r.start, start);
        assert_eq!(r.end, start + Duration::days(1));
    }

    #[test]
    fn preset_yesterday() {
        let r = TimeRange::from(TimeRangePreset::Yesterday);
        let today = today_start_utc();
        assert_eq!(r.start, today - Duration::days(1));
        assert_eq!(r.end, today);
    }

    #[test]
    fn preset_this_week() {
        let r = TimeRange::from(TimeRangePreset::ThisWeek);
        let start = this_week_start_utc();
        assert_eq!(r.start, start);
        assert_eq!(r.end, start + Duration::days(7));
    }

    #[test]
    fn preset_last_week() {
        let r = TimeRange::from(TimeRangePreset::LastWeek);
        let this_start = this_week_start_utc();
        assert_eq!(r.start, this_start - Duration::days(7));
        assert_eq!(r.end, this_start);
    }

    #[test]
    fn preset_this_month() {
        let local_now = Local::now().date_naive();
        let start = month_start_utc(local_now.year(), local_now.month());
        let end = if local_now.month() == 12 {
            month_start_utc(local_now.year() + 1, 1)
        } else {
            month_start_utc(local_now.year(), local_now.month() + 1)
        };
        let r = TimeRange::from(TimeRangePreset::ThisMonth);
        assert_eq!(r.start, start);
        assert_eq!(r.end, end);
    }

    #[test]
    fn preset_last_month() {
        let local_now = Local::now().date_naive();
        let (y, m) = if local_now.month() == 1 {
            (local_now.year() - 1, 12)
        } else {
            (local_now.year(), local_now.month() - 1)
        };
        let start = month_start_utc(y, m);
        let end = month_start_utc(local_now.year(), local_now.month());
        let r = TimeRange::from(TimeRangePreset::LastMonth);
        assert_eq!(r.start, start);
        assert_eq!(r.end, end);
    }

    #[test]
    fn preset_last_n_minutes_has_correct_span() {
        let before = Utc::now();
        let r = TimeRange::from(TimeRangePreset::LastNMinutes(15));
        let after = Utc::now();

        let span = r.end - r.start;
        assert_eq!(span, Duration::minutes(15));
        assert!(r.end >= before && r.end <= after);
    }

    #[test]
    fn preset_last_n_days_excluding_today() {
        let today = today_start_utc();
        let r = TimeRange::from(TimeRangePreset::LastNDays(3, false));
        assert_eq!(r.start, today - Duration::days(2));
        assert_eq!(r.end, today);
    }

    #[test]
    fn preset_last_n_days_including_today() {
        let today = today_start_utc();
        let r = TimeRange::from(TimeRangePreset::LastNDays(3, true));
        assert_eq!(r.start, today - Duration::days(3));
        assert_eq!(r.end, today + Duration::days(1));
    }

    #[test]
    fn preset_last_n_weeks_excluding_this_week() {
        let this_start = this_week_start_utc();
        let r = TimeRange::from(TimeRangePreset::LastNWeeks(2, false));
        assert_eq!(r.end, this_start);
        assert_eq!(
            r.start,
            (this_start + Duration::days(7)) - Duration::weeks(2)
        );
    }

    #[test]
    fn preset_last_n_weeks_including_this_week() {
        let this_start = this_week_start_utc();
        let r = TimeRange::from(TimeRangePreset::LastNWeeks(2, true));
        assert_eq!(r.end, this_start + Duration::days(7));
        assert_eq!(
            r.start,
            (this_start + Duration::days(7)) - Duration::weeks(2)
        );
    }

    #[test]
    fn preset_last_n_months_excluding_this_month() {
        let local_today = Local::now().date_naive();
        let n = 3i64;
        let include_this = false;

        let mut year = local_today.year();
        let mut month = local_today.month();

        if !include_this {
            month -= 1;
            if month <= 0 {
                month += 12;
                year -= 1;
            }
        }

        let mut start_month = month - n as u32 + 1;
        let mut start_year = year;
        while start_month <= 0 {
            start_month += 12;
            start_year -= 1;
        }

        let expected_start = month_start_utc(start_year, start_month);
        let expected_end = Local
            .with_ymd_and_hms(year, (month as u32) + 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| {
                Local
                    .with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0)
                    .single()
                    .unwrap()
            })
            .with_timezone(&Utc);
        let r = TimeRange::from(TimeRangePreset::LastNMonths(n, include_this));
        assert_eq!(r.start, expected_start);
        assert_eq!(r.end, expected_end);
    }

    #[test]
    fn preset_last_n_years_excluding_this_year() {
        let local_today = Local::now().date_naive();
        let end_year = local_today.year();
        let start_year = end_year - 2;
        let start = month_start_utc(start_year, 1);
        let end = month_start_utc(end_year, 1);

        let r = TimeRange::from(TimeRangePreset::LastNYears(2, false));
        assert_eq!(r.start, start);
        assert_eq!(r.end, end);
    }

    #[test]
    fn preset_last_n_years_including_this_year() {
        let local_today = Local::now().date_naive();
        let end_year = local_today.year() + 1;
        let start_year = end_year - 2;
        let start = month_start_utc(start_year, 1);
        let end = month_start_utc(end_year, 1);

        let r = TimeRange::from(TimeRangePreset::LastNYears(2, true));
        assert_eq!(r.start, start);
        assert_eq!(r.end, end);
    }

    #[test]
    fn preset_custom_passthrough() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        let r = TimeRange::from(TimeRangePreset::Custom {
            start,
            end,
            bucket: TimeBucket::Hour,
        });

        assert_eq!(r.start, start);
        assert_eq!(r.end, end);
    }
}
