//! Deterministic test helpers: simulated network, clock and faults.

use std::cell::Cell;
use std::collections::VecDeque;
use std::time::Duration;

pub use fabrickit_core::NodeId;
use fabrickit_core::{Clock, Message};

/// SplitMix64 generator. Its output for a seed is fixed forever, so failing
/// seeds always replay the same run.
#[derive(Debug, Clone)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// In-memory network.
///
/// [`SimNetwork::new`] delivers messages in send order.
/// [`SimNetwork::with_seed`] delivers them in a random order fixed by the seed.
#[derive(Debug, Default)]
pub struct SimNetwork {
    in_flight: VecDeque<Message>,
    rng: Option<SplitMix64>,
}

impl SimNetwork {
    /// Creates an empty network that delivers in send order.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty network that reorders delivery. The same seed and
    /// the same sends always give the same delivery order.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            in_flight: VecDeque::new(),
            rng: Some(SplitMix64(seed)),
        }
    }

    /// Queues `msg` for delivery.
    pub fn send(&mut self, msg: Message) {
        self.in_flight.push_back(msg);
    }

    /// Removes and returns the next message to deliver, if any.
    pub fn deliver_next(&mut self) -> Option<Message> {
        match &mut self.rng {
            None => self.in_flight.pop_front(),
            Some(rng) if !self.in_flight.is_empty() => {
                let index = (rng.next_u64() % self.in_flight.len() as u64) as usize;
                self.in_flight.remove(index)
            }
            Some(_) => None,
        }
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

    fn delivery_order(seed: u64) -> Vec<Vec<u8>> {
        let mut net = SimNetwork::with_seed(seed);
        for i in 0..10u8 {
            net.send(msg(1, 2, &[i]));
        }
        std::iter::from_fn(|| net.deliver_next())
            .map(|m| m.payload)
            .collect()
    }

    #[test]
    fn splitmix64_matches_reference_output() {
        let mut rng = SplitMix64(0);
        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
    }

    #[test]
    fn same_seed_gives_same_order() {
        assert_eq!(delivery_order(42), delivery_order(42));
    }

    #[test]
    fn seed_changes_order() {
        let in_order: Vec<Vec<u8>> = (0..10u8).map(|i| vec![i]).collect();
        assert_ne!(delivery_order(42), in_order);
        assert_ne!(delivery_order(42), delivery_order(7));
    }

    #[test]
    fn seeded_network_delivers_every_message_once() {
        let mut order = delivery_order(42);
        order.sort();
        let expected: Vec<Vec<u8>> = (0..10u8).map(|i| vec![i]).collect();
        assert_eq!(order, expected);
    }

    #[test]
    fn empty_seeded_network_delivers_nothing() {
        assert_eq!(SimNetwork::with_seed(1).deliver_next(), None);
    }

    #[test]
    #[should_panic(expected = "overflowed")]
    fn advance_panics_on_overflow() {
        let clock = SimClock::new();
        clock.advance(Duration::MAX);
        clock.advance(Duration::from_nanos(1));
    }
}
