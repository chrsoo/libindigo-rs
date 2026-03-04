//! Device types and information.
//!
//! This module provides types for representing INDIGO devices and their metadata.

use std::collections::HashMap;

/// Represents an INDIGO device.
#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    /// Device name (unique identifier).
    pub name: String,

    /// Device information.
    pub info: DeviceInfo,

    /// Device properties (property name -> property).
    /// TODO: This will be populated in future phases.
    pub properties: HashMap<String, ()>,
}

/// Information about a device.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeviceInfo {
    /// Device interface bitmap.
    pub interface: u32,

    /// Device version.
    pub version: Option<String>,

    /// Device driver name.
    pub driver_name: Option<String>,

    /// Device driver version.
    pub driver_version: Option<String>,

    /// Device driver interface.
    pub driver_interface: Option<u32>,
}

impl Device {
    /// Creates a new device with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Device {
            name: name.into(),
            info: DeviceInfo::default(),
            properties: HashMap::new(),
        }
    }

    /// Creates a new device with the given name and info.
    pub fn with_info(name: impl Into<String>, info: DeviceInfo) -> Self {
        Device {
            name: name.into(),
            info,
            properties: HashMap::new(),
        }
    }
}

impl DeviceInfo {
    /// Creates a new device info with the given interface.
    pub fn new(interface: u32) -> Self {
        DeviceInfo {
            interface,
            ..Default::default()
        }
    }

    /// Sets the version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the driver name.
    pub fn with_driver_name(mut self, driver_name: impl Into<String>) -> Self {
        self.driver_name = Some(driver_name.into());
        self
    }

    /// Sets the driver version.
    pub fn with_driver_version(mut self, driver_version: impl Into<String>) -> Self {
        self.driver_version = Some(driver_version.into());
        self
    }

    /// Sets the driver interface.
    pub fn with_driver_interface(mut self, driver_interface: u32) -> Self {
        self.driver_interface = Some(driver_interface);
        self
    }
}
