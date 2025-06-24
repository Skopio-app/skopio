use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum TimeBucket {
    Day,
    Week,
    Month,
    Hour,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub enum TimeRangePreset {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    LastNDays(i64),
    LastNWeeks(i64),
    Custom {
        start: NaiveDateTime,
        end: NaiveDateTime,
        bucket: TimeBucket,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct TimeRange {
    start: NaiveDateTime,
    end: NaiveDateTime,
    bucket: TimeBucket,
}

impl From<TimeRangePreset> for TimeRange {
    fn from(preset: TimeRangePreset) -> Self {
        let now = Local::now().naive_local().date();

        match preset {
            TimeRangePreset::Today => {
                let start = now.and_hms_opt(0, 0, 0).unwrap();
                let end = start + Duration::days(1);
                Self {
                    start,
                    end,
                    bucket: TimeBucket::Hour,
                }
            }
            TimeRangePreset::Yesterday => {
                let start = (now - Duration::days(1)).and_hms_opt(0, 0, 0).unwrap();
                let end = start + Duration::days(1);
                Self {
                    start,
                    end,
                    bucket: TimeBucket::Hour,
                }
            }
            TimeRangePreset::ThisWeek => {
                let start = now - Duration::days(now.weekday().num_days_from_monday() as i64);
                let end = start + Duration::days(7);
                Self {
                    start: start.and_hms_opt(0, 0, 0).unwrap(),
                    end: end.and_hms_opt(0, 0, 0).unwrap(),
                    bucket: TimeBucket::Day,
                }
            }
            TimeRangePreset::LastWeek => {
                let start = now - Duration::days(now.weekday().num_days_from_monday() as i64 + 7);
                let end = start + Duration::days(7);
                Self {
                    start: start.and_hms_opt(0, 0, 0).unwrap(),
                    end: end.and_hms_opt(0, 0, 0).unwrap(),
                    bucket: TimeBucket::Day,
                }
            }
            TimeRangePreset::ThisMonth => {
                let start = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
                let end = if now.month() == 12 {
                    NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
                };
                Self {
                    start: start.and_hms_opt(0, 0, 0).unwrap(),
                    end: end.and_hms_opt(0, 0, 0).unwrap(),
                    bucket: TimeBucket::Day,
                }
            }
            TimeRangePreset::LastMonth => {
                let (year, month) = if now.month() == 1 {
                    (now.year() - 1, 12)
                } else {
                    (now.year(), now.month() - 1)
                };
                let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                let end = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
                Self {
                    start: start.and_hms_opt(0, 0, 0).unwrap(),
                    end: end.and_hms_opt(0, 0, 0).unwrap(),
                    bucket: TimeBucket::Day,
                }
            }
            TimeRangePreset::LastNDays(n) => {
                let clamped = n.clamp(1, 31);
                let start = now - Duration::days(clamped);
                let end = now;
                Self {
                    start: start.and_hms_opt(0, 0, 0).unwrap(),
                    end: end.and_hms_opt(0, 0, 0).unwrap(),
                    bucket: TimeBucket::Day,
                }
            }
            TimeRangePreset::LastNWeeks(n) => {
                let clamped = n.clamp(1, 4);
                let start = now - Duration::weeks(clamped);
                let end = now;
                Self {
                    start: start.and_hms_opt(0, 0, 0).unwrap(),
                    end: end.and_hms_opt(0, 0, 0).unwrap(),
                    bucket: TimeBucket::Week,
                }
            }
            TimeRangePreset::Custom { start, end, bucket } => Self { start, end, bucket },
        }
    }
}

impl TimeRange {
    pub fn start(&self) -> NaiveDateTime {
        self.start
    }
    pub fn end(&self) -> NaiveDateTime {
        self.end
    }
    pub fn bucket(&self) -> TimeBucket {
        self.bucket
    }
    pub fn as_tuple(&self) -> (NaiveDateTime, NaiveDateTime, TimeBucket) {
        (self.start, self.end, self.bucket)
    }
}
