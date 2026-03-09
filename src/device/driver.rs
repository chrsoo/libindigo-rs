//! Core DeviceDriver trait and related types.

use super::context::DeviceContext;
use crate::error::Result;
use crate::types::{BlobTransferMode, Property};

/// Information about a device driver
#[derive(Debug, Clone)]
pub struct DriverInfo {
    /// Unique name for this driver
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Driver version string
    pub version: String,
    /// INDIGO device interfaces this driver implements (bitmask)
    pub interfaces: u32,
}

/// INDIGO device interface types (bitmask values matching INDIGO C API)
/// These are the standard INDIGO interface types from indigo_driver.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DeviceInterface {
    /// General device
    General = 0,
    /// CCD camera
    Ccd = 1 << 0,
    /// Filter wheel
    FilterWheel = 1 << 1,
    /// Focuser
    Focuser = 1 << 2,
    /// Mount
    Mount = 1 << 3,
    /// Guider
    Guider = 1 << 4,
    /// Dome
    Dome = 1 << 5,
    /// GPS
    Gps = 1 << 6,
    /// Weather station
    Weather = 1 << 7,
    /// Auxiliary device
    Aux = 1 << 8,
    /// Rotator
    Rotator = 1 << 9,
    /// Adaptive optics
    Ao = 1 << 10,
    /// Flat panel
    FlatPanel = 1 << 11,
    /// Power/relay board
    PowerBoard = 1 << 12,
}

impl DeviceInterface {
    /// Combine multiple interfaces into a bitmask
    pub fn combine(interfaces: &[DeviceInterface]) -> u32 {
        interfaces.iter().fold(0u32, |acc, i| acc | *i as u32)
    }
}

/// The core Device Driver trait — the SPI for implementing INDIGO devices in Rust.
///
/// Implementors of this trait can register properties, respond to client requests,
/// and manage device state through the INDIGO protocol.
///
/// # Lifecycle
///
/// 1. `attach()` — Called when the driver is loaded. Register properties here.
/// 2. `change_property()` — Called when a client modifies a property value.
/// 3. `enable_blob()` — Called when a client requests BLOB mode change.
/// 4. `detach()` — Called when the driver is being unloaded. Clean up resources.
///
/// # Example
///
/// ```rust,no_run
/// use libindigo::device::{DeviceDriver, DeviceContext, DriverInfo, DeviceInterface};
/// use libindigo::error::Result;
/// use libindigo::types::{Property, PropertyState};
///
/// struct MyCamera;
///
/// #[async_trait::async_trait]
/// impl DeviceDriver for MyCamera {
///     fn info(&self) -> DriverInfo {
///         DriverInfo {
///             name: "My Camera".into(),
///             description: "Example camera driver".into(),
///             version: "1.0.0".into(),
///             interfaces: DeviceInterface::Ccd as u32,
///         }
///     }
///
///     async fn attach(&mut self, ctx: &mut DeviceContext) -> Result<()> {
///         // Register properties during attach
///         ctx.property_manager().register_standard_connection()?;
///         Ok(())
///     }
///
///     async fn change_property(&mut self, ctx: &mut DeviceContext, property: &Property) -> Result<()> {
///         // Handle property changes from clients
///         Ok(())
///     }
///
///     async fn detach(&mut self, ctx: &mut DeviceContext) -> Result<()> {
///         Ok(())
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait DeviceDriver: Send + Sync {
    /// Return driver metadata/info
    fn info(&self) -> DriverInfo;

    /// Called when the driver is attached/loaded.
    /// Use this to register properties via the DeviceContext.
    async fn attach(&mut self, ctx: &mut DeviceContext) -> Result<()>;

    /// Called when a client changes a property value.
    /// The property contains the new requested values.
    async fn change_property(&mut self, ctx: &mut DeviceContext, property: &Property)
        -> Result<()>;

    /// Called when a client requests a BLOB mode change.
    /// Default implementation accepts all modes.
    async fn enable_blob(
        &mut self,
        _ctx: &mut DeviceContext,
        _device: &str,
        _name: Option<&str>,
        _mode: BlobTransferMode,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when the driver is being detached/unloaded.
    /// Clean up resources here.
    async fn detach(&mut self, ctx: &mut DeviceContext) -> Result<()>;
}
