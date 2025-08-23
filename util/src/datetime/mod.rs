pub mod unix;

use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    format_seconds(duration.as_secs())
}

pub fn format_seconds(time: u64) -> String {
    let hours = time / 3600;
    let minutes = (time % 3600) / 60;
    let seconds = time % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn format_duration_with_precision(duration: Duration) -> String {
    let time = duration.as_secs() as f64 + duration.subsec_micros() as f64 / 1_000_000.0;
    format_seconds_with_precision(time)
}

pub fn format_seconds_with_precision(time: f64) -> String {
    let seconds = time as u64;
    let microseconds = ((time.fract() * 1_000_000.0) as u32) / 10;

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    format!(
        "{:02}:{:02}:{:02}.{:05}",
        hours,
        minutes,
        seconds,
        microseconds / 10
    )
}
