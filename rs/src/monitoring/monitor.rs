//! Main monitoring orchestrator that ties everything together.

use crate::monitoring::heartbeat::HeartbeatChecker;
use crate::monitoring::server_check::ServerChecker;
use crate::monitoring::status::StatusTracker;
use libindigo::client::monitoring::{AvailabilityStatus, MonitoringConfig, MonitoringEvent};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

/// Server monitor that orchestrates heartbeat and server checks.
///
/// This is the main entry point for the monitoring system. It spawns a
/// background task that performs periodic checks and emits status change events.
pub struct ServerMonitor {
    config: MonitoringConfig,
    state: Arc<Mutex<MonitorState>>,
}

struct MonitorState {
    task_handle: Option<JoinHandle<()>>,
    event_tx: Option<mpsc::UnboundedSender<MonitoringEvent>>,
}

impl ServerMonitor {
    /// Create a new server monitor with the given configuration.
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(MonitorState {
                task_handle: None,
                event_tx: None,
            })),
        }
    }

    /// Start monitoring and return a receiver for monitoring events.
    ///
    /// This spawns a background task that performs periodic checks.
    pub async fn start(&self) -> mpsc::UnboundedReceiver<MonitoringEvent> {
        let (tx, rx) = mpsc::unbounded_channel();

        let config = self.config.clone();
        let state = Arc::clone(&self.state);
        let tx_clone = tx.clone();

        // Spawn monitoring task
        let handle = tokio::spawn(async move {
            Self::monitoring_loop(config, tx_clone).await;
        });

        // Store handle and sender
        let mut state = state.lock().await;
        state.task_handle = Some(handle);
        state.event_tx = Some(tx);

        rx
    }

    /// Stop monitoring.
    pub async fn stop(&self) {
        let mut state = self.state.lock().await;

        if let Some(handle) = state.task_handle.take() {
            handle.abort();
        }

        state.event_tx = None;
    }

    /// Main monitoring loop.
    async fn monitoring_loop(
        config: MonitoringConfig,
        event_tx: mpsc::UnboundedSender<MonitoringEvent>,
    ) {
        tracing::info!(
            "Starting monitoring for {} (ping_interval={:?}, window_size={})",
            config.server_addr,
            config.ping_interval,
            config.window_size
        );

        // Create checkers
        let heartbeat_checker = HeartbeatChecker::new(
            config.server_addr.ip(),
            config.server_addr.port(),
            config.connection_timeout,
            config.use_icmp,
        );

        let server_checker = ServerChecker::new(config.server_addr, config.connection_timeout);

        // Create status tracker
        let mut status_tracker =
            StatusTracker::new(config.window_size, config.response_time_threshold);

        // Tracking for server check interval
        let mut ping_count = 0;
        let server_check_every = (config.ping_interval.as_secs_f64() * 5.0) as u64; // Check server every ~5 pings

        loop {
            // Perform heartbeat check
            let ping_result = heartbeat_checker.check().await;

            // Emit low-level ping event
            let _ = event_tx.send(MonitoringEvent::PingResult {
                success: ping_result.success,
                duration: ping_result.duration,
            });

            // Record ping result and check for status change
            if let Some(new_status) = status_tracker.record_ping(ping_result) {
                Self::emit_status_change(&event_tx, status_tracker.current_status(), new_status);
            }

            // Perform server check periodically
            ping_count += 1;
            if ping_count % server_check_every == 0 {
                let handshake_result = server_checker.check().await;

                // Emit low-level handshake event
                let _ = event_tx.send(MonitoringEvent::HandshakeResult {
                    success: handshake_result.success,
                });

                // Record handshake result and check for status change
                if let Some(new_status) = status_tracker.record_handshake(handshake_result) {
                    Self::emit_status_change(
                        &event_tx,
                        status_tracker.current_status(),
                        new_status,
                    );
                }
            }

            // Sleep until next check
            tokio::time::sleep(config.ping_interval).await;
        }
    }

    /// Emit a status change event.
    fn emit_status_change(
        event_tx: &mpsc::UnboundedSender<MonitoringEvent>,
        previous: AvailabilityStatus,
        current: AvailabilityStatus,
    ) {
        tracing::info!("Status changed from {} to {}", previous, current);

        let _ = event_tx.send(MonitoringEvent::StatusChanged { previous, current });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_monitor_creation() {
        let config = MonitoringConfig::new("127.0.0.1:7624".parse().unwrap());
        let monitor = ServerMonitor::new(config);

        // Should be able to create monitor
        assert!(monitor.state.lock().await.task_handle.is_none());
    }

    #[tokio::test]
    async fn test_monitor_start_stop() {
        let config = MonitoringConfig::new("127.0.0.1:9999".parse().unwrap())
            .with_ping_interval(Duration::from_millis(100));

        let monitor = ServerMonitor::new(config);

        // Start monitoring
        let mut rx = monitor.start().await;

        // Should receive some events
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Stop monitoring
        monitor.stop().await;

        // Verify we received events
        let mut event_count = 0;
        while rx.try_recv().is_ok() {
            event_count += 1;
        }

        assert!(event_count > 0, "Should have received at least one event");
    }
}
