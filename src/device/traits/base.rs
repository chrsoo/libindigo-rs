//! Base Device trait shared by all INDIGO device types.

use crate::device::DeviceInterface;
use crate::error::Result;
use crate::types::{Property, PropertyState};

/// Common interface shared by all INDIGO device types.
///
/// Provides basic operations like connection management, device info,
/// and generic property access.
#[async_trait::async_trait]
pub trait Device: Send + Sync {
    /// Get the device name
    fn name(&self) -> &str;

    /// Get the device type/interface
    fn device_type(&self) -> DeviceInterface;

    /// Connect to the device
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the device
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if the device is connected
    fn is_connected(&self) -> bool;

    /// Get device description/info
    fn description(&self) -> Option<&str>;

    /// Get driver version
    fn driver_version(&self) -> Option<&str>;

    /// Get a property by name
    async fn get_property(&self, name: &str) -> Result<Option<Property>>;

    /// Get all properties for this device
    async fn get_properties(&self) -> Result<Vec<Property>>;

    /// Wait for a property to reach a specific state, with timeout
    async fn wait_for_property_state(
        &self,
        name: &str,
        state: PropertyState,
        timeout: std::time::Duration,
    ) -> Result<Property>;
}
