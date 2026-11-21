use std::time::Duration;

use crate::access::BoardAccess;
use crate::protocol;

#[derive(Clone, Copy)]
pub struct Pacing {
    pub first_pause: Duration,
    pub longest_pause: Duration,
    pub failures_before_asking_the_board: u32,
    pub longest_wait_for_a_stream: Duration,
    pub longest_silence: Duration,
}

impl Pacing {
    pub fn for_the_app() -> Pacing {
        Pacing {
            first_pause: Duration::from_secs(1),
            longest_pause: Duration::from_secs(30),
            failures_before_asking_the_board: 3,
            longest_wait_for_a_stream: Duration::from_secs(10),
            longest_silence: Duration::from_secs(45),
        }
    }
}

pub(crate) struct BoardGone;

pub(crate) struct Retries<'access> {
    pacing: Pacing,
    access: &'access BoardAccess,
    pause: Duration,
    failures_in_a_row: u32,
}

impl<'access> Retries<'access> {
    pub(crate) fn new(pacing: Pacing, access: &'access BoardAccess) -> Retries<'access> {
        Retries {
            pacing,
            access,
            pause: pacing.first_pause,
            failures_in_a_row: 0,
        }
    }

    pub(crate) async fn wait_after_failure(&mut self) -> Result<(), BoardGone> {
        self.failures_in_a_row = self.failures_in_a_row.saturating_add(1);
        let time_to_ask = self.failures_in_a_row >= self.pacing.failures_before_asking_the_board;
        if time_to_ask && !protocol::board_still_answers(self.access).await {
            return Err(BoardGone);
        }
        tokio::time::sleep(self.pause).await;
        self.pause = (self.pause * 2).min(self.pacing.longest_pause);
        Ok(())
    }

    pub(crate) fn succeeded(&mut self) {
        self.failures_in_a_row = 0;
        self.pause = self.pacing.first_pause;
    }
}
