//! DeviceProxy - bridge between high-level traits and property-based ClientStrategy.

use crate::error::{IndigoError, Result};
use crate::types::{Property, PropertyValue};
use std::collections::HashMap;

/// Bridge between high-level device traits and the property-based ClientStrategy.
///
/// DeviceProxy holds a reference to the client strategy and provides helper
/// methods for reading/writing properties that device trait implementations use.
pub struct DeviceProxy {
    /// Device name in INDIGO
    device_name: String,
    /// Device description
    description: Option<String>,
    /// Driver version
    driver_version: Option<String>,
    /// Connected state
    connected: bool,
    /// Cached properties
    properties: HashMap<String, Property>,
}

impl DeviceProxy {
    /// Create a new device proxy for the named device
    pub fn new(device_name: impl Into<String>) -> Self {
        Self {
            device_name: device_name.into(),
            description: None,
            driver_version: None,
            connected: false,
            properties: HashMap::new(),
        }
    }

    /// Get the device name
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Set connection state
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    /// Get device description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set device description
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Get driver version
    pub fn driver_version(&self) -> Option<&str> {
        self.driver_version.as_deref()
    }

    /// Set driver version
    pub fn set_driver_version(&mut self, version: impl Into<String>) {
        self.driver_version = Some(version.into());
    }

    /// Cache a property value
    pub fn cache_property(&mut self, property: Property) {
        self.properties.insert(property.name.clone(), property);
    }

    /// Get a cached property
    pub fn get_cached_property(&self, name: &str) -> Option<&Property> {
        self.properties.get(name)
    }

    /// Get a specific item value from a cached property
    pub fn get_property_item(
        &self,
        property_name: &str,
        item_name: &str,
    ) -> Option<&PropertyValue> {
        self.properties
            .get(property_name)
            .and_then(|p| p.items.get(item_name))
            .map(|item| &item.value)
    }

    /// Get a number value from a cached property item
    pub fn get_number(&self, property_name: &str, item_name: &str) -> Result<f64> {
        match self.get_property_item(property_name, item_name) {
            Some(PropertyValue::Number { value, .. }) => Ok(*value),
            Some(_) => Err(IndigoError::InvalidParameter(format!(
                "Property {}.{} is not a number",
                property_name, item_name
            ))),
            None => Err(IndigoError::PropertyNotFound(format!(
                "{}.{}",
                property_name, item_name
            ))),
        }
    }

    /// Get a text value from a cached property item
    pub fn get_text(&self, property_name: &str, item_name: &str) -> Result<String> {
        match self.get_property_item(property_name, item_name) {
            Some(PropertyValue::Text(s)) => Ok(s.clone()),
            Some(_) => Err(IndigoError::InvalidParameter(format!(
                "Property {}.{} is not text",
                property_name, item_name
            ))),
            None => Err(IndigoError::PropertyNotFound(format!(
                "{}.{}",
                property_name, item_name
            ))),
        }
    }

    /// Get a switch value from a cached property item
    pub fn get_switch(&self, property_name: &str, item_name: &str) -> Result<bool> {
        match self.get_property_item(property_name, item_name) {
            Some(PropertyValue::Switch { state }) => Ok(*state == crate::types::SwitchState::On),
            Some(_) => Err(IndigoError::InvalidParameter(format!(
                "Property {}.{} is not a switch",
                property_name, item_name
            ))),
            None => Err(IndigoError::PropertyNotFound(format!(
                "{}.{}",
                property_name, item_name
            ))),
        }
    }

    /// Get a BLOB value from a cached property item
    pub fn get_blob(&self, property_name: &str, item_name: &str) -> Result<(Vec<u8>, String)> {
        match self.get_property_item(property_name, item_name) {
            Some(PropertyValue::Blob { data, format, .. }) => Ok((data.clone(), format.clone())),
            Some(_) => Err(IndigoError::InvalidParameter(format!(
                "Property {}.{} is not a BLOB",
                property_name, item_name
            ))),
            None => Err(IndigoError::PropertyNotFound(format!(
                "{}.{}",
                property_name, item_name
            ))),
        }
    }

    /// Clear all cached properties
    pub fn clear_cache(&mut self) {
        self.properties.clear();
    }
}
