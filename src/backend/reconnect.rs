//! Retry timing shared by backend transports.

use std::time::Duration;

/// Retry only failed initialization/transports; healthy sessions use native events.
pub struct ReconnectBackoff {
    delay: Duration,
}

impl Default for ReconnectBackoff {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(250),
        }
    }
}

impl ReconnectBackoff {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn next_delay(&mut self) -> Duration {
        let delay = self.delay;

        self.delay = (delay * 2).min(Duration::from_secs(10));

        delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retries_are_bounded_and_reset_after_recovery() {
        let mut retry = ReconnectBackoff::default();

        assert_eq!(retry.next_delay(), Duration::from_millis(250));

        for _ in 0..10 {
            retry.next_delay();
        }

        assert_eq!(retry.next_delay(), Duration::from_secs(10));

        retry.reset();

        assert_eq!(retry.next_delay(), Duration::from_millis(250));
    }
}
