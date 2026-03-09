//! Property lifecycle management for device drivers.

use crate::error::{IndigoError, Result};
use crate::types::{
    Property, PropertyItem, PropertyPerm, PropertyState, PropertyType, PropertyValue,
};
use std::collections::HashMap;

/// Manages property registration, state, and updates for a device driver.
///
/// The PropertyManager maintains the canonical state of all properties
/// owned by a device driver and provides methods to define, update,
/// and remove properties.
pub struct PropertyManager {
    /// Device name this manager belongs to
    device_name: String,
    /// Registered properties indexed by name
    properties: HashMap<String, Property>,
    /// Pending property updates to be sent to clients
    pending_updates: Vec<PropertyUpdate>,
}

/// A pending property update to be sent to clients
#[derive(Debug, Clone)]
pub struct PropertyUpdate {
    pub property_name: String,
    pub values: Vec<(String, PropertyValue)>,
    pub state: PropertyState,
}

impl PropertyManager {
    /// Create a new PropertyManager for the given device
    pub fn new(device_name: impl Into<String>) -> Self {
        Self {
            device_name: device_name.into(),
            properties: HashMap::new(),
            pending_updates: Vec::new(),
        }
    }

    /// Get the device name
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Register a new property definition.
    /// Returns error if a property with the same name already exists.
    pub fn define_property(&mut self, property: Property) -> Result<()> {
        if self.properties.contains_key(&property.name) {
            return Err(IndigoError::PropertyAlreadyExists(property.name.clone()));
        }
        self.properties.insert(property.name.clone(), property);
        Ok(())
    }

    /// Register the standard CONNECTION property (Switch type).
    /// This is a convenience method as almost all devices need this.
    pub fn register_standard_connection(&mut self) -> Result<()> {
        let mut items = HashMap::new();
        items.insert(
            "CONNECTED".to_string(),
            PropertyItem::new(
                "CONNECTED",
                "Connected",
                PropertyValue::switch(crate::types::SwitchState::Off),
            ),
        );
        items.insert(
            "DISCONNECTED".to_string(),
            PropertyItem::new(
                "DISCONNECTED",
                "Disconnected",
                PropertyValue::switch(crate::types::SwitchState::On),
            ),
        );

        let property = Property {
            device: self.device_name.clone(),
            name: "CONNECTION".to_string(),
            group: "Main".to_string(),
            label: "Connection".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Switch,
            items,
            timeout: None,
            timestamp: None,
            message: None,
        };
        self.define_property(property)
    }

    /// Register the standard DEVICE_INFO property (Text type, read-only).
    pub fn register_device_info(
        &mut self,
        description: &str,
        version: &str,
        interface: u32,
    ) -> Result<()> {
        let mut items = HashMap::new();
        items.insert(
            "DEVICE_INTERFACE".to_string(),
            PropertyItem::new(
                "DEVICE_INTERFACE",
                "Interface",
                PropertyValue::text(interface.to_string()),
            ),
        );
        items.insert(
            "DEVICE_DESCRIPTION".to_string(),
            PropertyItem::new(
                "DEVICE_DESCRIPTION",
                "Description",
                PropertyValue::text(description),
            ),
        );
        items.insert(
            "DRIVER_VERSION".to_string(),
            PropertyItem::new("DRIVER_VERSION", "Version", PropertyValue::text(version)),
        );

        let property = Property {
            device: self.device_name.clone(),
            name: "INFO".to_string(),
            group: "Main".to_string(),
            label: "Device Info".to_string(),
            state: PropertyState::Ok,
            perm: PropertyPerm::ReadOnly,
            property_type: PropertyType::Text,
            items,
            timeout: None,
            timestamp: None,
            message: None,
        };
        self.define_property(property)
    }

    /// Get a registered property by name
    pub fn get_property(&self, name: &str) -> Option<&Property> {
        self.properties.get(name)
    }

    /// Get a mutable reference to a registered property
    pub fn get_property_mut(&mut self, name: &str) -> Option<&mut Property> {
        self.properties.get_mut(name)
    }

    /// Update property state and values, queueing the update for sending to clients.
    pub fn update_property(
        &mut self,
        name: &str,
        state: PropertyState,
        values: Vec<(String, PropertyValue)>,
    ) -> Result<()> {
        let property = self
            .properties
            .get_mut(name)
            .ok_or_else(|| IndigoError::PropertyNotFound(name.to_string()))?;

        property.state = state;
        // Update matching items
        for (item_name, new_value) in &values {
            if let Some(item) = property.items.get_mut(item_name) {
                item.value = new_value.clone();
            }
        }

        self.pending_updates.push(PropertyUpdate {
            property_name: name.to_string(),
            values,
            state,
        });

        Ok(())
    }

    /// Remove a property definition
    pub fn delete_property(&mut self, name: &str) -> Result<()> {
        self.properties
            .remove(name)
            .ok_or_else(|| IndigoError::PropertyNotFound(name.to_string()))?;
        Ok(())
    }

    /// Remove all properties
    pub fn delete_all_properties(&mut self) {
        self.properties.clear();
        self.pending_updates.clear();
    }

    /// Get and clear pending updates (consumed by the runtime to send to clients)
    pub fn drain_pending_updates(&mut self) -> Vec<PropertyUpdate> {
        std::mem::take(&mut self.pending_updates)
    }

    /// Get all registered properties
    pub fn properties(&self) -> impl Iterator<Item = &Property> {
        self.properties.values()
    }

    /// Check if a property is registered
    pub fn has_property(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    /// Get number of registered properties
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }
}
