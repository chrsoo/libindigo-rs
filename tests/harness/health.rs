#![allow(dead_code)]

//! Health Monitor for INDIGO server
//!
//! This module provides health checking and readiness detection for the INDIGO server,
//! including TCP connectivity checks, retry logic with exponential backoff, and protocol verification.

use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Server health status
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub reachable: bool,
    pub protocol_responsive: bool,
    pub uptime: Duration,
    pub last_check: Instant,
}

/// Health monitor for INDIGO server
pub struct HealthMonitor {
    address: String,
    timeout_duration: Duration,
    start_time: Instant,
    retry_config: RetryConfig,
}

/// Configuration for retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum number of retries
    pub max_retries: usize,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            max_retries: 20,
        }
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(address: String) -> Self {
        Self {
            address,
            timeout_duration: Duration::from_secs(2),
            start_time: Instant::now(),
            retry_config: RetryConfig::default(),
        }
    }

    /// Set the timeout duration for health checks
    pub fn with_timeout(mut self, timeout_duration: Duration) -> Self {
        self.timeout_duration = timeout_duration;
        self
    }

    /// Set custom retry configuration
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Check if server is reachable via TCP
    pub async fn check_connectivity(&self) -> Result<bool, String> {
        match timeout(self.timeout_duration, TcpStream::connect(&self.address)).await {
            Ok(Ok(_stream)) => {
                eprintln!("[HEALTH] Server is reachable at {}", self.address);
                Ok(true)
            }
            Ok(Err(e)) => {
                eprintln!("[HEALTH] Connection failed: {}", e);
                Ok(false)
            }
            Err(_) => {
                eprintln!(
                    "[HEALTH] Connection timeout after {:?}",
                    self.timeout_duration
                );
                Ok(false)
            }
        }
    }

    /// Wait for server to be ready with exponential backoff
    pub async fn wait_for_ready(&self, max_wait: Duration) -> Result<(), String> {
        let start = Instant::now();
        let mut attempts = 0;
        let mut current_delay = self.retry_config.initial_delay;

        eprintln!(
            "[HEALTH] Waiting for server to be ready at {} (max wait: {:?})",
            self.address, max_wait
        );

        while start.elapsed() < max_wait && attempts < self.retry_config.max_retries {
            attempts += 1;

            match self.check_connectivity().await {
                Ok(true) => {
                    eprintln!(
                        "[HEALTH] Server is reachable after {} attempts ({:?})",
                        attempts,
                        start.elapsed()
                    );

                    // Give it a moment to fully initialize
                    tokio::time::sleep(Duration::from_millis(500)).await;

                    // Verify it's still reachable
                    if self.check_connectivity().await.unwrap_or(false) {
                        eprintln!(
                            "[HEALTH] Server is ready! Total time: {:?}",
                            start.elapsed()
                        );
                        return Ok(());
                    } else {
                        eprintln!("[HEALTH] Server became unreachable during verification");
                    }
                }
                Ok(false) => {
                    // Not ready yet, wait with exponential backoff
                    eprintln!(
                        "[HEALTH] Attempt {}/{}: Server not ready, waiting {:?}",
                        attempts, self.retry_config.max_retries, current_delay
                    );
                    tokio::time::sleep(current_delay).await;

                    // Exponential backoff
                    current_delay = Duration::from_millis(
                        (current_delay.as_millis() as f64 * self.retry_config.backoff_multiplier)
                            as u64,
                    )
                    .min(self.retry_config.max_delay);
                }
                Err(e) => {
                    // Give up after a few failed attempts
                    if attempts > 3 {
                        return Err(format!(
                            "Health check error after {} attempts: {}",
                            attempts, e
                        ));
                    }
                    eprintln!("[HEALTH] Health check error (attempt {}): {}", attempts, e);
                    tokio::time::sleep(current_delay).await;
                }
            }
        }

        Err(format!(
            "Server failed to become ready within {:?} ({} attempts, elapsed: {:?})",
            max_wait,
            attempts,
            start.elapsed()
        ))
    }

    /// Wait for server to be ready with custom retry strategy
    pub async fn wait_for_ready_with_retries(
        &self,
        max_wait: Duration,
        retry_config: RetryConfig,
    ) -> Result<(), String> {
        let start = Instant::now();
        let mut attempts = 0;
        let mut current_delay = retry_config.initial_delay;

        eprintln!(
            "[HEALTH] Waiting for server with custom retry config (max retries: {})",
            retry_config.max_retries
        );

        while start.elapsed() < max_wait && attempts < retry_config.max_retries {
            attempts += 1;

            if self.check_connectivity().await.unwrap_or(false) {
                // Additional verification
                tokio::time::sleep(Duration::from_millis(500)).await;
                if self.check_connectivity().await.unwrap_or(false) {
                    eprintln!("[HEALTH] Server ready after {} attempts", attempts);
                    return Ok(());
                }
            }

            tokio::time::sleep(current_delay).await;
            current_delay = Duration::from_millis(
                (current_delay.as_millis() as f64 * retry_config.backoff_multiplier) as u64,
            )
            .min(retry_config.max_delay);
        }

        Err(format!(
            "Server not ready after {} attempts ({:?})",
            attempts,
            start.elapsed()
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

    /// Perform a comprehensive health check
    pub async fn comprehensive_check(&self) -> Result<ServerStatus, String> {
        let reachable = self.check_connectivity().await.unwrap_or(false);
        let protocol_responsive = if reachable {
            self.verify_protocol().await.unwrap_or(false)
        } else {
            false
        };

        Ok(ServerStatus {
            reachable,
            protocol_responsive,
            uptime: self.start_time.elapsed(),
            last_check: Instant::now(),
        })
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
            last_check: Instant::now(),
        }
    }

    /// Perform multiple health checks and return success rate
    pub async fn check_stability(&self, num_checks: usize, delay: Duration) -> f64 {
        let mut successful = 0;

        for i in 0..num_checks {
            if self.check_connectivity().await.unwrap_or(false) {
                successful += 1;
            }
            if i < num_checks - 1 {
                tokio::time::sleep(delay).await;
            }
        }

        successful as f64 / num_checks as f64
    }

    /// Get server address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get uptime since monitor creation
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
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

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(5));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.max_retries, 20);
    }

    #[test]
    fn test_health_monitor_with_retry_config() {
        let retry_config = RetryConfig {
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            max_retries: 10,
        };

        let monitor = HealthMonitor::new("localhost:7624".to_string())
            .with_retry_config(retry_config.clone());

        assert_eq!(
            monitor.retry_config.initial_delay,
            Duration::from_millis(200)
        );
        assert_eq!(monitor.retry_config.max_retries, 10);
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

    #[tokio::test]
    async fn test_get_status_unreachable() {
        let monitor = HealthMonitor::new("localhost:65534".to_string())
            .with_timeout(Duration::from_millis(100));

        let status = monitor.get_status().await;
        assert_eq!(status.reachable, false);
        assert_eq!(status.protocol_responsive, false);
    }
}
