//! Integration tests for libindigo Phase 2: Async FFI Strategy
//!
//! These tests verify the async FFI client strategy implementation.
//! Tests that require a running INDIGO server use the test harness for automatic
//! server management.

use libindigo::client::ClientBuilder;
use libindigo::error::IndigoError;
use libindigo::types::{Property, PropertyPerm, PropertyState, PropertyType};

#[path = "harness/mod.rs"]
mod harness;
use harness::TestHarness;

/// Test that we can create a client with async FFI strategy.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[tokio::test]
async fn test_create_async_ffi_client() {
    let result = ClientBuilder::new().with_async_ffi_strategy().build();
    assert!(result.is_ok(), "Failed to create async FFI client");
}

/// Test that creating a client without a strategy fails.
#[tokio::test]
async fn test_create_client_without_strategy_fails() {
    let result = ClientBuilder::new().build();
    assert!(result.is_err(), "Should fail without strategy");

    match result {
        Err(IndigoError::InvalidParameter(msg)) => {
            assert!(msg.contains("No strategy configured"));
        }
        Err(e) => panic!("Expected InvalidParameter error, got: {:?}", e),
        Ok(_) => panic!("Expected error but got Ok"),
    }
}

/// Test that connecting with an invalid URL fails.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[tokio::test]
async fn test_connect_invalid_url() {
    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // Test invalid URL format (missing port)
    let result = client.connect("localhost").await;
    assert!(result.is_err(), "Should fail with invalid URL");

    if let Err(IndigoError::InvalidParameter(msg)) = result {
        assert!(msg.contains("Invalid URL format"));
    } else {
        panic!("Expected InvalidParameter error");
    }
}

/// Test that connecting with an invalid port fails.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[tokio::test]
async fn test_connect_invalid_port() {
    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // Test invalid port number
    let result = client.connect("localhost:invalid").await;
    assert!(result.is_err(), "Should fail with invalid port");

    if let Err(IndigoError::InvalidParameter(msg)) = result {
        assert!(msg.contains("Invalid port number"));
    } else {
        panic!("Expected InvalidParameter error");
    }
}

/// Test that operations fail when not connected.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[tokio::test]
async fn test_operations_fail_when_not_connected() {
    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // Test disconnect without connecting
    let result = client.disconnect().await;
    assert!(result.is_err(), "Should fail when not connected");

    if let Err(IndigoError::InvalidState(msg)) = result {
        assert!(msg.contains("Not connected"));
    } else {
        panic!("Expected InvalidState error");
    }

    // Test enumerate_properties without connecting
    let result = client.enumerate_properties(None).await;
    assert!(result.is_err(), "Should fail when not connected");

    // Test send_property without connecting
    let property = Property::builder()
        .device("Test Device")
        .name("TEST_PROPERTY")
        .property_type(PropertyType::Switch)
        .build()
        .expect("Failed to build property");

    let result = client.send_property(property).await;
    assert!(result.is_err(), "Should fail when not connected");
}

/// Test connecting to a real INDIGO server using the test harness.
///
/// This test uses the test harness to automatically manage the INDIGO server.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[tokio::test]
async fn test_connect_to_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test harness
    TestHarness::initialize().await?;

    // Skip if harness not available
    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return Ok(());
    }

    // Reset state for test isolation
    TestHarness::reset_for_test().await?;

    // Get server address from harness
    let addr = TestHarness::server_address()?;

    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // Connect to INDIGO server
    let result = client.connect(&addr).await;
    assert!(result.is_ok(), "Failed to connect to server: {:?}", result);

    // Disconnect
    let result = client.disconnect().await;
    assert!(result.is_ok(), "Failed to disconnect: {:?}", result);

    Ok(())
}

