use lazy_static::lazy_static;
use regex::Regex;
use std::time::Duration;

pub enum Lowest {
    Seconds,
    MilliSeconds,
}

pub fn format_duration(duration: Duration, lowest: Lowest) -> String {
    let duration = humantime::format_duration(duration).to_string();
    truncate(duration, lowest)
}

fn truncate(s: String, lowest: Lowest) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"^(?P<start>.*?)?(?P<milli>\s*\d+ms)?(?P<micro>\s*\d+us)?(?P<nano>\s*\d+ns)?$"
        )
        .expect("compiling regex");
    };
    let s = match lowest {
        Lowest::Seconds => RE.replace(&s, "$start"),
        Lowest::MilliSeconds => RE.replace(&s, "$start$milli"),
    };
    s.to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lowest_seconds() {
        assert_eq!(
            truncate("2h 12m 123s 432ms 12us 1ns".to_string(), Lowest::Seconds),
            "2h 12m 123s"
        );
        assert_eq!(truncate("2h 12m".to_string(), Lowest::Seconds), "2h 12m");
        assert_eq!(
            truncate("2h 12m 123s 432ms 12us".to_string(), Lowest::Seconds),
            "2h 12m 123s"
        );
    }

    #[test]
    fn lowest_milliseconds() {
        assert_eq!(
            truncate(
                "2h 12m 123s 432ms 12us 1ns".to_string(),
                Lowest::MilliSeconds
            ),
            "2h 12m 123s 432ms"
        );
        assert_eq!(
            truncate("2h 12m".to_string(), Lowest::MilliSeconds),
            "2h 12m"
        );
        assert_eq!(
            truncate("2h 12m 123s 432ms 12us".to_string(), Lowest::MilliSeconds),
            "2h 12m 123s 432ms"
        );
    }
}
