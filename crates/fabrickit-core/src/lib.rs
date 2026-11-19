//! Shared types used by every FabricKit crate.

use std::fmt;
use std::time::Duration;

/// Unique identifier of a node in a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node-{}", self.0)
    }
}

/// A payload sent from one node to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Node that sent the message.
    pub from: NodeId,
    /// Node the message is addressed to.
    pub to: NodeId,
    /// Opaque message bytes.
    pub payload: Vec<u8>,
}

/// Source of monotonic time.
///
/// Code that needs time takes a `Clock` instead of reading the system clock,
/// so tests can control time.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use fabrickit_core::Clock;
///
/// struct FixedClock(Duration);
///
/// impl Clock for FixedClock {
///     fn now(&self) -> Duration {
///         self.0
///     }
/// }
///
/// let clock = FixedClock(Duration::from_secs(5));
/// assert_eq!(clock.now(), Duration::from_secs(5));
/// ```
pub trait Clock {
    /// Time elapsed since the clock's start. Never decreases.
    fn now(&self) -> Duration;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_display() {
        assert_eq!(NodeId(7).to_string(), "node-7");
    }

    #[test]
    fn message_keeps_fields() {
        let msg = Message {
            from: NodeId(1),
            to: NodeId(2),
            payload: b"ping".to_vec(),
        };
        assert_eq!(msg.from, NodeId(1));
        assert_eq!(msg.to, NodeId(2));
        assert_eq!(msg.payload, b"ping");
        assert_eq!(msg.clone(), msg);
    }
}
