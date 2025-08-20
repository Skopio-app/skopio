pub fn format_duration(seconds: u64) -> String {
    let total = (seconds as f64).floor() as u64;
    let hrs = total / 3600;
    let mins = (total % 3600) / 60;
    let secs = total % 60;

    let hrs_str = format!("{:01}h", hrs as i64);
    let min_str = format!("{:02}m", mins as i64);
    let sec_str = format!("{:02}s", secs as i64);

    if hrs > 0 {
        return format!("{hrs_str} {min_str} {sec_str}");
    } else if mins > 0 {
        return format!("{min_str} {sec_str}");
    } else {
        sec_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_seconds_only() {
        assert_eq!(format_duration(0), "00s");
        assert_eq!(format_duration(5), "05s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn formats_minutes_and_seconds() {
        assert_eq!(format_duration(60), "01m 00s");
        assert_eq!(format_duration(61), "01m 01s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn formats_hours_minutes_seconds() {
        assert_eq!(format_duration(3600), "1h 00m 00s");
        assert_eq!(format_duration(3661), "1h 01m 01s");
        assert_eq!(format_duration(86399), "23h 59m 59s");
    }

    #[test]
    fn formats_large_hours() {
        let secs = 100 * 3600 + 5;
        assert_eq!(format_duration(secs), "100h 00m 05s");
    }
}
