#![allow(dead_code)]

//! Health Monitor for INDIGO server
//!
//! This module provides health checking and readiness detection for the INDIGO server,
//! including TCP connectivity checks and protocol verification.

use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Server health status
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub reachable: bool,
    pub protocol_responsive: bool,
    pub uptime: Duration,
}

/// Health monitor for INDIGO server
pub struct HealthMonitor {
    address: String,
    timeout_duration: Duration,
    start_time: Instant,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(address: String) -> Self {
        Self {
            address,
            timeout_duration: Duration::from_secs(2),
            start_time: Instant::now(),
        }
    }

    /// Set the timeout duration for health checks
    pub fn with_timeout(mut self, timeout_duration: Duration) -> Self {
        self.timeout_duration = timeout_duration;
        self
    }

    /// Check if server is reachable via TCP
    pub async fn check_connectivity(&self) -> Result<bool, String> {
        match timeout(self.timeout_duration, TcpStream::connect(&self.address)).await {
            Ok(Ok(_stream)) => Ok(true),
            Ok(Err(_e)) => Ok(false), // Connection failed but no timeout
            Err(_) => Ok(false),      // Timeout
        }
    }

    /// Wait for server to be ready
    pub async fn wait_for_ready(&self, max_wait: Duration) -> Result<(), String> {
        let start = Instant::now();
        let mut attempts = 0;
        let max_attempts = (max_wait.as_secs() * 2) as usize; // 2 attempts per second

        while start.elapsed() < max_wait && attempts < max_attempts {
            attempts += 1;

            match self.check_connectivity().await {
                Ok(true) => {
                    // Server is reachable, give it a moment to fully initialize
                    tokio::time::sleep(Duration::from_millis(500)).await;

                    // Verify it's still reachable
                    if self.check_connectivity().await.unwrap_or(false) {
                        return Ok(());
                    }
                }
                Ok(false) => {
                    // Not ready yet, wait and retry
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    // Give up after a few failed attempts
                    if attempts > 3 {
                        return Err(format!(
                            "Health check error after {} attempts: {}",
                            attempts, e
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }

        Err(format!(
            "Server failed to become ready within {:?} ({} attempts)",
            max_wait, attempts
        ))
    }

    /// Verify server responds to protocol messages
    ///
    /// Note: This is a basic connectivity check. Full protocol verification
    /// would require importing the transport layer, which we avoid to keep
    /// the harness independent.
    pub async fn verify_protocol(&self) -> Result<bool, String> {
        // For now, just verify TCP connectivity
        // Full protocol verification can be added later if needed
        self.check_connectivity().await
    }

    /// Get current server status
    pub async fn get_status(&self) -> ServerStatus {
        let reachable = self.check_connectivity().await.unwrap_or(false);
        let protocol_responsive = if reachable {
            self.verify_protocol().await.unwrap_or(false)
        } else {
            false
        };

        ServerStatus {
            reachable,
            protocol_responsive,
            uptime: self.start_time.elapsed(),
        }
    }

    /// Get server address
    pub fn address(&self) -> &str {
        &self.address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new("localhost:7624".to_string());
        assert_eq!(monitor.address(), "localhost:7624");
    }

    #[test]
    fn test_health_monitor_with_timeout() {
        let monitor =
            HealthMonitor::new("localhost:7624".to_string()).with_timeout(Duration::from_secs(5));
        assert_eq!(monitor.timeout_duration, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_check_connectivity_unreachable() {
        // Use a port that's unlikely to be in use
        let monitor = HealthMonitor::new("localhost:65534".to_string())
            .with_timeout(Duration::from_millis(100));

        let result = monitor.check_connectivity().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
}
