//! Example: Continuous server discovery
//!
//! This example demonstrates continuous monitoring for server changes.
//!
//! Run with:
//! ```bash
//! cargo run --example continuous_discovery --features auto
//! ```

#[cfg(feature = "auto")]
use libindigo::discovery::{DiscoveryConfig, DiscoveryEvent, ServerDiscoveryApi};
#[cfg(feature = "auto")]
use std::time::Duration;

#[cfg(feature = "auto")]
#[tokio::main]
async fn main() -> libindigo::error::Result<()> {
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

#[cfg(not(feature = "auto"))]
fn main() {
    eprintln!("This example requires the 'auto' feature.");
    eprintln!("Run with: cargo run --example continuous_discovery --features auto");
    std::process::exit(1);
}
