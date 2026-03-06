//! Property types and builders for INDIGO properties.
//!
//! This module provides types for representing INDIGO properties and their items,
//! along with builder patterns for ergonomic construction.

use super::value::PropertyValue;
use std::collections::HashMap;

/// Represents an INDIGO property with its metadata and items.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// Device name this property belongs to.
    pub device: String,

    /// Property name.
    pub name: String,

    /// Property group (for UI organization).
    pub group: String,

    /// Human-readable label.
    pub label: String,

    /// Current state of the property.
    pub state: PropertyState,

    /// Permission level for the property.
    pub perm: PropertyPerm,

    /// Type of the property.
    pub property_type: PropertyType,

    /// Property items (name -> value mapping).
    pub items: HashMap<String, PropertyItem>,

    /// Optional timeout in seconds.
    pub timeout: Option<f64>,

    /// Optional timestamp.
    pub timestamp: Option<String>,

    /// Optional message.
    pub message: Option<String>,
}

/// Represents a single item within a property.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyItem {
    /// Item name.
    pub name: String,

    /// Human-readable label.
    pub label: String,

    /// Item value.
    pub value: PropertyValue,
}

/// State of a property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PropertyState {
    /// Property is idle.
    #[default]
    Idle,
    /// Property operation completed successfully.
    Ok,
    /// Property operation is in progress.
    Busy,
    /// Property operation failed or is in alert state.
    Alert,
}

impl PropertyState {
    /// Parses a property state from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid property state.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Idle" => Ok(PropertyState::Idle),
            "Ok" => Ok(PropertyState::Ok),
            "Busy" => Ok(PropertyState::Busy),
            "Alert" => Ok(PropertyState::Alert),
            _ => Err(format!("Invalid property state: {}", s)),
        }
    }

    /// Converts the property state to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            PropertyState::Idle => "Idle",
            PropertyState::Ok => "Ok",
            PropertyState::Busy => "Busy",
            PropertyState::Alert => "Alert",
        }
    }
}

impl std::fmt::Display for PropertyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Permission level for a property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PropertyPerm {
    /// Read-only property.
    ReadOnly,
    /// Write-only property.
    WriteOnly,
    /// Read-write property.
    #[default]
    ReadWrite,
}

impl PropertyPerm {
    /// Parses a property permission from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid property permission.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ro" => Ok(PropertyPerm::ReadOnly),
            "wo" => Ok(PropertyPerm::WriteOnly),
            "rw" => Ok(PropertyPerm::ReadWrite),
            _ => Err(format!("Invalid property permission: {}", s)),
        }
    }

    /// Converts the property permission to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            PropertyPerm::ReadOnly => "ro",
            PropertyPerm::WriteOnly => "wo",
            PropertyPerm::ReadWrite => "rw",
        }
    }
}

impl std::fmt::Display for PropertyPerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Type of property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyType {
    /// Text property.
    Text,
    /// Number property.
    Number,
    /// Switch property.
    Switch,
    /// Light property (read-only indicator).
    Light,
    /// BLOB property (Binary Large Object).
    Blob,
}

impl std::fmt::Display for PropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyType::Text => write!(f, "text"),
            PropertyType::Number => write!(f, "number"),
            PropertyType::Switch => write!(f, "switch"),
            PropertyType::Light => write!(f, "light"),
            PropertyType::Blob => write!(f, "blob"),
        }
    }
}

impl Property {
    /// Creates a new property builder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::types::Property;
    ///
    /// let property = Property::builder()
    ///     .device("CCD Simulator")
    ///     .name("CCD_EXPOSURE")
    ///     .build();
    /// ```
    pub fn builder() -> PropertyBuilder {
        PropertyBuilder::default()
    }

    /// Gets the full property key (device.name).
    pub fn key(&self) -> String {
        format!("{}.{}", self.device, self.name)
    }
}

/// Builder for constructing properties.
#[derive(Default)]
pub struct PropertyBuilder {
    device: Option<String>,
    name: Option<String>,
    group: Option<String>,
    label: Option<String>,
    state: PropertyState,
    perm: PropertyPerm,
    property_type: Option<PropertyType>,
    items: HashMap<String, PropertyItem>,
    timeout: Option<f64>,
    timestamp: Option<String>,
    message: Option<String>,
}

impl PropertyBuilder {
    /// Sets the device name.
    pub fn device(mut self, device: impl Into<String>) -> Self {
        self.device = Some(device.into());
        self
    }

    /// Sets the property name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the property group.
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Sets the property label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the property state.
    pub fn state(mut self, state: PropertyState) -> Self {
        self.state = state;
        self
    }

    /// Sets the property permission.
    pub fn perm(mut self, perm: PropertyPerm) -> Self {
        self.perm = perm;
        self
    }

    /// Sets the property type.
    pub fn property_type(mut self, property_type: PropertyType) -> Self {
        self.property_type = Some(property_type);
        self
    }

    /// Adds an item to the property.
    pub fn item(mut self, item: PropertyItem) -> Self {
        self.items.insert(item.name.clone(), item);
        self
    }

    /// Sets the timeout.
    pub fn timeout(mut self, timeout: f64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the timestamp.
    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Sets the message.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Builds the property.
    ///
    /// # Panics
    ///
    /// Panics if required fields (device, name, property_type) are not set.
    /// TODO: Return Result instead of panicking in future phases.
    pub fn build(self) -> Property {
        Property {
            device: self.device.expect("device is required"),
            name: self.name.expect("name is required"),
            group: self.group.unwrap_or_default(),
            label: self.label.unwrap_or_default(),
            state: self.state,
            perm: self.perm,
            property_type: self.property_type.expect("property_type is required"),
            items: self.items,
            timeout: self.timeout,
            timestamp: self.timestamp,
            message: self.message,
        }
    }
}

impl PropertyItem {
    /// Creates a new property item.
    pub fn new(name: impl Into<String>, label: impl Into<String>, value: PropertyValue) -> Self {
        PropertyItem {
            name: name.into(),
            label: label.into(),
            value,
        }
    }
}
