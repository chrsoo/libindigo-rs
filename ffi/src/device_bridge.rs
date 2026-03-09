//! Bridge between Rust DeviceDriver implementations and the C INDIGO driver API.
//!
//! This module allows Rust device drivers to be registered with the C INDIGO
//! server, converting between the C callback model and the Rust async trait model.
//!
//! # Architecture
//!
//! The bridge works by:
//!
//! 1. Wrapping a Rust `DeviceDriver` implementation
//! 2. Registering C callbacks with the INDIGO server
//! 3. Translating C callbacks into Rust trait method calls
//! 4. Managing the lifecycle of the driver within the C server
//!
//! # Example
//!
//! ```ignore
//! use libindigo_ffi::device_bridge::FfiDriverBridge;
//! use libindigo::device::DeviceDriver;
//!
//! struct MyDriver;
//! impl DeviceDriver for MyDriver {
//!     // ... implementation
//! }
//!
//! let driver = Box::new(MyDriver);
//! let bridge = FfiDriverBridge::new(driver);
//! bridge.register().await?;
//! ```

use libindigo::device::{DeviceContext, DeviceDriver, DriverInfo};
use libindigo::error::{IndigoError, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Bridge that wraps a Rust DeviceDriver for use with the C INDIGO server.
///
/// This bridge manages the lifecycle of a Rust device driver within the
/// C INDIGO server environment, handling the translation between C callbacks
/// and Rust async trait methods.
///
/// # Example
///
/// ```ignore
/// use libindigo_ffi::device_bridge::FfiDriverBridge;
///
/// let driver = Box::new(MyDriver::new());
/// let bridge = FfiDriverBridge::new(driver);
///
/// // Register with the C INDIGO server
/// bridge.register().await?;
///
/// // Driver is now active and will receive callbacks
///
/// // When done, unregister
/// bridge.unregister().await?;
/// ```
pub struct FfiDriverBridge {
    /// The wrapped Rust device driver.
    driver: Arc<Mutex<Box<dyn DeviceDriver>>>,

    /// Driver information cached from the driver.
    info: DriverInfo,

    /// Whether the driver is currently registered.
    registered: Arc<Mutex<bool>>,
}

impl FfiDriverBridge {
    /// Creates a new bridge wrapping a Rust device driver.
    ///
    /// The driver's `info()` method is called immediately to cache
    /// the driver metadata.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let driver = Box::new(MyDriver::new());
    /// let bridge = FfiDriverBridge::new(driver);
    /// ```
    pub fn new(driver: Box<dyn DeviceDriver>) -> Self {
        let info = driver.info();
        Self {
            driver: Arc::new(Mutex::new(driver)),
            info,
            registered: Arc::new(Mutex::new(false)),
        }
    }

    /// Returns the driver information.
    pub fn info(&self) -> &DriverInfo {
        &self.info
    }

    /// Registers this driver with the C INDIGO server.
    ///
    /// This method:
    /// 1. Checks if the driver is already registered
    /// 2. Calls the C INDIGO driver registration function
    /// 3. Sets up callbacks for property changes
    /// 4. Calls the driver's `attach()` method
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The driver is already registered
    /// - The C INDIGO library is not available
    /// - Registration with the C server fails
    /// - The driver's `attach()` method fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bridge = FfiDriverBridge::new(driver);
    /// bridge.register().await?;
    /// ```
    #[cfg(feature = "sys-available")]
    pub async fn register(&self) -> Result<()> {
        let mut registered = self.registered.lock().await;
        if *registered {
            return Err(IndigoError::DriverAlreadyRegistered(self.info.name.clone()));
        }

        info!("Registering FFI driver: {}", self.info.name);

        // TODO: Implement actual C INDIGO driver registration
        // This would involve:
        // 1. Allocating a C indigo_driver structure
        // 2. Setting up callback function pointers
        // 3. Calling indigo_attach_driver()
        // 4. Creating a DeviceContext for the driver
        // 5. Calling driver.attach(ctx)

        warn!("FFI driver registration not yet fully implemented");

        // For now, just call the driver's attach method with a mock context
        let mut driver = self.driver.lock().await;
        let mut ctx = DeviceContext::new(self.info.name.clone());
        driver.attach(&mut ctx).await?;

        *registered = true;
        Ok(())
    }

    /// Stub implementation when sys crate is not available.
    #[cfg(not(feature = "sys-available"))]
    pub async fn register(&self) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI not available - sys crate not built".to_string(),
        ))
    }

    /// Unregisters this driver from the C INDIGO server.
    ///
    /// This method:
    /// 1. Checks if the driver is registered
    /// 2. Calls the driver's `detach()` method
    /// 3. Unregisters from the C INDIGO server
    /// 4. Cleans up callbacks and resources
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The driver is not registered
    /// - The driver's `detach()` method fails
    /// - Unregistration from the C server fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// bridge.unregister().await?;
    /// ```
    #[cfg(feature = "sys-available")]
    pub async fn unregister(&self) -> Result<()> {
        let mut registered = self.registered.lock().await;
        if !*registered {
            return Err(IndigoError::DriverNotAttached(self.info.name.clone()));
        }

        info!("Unregistering FFI driver: {}", self.info.name);

        // Call the driver's detach method
        let mut driver = self.driver.lock().await;
        let mut ctx = DeviceContext::new(self.info.name.clone());
        driver.detach(&mut ctx).await?;

        // TODO: Implement actual C INDIGO driver unregistration
        // This would involve:
        // 1. Calling indigo_detach_driver()
        // 2. Freeing the C indigo_driver structure
        // 3. Cleaning up callbacks

        warn!("FFI driver unregistration not yet fully implemented");

        *registered = false;
        Ok(())
    }

    /// Stub implementation when sys crate is not available.
    #[cfg(not(feature = "sys-available"))]
    pub async fn unregister(&self) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI not available - sys crate not built".to_string(),
        ))
    }

    /// Checks if the driver is currently registered.
    pub async fn is_registered(&self) -> bool {
        *self.registered.lock().await
    }
}

