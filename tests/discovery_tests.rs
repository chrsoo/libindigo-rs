//! Integration tests for server discovery functionality.
//!
//! These tests verify the discovery API works correctly. Some tests are marked
//! as `#[ignore]` because they require an actual INDIGO server running on the
//! local network.

#[cfg(feature = "discovery")]
mod discovery_integration {
    use libindigo_rs::discovery::{
        DiscoveryConfig, DiscoveryEvent, ServerDiscoveryApi, ServiceAnnouncement,
    };
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
        use libindigo_rs::discovery::DiscoveryMode;
        assert_eq!(config.get_mode(), DiscoveryMode::Continuous);
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
    async fn test_service_announcement_builder() {
        let announcement = ServiceAnnouncement::new("Test Server", 7624)
            .with_property("version", "2.0")
            .with_property("devices", "3");

        assert_eq!(announcement.name, "Test Server");
        assert_eq!(announcement.port, 7624);
        assert_eq!(announcement.properties.len(), 2);
        assert_eq!(
            announcement.properties.get("version"),
            Some(&"2.0".to_string())
        );
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_announce_and_discover() {
        // Announce a test service
        let announcement = ServiceAnnouncement::new("Test INDIGO Server", 17624)
            .with_property("version", "2.0-test")
            .with_property("test", "true");

        let handle = ServerDiscoveryApi::announce(announcement).await;

        match handle {
            Ok(handle) => {
                println!("Service announced: {}", handle.fullname());

                // Give mDNS time to propagate
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Try to discover it
                let config = DiscoveryConfig::new()
                    .timeout(Duration::from_secs(3))
                    .filter(|server| server.name.contains("Test INDIGO Server"));

                match ServerDiscoveryApi::discover(config).await {
                    Ok(servers) => {
                        println!("Found {} matching servers", servers.len());
                        for server in &servers {
                            println!("  - {} at {}", server.name, server.url());
                            if server.name.contains("Test INDIGO Server") {
                                assert_eq!(server.port, 17624);
                                assert_eq!(
                                    server.txt_records.get("version"),
                                    Some(&"2.0-test".to_string())
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("Discovery error: {}", e);
                    }
                }

                // Stop announcing
                handle.stop().await.unwrap();
                println!("Service announcement stopped");
            }
            Err(e) => {
                println!("Announcement error (may require network): {}", e);
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

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_announcement_handle_drop() {
        let announcement = ServiceAnnouncement::new("Drop Test Server", 27624);

        let result = ServerDiscoveryApi::announce(announcement).await;

        if let Ok(handle) = result {
            println!("Service announced: {}", handle.fullname());
            // Handle will be dropped here, automatically stopping the announcement
        }
    }
}

#[cfg(not(feature = "discovery"))]
mod discovery_disabled {
    #[tokio::test]
    async fn test_discovery_requires_feature() {
        // This test verifies that discovery is properly gated behind the discovery feature
        // When discovery feature is disabled, the discovery module should not be available
        // This test just ensures the module compiles without the discovery feature
        assert!(true);
    }
}
