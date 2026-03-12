//! Example: Continuous server discovery
//!
//! This example demonstrates continuous monitoring for server changes.
//!
//! Run with:
//! ```bash
//! cargo run --example continuous_discovery --features discovery
//! ```

#[cfg(feature = "discovery")]
use libindigo_rs::discovery::{DiscoveryConfig, DiscoveryEvent, ServerDiscoveryApi};
#[cfg(feature = "discovery")]
use std::time::Duration;

#[cfg(feature = "discovery")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting continuous INDIGO server discovery...");
    println!("Press Ctrl+C to stop\n");

    let config = DiscoveryConfig::continuous().timeout(Duration::from_secs(3));

    let mut discovery = ServerDiscoveryApi::start_continuous(config).await?;

    // Monitor for events
    while let Some(event) = discovery.next_event().await {
        match event {
            DiscoveryEvent::ServerAdded(server) => {
                println!("✓ Server ADDED: {} at {}", server.name, server.url());
            }
            DiscoveryEvent::ServerRemoved(id) => {
                println!("✗ Server REMOVED: {}", id);
            }
            DiscoveryEvent::ServerUpdated(server) => {
                println!("↻ Server UPDATED: {} at {}", server.name, server.url());
            }
            DiscoveryEvent::DiscoveryComplete => {
                println!("\n✓ Initial discovery complete\n");

                // Show current servers
                let servers = discovery.servers();
                println!("Currently {} server(s) online:", servers.len());
                for server in servers {
                    println!("  - {} at {}", server.name, server.url());
                }
                println!("\nMonitoring for changes...\n");
            }
            DiscoveryEvent::Error(msg) => {
                eprintln!("⚠ Discovery error: {}", msg);
            }
        }
    }

    discovery.stop().await?;
    println!("\nDiscovery stopped");

    Ok(())
}

#[cfg(not(feature = "discovery"))]
fn main() {
    eprintln!("This example requires the 'discovery' feature.");
    eprintln!("Run with: cargo run --example continuous_discovery --features discovery");
    std::process::exit(1);
}