// ============================================================================
// C Callback Handlers for Device Drivers
// ============================================================================
//
// These functions would be registered with the C INDIGO server and called
// when property changes occur. They need to translate C calls into Rust
// trait method calls.

/// Context structure passed to C callbacks.
///
/// This contains the information needed to route callbacks back to the
/// appropriate Rust driver instance.
#[repr(C)]
struct DriverCallbackContext {
    /// Pointer to the FfiDriverBridge (as Arc).
    /// This is stored as a raw pointer but must be carefully managed.
    _bridge_ptr: *const (),
}

/// C callback for property change requests.
///
/// # Safety
///
/// This function is called by C code and must handle all errors gracefully.
/// The pointers must be valid for the duration of the call.
#[cfg(feature = "sys-available")]
#[no_mangle]
pub unsafe extern "C" fn indigo_driver_change_property_callback(
    _device: *mut libindigo_sys::indigo_device,
    _client: *mut libindigo_sys::indigo_client,
    _property: *mut libindigo_sys::indigo_property,
) -> libindigo_sys::indigo_result {
    // TODO: Implement property change callback
    // This would:
    // 1. Extract the DriverCallbackContext from device user data
    // 2. Convert the C property to Rust Property
    // 3. Call driver.change_property() in an async context
    // 4. Return the appropriate result code

    debug!("Driver property change callback received");

    // For now, return success
    libindigo_sys::indigo_result(0) // INDIGO_OK
}

/// Stub implementation when sys crate is not available.
#[cfg(not(feature = "sys-available"))]
pub unsafe extern "C" fn indigo_driver_change_property_callback() -> i32 {
    0 // Return success code
}

/// C callback for driver attach.
///
/// # Safety
///
/// This function is called by C code and must handle all errors gracefully.
#[cfg(feature = "sys-available")]
#[no_mangle]
pub unsafe extern "C" fn indigo_driver_attach_callback(
    _device: *mut libindigo_sys::indigo_device,
) -> libindigo_sys::indigo_result {
    debug!("Driver attach callback received");
    libindigo_sys::indigo_result(0) // INDIGO_OK
}

/// Stub implementation when sys crate is not available.
#[cfg(not(feature = "sys-available"))]
pub unsafe extern "C" fn indigo_driver_attach_callback() -> i32 {
    0
}

/// C callback for driver detach.
///
/// # Safety
///
/// This function is called by C code and must handle all errors gracefully.
#[cfg(feature = "sys-available")]
#[no_mangle]
pub unsafe extern "C" fn indigo_driver_detach_callback(
    _device: *mut libindigo_sys::indigo_device,
) -> libindigo_sys::indigo_result {
    debug!("Driver detach callback received");
    libindigo_sys::indigo_result(0) // INDIGO_OK
}

/// Stub implementation when sys crate is not available.
#[cfg(not(feature = "sys-available"))]
pub unsafe extern "C" fn indigo_driver_detach_callback() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use libindigo::device::{DeviceInterface, DriverInfo};
    use libindigo::types::Property;

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

        async fn attach(&mut self, _ctx: &mut DeviceContext) -> Result<()> {
            Ok(())
        }

        async fn change_property(
            &mut self,
            _ctx: &mut DeviceContext,
            _property: &Property,
        ) -> Result<()> {
            Ok(())
        }

        async fn detach(&mut self, _ctx: &mut DeviceContext) -> Result<()> {
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

    #[cfg(not(feature = "sys-available"))]
    #[tokio::test]
    async fn test_register_without_sys_fails() {
        let driver = Box::new(MockDriver {
            name: "Test Driver".to_string(),
        });
        let bridge = FfiDriverBridge::new(driver);
        let result = bridge.register().await;
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }
}
