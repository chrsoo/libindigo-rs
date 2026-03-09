//! Mock device management for the INDIGO server.

use super::property::{MockProperty, PropertyUpdate};
use libindigo::error::{IndigoError, Result};
use std::collections::HashMap;

/// Registry of mock devices
pub struct DeviceRegistry {
    /// Devices by name
    devices: HashMap<String, MockDevice>,
}

/// A mock INDIGO device
#[derive(Debug, Clone)]
pub struct MockDevice {
    /// Device name (unique identifier)
    pub name: String,

    /// Device interface bitmap
    pub interface: u32,

    /// Properties owned by this device
    pub properties: HashMap<String, MockProperty>,

    /// Device metadata
    pub metadata: DeviceMetadata,
}

/// Device metadata
#[derive(Debug, Clone, Default)]
pub struct DeviceMetadata {
    pub version: Option<String>,
    pub driver_name: Option<String>,
    pub driver_version: Option<String>,
    pub driver_interface: Option<u32>,
}

impl DeviceRegistry {
    /// Create a new empty device registry
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Add a device to the registry
    pub fn add_device(&mut self, device: MockDevice) {
        self.devices.insert(device.name.clone(), device);
    }

    /// Get a device by name
    pub fn get_device(&self, name: &str) -> Option<&MockDevice> {
        self.devices.get(name)
    }

    /// Get a mutable device by name
    pub fn get_device_mut(&mut self, name: &str) -> Option<&mut MockDevice> {
        self.devices.get_mut(name)
    }

    /// List all devices
    pub fn list_devices(&self) -> Vec<&MockDevice> {
        self.devices.values().collect()
    }

    /// Add a property to a device
    pub fn add_property(&mut self, device: &str, property: MockProperty) -> Result<()> {
        let dev = self
            .get_device_mut(device)
            .ok_or_else(|| IndigoError::ProtocolError(format!("Device '{}' not found", device)))?;

        dev.properties.insert(property.name.clone(), property);
        Ok(())
    }

    /// Get a property from a device
    pub fn get_property(&self, device: &str, name: &str) -> Option<&MockProperty> {
        self.get_device(device).and_then(|d| d.properties.get(name))
    }

    /// Get a mutable property from a device
    pub fn get_property_mut(&mut self, device: &str, name: &str) -> Option<&mut MockProperty> {
        self.get_device_mut(device)
            .and_then(|d| d.properties.get_mut(name))
    }

    /// Update a property value
    pub fn update_property(
        &mut self,
        device: &str,
        name: &str,
        update: PropertyUpdate,
    ) -> Result<()> {
        let property = self.get_property_mut(device, name).ok_or_else(|| {
            IndigoError::ProtocolError(format!("Property '{}.{}' not found", device, name))
        })?;

        property.apply_update(update)
    }

    /// List all properties, optionally filtered by device
    pub fn list_properties(&self, device_filter: Option<&str>) -> Vec<(&str, &MockProperty)> {
        let mut result = Vec::new();

        for (device_name, device) in &self.devices {
            // Apply device filter
            if let Some(filter) = device_filter {
                if device_name != filter {
                    continue;
                }
            }

            for property in device.properties.values() {
                result.push((device_name.as_str(), property));
            }
        }

        result
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_server::property::{
        NumberValue, PropertyItem, PropertyType, PropertyTypeMetadata, PropertyValue,
    };
    use libindigo_rs::protocol::{PropertyPerm, PropertyState};

    #[test]
    fn test_device_registry() {
        let mut registry = DeviceRegistry::new();

        let device = MockDevice {
            name: "Test Device".to_string(),
            interface: 0x01,
            properties: HashMap::new(),
            metadata: DeviceMetadata::default(),
        };

        registry.add_device(device);
        assert!(registry.get_device("Test Device").is_some());
        assert_eq!(registry.list_devices().len(), 1);
    }

    #[test]
    fn test_add_property() {
        let mut registry = DeviceRegistry::new();

        let device = MockDevice {
            name: "Test Device".to_string(),
            interface: 0x01,
            properties: HashMap::new(),
            metadata: DeviceMetadata::default(),
        };

        registry.add_device(device);

        let property = MockProperty {
            device: "Test Device".to_string(),
            name: "TEST_PROP".to_string(),
            group: "Main".to_string(),
            label: "Test Property".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Number,
            items: vec![PropertyItem {
                name: "VALUE".to_string(),
                label: "Value".to_string(),
                value: PropertyValue::Number(NumberValue {
                    value: 0.0,
                    format: "%.2f".to_string(),
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                }),
            }],
            timeout: None,
            timestamp: None,
            message: None,
            type_metadata: PropertyTypeMetadata::Number,
        };

        registry.add_property("Test Device", property).unwrap();
        assert!(registry.get_property("Test Device", "TEST_PROP").is_some());
    }

    #[test]
    fn test_list_properties() {
        let mut registry = DeviceRegistry::new();

        let mut device = MockDevice {
            name: "Test Device".to_string(),
            interface: 0x01,
            properties: HashMap::new(),
            metadata: DeviceMetadata::default(),
        };

        let property = MockProperty {
            device: "Test Device".to_string(),
            name: "TEST_PROP".to_string(),
            group: "Main".to_string(),
            label: "Test Property".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Number,
            items: vec![],
            timeout: None,
            timestamp: None,
            message: None,
            type_metadata: PropertyTypeMetadata::Number,
        };

        device.properties.insert("TEST_PROP".to_string(), property);
        registry.add_device(device);

        let props = registry.list_properties(None);
        assert_eq!(props.len(), 1);

        let props_filtered = registry.list_properties(Some("Test Device"));
        assert_eq!(props_filtered.len(), 1);

        let props_no_match = registry.list_properties(Some("Other Device"));
        assert_eq!(props_no_match.len(), 0);
    }
}
