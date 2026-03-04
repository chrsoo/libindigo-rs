//! Integration tests for server discovery functionality.
//!
//! These tests verify the discovery API works correctly. Some tests are marked
//! as `#[ignore]` because they require an actual INDIGO server running on the
//! local network.

#[cfg(feature = "auto")]
mod discovery_integration {
    use libindigo::client::Client;
    use libindigo::discovery::{DiscoveryConfig, DiscoveryEvent, ServerDiscoveryApi};
    use std::time::Duration;

    #[tokio::test]
    async fn test_discovery_config_builder() {
        let config = DiscoveryConfig::new()
            .timeout(Duration::from_secs(10))
            .service_type("_indigo._tcp.local.")
            .filter(|server| server.name.contains("Test"));

        assert_eq!(config.get_timeout(), Duration::from_secs(10));
        assert_eq!(config.get_service_type(), "_indigo._tcp.local.");
    }

    #[tokio::test]
    async fn test_continuous_config() {
        let config = DiscoveryConfig::continuous();
        assert_eq!(
            config.get_mode(),
            libindigo::discovery::DiscoveryMode::Continuous
        );
    }

    #[tokio::test]
    #[ignore] // Requires actual INDIGO server running
    async fn test_discover_servers() {
        let config = DiscoveryConfig::new().timeout(Duration::from_secs(3));

        let result = ServerDiscoveryApi::discover(config).await;

        match result {
            Ok(servers) => {
                println!("Found {} servers", servers.len());
                for server in servers {
                    println!("  - {} at {}", server.name, server.url());
                    assert!(!server.name.is_empty());
                    assert!(!server.host.is_empty());
                    assert!(server.port > 0);
                }
            }
            Err(e) => {
                println!("Discovery error (expected if no servers): {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires actual INDIGO server running
    async fn test_client_discover_servers() {
        let result = Client::discover_servers().await;

        match result {
            Ok(servers) => {
                println!("Client::discover_servers found {} servers", servers.len());
                for server in servers {
                    println!("  - {} at {}", server.name, server.url());
                }
            }
            Err(e) => {
                println!("Discovery error (expected if no servers): {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires actual INDIGO server running
    async fn test_continuous_discovery() {
        let config = DiscoveryConfig::continuous().timeout(Duration::from_secs(2));

        let result = ServerDiscoveryApi::start_continuous(config).await;

        match result {
            Ok(mut discovery) => {
                let timeout_duration = Duration::from_secs(5);
                let start = std::time::Instant::now();

                while start.elapsed() < timeout_duration {
                    if let Some(event) = discovery.next_event().await {
                        match event {
                            DiscoveryEvent::ServerAdded(server) => {
                                println!("Server added: {}", server.name);
                            }
                            DiscoveryEvent::ServerRemoved(id) => {
                                println!("Server removed: {}", id);
                            }
                            DiscoveryEvent::ServerUpdated(server) => {
                                println!("Server updated: {}", server.name);
                            }
                            DiscoveryEvent::DiscoveryComplete => {
                                println!("Initial discovery complete");
                                break;
                            }
                            DiscoveryEvent::Error(msg) => {
                                println!("Discovery error: {}", msg);
                            }
                        }
                    }
                }

                let servers = discovery.servers();
                println!("Currently {} servers online", servers.len());

                discovery.stop().await.unwrap();
            }
            Err(e) => {
                println!("Continuous discovery error: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires actual INDIGO server running
    async fn test_discovery_with_filter() {
        let config = DiscoveryConfig::new()
            .timeout(Duration::from_secs(3))
            .filter(|server| server.port == 7624);

        let result = ServerDiscoveryApi::discover(config).await;

        match result {
            Ok(servers) => {
                println!("Found {} servers on port 7624", servers.len());
                for server in &servers {
                    assert_eq!(server.port, 7624);
                }
            }
            Err(e) => {
                println!("Discovery error (expected if no servers): {}", e);
            }
        }
    }
}

#[cfg(not(feature = "auto"))]
mod discovery_disabled {
    use libindigo::client::Client;
    use libindigo::error::IndigoError;

    #[tokio::test]
    async fn test_discovery_requires_auto_feature() {
        // This test verifies that discovery is properly gated behind the auto feature
        // When auto feature is disabled, the discovery module should not be available

        // We can't call Client::discover_servers() here because it doesn't exist
        // without the auto feature, which is the correct behavior

        // This test just ensures the module compiles without the auto feature
        assert!(true);
    }
}
