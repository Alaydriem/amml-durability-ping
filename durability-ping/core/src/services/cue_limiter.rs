use std::time::Duration;

pub struct CueLimiter {
    last: Option<Duration>,
}

impl CueLimiter {
    pub const GAP: Duration = Duration::from_secs(1);

    pub fn new() -> Self {
        Self { last: None }
    }

    pub fn allow(&mut self, now: Duration) -> bool {
        match self.last {
            Some(last) if now.saturating_sub(last) < Self::GAP => false,
            _ => {
                self.last = Some(now);
                true
            }
        }
    }
}

impl Default for CueLimiter {
    fn default() -> Self {
        Self::new()
    }
}
