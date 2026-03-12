//! Host-level heartbeat checking via ICMP ping or TCP connect.

use crate::monitoring::status::PingResult;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

/// Heartbeat checker for host availability.
///
/// Attempts ICMP ping first (when enabled), falls back to TCP connect.
pub struct HeartbeatChecker {
    target: IpAddr,
    port: u16,
    timeout: Duration,
    use_icmp: bool,
}

impl HeartbeatChecker {
    /// Create a new heartbeat checker.
    pub fn new(target: IpAddr, port: u16, timeout: Duration, use_icmp: bool) -> Self {
        Self {
            target,
            port,
            timeout,
            use_icmp,
        }
    }

    /// Perform a heartbeat check.
    ///
    /// Returns a PingResult indicating success/failure and duration.
    pub async fn check(&self) -> PingResult {
        // Try ICMP first if enabled
        if self.use_icmp {
            match self.icmp_ping().await {
                Ok(result) => return result,
                Err(e) => {
                    tracing::warn!("ICMP ping unavailable, falling back to TCP connect: {}", e);
                    // Fall through to TCP
                }
            }
        }

        // Use TCP connect as fallback or primary method
        self.tcp_connect().await
    }

    /// Perform ICMP ping using surge-ping.
    #[cfg(feature = "monitoring")]
    async fn icmp_ping(&self) -> Result<PingResult, String> {
        use surge_ping::{Client, Config, PingIdentifier, PingSequence};

        tracing::trace!("ICMP ping request sent to {}", self.target);

        // Create ICMP client
        let config = Config::default();
        let client =
            Client::new(&config).map_err(|e| format!("Failed to create ICMP client: {}", e))?;

        // Send ping
        let payload = [0; 8];
        let mut pinger = client
            .pinger(self.target, PingIdentifier(rand::random()))
            .await;

        // Wait for response with timeout
        match tokio::time::timeout(self.timeout, pinger.ping(PingSequence(0), &payload)).await {
            Ok(Ok((_, duration))) => {
                tracing::trace!(
                    "ICMP ping response received from {}: rtt={:?}",
                    self.target,
                    duration
                );
                Ok(PingResult {
                    success: true,
                    duration: Some(duration),
                })
            }
            Ok(Err(e)) => {
                tracing::debug!("ICMP ping failed for {}: {}", self.target, e);
                Ok(PingResult {
                    success: false,
                    duration: None,
                })
            }
            Err(_) => {
                tracing::debug!("ICMP ping timeout for {}", self.target);
                Ok(PingResult {
                    success: false,
                    duration: None,
                })
            }
        }
    }

    /// Perform TCP connect check.
    async fn tcp_connect(&self) -> PingResult {
        let addr = SocketAddr::new(self.target, self.port);
        tracing::trace!("TCP connect check sent to {}", addr);

        let start = Instant::now();

        match tokio::time::timeout(self.timeout, tokio::net::TcpStream::connect(addr)).await {
            Ok(Ok(_stream)) => {
                let duration = start.elapsed();
                tracing::trace!("TCP connect succeeded to {}: rtt={:?}", addr, duration);
                PingResult {
                    success: true,
                    duration: Some(duration),
                }
            }
            Ok(Err(e)) => {
                tracing::debug!("TCP connect failed for {}: {}", addr, e);
                PingResult {
                    success: false,
                    duration: None,
                }
            }
            Err(_) => {
                tracing::debug!("TCP connect timeout for {}", addr);
                PingResult {
                    success: false,
                    duration: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_connect_localhost() {
        // This test assumes nothing is listening on port 9999
        let checker = HeartbeatChecker::new(
            "127.0.0.1".parse().unwrap(),
            9999,
            Duration::from_millis(100),
            false,
        );

        let result = checker.check().await;
        // Should fail since nothing is listening
        assert!(!result.success);
    }

    #[test]
    fn test_heartbeat_checker_creation() {
        let checker = HeartbeatChecker::new(
            "192.168.1.1".parse().unwrap(),
            7624,
            Duration::from_secs(2),
            true,
        );

        assert_eq!(checker.target, "192.168.1.1".parse::<IpAddr>().unwrap());
        assert_eq!(checker.port, 7624);
        assert_eq!(checker.timeout, Duration::from_secs(2));
        assert!(checker.use_icmp);
    }
}
