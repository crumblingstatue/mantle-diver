fn minutes_elapsed(ticks: u64) -> u64 {
    (ticks % TICKS_PER_DAY) / TICKS_PER_MIN
}

pub fn ticks_hm(ticks: u64) -> (u64, u64) {
    let min = minutes_elapsed(ticks);
    let h = min / 60;
    let m = min % 60;
    (h, m)
}

pub fn tick_of_day(ticks: u64) -> u64 {
    ticks % TICKS_PER_DAY
}

pub const MINS_PER_DAY: u64 = 1440;
pub const TICKS_PER_MIN: u64 = 60;
pub const TICKS_PER_DAY: u64 = MINS_PER_DAY * TICKS_PER_MIN;
pub const HOUR_IN_TICKS: u64 = TICKS_PER_MIN * 60;
pub const NOON: u64 = TICKS_PER_DAY / 2;

/// Returns the daylight (between 0 and 255) for the current time of day
/// `time_of_day` should be between 0 and `TICKS_PER_DAY`.
///
/// Maximum brightness is at `NOON`, and it decreases the further away `time_of_day` is from `NOON`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "Return value should be between 0 and 255"
)]
pub fn daylight(time_of_day: u64) -> u8 {
    let distance_from_noon = if time_of_day > NOON {
        time_of_day - NOON
    } else {
        NOON - time_of_day
    };
    let brightness = 255 - (distance_from_noon * 255 / NOON);
    brightness as u8
}
