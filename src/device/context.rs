//! Runtime context for device drivers.

use super::property_manager::PropertyManager;

/// Runtime context passed to device driver callbacks.
///
/// Provides access to the property manager and other runtime services
/// that device drivers need during their lifecycle.
pub struct DeviceContext {
    property_manager: PropertyManager,
    /// Whether the device is currently connected
    connected: bool,
}

impl DeviceContext {
    /// Create a new device context
    pub fn new(device_name: impl Into<String>) -> Self {
        Self {
            property_manager: PropertyManager::new(device_name),
            connected: false,
        }
    }

    /// Get the property manager
    pub fn property_manager(&mut self) -> &mut PropertyManager {
        &mut self.property_manager
    }

    /// Get read-only access to the property manager
    pub fn properties(&self) -> &PropertyManager {
        &self.property_manager
    }

    /// Check if the device is currently connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Set the connected state (called by the runtime, not usually by drivers directly)
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }
}
