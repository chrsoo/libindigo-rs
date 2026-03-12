//! Example: Simple server discovery
//!
//! This example demonstrates basic one-shot server discovery.
//!
//! Run with:
//! ```bash
//! cargo run --example discover_servers --features discovery
//! ```

#[cfg(feature = "discovery")]
use libindigo_rs::discovery::{DiscoveryConfig, ServerDiscoveryApi};
#[cfg(feature = "discovery")]
use libindigo_rs::{ClientBuilder, RsClientStrategy};

#[cfg(feature = "discovery")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Simple discovery with default settings (5 second timeout)
    println!("Discovering INDIGO servers...");

    let config = DiscoveryConfig::new();
    let servers = ServerDiscoveryApi::discover(config).await?;

    println!("\nFound {} INDIGO server(s):", servers.len());
    for server in &servers {
        println!("\n  Name: {}", server.name);
        println!("  URL:  {}", server.url());
        println!("  Host: {}", server.host);
        println!("  Port: {}", server.port);

        if !server.addresses.is_empty() {
            println!("  Addresses:");
            for addr in &server.addresses {
                println!("    - {}", addr);
            }
        }

        if !server.txt_records.is_empty() {
            println!("  TXT Records:");
            for (key, value) in &server.txt_records {
                println!("    {}: {}", key, value);
            }
        }
    }

    // Try to connect to the first server if available
    if let Some(server) = servers.first() {
        println!("\n\nConnecting to {}...", server.name);

        let strategy = Box::new(RsClientStrategy::new());
        let mut client = ClientBuilder::new().with_strategy(strategy).build()?;

        client.strategy_mut().connect(&server.url()).await?;
        println!("Connected successfully!");

        client.strategy_mut().enumerate_properties(None).await?;
        println!("Enumerated properties");

        client.strategy_mut().disconnect().await?;
        println!("Disconnected");
    } else {
        println!("\nNo servers found. Make sure an INDIGO server is running on your network.");
    }

    Ok(())
}

#[cfg(not(feature = "discovery"))]
fn main() {
    eprintln!("This example requires the 'discovery' feature.");
    eprintln!("Run with: cargo run --example discover_servers --features discovery");
    std::process::exit(1);
}
