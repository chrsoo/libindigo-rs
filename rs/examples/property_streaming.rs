//! Property Streaming Example
//!
//! This example demonstrates how to connect to an INDIGO server and stream
//! property updates in real-time using the subscribe_properties API.
//!
//! # Usage
//!
//! ```bash
//! # Run with default logging
//! cargo run --example property_streaming --features tracing-subscriber
//!
//! # Run with debug logging
//! RUST_LOG=libindigo_rs=debug cargo run --example property_streaming --features tracing-subscriber
//!
//! # Run with trace logging (very verbose)
//! RUST_LOG=libindigo_rs=trace cargo run --example property_streaming --features tracing-subscriber
//! ```
//!
//! # What This Example Shows
//!
//! 1. How to configure protocol negotiation (XML vs JSON)
//! 2. How to connect to an INDIGO server
//! 3. How to subscribe to property updates
//! 4. How to enumerate properties from the server
//! 5. How to receive and process property updates in real-time

use libindigo::client::strategy::ClientStrategy;
use libindigo_rs::protocol_negotiation::{ProtocolNegotiator, ProtocolType};
use libindigo_rs::RsClientStrategy;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("libindigo_rs=info")),
        )
        .init();

    // Configure the server address
    // Change this to your INDIGO server address
    let server_address =
        std::env::var("INDIGO_SERVER").unwrap_or_else(|_| "localhost:7624".to_string());

    println!("=== INDIGO Property Streaming Example ===\n");
    println!("Server: {}", server_address);

    // Create a client strategy with XML protocol
    // Note: Some servers only support XML, not JSON
    // For servers that support both, you can use the default (JSON-first with XML fallback)
    let negotiator = ProtocolNegotiator::new(ProtocolType::Xml, false);
    let mut strategy = RsClientStrategy::with_protocol_negotiator(negotiator);

    // Connect to the server
    println!("\nConnecting to {}...", server_address);
    strategy.connect(&server_address).await?;
    println!("✓ Connected successfully!\n");

    // Subscribe to property updates
    // This creates a channel that will receive all property updates from the server
    let mut property_stream = strategy.subscribe_properties().await;

    // Request all properties from the server
    // This sends a getProperties message which triggers the server to send
    // all current property definitions
    println!("Requesting properties from server...");
    strategy.enumerate_properties(None).await?;
    println!("✓ Request sent\n");

    // Receive and display properties
    println!("Receiving properties (will timeout after 5 seconds of inactivity):\n");
    println!("{:<30} {:<40} {:>6}", "Device", "Property", "Items");
    println!("{}", "=".repeat(80));

    let mut property_count = 0;
    let receive_timeout = Duration::from_secs(5);

    loop {
        match timeout(receive_timeout, property_stream.recv()).await {
            Ok(Some(property)) => {
                property_count += 1;
                println!(
                    "{:<30} {:<40} {:>6}",
                    property.device,
                    property.name,
                    property.items.len()
                );
            }
            Ok(None) => {
                println!("\n✗ Property stream closed");
                break;
            }
            Err(_) => {
                println!("\n✓ Timeout - no more properties received");
                break;
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("Total properties received: {}", property_count);

    // Optionally, you can request properties for a specific device
    if property_count > 0 {
        println!("\n--- Example: Requesting properties for 'Server' device ---");
        strategy.enumerate_properties(Some("Server")).await?;

        // Receive Server properties
        let mut server_count = 0;
        loop {
            match timeout(Duration::from_secs(2), property_stream.recv()).await {
                Ok(Some(property)) if property.device == "Server" => {
                    server_count += 1;
                    println!("  {} ({})", property.name, property.items.len());
                }
                Ok(Some(_)) => continue, // Skip non-Server properties
                _ => break,
            }
        }
        println!("Server properties: {}", server_count);
    }

    // Disconnect
    println!("\nDisconnecting...");
    strategy.disconnect().await?;
    println!("✓ Disconnected\n");

    Ok(())
}
