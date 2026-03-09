//! Driver registration and lifecycle management.

use super::context::DeviceContext;
use super::driver::{DeviceDriver, DriverInfo};
use crate::error::{IndigoError, Result};
use std::collections::HashMap;

/// Entry in the driver registry
struct DriverEntry {
    driver: Box<dyn DeviceDriver>,
    context: DeviceContext,
    attached: bool,
}

/// Registry for managing device driver lifecycle.
///
/// The DriverRegistry handles registration, attachment, detachment,
/// and routing of property changes to the appropriate drivers.
pub struct DriverRegistry {
    drivers: HashMap<String, DriverEntry>,
}

impl DriverRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
        }
    }

    /// Register a device driver. The driver is not yet attached.
    pub fn register(&mut self, driver: Box<dyn DeviceDriver>) -> Result<()> {
        let info = driver.info();
        if self.drivers.contains_key(&info.name) {
            return Err(IndigoError::DriverAlreadyRegistered(info.name));
        }
        let context = DeviceContext::new(&info.name);
        self.drivers.insert(
            info.name.clone(),
            DriverEntry {
                driver,
                context,
                attached: false,
            },
        );
        Ok(())
    }

    /// Attach (initialize) a registered driver
    pub async fn attach(&mut self, name: &str) -> Result<()> {
        let entry = self
            .drivers
            .get_mut(name)
            .ok_or_else(|| IndigoError::DriverNotFound(name.to_string()))?;
        if entry.attached {
            return Err(IndigoError::DriverAlreadyAttached(name.to_string()));
        }
        entry.driver.attach(&mut entry.context).await?;
        entry.attached = true;
        Ok(())
    }

    /// Attach all registered drivers
    pub async fn attach_all(&mut self) -> Result<()> {
        let names: Vec<String> = self.drivers.keys().cloned().collect();
        for name in names {
            self.attach(&name).await?;
        }
        Ok(())
    }

    /// Detach a driver, cleaning up its resources
    pub async fn detach(&mut self, name: &str) -> Result<()> {
        let entry = self
            .drivers
            .get_mut(name)
            .ok_or_else(|| IndigoError::DriverNotFound(name.to_string()))?;
        if !entry.attached {
            return Err(IndigoError::DriverNotAttached(name.to_string()));
        }
        entry.driver.detach(&mut entry.context).await?;
        entry.attached = false;
        Ok(())
    }

    /// Detach all attached drivers
    pub async fn detach_all(&mut self) -> Result<()> {
        let names: Vec<String> = self
            .drivers
            .iter()
            .filter(|(_, e)| e.attached)
            .map(|(n, _)| n.clone())
            .collect();
        for name in names {
            self.detach(&name).await?;
        }
        Ok(())
    }

    /// Route a property change to the appropriate driver
    pub async fn handle_property_change(
        &mut self,
        device: &str,
        property: &crate::types::Property,
    ) -> Result<()> {
        let entry = self
            .drivers
            .get_mut(device)
            .ok_or_else(|| IndigoError::DriverNotFound(device.to_string()))?;
        if !entry.attached {
            return Err(IndigoError::DriverNotAttached(device.to_string()));
        }
        entry
            .driver
            .change_property(&mut entry.context, property)
            .await
    }

    /// Get information about all registered drivers
    pub fn list_drivers(&self) -> Vec<(DriverInfo, bool)> {
        self.drivers
            .values()
            .map(|e| (e.driver.info(), e.attached))
            .collect()
    }

    /// Check if a driver is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.drivers.contains_key(name)
    }

    /// Check if a driver is attached
    pub fn is_attached(&self, name: &str) -> bool {
        self.drivers.get(name).map_or(false, |e| e.attached)
    }

    /// Get the number of registered drivers
    pub fn count(&self) -> usize {
        self.drivers.len()
    }

    /// Unregister a driver (must be detached first)
    pub fn unregister(&mut self, name: &str) -> Result<()> {
        let entry = self
            .drivers
            .get(name)
            .ok_or_else(|| IndigoError::DriverNotFound(name.to_string()))?;
        if entry.attached {
            return Err(IndigoError::DriverStillAttached(name.to_string()));
        }
        self.drivers.remove(name);
        Ok(())
    }
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::new()
    }
}
