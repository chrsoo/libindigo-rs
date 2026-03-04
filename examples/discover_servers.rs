//! Example: Simple server discovery
//!
//! This example demonstrates basic one-shot server discovery.
//!
//! Run with:
//! ```bash
//! cargo run --example discover_servers --features auto
//! ```

#[cfg(feature = "auto")]
use libindigo::prelude::*;

#[cfg(feature = "auto")]
#[tokio::main]
async fn main() -> Result<()> {
    // Simple discovery with default settings (5 second timeout)
    println!("Discovering INDIGO servers...");

    let servers = Client::discover_servers().await?;

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

        let mut client = ClientBuilder::new().with_rs_strategy().build()?;

        client.connect(&server.url()).await?;
        println!("Connected successfully!");

        client.enumerate_properties(None).await?;
        println!("Enumerated properties");

        client.disconnect().await?;
        println!("Disconnected");
    } else {
        println!("\nNo servers found. Make sure an INDIGO server is running on your network.");
    }

    Ok(())
}

#[cfg(not(feature = "auto"))]
fn main() {
    eprintln!("This example requires the 'auto' feature.");
    eprintln!("Run with: cargo run --example discover_servers --features auto");
    std::process::exit(1);
}
