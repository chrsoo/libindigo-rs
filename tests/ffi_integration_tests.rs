//! Integration tests for FFI implementation.
//!
//! These tests verify the FFI integration layer between Rust and the C INDIGO library.
//! Tests are conditional on the `sys-available` feature, which indicates whether the
//! C library was successfully built.

#![cfg(test)]

use libindigo::error::IndigoError;
use libindigo::types::{BlobTransferMode, Property, PropertyState, PropertyType};
use std::collections::HashMap;

// ============================================================================
// Conversion Tests
// ============================================================================

#[cfg(feature = "sys-available")]
mod conversion_tests {
    use super::*;
    use libindigo_ffi::conversion::*;

    #[test]
    fn test_blob_mode_conversions() {
        assert_eq!(blob_mode_to_c_str(BlobTransferMode::Never), "Never");
        assert_eq!(blob_mode_to_c_str(BlobTransferMode::Also), "Also");
        assert_eq!(blob_mode_to_c_str(BlobTransferMode::Only), "Only");

        assert_eq!(
            blob_mode_from_c_str("Never").unwrap(),
            BlobTransferMode::Never
        );
        assert_eq!(
            blob_mode_from_c_str("Also").unwrap(),
            BlobTransferMode::Also
        );
        assert_eq!(
            blob_mode_from_c_str("Only").unwrap(),
            BlobTransferMode::Only
        );
    }

    #[test]
    fn test_invalid_blob_mode() {
        let result = blob_mode_from_c_str("Invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_string_to_c_string() {
        let s = "test string";
        let c_str = string_to_c_string(s).unwrap();
        assert_eq!(c_str.to_str().unwrap(), s);
    }

    #[test]
    fn test_string_with_null_byte_fails() {
        let s = "test\0string";
        let result = string_to_c_string(s);
        assert!(result.is_err());
    }

    #[test]
    fn test_property_from_c_null_pointer() {
        let result = unsafe { property_from_c(std::ptr::null()) };
        assert!(result.is_err());
        match result {
            Err(IndigoError::InvalidParameter(_)) => (),
            _ => panic!("Expected InvalidParameter error"),
        }
    }
}

#[cfg(not(feature = "sys-available"))]
mod conversion_tests_no_sys {
    use super::*;
    use libindigo_ffi::conversion::*;

    #[test]
    fn test_property_from_c_not_available() {
        let result = unsafe { property_from_c(std::ptr::null()) };
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }

    #[test]
    fn test_property_to_c_not_available() {
        let prop = Property {
            device: "Test".to_string(),
            name: "TEST".to_string(),
            group: "Test".to_string(),
            label: "Test".to_string(),
            state: PropertyState::Idle,
            perm: libindigo::types::PropertyPerm::ReadWrite,
            property_type: PropertyType::Text,
            items: HashMap::new(),
            timeout: None,
            timestamp: None,
            message: None,
        };

        let result = unsafe { property_to_c(&prop) };
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }
}

// ============================================================================
// Callback Handler Tests
// ============================================================================

mod callback_tests {
    use super::*;
    use libindigo_ffi::callback::{CallbackHandler, FfiEvent};

    #[test]
    fn test_callback_handler_creation() {
        let handler = CallbackHandler::new();
        let _rx = handler.subscribe();
        // Handler should be created successfully
    }

    #[test]
    fn test_multiple_subscribers() {
        let handler = CallbackHandler::new();
        let _rx1 = handler.subscribe();
        let _rx2 = handler.subscribe();
        let _rx3 = handler.subscribe();
        // Multiple subscribers should work
    }

    #[tokio::test]
    async fn test_event_forwarding() {
        let handler = CallbackHandler::new();
        let mut rx = handler.subscribe();

        // Get the sync sender
        let sender = handler.sync_sender().unwrap();

        // Start the bridge task
        let bridge_handle = tokio::spawn(async move {
            handler.start().await.unwrap();
        });

        // Send a test event
        let test_event = FfiEvent::ConnectionChanged(true);
        sender.send(test_event).unwrap();

        // Receive the event
        let received = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Failed to receive event");

        match received {
            FfiEvent::ConnectionChanged(true) => (),
            _ => panic!("Unexpected event received"),
        }

        // Clean up
        drop(sender);
        let _ = tokio::time::timeout(std::time::Duration::from_millis(100), bridge_handle).await;
    }
}

// ============================================================================
// FFI Client Strategy Tests
// ============================================================================

#[cfg(feature = "sys-available")]
mod ffi_strategy_tests {
    use super::*;
    use libindigo::client::ClientStrategy;
    use libindigo_ffi::FfiClientStrategy;

