//! Property state management for the mock INDIGO server.

use libindigo::error::{IndigoError, Result};
use libindigo_rs::protocol::{PropertyPerm, PropertyState, SwitchRule};
use std::collections::HashMap;

/// Type of property
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Text,
    Number,
    Switch,
    Light,
    Blob,
}

/// Type-specific property metadata
#[derive(Debug, Clone)]
pub enum PropertyTypeMetadata {
    Text,
    Number,
    Switch { rule: SwitchRule },
    Light,
    Blob,
}

/// A mock INDIGO property
#[derive(Debug, Clone)]
pub struct MockProperty {
    /// Property metadata
    pub device: String,
    pub name: String,
    pub group: String,
    pub label: String,
    pub state: PropertyState,
    pub perm: PropertyPerm,
    pub property_type: PropertyType,

    /// Property items
    pub items: Vec<PropertyItem>,

    /// Optional fields
    pub timeout: Option<f64>,
    pub timestamp: Option<String>,
    pub message: Option<String>,

    /// Type-specific metadata
    pub type_metadata: PropertyTypeMetadata,
}

/// A property item (element)
#[derive(Debug, Clone)]
pub struct PropertyItem {
    pub name: String,
    pub label: String,
    pub value: PropertyValue,
}

/// Property value types
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Text(String),
    Number(NumberValue),
    Switch(bool),
    Light(PropertyState),
    Blob(BlobValue),
}

/// Number value with constraints
#[derive(Debug, Clone, PartialEq)]
pub struct NumberValue {
    pub value: f64,
    pub format: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

/// BLOB value (URL reference only for JSON protocol)
#[derive(Debug, Clone, PartialEq)]
pub struct BlobValue {
    pub url: String,
    pub format: String,
    pub size: usize,
}

/// Property update request
#[derive(Debug, Clone)]
pub struct PropertyUpdate {
    pub state: Option<PropertyState>,
    pub items: Vec<(String, PropertyValue)>,
    pub message: Option<String>,
}

impl MockProperty {
    /// Apply an update to this property
    pub fn apply_update(&mut self, update: PropertyUpdate) -> Result<()> {
        // Update state if provided
        if let Some(state) = update.state {
            self.state = state;
        }

        // Update message if provided
        if let Some(message) = update.message {
            self.message = Some(message);
        }

        // Update items
        for (item_name, new_value) in update.items {
            let item = self
                .items
                .iter_mut()
                .find(|i| i.name == item_name)
                .ok_or_else(|| {
                    IndigoError::ProtocolError(format!(
                        "Item '{}' not found in property '{}.{}'",
                        item_name, self.device, self.name
                    ))
                })?;

            // Validate value type matches
            match (&item.value, &new_value) {
                (PropertyValue::Text(_), PropertyValue::Text(_))
                | (PropertyValue::Number(_), PropertyValue::Number(_))
                | (PropertyValue::Switch(_), PropertyValue::Switch(_))
                | (PropertyValue::Light(_), PropertyValue::Light(_))
                | (PropertyValue::Blob(_), PropertyValue::Blob(_)) => {
                    item.value = new_value;
                }
                _ => {
                    return Err(IndigoError::ProtocolError(format!(
                        "Type mismatch for item '{}' in property '{}.{}'",
                        item_name, self.device, self.name
                    )));
                }
            }
        }

        // Update timestamp
        self.timestamp = Some(Self::current_timestamp());

        Ok(())
    }

    /// Generate current timestamp in ISO 8601 format
    fn current_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_update() {
        let mut property = MockProperty {
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

        let update = PropertyUpdate {
            state: Some(PropertyState::Ok),
            items: vec![(
                "VALUE".to_string(),
                PropertyValue::Number(NumberValue {
                    value: 42.0,
                    format: "%.2f".to_string(),
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                }),
            )],
            message: Some("Updated".to_string()),
        };

        property.apply_update(update).unwrap();

        assert_eq!(property.state, PropertyState::Ok);
        assert_eq!(property.message, Some("Updated".to_string()));
        if let PropertyValue::Number(num) = &property.items[0].value {
            assert_eq!(num.value, 42.0);
        } else {
            panic!("Expected Number value");
        }
    }
}
