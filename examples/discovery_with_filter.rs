//! Example: Server discovery with filtering
//!
//! This example demonstrates how to filter discovered servers
//! based on custom criteria.
//!
//! Run with:
//! ```bash
//! cargo run --example discovery_with_filter --features discovery
//! ```

#[cfg(feature = "discovery")]
use libindigo_rs::discovery::{DiscoveryConfig, ServerDiscoveryApi};
#[cfg(feature = "discovery")]
use std::time::Duration;

#[cfg(feature = "discovery")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Discovering INDIGO servers with filters...\n");

    // Example 1: Filter by name pattern
    println!("1. Servers with 'Simulator' in name:");
    let config = DiscoveryConfig::new()
        .timeout(Duration::from_secs(5))
        .filter(|server| server.name.contains("Simulator"));

    match ServerDiscoveryApi::discover(config).await {
        Ok(servers) => {
            if servers.is_empty() {
                println!("   No matching servers found");
            } else {
                for server in servers {
                    println!("   - {}", server.name);
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 2: Filter by port
    println!("\n2. Servers on port 7624:");
    let config = DiscoveryConfig::new()
        .timeout(Duration::from_secs(5))
        .filter(|server| server.port == 7624);

    match ServerDiscoveryApi::discover(config).await {
        Ok(servers) => {
            if servers.is_empty() {
                println!("   No matching servers found");
            } else {
                for server in servers {
                    println!("   - {} at {}", server.name, server.url());
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 3: Filter by TXT record
    println!("\n3. Servers with specific TXT records:");
    let config = DiscoveryConfig::new()
        .timeout(Duration::from_secs(5))
        .filter(|server| server.txt_records.contains_key("version"));

    match ServerDiscoveryApi::discover(config).await {
        Ok(servers) => {
            if servers.is_empty() {
                println!("   No matching servers found");
            } else {
                for server in servers {
                    if let Some(version) = server.txt_records.get("version") {
                        println!("   - {} (version: {})", server.name, version);
                    }
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 4: Complex filter
    println!("\n4. Servers matching complex criteria:");
    let config = DiscoveryConfig::new()
        .timeout(Duration::from_secs(5))
        .filter(|server| {
            // Must be on standard port and have at least one IP address
            server.port == 7624 && !server.addresses.is_empty()
        });

    match ServerDiscoveryApi::discover(config).await {
        Ok(servers) => {
            if servers.is_empty() {
                println!("   No matching servers found");
            } else {
                for server in servers {
                    println!(
                        "   - {} with {} address(es)",
                        server.name,
                        server.addresses.len()
                    );
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    Ok(())
}

#[cfg(not(feature = "discovery"))]
fn main() {
    eprintln!("This example requires the 'discovery' feature.");
    eprintln!("Run with: cargo run --example discovery_with_filter --features discovery");
    std::process::exit(1);
}
