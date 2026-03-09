//! Integration tests for the mock INDIGO server.

mod mock_server;

use mock_server::MockServerBuilder;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::test]
async fn test_mock_server_basic() {
    // Start mock server with CCD simulator
    let server = MockServerBuilder::new()
        .with_ccd_simulator()
        .build()
        .await
        .expect("Failed to start mock server");

    let addr = server.addr();
    println!("Mock server listening on {}", addr);

    // Verify server is running
    assert!(addr.port() > 0);

    // Check devices were added
    let devices = server.list_devices().await;
    assert_eq!(devices.len(), 1);
    assert!(devices.contains(&"CCD Simulator".to_string()));

    // Shutdown server
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_mock_server_with_multiple_devices() {
    // Start mock server with multiple devices
    let server = MockServerBuilder::new()
        .with_ccd_simulator()
        .with_mount_simulator()
        .build()
        .await
        .expect("Failed to start mock server");

    let devices = server.list_devices().await;
    assert_eq!(devices.len(), 2);
    assert!(devices.contains(&"CCD Simulator".to_string()));
    assert!(devices.contains(&"Mount Simulator".to_string()));

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_mock_server_connection() {
    // Start mock server
    let server = MockServerBuilder::new()
        .with_ccd_simulator()
        .build()
        .await
        .expect("Failed to start mock server");

    let addr = server.addr();

    // Connect to server
    let stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to mock server");

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send getProperties request
    let request = r#"{"getProperties":{"version":512}}"#;
    writer
        .write_all(request.as_bytes())
        .await
        .expect("Failed to write request");
    writer
        .write_all(b"\n")
        .await
        .expect("Failed to write newline");
    writer.flush().await.expect("Failed to flush");

    // Read responses (should get property definitions)
    let mut line = String::new();
    let mut response_count = 0;

    // Read up to 10 responses or timeout after 1 second
    for _ in 0..10 {
        line.clear();
        match tokio::time::timeout(
            std::time::Duration::from_secs(1),
            reader.read_line(&mut line),
        )
        .await
        {
            Ok(Ok(n)) if n > 0 => {
                response_count += 1;
                println!("Received response {}: {}", response_count, line.trim());
            }
            _ => break,
        }
    }

    // Should have received at least one property definition
    assert!(
        response_count > 0,
        "Expected to receive property definitions"
    );

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_mock_server_property_update() {
    use mock_server::{NumberValue, PropertyUpdate, PropertyValue};

    // Start mock server
    let server = MockServerBuilder::new()
        .with_ccd_simulator()
        .build()
        .await
        .expect("Failed to start mock server");

    // Get initial temperature
    let property = server
        .get_property("CCD Simulator", "CCD_TEMPERATURE")
        .await
        .expect("Property not found");

    assert_eq!(property.name, "CCD_TEMPERATURE");

    // Update temperature
    let update = PropertyUpdate {
        state: Some(libindigo_rs::protocol::PropertyState::Ok),
        items: vec![(
            "CCD_TEMPERATURE_VALUE".to_string(),
            PropertyValue::Number(NumberValue {
                value: -10.5,
                format: "%.2f".to_string(),
                min: -50.0,
                max: 50.0,
                step: 0.1,
            }),
        )],
        message: Some("Temperature updated".to_string()),
    };

    server
        .update_property("CCD Simulator", "CCD_TEMPERATURE", update)
        .await
        .expect("Failed to update property");

    // Verify update
    let updated_property = server
        .get_property("CCD Simulator", "CCD_TEMPERATURE")
        .await
        .expect("Property not found");

    assert_eq!(
        updated_property.state,
        libindigo_rs::protocol::PropertyState::Ok
    );
    assert_eq!(
        updated_property.message,
        Some("Temperature updated".to_string())
    );

    // Check temperature value
    if let PropertyValue::Number(num) = &updated_property.items[0].value {
        assert_eq!(num.value, -10.5);
    } else {
        panic!("Expected Number value");
    }

    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_mock_server_stats() {
    // Start mock server
    let server = MockServerBuilder::new()
        .with_ccd_simulator()
        .build()
        .await
        .expect("Failed to start mock server");

    let addr = server.addr();

    // Connect to server
    let stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to mock server");

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send a request
    let request = r#"{"getProperties":{"version":512}}"#;
    writer.write_all(request.as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read one response
    let mut line = String::new();
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        reader.read_line(&mut line),
    )
    .await;

    // Check stats
    let stats = server.stats().await;
    assert!(stats.total_connections >= 1);
    assert!(stats.messages_received >= 1);
    assert!(stats.messages_sent >= 1);

    server.shutdown().await.expect("Failed to shutdown server");
}
