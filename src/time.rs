fn minutes_elapsed(ticks: u64) -> u64 {
    (ticks % TICKS_PER_DAY) / TICKS_PER_MIN
}

pub fn ticks_hm(ticks: u64) -> (u64, u64) {
    let min = minutes_elapsed(ticks);
    let h = min / 60;
    let m = min % 60;
    (h, m)
}

pub const MINS_PER_DAY: u64 = 1440;
pub const TICKS_PER_MIN: u64 = 60;
pub const TICKS_PER_DAY: u64 = MINS_PER_DAY * TICKS_PER_MIN;
pub const HOUR_IN_TICKS: u64 = TICKS_PER_MIN * 60;
