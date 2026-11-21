use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;
const FIRST_YEAR: u64 = 1970;
const MONTHS_OF_A_COMMON_YEAR: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTHS_OF_A_LEAP_YEAR: [u64; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub fn board_time(moment: SystemTime) -> String {
    let since_the_epoch = moment.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = since_the_epoch.as_secs();
    let millisecond = since_the_epoch.subsec_millis();
    let second_of_the_day = seconds % SECONDS_PER_DAY;
    let hour = second_of_the_day / SECONDS_PER_HOUR;
    let minute = second_of_the_day % SECONDS_PER_HOUR / SECONDS_PER_MINUTE;
    let second = second_of_the_day % SECONDS_PER_MINUTE;

    let mut days_left = seconds / SECONDS_PER_DAY;
    let mut year = FIRST_YEAR;
    while days_left >= days_in_year(year) {
        days_left -= days_in_year(year);
        year += 1;
    }
    let mut month = 1;
    for days_in_the_month in months_of(year) {
        if days_left < days_in_the_month {
            break;
        }
        days_left -= days_in_the_month;
        month += 1;
    }
    let day = days_left + 1;

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millisecond:03}Z")
}

fn is_leap_year(year: u64) -> bool {
    let divisible_by = |divisor: u64| year.is_multiple_of(divisor);
    divisible_by(4) && (!divisible_by(100) || divisible_by(400))
}

fn months_of(year: u64) -> [u64; 12] {
    if is_leap_year(year) {
        return MONTHS_OF_A_LEAP_YEAR;
    }
    MONTHS_OF_A_COMMON_YEAR
}

fn days_in_year(year: u64) -> u64 {
    months_of(year).iter().sum()
}
