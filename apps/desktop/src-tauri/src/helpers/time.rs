pub fn format_duration(seconds: u64) -> String {
    let total = (seconds as f64).floor() as u64;
    let hrs = total / 3600;
    let mins = (total % 3600) / 60;
    let secs = total % 60;

    let hrs_str = format!("{:02}h", hrs as i64);
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
