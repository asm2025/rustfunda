pub mod unix;

use chrono::{Local, TimeZone};
use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    format_seconds_long(duration.as_micros())
}

pub fn format_seconds(time: u64) -> String {
    Local
        .timestamp_opt(time as i64, 0)
        .single()
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "invalid time".to_string())
}

pub fn format_seconds_long(time: u128) -> String {
    let secs = (time / 1_000_000) as i64;
    let micros_in_sec = (time % 1_000_000) as u32;
    let nanos = micros_in_sec * 1_000;
    Local
        .timestamp_opt(secs, nanos)
        .single()
        .map(|dt| dt.format("%H:%M:%S%.6f").to_string())
        .unwrap_or_else(|| "invalid time".to_string())
}
