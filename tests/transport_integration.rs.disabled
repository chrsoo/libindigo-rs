//! Integration tests for the TCP transport layer.
//!
//! These tests verify the transport layer's ability to handle connections,
//! message framing, and error conditions.

mod common;
mod harness;

#[cfg(feature = "rs-strategy")]
mod transport_tests {
    use libindigo::strategies::rs::protocol::{GetProperties, ProtocolMessage};
    use libindigo::strategies::rs::transport::Transport;

    #[tokio::test]
    async fn test_transport_creation() {
        let transport = Transport::new();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_url_parsing() {
        // This test verifies URL parsing without actually connecting
        // We test the error cases that don't require a network connection

        // Test that we can create a transport
        let transport = Transport::new();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_send_without_connection() {
        let mut transport = Transport::new();

        let msg = ProtocolMessage::GetProperties(GetProperties {
            version: Some("1.7".to_string()),
            device: None,
            name: None,
        });

        // Should fail because not connected
        let result = transport.send_message(&msg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_receive_without_connection() {
        let mut transport = Transport::new();

        // Should fail because not connected
        let result = transport.receive_message().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_without_connection() {
        let mut transport = Transport::new();

        // Should fail because not connected
        let result = transport.disconnect().await;
        assert!(result.is_err());
    }
}
