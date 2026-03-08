//! Property Streaming Example
//!
//! This example demonstrates how to subscribe to property updates from an INDIGO server
//! using the multi-subscriber pattern. Multiple subscribers can receive property updates
//! simultaneously without interfering with each other.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example property_streaming
//! ```
//!
//! # Prerequisites
//!
//! - An INDIGO server must be running on localhost:7624
//! - At least one device should be connected to the server
//!
//! # What This Example Does
//!
//! 1. Connects to an INDIGO server
//! 2. Creates multiple property subscribers
//! 3. Receives and displays property updates in real-time
//! 4. Demonstrates proper error handling and graceful shutdown

use libindigo::client::strategy::ClientStrategy;
use libindigo::error::Result;
use libindigo_rs::RsClientStrategy;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging for better visibility
    env_logger::init();

    println!("=== INDIGO Property Streaming Example ===\n");

    // Create the Rust client strategy
    let mut strategy = RsClientStrategy::new();

    // Connect to the INDIGO server
    println!("Connecting to INDIGO server at localhost:7624...");
    match strategy.connect("localhost:7624").await {
        Ok(_) => println!("✓ Connected successfully!\n"),
        Err(e) => {
            eprintln!("✗ Failed to connect: {}", e);
            eprintln!("\nMake sure an INDIGO server is running on localhost:7624");
            eprintln!("You can start one with: indigo_server");
            return Err(e);
        }
    }

    // Subscribe to property updates
    // This creates a new receiver channel that will receive all property updates
    println!("Subscribing to property updates...");
    let mut property_receiver = strategy.subscribe_properties().await;
    println!("✓ Subscribed to property stream\n");

    // You can create multiple subscribers if needed
    // Each subscriber will receive all property updates independently
    let mut second_subscriber = strategy.subscribe_properties().await;

    // Spawn a task to handle the second subscriber
    // This demonstrates that multiple subscribers can work simultaneously
    tokio::spawn(async move {
        let mut count = 0;
        while let Some(property) = second_subscriber.recv().await {
            count += 1;
            if count <= 3 {
                println!(
                    "[Subscriber 2] Received property: {}.{} (state: {:?})",
                    property.device, property.name, property.state
                );
            }
        }
    });

    println!("Listening for property updates (Ctrl+C to stop)...\n");
    println!("---");

    // Main loop: receive and display property updates
    let mut property_count = 0;
    let max_properties = 20; // Limit for demo purposes

    while let Some(property) = property_receiver.recv().await {
        property_count += 1;

        // Display property information
        println!(
            "Property #{}: {}.{}",
            property_count, property.device, property.name
        );
        println!("  Type:      {:?}", property.property_type);
        println!("  State:     {:?}", property.state);
        println!("  Group:     {}", property.group);
        println!("  Label:     {}", property.label);
        println!("  Items:     {} item(s)", property.items.len());

        // Display first few items as examples
        let mut item_count = 0;
        for (name, item) in &property.items {
            if item_count < 3 {
                println!("    - {}: {:?}", name, item.value);
                item_count += 1;
            }
        }
        if property.items.len() > 3 {
            println!("    ... and {} more items", property.items.len() - 3);
        }

        println!("---");

        // Stop after receiving a certain number of properties (for demo)
        if property_count >= max_properties {
            println!("\nReceived {} properties. Stopping demo.", max_properties);
            break;
        }
    }

    // Graceful shutdown
    println!("\nDisconnecting from server...");
    strategy.disconnect().await?;
    println!("✓ Disconnected successfully");

    // Give the second subscriber task time to finish
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\n=== Example Complete ===");
    Ok(())
}
