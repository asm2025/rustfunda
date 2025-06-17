use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let microseconds = duration.subsec_micros();
    format!(
        "{:02}:{:02}:{:02}.{:05}",
        hours,
        minutes,
        seconds,
        microseconds / 10
    )
}
