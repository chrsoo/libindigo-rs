//! Server-level INDIGO handshake checking.

use crate::monitoring::status::HandshakeResult;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncReadExt;

/// Server handshake checker.
///
/// Connects to the INDIGO server port and verifies it responds with
/// a valid INDIGO protocol greeting.
pub struct ServerChecker {
    addr: SocketAddr,
    timeout: Duration,
}

impl ServerChecker {
    /// Create a new server checker.
    pub fn new(addr: SocketAddr, timeout: Duration) -> Self {
        Self { addr, timeout }
    }

    /// Perform a server handshake check.
    ///
    /// Returns a HandshakeResult indicating success/failure.
    pub async fn check(&self) -> HandshakeResult {
        tracing::trace!("TCP handshake attempt to {}", self.addr);

        match tokio::time::timeout(self.timeout, self.try_handshake()).await {
            Ok(Ok(())) => {
                tracing::debug!("Server handshake succeeded for {}", self.addr);
                HandshakeResult { success: true }
            }
            Ok(Err(e)) => {
                tracing::debug!("Server handshake failed for {}: {}", self.addr, e);
                HandshakeResult { success: false }
            }
            Err(_) => {
                tracing::debug!("Server handshake timeout for {}", self.addr);
                HandshakeResult { success: false }
            }
        }
    }

    /// Attempt to connect and read initial greeting from server.
    async fn try_handshake(&self) -> Result<(), String> {
        // Connect to server
        let mut stream = tokio::net::TcpStream::connect(self.addr)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Read initial greeting (INDIGO servers send XML or JSON greeting)
        let mut buf = vec![0u8; 1024];
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| format!("Read failed: {}", e))?;

        if n == 0 {
            return Err("No data received".to_string());
        }

        // Convert to string and check for INDIGO markers
        let greeting = String::from_utf8_lossy(&buf[..n]);

        // Check for INDIGO protocol markers (XML or JSON)
        if greeting.contains("<getProperties") || greeting.contains("\"getProperties\"") {
            Ok(())
        } else {
            Err("Invalid INDIGO greeting".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_check_no_server() {
        // This test assumes nothing is listening on port 9999
        let checker = ServerChecker::new(
            "127.0.0.1:9999".parse().unwrap(),
            Duration::from_millis(100),
        );

        let result = checker.check().await;
        // Should fail since nothing is listening
        assert!(!result.success);
    }

    #[test]
    fn test_server_checker_creation() {
        let addr: SocketAddr = "192.168.1.1:7624".parse().unwrap();
        let checker = ServerChecker::new(addr, Duration::from_secs(2));

        assert_eq!(checker.addr, addr);
        assert_eq!(checker.timeout, Duration::from_secs(2));
    }
}
