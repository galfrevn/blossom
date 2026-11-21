use std::time::{Duration, SystemTime, UNIX_EPOCH};

use blossom::time::board_time;

fn milliseconds_after_the_epoch(milliseconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(milliseconds)
}

#[test]
fn the_epoch_reads_as_its_first_millisecond() {
    assert_eq!(board_time(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
}

#[test]
fn a_moment_reads_as_the_board_writes_its_times() {
    assert_eq!(
        board_time(milliseconds_after_the_epoch(1_795_255_200_123)),
        "2026-11-21T10:00:00.123Z"
    );
}

#[test]
fn the_last_moment_of_a_year_stays_in_that_year() {
    assert_eq!(
        board_time(milliseconds_after_the_epoch(1_798_761_599_000)),
        "2026-12-31T23:59:59.000Z"
    );
}

#[test]
fn a_leap_day_is_a_day_of_its_own() {
    assert_eq!(
        board_time(milliseconds_after_the_epoch(1_835_481_599_999)),
        "2028-02-29T23:59:59.999Z"
    );
}

#[test]
fn a_century_divisible_by_four_hundred_is_a_leap_year() {
    assert_eq!(
        board_time(milliseconds_after_the_epoch(951_868_800_000)),
        "2000-03-01T00:00:00.000Z"
    );
}

#[test]
fn any_other_century_is_not_a_leap_year() {
    assert_eq!(
        board_time(milliseconds_after_the_epoch(4_107_542_400_000)),
        "2100-03-01T00:00:00.000Z"
    );
}

#[test]
fn a_moment_before_the_epoch_reads_as_the_epoch() {
    let before_the_epoch = UNIX_EPOCH - Duration::from_secs(1);

    assert_eq!(board_time(before_the_epoch), "1970-01-01T00:00:00.000Z");
}
