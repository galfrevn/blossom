use std::time::Duration;

#[derive(Clone, Copy)]
pub struct Pacing {
    first_pause: Duration,
    longest_pause: Duration,
}

impl Pacing {
    pub fn new(first_pause: Duration, longest_pause: Duration) -> Pacing {
        Pacing {
            first_pause,
            longest_pause,
        }
    }

    pub fn for_the_app() -> Pacing {
        Pacing::new(Duration::from_secs(1), Duration::from_secs(30))
    }

    pub(crate) fn first_pause(&self) -> Duration {
        self.first_pause
    }

    pub(crate) fn pause_after(&self, pause: Duration) -> Duration {
        (pause * 2).min(self.longest_pause)
    }
}