/// Test enumerating properties from a real INDIGO server using the test harness.
///
/// This test uses the test harness to automatically manage the INDIGO server.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[tokio::test]
async fn test_enumerate_properties() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test harness
    TestHarness::initialize().await?;

    // Skip if harness not available
    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return Ok(());
    }

    // Reset state for test isolation
    TestHarness::reset_for_test().await?;

    // Get server address from harness
    let addr = TestHarness::server_address()?;

    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // Connect to INDIGO server
    client
        .connect(&addr)
        .await
        .expect("Failed to connect to server");

    // Enumerate all properties
    let result = client.enumerate_properties(None).await;
    assert!(
        result.is_ok(),
        "Failed to enumerate properties: {:?}",
        result
    );

    // Enumerate properties for a specific device (if it exists)
    let result = client.enumerate_properties(Some("CCD Simulator")).await;
    // This may fail if the device doesn't exist, which is okay

    // Disconnect
    client.disconnect().await.expect("Failed to disconnect");

    Ok(())
}

/// Test sending a property to a real INDIGO server using the test harness.
///
/// This test uses the test harness to automatically manage the INDIGO server.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[tokio::test]
async fn test_send_property() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test harness
    TestHarness::initialize().await?;

    // Skip if harness not available
    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return Ok(());
    }

    // Reset state for test isolation
    TestHarness::reset_for_test().await?;

    // Get server address from harness
    let addr = TestHarness::server_address()?;

    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // Connect to INDIGO server
    client
        .connect(&addr)
        .await
        .expect("Failed to connect to server");

    // Create a test property
    let property = Property::builder()
        .device("CCD Simulator")
        .name("CONNECTION")
        .group("Main")
        .label("Connection")
        .state(PropertyState::Idle)
        .perm(PropertyPerm::ReadWrite)
        .property_type(PropertyType::Switch)
        .build()
        .expect("Failed to build property");

    // Send the property
    let result = client.send_property(property).await;
    assert!(result.is_ok(), "Failed to send property: {:?}", result);

    // Disconnect
    client.disconnect().await.expect("Failed to disconnect");

    Ok(())
}

/// Test that we can't connect twice using the test harness.
///
/// This test uses the test harness to automatically manage the INDIGO server.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[tokio::test]
async fn test_cannot_connect_twice() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test harness
    TestHarness::initialize().await?;

    // Skip if harness not available
    if !TestHarness::is_available() {
        eprintln!("Skipping test: INDIGO server not available");
        return Ok(());
    }

    // Reset state for test isolation
    TestHarness::reset_for_test().await?;

    // Get server address from harness
    let addr = TestHarness::server_address()?;

    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // First connection should succeed
    client
        .connect(&addr)
        .await
        .expect("Failed to connect to server");

    // Second connection should fail
    let result = client.connect(&addr).await;
    assert!(result.is_err(), "Should not be able to connect twice");

    if let Err(IndigoError::InvalidState(msg)) = result {
        assert!(msg.contains("Already connected"));
    } else {
        panic!("Expected InvalidState error");
    }

    // Cleanup
    client.disconnect().await.expect("Failed to disconnect");

    Ok(())
}

/// Test builder with different strategies.
/// Note: This test is disabled because the API has changed.
/// The builder now requires a strategy instance, not a method call.
#[cfg(feature = "ffi-strategy")]
#[test]
#[ignore = "API changed - builder requires strategy instance"]
fn test_builder_with_ffi_strategy() {
    // Old API: let result = ClientBuilder::new().with_ffi_strategy().build();
    // New API requires: ClientBuilder::new().with_strategy(Box::new(strategy)).build()
    // This test needs to be updated to use the new API
}

/// Test default builder.
#[test]
fn test_default_builder() {
    let builder = ClientBuilder::default();
    let result = builder.build();
    assert!(
        result.is_err(),
        "Default builder should fail without strategy"
    );
}