    #[test]
    fn test_ffi_strategy_creation() {
        let result = FfiClientStrategy::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_ffi_strategy_not_connected_initially() {
        let strategy = FfiClientStrategy::new().unwrap();
        assert!(!strategy.is_connected());
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        assert!(!strategy.is_connected());

        // Connect (stub implementation)
        let result = strategy.connect("localhost:7624").await;
        assert!(result.is_ok());
        assert!(strategy.is_connected());

        // Disconnect
        let result = strategy.disconnect().await;
        assert!(result.is_ok());
        assert!(!strategy.is_connected());
    }

    #[tokio::test]
    async fn test_double_connect_fails() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        strategy.connect("localhost:7624").await.unwrap();

        let result = strategy.connect("localhost:7624").await;
        assert!(result.is_err());
        match result {
            Err(IndigoError::InvalidState(_)) => (),
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[tokio::test]
    async fn test_disconnect_when_not_connected_fails() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        let result = strategy.disconnect().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enumerate_properties_when_not_connected_fails() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        let result = strategy.enumerate_properties(None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_property_when_not_connected_fails() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        let prop = Property {
            device: "Test".to_string(),
            name: "TEST".to_string(),
            group: "Test".to_string(),
            label: "Test".to_string(),
            state: PropertyState::Idle,
            perm: libindigo::types::PropertyPerm::ReadWrite,
            property_type: PropertyType::Text,
            items: HashMap::new(),
            timeout: None,
            timestamp: None,
            message: None,
        };

        let result = strategy.send_property(prop).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enable_blob_when_not_connected_fails() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        let result = strategy
            .enable_blob("Test", None, BlobTransferMode::Also)
            .await;
        assert!(result.is_err());
    }
}

#[cfg(not(feature = "sys-available"))]
mod ffi_strategy_tests_no_sys {
    use super::*;
    use libindigo_ffi::FfiClientStrategy;

    #[test]
    fn test_ffi_strategy_creation_fails_without_sys() {
        let result = FfiClientStrategy::new();
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }
}

// ============================================================================
// Async FFI Strategy Tests
// ============================================================================

#[cfg(all(feature = "sys-available", feature = "async"))]
mod async_ffi_strategy_tests {
    use super::*;
    use futures::StreamExt;
    use libindigo::client::ClientStrategy;
    use libindigo_ffi::AsyncFfiStrategy;

    #[test]
    fn test_async_ffi_strategy_creation() {
        let result = AsyncFfiStrategy::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_stream_creation() {
        let strategy = AsyncFfiStrategy::new().unwrap();
        let _stream = strategy.property_stream();
        // Stream should be created successfully
    }

    #[tokio::test]
    async fn test_multiple_streams() {
        let strategy = AsyncFfiStrategy::new().unwrap();
        let _stream1 = strategy.property_stream();
        let _stream2 = strategy.property_stream();
        // Multiple streams should work
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut strategy = AsyncFfiStrategy::new().unwrap();

        // Connect (stub implementation)
        let result = strategy.connect("localhost:7624").await;
        assert!(result.is_ok());

        // Disconnect
        let result = strategy.disconnect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enumerate_properties() {
        let mut strategy = AsyncFfiStrategy::new().unwrap();
        strategy.connect("localhost:7624").await.unwrap();

        let result = strategy.enumerate_properties(None).await;
        assert!(result.is_ok());

        let result = strategy.enumerate_properties(Some("Test Device")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_property() {
        let mut strategy = AsyncFfiStrategy::new().unwrap();
        strategy.connect("localhost:7624").await.unwrap();

        let prop = Property {
            device: "Test".to_string(),
            name: "TEST".to_string(),
            group: "Test".to_string(),
            label: "Test".to_string(),
            state: PropertyState::Idle,
            perm: libindigo::types::PropertyPerm::ReadWrite,
            property_type: PropertyType::Text,
            items: HashMap::new(),
            timeout: None,
            timestamp: None,
            message: None,
        };

        let result = strategy.send_property(prop).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enable_blob() {
        let mut strategy = AsyncFfiStrategy::new().unwrap();
        strategy.connect("localhost:7624").await.unwrap();

        let result = strategy
            .enable_blob("Test", None, BlobTransferMode::Also)
            .await;
        assert!(result.is_ok());

        let result = strategy
            .enable_blob("Test", Some("BLOB_PROP"), BlobTransferMode::Never)
            .await;
        assert!(result.is_ok());
    }
}

#[cfg(all(not(feature = "sys-available"), feature = "async"))]
mod async_ffi_strategy_tests_no_sys {
    use super::*;
    use libindigo_ffi::AsyncFfiStrategy;

    #[test]
    fn test_async_ffi_strategy_creation_fails_without_sys() {
        let result = AsyncFfiStrategy::new();
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }
}

// ============================================================================
// Device Bridge Tests
// ============================================================================

#[cfg(all(feature = "sys-available", feature = "device"))]
mod device_bridge_tests {
    use super::*;
    use libindigo::device::{DeviceContext, DeviceDriver, DeviceInterface, DriverInfo};
    use libindigo_ffi::device_bridge::FfiDriverBridge;

    // Mock driver for testing
    struct MockDriver {
        name: String,
    }

    #[async_trait::async_trait]
    impl DeviceDriver for MockDriver {
        fn info(&self) -> DriverInfo {
            DriverInfo {
                name: self.name.clone(),
                description: "Mock driver for testing".to_string(),
                version: "1.0.0".to_string(),
                interfaces: DeviceInterface::Ccd as u32,
            }
        }

        async fn attach(&mut self, _ctx: &mut DeviceContext) -> libindigo::error::Result<()> {
            Ok(())
        }

        async fn change_property(
            &mut self,
            _ctx: &mut DeviceContext,
            _property: &Property,
        ) -> libindigo::error::Result<()> {
            Ok(())
        }

        async fn detach(&mut self, _ctx: &mut DeviceContext) -> libindigo::error::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_bridge_creation() {
        let driver = Box::new(MockDriver {
            name: "Test Driver".to_string(),
        });
        let bridge = FfiDriverBridge::new(driver);
        assert_eq!(bridge.info().name, "Test Driver");
    }

    #[tokio::test]
    async fn test_bridge_not_registered_initially() {
        let driver = Box::new(MockDriver {
            name: "Test Driver".to_string(),
        });
        let bridge = FfiDriverBridge::new(driver);
        assert!(!bridge.is_registered().await);
    }

    #[tokio::test]
    async fn test_unregister_when_not_registered_fails() {
        let driver = Box::new(MockDriver {
            name: "Test Driver".to_string(),
        });
        let bridge = FfiDriverBridge::new(driver);
        let result = bridge.unregister().await;
        assert!(result.is_err());
    }
}

#[cfg(all(not(feature = "sys-available"), feature = "device"))]
mod device_bridge_tests_no_sys {
    use super::*;
    use libindigo::device::{DeviceContext, DeviceDriver, DeviceInterface, DriverInfo};
    use libindigo_ffi::device_bridge::FfiDriverBridge;

    struct MockDriver;

    #[async_trait::async_trait]
    impl DeviceDriver for MockDriver {
        fn info(&self) -> DriverInfo {
            DriverInfo {
                name: "Test".to_string(),
                description: "Test".to_string(),
                version: "1.0.0".to_string(),
                interfaces: DeviceInterface::Ccd as u32,
            }
        }

        async fn attach(&mut self, _ctx: &mut DeviceContext) -> libindigo::error::Result<()> {
            Ok(())
        }

        async fn change_property(
            &mut self,
            _ctx: &mut DeviceContext,
            _property: &Property,
        ) -> libindigo::error::Result<()> {
            Ok(())
        }

        async fn detach(&mut self, _ctx: &mut DeviceContext) -> libindigo::error::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_register_without_sys_fails() {
        let driver = Box::new(MockDriver);
        let bridge = FfiDriverBridge::new(driver);
        let result = bridge.register().await;
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }
}

// ============================================================================
// Feature Flag Tests
// ============================================================================

#[test]
fn test_sys_available_constant() {
    #[cfg(feature = "sys-available")]
    assert!(libindigo_ffi::SYS_AVAILABLE);

    #[cfg(not(feature = "sys-available"))]
    assert!(!libindigo_ffi::SYS_AVAILABLE);
}
