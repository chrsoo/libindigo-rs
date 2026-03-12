//! Example: Auto-connect to discovered server
//!
//! This example demonstrates discovering and automatically connecting
//! to the first available INDIGO server.
//!
//! Run with:
//! ```bash
//! cargo run --example auto_connect --features discovery
//! ```
//!
//! Note: This example requires libindigo-rs for the pure Rust client implementation.

#[cfg(feature = "discovery")]
use libindigo::discovery::DiscoveryConfig;
#[cfg(feature = "discovery")]
use libindigo::prelude::*;
#[cfg(feature = "discovery")]
use std::time::Duration;

#[cfg(feature = "discovery")]
#[tokio::main]
async fn main() -> Result<()> {
    println!("Auto-connecting to INDIGO server...\n");

    // Discover servers with custom timeout
    let config = DiscoveryConfig::new().timeout(Duration::from_secs(5));

    let servers = Client::discover_servers_with_config(config).await?;

    if servers.is_empty() {
        eprintln!("No INDIGO servers found on the network.");
        eprintln!("Make sure an INDIGO server is running and discoverable.");
        return Ok(());
    }

    println!("Found {} server(s):", servers.len());
    for (i, server) in servers.iter().enumerate() {
        println!("  {}. {} at {}", i + 1, server.name, server.url());
    }

    // Connect to the first server
    let server = &servers[0];
    println!("\nConnecting to: {}", server.name);

    let mut client = ClientBuilder::new().with_rs_strategy().build()?;

    client.connect(&server.url()).await?;
    println!("✓ Connected successfully!");

    // Enumerate properties
    println!("\nEnumerating properties...");
    client.enumerate_properties(None).await?;
    println!("✓ Properties enumerated");

    // Disconnect
    println!("\nDisconnecting...");
    client.disconnect().await?;
    println!("✓ Disconnected");

    Ok(())
}

#[cfg(not(feature = "discovery"))]
fn main() {
    eprintln!("This example requires the 'discovery' feature.");
    eprintln!("Run with: cargo run --example auto_connect --features discovery");
    std::process::exit(1);
}
