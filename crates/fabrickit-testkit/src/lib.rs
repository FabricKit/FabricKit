//! Deterministic test helpers: simulated network, clock and faults.

use std::cell::Cell;
use std::time::Duration;

use fabrickit_core::Clock;
pub use fabrickit_core::NodeId;

/// Clock whose time moves only when a test calls [`SimClock::advance`].
///
/// Starts at zero. Share it by reference so every component sees the same time.
#[derive(Debug, Default)]
pub struct SimClock {
    now: Cell<Duration>,
}

impl SimClock {
    /// Creates a clock at time zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves time forward by `by`.
    ///
    /// # Panics
    ///
    /// Panics if the new time overflows [`Duration`].
    pub fn advance(&self, by: Duration) {
        let next = self
            .now
            .get()
            .checked_add(by)
            .expect("SimClock time overflowed Duration");
        self.now.set(next);
    }
}

impl Clock for SimClock {
    fn now(&self) -> Duration {
        self.now.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero() {
        assert_eq!(SimClock::new().now(), Duration::ZERO);
    }

    #[test]
    fn advance_moves_time_forward() {
        let clock = SimClock::new();
        clock.advance(Duration::from_millis(250));
        clock.advance(Duration::from_millis(750));
        assert_eq!(clock.now(), Duration::from_secs(1));
    }

    #[test]
    fn time_stands_still_without_advance() {
        let clock = SimClock::new();
        clock.advance(Duration::from_secs(3));
        assert_eq!(clock.now(), clock.now());
    }

    #[test]
    #[should_panic(expected = "overflowed")]
    fn advance_panics_on_overflow() {
        let clock = SimClock::new();
        clock.advance(Duration::MAX);
        clock.advance(Duration::from_nanos(1));
    }
}
