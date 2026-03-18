use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use crate::protocol::SyncMessage;

const WINDOW_SIZE: usize = 8;
const PING_INTERVAL_MS: u64 = 2000;

/// Tracks round-trip time to each remote peer using NTP-like Ping/Pong.
pub struct ClockSync {
    epoch: Instant,
    per_peer: HashMap<String, PeerClock>,
    next_ping_id: u64,
}

struct PeerClock {
    /// RTT samples in microseconds
    samples: VecDeque<i64>,
    /// Median RTT in microseconds
    rtt_us: i64,
}

impl Default for ClockSync {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
            per_peer: HashMap::new(),
            next_ping_id: 0,
        }
    }
}

impl ClockSync {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current local time in microseconds since epoch.
    pub fn now_us(&self) -> i64 {
        self.epoch.elapsed().as_micros() as i64
    }

    /// Generate a Ping message to send to a peer.
    pub fn make_ping(&mut self) -> SyncMessage {
        let id = self.next_ping_id;
        self.next_ping_id += 1;
        SyncMessage::Ping {
            id,
            sent_at_us: self.now_us(),
        }
    }

    /// Handle an incoming Ping — return a Pong to send back.
    pub fn handle_ping(&self, id: u64, sent_at_us: i64) -> SyncMessage {
        SyncMessage::Pong {
            id,
            ping_sent_at_us: sent_at_us,
            pong_sent_at_us: self.now_us(),
        }
    }

    /// Handle an incoming Pong — update RTT estimate for the peer.
    pub fn handle_pong(&mut self, peer_id: &str, ping_sent_at_us: i64, _pong_sent_at_us: i64) {
        let now = self.now_us();
        let rtt = now - ping_sent_at_us;

        // Discard samples with negative RTT (clock anomaly)
        if rtt < 0 {
            return;
        }

        let clock = self.per_peer.entry(peer_id.to_string()).or_insert(PeerClock {
            samples: VecDeque::with_capacity(WINDOW_SIZE),
            rtt_us: 0,
        });

        clock.samples.push_back(rtt);
        if clock.samples.len() > WINDOW_SIZE {
            clock.samples.pop_front();
        }

        // Use median RTT (robust to outliers)
        let samples: Vec<i64> = clock.samples.iter().copied().collect();
        clock.rtt_us = Self::median_of(&samples);
    }

    /// Compute the median of a slice of RTT samples.
    /// Used internally by `handle_pong`.
    pub(crate) fn median_of(samples: &[i64]) -> i64 {
        let mut sorted: Vec<i64> = samples.to_vec();
        sorted.sort();
        sorted[sorted.len() / 2]
    }

    /// Get the estimated RTT for a peer in microseconds.
    pub fn rtt_us(&self, peer_id: &str) -> Option<i64> {
        self.per_peer.get(peer_id).map(|c| c.rtt_us)
    }

    /// Jitter estimate: mean absolute deviation from median RTT (microseconds).
    /// Returns `None` if fewer than 2 samples have been collected.
    pub fn jitter_us(&self, peer_id: &str) -> Option<i64> {
        let clock = self.per_peer.get(peer_id)?;
        if clock.samples.len() < 2 {
            return None;
        }
        let median = clock.rtt_us;
        let mad = clock
            .samples
            .iter()
            .map(|s| (s - median).abs())
            .sum::<i64>()
            / clock.samples.len() as i64;
        Some(mad)
    }

    /// Ping interval in milliseconds.
    pub fn ping_interval_ms() -> u64 {
        PING_INTERVAL_MS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jitter_zero_with_identical_samples() {
        let mut clock = ClockSync::new();
        // Manually inject identical RTT samples
        for _ in 0..4 {
            clock.handle_pong("peer-a", clock.now_us() - 10_000, 0);
        }
        // All samples are ~10ms, so jitter should be near zero
        let jitter = clock.jitter_us("peer-a").unwrap();
        // Allow small variance from timing
        assert!(jitter < 500, "jitter should be near zero, got {jitter}");
    }

    #[test]
    fn jitter_none_with_insufficient_samples() {
        let mut clock = ClockSync::new();
        clock.handle_pong("peer-a", clock.now_us() - 10_000, 0);
        assert_eq!(clock.jitter_us("peer-a"), None);
    }

    #[test]
    fn jitter_none_for_unknown_peer() {
        let clock = ClockSync::new();
        assert_eq!(clock.jitter_us("unknown"), None);
    }

    #[test]
    fn jitter_positive_with_varied_samples() {
        let mut clock = ClockSync::new();
        let peer_clock = clock.per_peer.entry("peer-a".to_string()).or_insert(PeerClock {
            samples: VecDeque::with_capacity(WINDOW_SIZE),
            rtt_us: 0,
        });
        // Insert known RTT samples: 10, 20, 30, 40 ms
        for &rtt in &[10_000i64, 20_000, 30_000, 40_000] {
            peer_clock.samples.push_back(rtt);
        }
        peer_clock.rtt_us = ClockSync::median_of(
            &peer_clock.samples.iter().copied().collect::<Vec<_>>(),
        );
        // median of [10000, 20000, 30000, 40000] = 30000 (index 2 of sorted)
        // MAD = (|10000-30000| + |20000-30000| + |30000-30000| + |40000-30000|) / 4
        //     = (20000 + 10000 + 0 + 10000) / 4 = 10000
        let jitter = clock.jitter_us("peer-a").unwrap();
        assert_eq!(jitter, 10_000);
    }

    #[test]
    fn rtt_us_returns_none_for_unknown_peer() {
        let clock = ClockSync::new();
        assert_eq!(clock.rtt_us("nonexistent"), None);
    }

    #[test]
    fn rtt_us_updates_on_pong() {
        let mut clock = ClockSync::new();
        let sent = clock.now_us() - 5000;
        clock.handle_pong("peer-a", sent, 0);
        let rtt = clock.rtt_us("peer-a").unwrap();
        assert!(rtt >= 4000 && rtt <= 6000, "rtt should be ~5000, got {rtt}");
    }
}
