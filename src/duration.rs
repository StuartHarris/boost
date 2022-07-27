use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let duration = duration.as_secs_f64();
    if duration > 60.0 {
        format!("{}m {:.2}s", duration as u32 / 60, duration % 60.0)
    } else {
        format!("{:.2}s", duration)
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
        assert_eq!(
            format_duration(Duration::from_millis(2021300)),
            "33m 41.30s"
        );
    }
}
