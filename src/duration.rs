use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs_f64();
    if secs > 60.0 * 60.0 {
        format!("{}h {:.0}m", secs as u32 / (60 * 60), secs / 60.0 % 60.0)
    } else if secs > 60.0 {
        format!("{}m {:.0}s", secs as u32 / 60, secs % 60.0)
    } else {
        format!("{:.2}s", secs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn format_seconds() {
        assert_eq!(format_duration(Duration::from_millis(2022)), "2.02s");
    }

    #[test]
    fn format_minutes() {
        assert_eq!(format_duration(Duration::from_millis(2021300)), "33m 41s");
    }

    #[test]
    fn format_hours() {
        assert_eq!(
            format_duration(Duration::from_secs((2 * 60 * 60) + (40 * 60) + 21)),
            "2h 40m"
        );
    }
}