/// Test property builder for integration tests.
#[test]
fn test_property_builder() {
    let property = Property::builder()
        .device("Test Device")
        .name("TEST_PROPERTY")
        .group("Test Group")
        .label("Test Property")
        .state(PropertyState::Ok)
        .perm(PropertyPerm::ReadWrite)
        .property_type(PropertyType::Number)
        .timeout(10.0)
        .timestamp("2024-01-01T00:00:00")
        .message("Test message")
        .build()
        .expect("Failed to build property");

    assert_eq!(property.device, "Test Device");
    assert_eq!(property.name, "TEST_PROPERTY");
    assert_eq!(property.group, "Test Group");
    assert_eq!(property.label, "Test Property");
    assert_eq!(property.state, PropertyState::Ok);
    assert_eq!(property.perm, PropertyPerm::ReadWrite);
    assert_eq!(property.property_type, PropertyType::Number);
    assert_eq!(property.timeout, Some(10.0));
    assert_eq!(property.timestamp, Some("2024-01-01T00:00:00".to_string()));
    assert_eq!(property.message, Some("Test message".to_string()));
}

/// Test property builder error handling - missing device.
#[test]
fn test_property_builder_missing_device() {
    let result = Property::builder()
        .name("TEST_PROPERTY")
        .property_type(PropertyType::Number)
        .build();

    assert!(result.is_err(), "Should fail when device is missing");
    if let Err(e) = result {
        assert_eq!(
            e,
            libindigo::error::PropertyBuilderError::MissingDevice,
            "Expected MissingDevice error"
        );
    }
}

/// Test property builder error handling - missing name.
#[test]
fn test_property_builder_missing_name() {
    let result = Property::builder()
        .device("Test Device")
        .property_type(PropertyType::Number)
        .build();

    assert!(result.is_err(), "Should fail when name is missing");
    if let Err(e) = result {
        assert_eq!(
            e,
            libindigo::error::PropertyBuilderError::MissingName,
            "Expected MissingName error"
        );
    }
}

/// Test property builder error handling - missing property_type.
#[test]
fn test_property_builder_missing_property_type() {
    let result = Property::builder()
        .device("Test Device")
        .name("TEST_PROPERTY")
        .build();

    assert!(result.is_err(), "Should fail when property_type is missing");
    if let Err(e) = result {
        assert_eq!(
            e,
            libindigo::error::PropertyBuilderError::MissingPropertyType,
            "Expected MissingPropertyType error"
        );
    }
}

/// Test property builder error handling - all fields missing.
#[test]
fn test_property_builder_all_fields_missing() {
    let result = Property::builder().build();

    assert!(
        result.is_err(),
        "Should fail when all required fields are missing"
    );
    // Should fail on the first missing field (device)
    if let Err(e) = result {
        assert_eq!(
            e,
            libindigo::error::PropertyBuilderError::MissingDevice,
            "Expected MissingDevice error"
        );
    }
}

/// Test property builder success with all required fields.
#[test]
fn test_property_builder_success_minimal() {
    let result = Property::builder()
        .device("Test Device")
        .name("TEST_PROPERTY")
        .property_type(PropertyType::Number)
        .build();

    assert!(result.is_ok(), "Should succeed with all required fields");
    let property = result.unwrap();
    assert_eq!(property.device, "Test Device");
    assert_eq!(property.name, "TEST_PROPERTY");
    assert_eq!(property.property_type, PropertyType::Number);
    // Optional fields should have defaults
    assert_eq!(property.group, "");
    assert_eq!(property.label, "");
}

/// Test that client strategy can be accessed.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[test]
fn test_client_strategy_access() {
    let client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // Test that we can access the strategy
    let _strategy = client.strategy();
}

/// Test that client strategy can be mutably accessed.
#[cfg(all(feature = "ffi-strategy", feature = "async"))]
#[test]
fn test_client_strategy_mut_access() {
    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()
        .expect("Failed to create client");

    // Test that we can mutably access the strategy
    let _strategy = client.strategy_mut();
}
