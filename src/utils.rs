use std::error::Error;
pub type MyResult<T> = Result<T, Box<dyn Error>>;

pub const SECONDS_IN_MINUTES: u64 = 60;
pub const MINUTES_IN_HOURS: u64 = 60;
pub const SECONDS_IN_HOURS: u64 = SECONDS_IN_MINUTES * MINUTES_IN_HOURS;

pub fn format_secs(mut secs: u64) -> [u64; 3] {
    let hours = secs / SECONDS_IN_HOURS;
    secs = secs - (hours * SECONDS_IN_HOURS);
    let minutes = secs / SECONDS_IN_MINUTES;
    let seconds = secs % SECONDS_IN_MINUTES;

    return [hours, minutes, seconds];
}

pub fn to_secs(hours: u64, minutes: u64, secs: u64) -> u64 {
    let hours_in_seconds = hours * SECONDS_IN_HOURS;
    let minutes_in_seconds = minutes * SECONDS_IN_MINUTES;

    hours_in_seconds + minutes_in_seconds + secs
}

pub fn get_percentage(value: u64, total_value: u64) -> u16 {
    ((value as f32 / total_value as f32) * 100.0) as u16
}
