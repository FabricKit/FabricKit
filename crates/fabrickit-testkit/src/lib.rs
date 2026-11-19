//! Deterministic test helpers: simulated network, clock and faults.

use std::cell::Cell;
use std::collections::VecDeque;
use std::time::Duration;

pub use fabrickit_core::NodeId;
use fabrickit_core::{Clock, Message};

/// In-memory network that delivers messages in the order they were sent.
#[derive(Debug, Default)]
pub struct SimNetwork {
    in_flight: VecDeque<Message>,
}

impl SimNetwork {
    /// Creates an empty network.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues `msg` for delivery.
    pub fn send(&mut self, msg: Message) {
        self.in_flight.push_back(msg);
    }

    /// Removes and returns the oldest undelivered message, if any.
    pub fn deliver_next(&mut self) -> Option<Message> {
        self.in_flight.pop_front()
    }
}

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

    fn msg(from: u64, to: u64, payload: &[u8]) -> Message {
        Message {
            from: NodeId(from),
            to: NodeId(to),
            payload: payload.to_vec(),
        }
    }

    #[test]
    fn empty_network_delivers_nothing() {
        assert_eq!(SimNetwork::new().deliver_next(), None);
    }

    #[test]
    fn delivers_in_send_order() {
        let mut net = SimNetwork::new();
        net.send(msg(1, 2, b"a"));
        net.send(msg(2, 3, b"b"));
        net.send(msg(3, 1, b"c"));
        assert_eq!(net.deliver_next(), Some(msg(1, 2, b"a")));
        assert_eq!(net.deliver_next(), Some(msg(2, 3, b"b")));
        assert_eq!(net.deliver_next(), Some(msg(3, 1, b"c")));
        assert_eq!(net.deliver_next(), None);
    }

    #[test]
    #[should_panic(expected = "overflowed")]
    fn advance_panics_on_overflow() {
        let clock = SimClock::new();
        clock.advance(Duration::MAX);
        clock.advance(Duration::from_nanos(1));
    }
}
