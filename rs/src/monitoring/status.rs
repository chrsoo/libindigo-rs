//! Status tracking with rolling window for monitoring.

use libindigo::client::monitoring::AvailabilityStatus;
use std::collections::VecDeque;
use std::time::Duration;

/// Result of a ping check.
#[derive(Debug, Clone)]
pub struct PingResult {
    pub success: bool,
    pub duration: Option<Duration>,
}

/// Result of a server handshake check.
#[derive(Debug, Clone)]
pub struct HandshakeResult {
    pub success: bool,
}

/// Rolling window status tracker.
///
/// Maintains a rolling window of ping and handshake results and computes
/// the current availability status based on thresholds.
pub struct StatusTracker {
    /// Rolling window of ping results
    ping_window: VecDeque<PingResult>,
    /// Rolling window of handshake results
    handshake_window: VecDeque<HandshakeResult>,
    /// Maximum window size
    window_size: usize,
    /// Response time threshold for degraded status
    response_time_threshold: Duration,
    /// Current computed status
    current_status: AvailabilityStatus,
}

impl StatusTracker {
    /// Create a new status tracker.
    pub fn new(window_size: usize, response_time_threshold: Duration) -> Self {
        Self {
            ping_window: VecDeque::with_capacity(window_size),
            handshake_window: VecDeque::with_capacity(window_size),
            window_size,
            response_time_threshold,
            current_status: AvailabilityStatus::Unavailable,
        }
    }

    /// Record a ping result and return the new status if it changed.
    pub fn record_ping(&mut self, result: PingResult) -> Option<AvailabilityStatus> {
        // Add to window
        self.ping_window.push_back(result);

        // Trim window if needed
        if self.ping_window.len() > self.window_size {
            self.ping_window.pop_front();
        }

        // Recompute status
        self.compute_status()
    }

    /// Record a handshake result and return the new status if it changed.
    pub fn record_handshake(&mut self, result: HandshakeResult) -> Option<AvailabilityStatus> {
        // Add to window
        self.handshake_window.push_back(result);

        // Trim window if needed
        if self.handshake_window.len() > self.window_size {
            self.handshake_window.pop_front();
        }

        // Recompute status
        self.compute_status()
    }

    /// Get the current status.
    pub fn current_status(&self) -> AvailabilityStatus {
        self.current_status
    }

    /// Compute the current status based on the rolling windows.
    ///
    /// Returns Some(new_status) if the status changed, None otherwise.
    fn compute_status(&mut self) -> Option<AvailabilityStatus> {
        let new_status = self.determine_status();

        if new_status != self.current_status {
            let old_status = self.current_status;
            self.current_status = new_status;
            tracing::trace!(
                "Status window state changed: {:?} -> {:?}",
                old_status,
                new_status
            );
            Some(new_status)
        } else {
            None
        }
    }

    /// Determine the status based on current window contents.
    fn determine_status(&self) -> AvailabilityStatus {
        // If we don't have enough data, maintain current status
        if self.ping_window.is_empty() {
            return self.current_status;
        }

        // Check if host is reachable (recent pings successful)
        let recent_pings_successful = self
            .ping_window
            .iter()
            .rev()
            .take(self.window_size.min(3))
            .all(|p| p.success);

        if !recent_pings_successful {
            // Host unreachable
            return AvailabilityStatus::Unavailable;
        }

        // Host is reachable, check server handshake
        if self.handshake_window.is_empty() {
            // No handshake data yet, but host is up
            return AvailabilityStatus::Degraded;
        }

        let recent_handshakes_successful = self
            .handshake_window
            .iter()
            .rev()
            .take(self.window_size.min(3))
            .all(|h| h.success);

        if !recent_handshakes_successful {
            // Host up but server not responding
            return AvailabilityStatus::Degraded;
        }

        // Check response times
        let has_slow_responses = self
            .ping_window
            .iter()
            .rev()
            .take(self.window_size.min(3))
            .any(|p| {
                p.duration
                    .map(|d| d > self.response_time_threshold)
                    .unwrap_or(false)
            });

        if has_slow_responses {
            // Slow responses
            return AvailabilityStatus::Degraded;
        }

        // Everything looks good
        AvailabilityStatus::Available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_status_unavailable() {
        let tracker = StatusTracker::new(5, Duration::from_secs(1));
        assert_eq!(tracker.current_status(), AvailabilityStatus::Unavailable);
    }

    #[test]
    fn test_successful_pings_without_handshake() {
        let mut tracker = StatusTracker::new(5, Duration::from_secs(1));

        // Record successful pings
        for _ in 0..3 {
            tracker.record_ping(PingResult {
                success: true,
                duration: Some(Duration::from_millis(50)),
            });
        }

        // Should be degraded (host up, no handshake data)
        assert_eq!(tracker.current_status(), AvailabilityStatus::Degraded);
    }

    #[test]
    fn test_successful_pings_and_handshakes() {
        let mut tracker = StatusTracker::new(5, Duration::from_secs(1));

        // Record successful pings
        for _ in 0..3 {
            tracker.record_ping(PingResult {
                success: true,
                duration: Some(Duration::from_millis(50)),
            });
        }

        // Record successful handshakes
        for _ in 0..3 {
            tracker.record_handshake(HandshakeResult { success: true });
        }

        // Should be available
        assert_eq!(tracker.current_status(), AvailabilityStatus::Available);
    }

    #[test]
    fn test_failed_pings() {
        let mut tracker = StatusTracker::new(5, Duration::from_secs(1));

        // Record failed pings
        for _ in 0..3 {
            tracker.record_ping(PingResult {
                success: false,
                duration: None,
            });
        }

        // Should be unavailable
        assert_eq!(tracker.current_status(), AvailabilityStatus::Unavailable);
    }

    #[test]
    fn test_slow_responses() {
        let mut tracker = StatusTracker::new(5, Duration::from_secs(1));

        // Record slow pings
        for _ in 0..3 {
            tracker.record_ping(PingResult {
                success: true,
                duration: Some(Duration::from_secs(2)), // Exceeds threshold
            });
        }

        // Record successful handshakes
        for _ in 0..3 {
            tracker.record_handshake(HandshakeResult { success: true });
        }

        // Should be degraded due to slow responses
        assert_eq!(tracker.current_status(), AvailabilityStatus::Degraded);
    }

    #[test]
    fn test_status_change_detection() {
        let mut tracker = StatusTracker::new(5, Duration::from_secs(1));

        // First ping should change status
        let status_change = tracker.record_ping(PingResult {
            success: true,
            duration: Some(Duration::from_millis(50)),
        });
        assert!(status_change.is_some());

        // Same status should not trigger change
        let status_change = tracker.record_ping(PingResult {
            success: true,
            duration: Some(Duration::from_millis(50)),
        });
        assert!(status_change.is_none());
    }
}
