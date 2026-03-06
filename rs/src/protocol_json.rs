//! INDIGO JSON Protocol Parser and Serializer
//!
//! This module handles parsing and serialization of INDIGO JSON protocol messages.
//!
//! # Overview
//!
//! The INDIGO JSON protocol provides the same features as XML protocol version 2.0,
//! but in JSON format. Key differences from XML:
//!
//! - **Version**: JSON protocol uses version number 512 (equivalent to XML 2.0)
//! - **BLOB Handling**: JSON protocol **only supports URL-referenced BLOBs**, no inline BASE64 data
//! - **Boolean Values**: Switch states use `true`/`false` instead of "On"/"Off"
//! - **Numeric Values**: Numbers are JSON numbers, not strings
//! - **Message Structure**: XML tag names become JSON object keys, attributes become properties,
//!   nested items go in `items` array
//!
//! # Example
//!
//! ```ignore
//! use libindigo::strategies::rs::protocol_json::{JsonProtocolParser, JsonProtocolSerializer};
//!
//! let json = r#"{"getProperties": {"version": 512, "device": "CCD Simulator"}}"#;
//! let message = JsonProtocolParser::parse_message(json)?;
//! ```

use libindigo::error::{IndigoError, Result};
use serde_json::{json, Map, Value};

// Re-export protocol types from the main protocol module
pub use crate::protocol::{
    BLOBEnable, DefBLOB, DefBLOBVector, DefLight, DefLightVector, DefNumber, DefNumberVector,
    DefSwitch, DefSwitchVector, DefText, DefTextVector, DelProperty, EnableBLOB, GetProperties,
    Message, NewBLOBVector, NewNumberVector, NewSwitchVector, NewTextVector, NewVectorAttributes,
    OneBLOB, OneLight, OneNumber, OneSwitch, OneText, PropertyPerm, PropertyState, ProtocolMessage,
    SetBLOBVector, SetLightVector, SetNumberVector, SetSwitchVector, SetTextVector,
    SetVectorAttributes, SwitchRule, SwitchState, VectorAttributes,
};

// ============================================================================
// JSON Protocol Constants
// ============================================================================

/// JSON protocol version number (equivalent to XML 2.0)
pub const JSON_PROTOCOL_VERSION: u32 = 512;

// ============================================================================
// JSON-specific Helper Structures
// ============================================================================

/// Helper for serializing/deserializing switch states as booleans in JSON
impl SwitchState {
    /// Converts switch state to boolean (for JSON)
    pub fn to_bool(&self) -> bool {
        match self {
            SwitchState::On => true,
            SwitchState::Off => false,
        }
    }

    /// Creates switch state from boolean (for JSON)
    pub fn from_bool(value: bool) -> Self {
        if value {
            SwitchState::On
        } else {
            SwitchState::Off
        }
    }
}

// ============================================================================
// JSON Protocol Parser
// ============================================================================

/// JSON protocol message parser.
///
/// Parses INDIGO JSON protocol messages from JSON strings.
pub struct JsonProtocolParser;

impl JsonProtocolParser {
    /// Parses a single protocol message from JSON string.
    ///
    /// The JSON should contain a single message object with the message type as the key.
    pub fn parse_message(json: &str) -> Result<ProtocolMessage> {
        let value: Value = serde_json::from_str(json)
            .map_err(|e| IndigoError::ParseError(format!("JSON parse error: {}", e)))?;

        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected JSON object".to_string()))?;

        // Get the message type (should be the only key at root level)
        if obj.len() != 1 {
            return Err(IndigoError::ParseError(
                "Expected single message type key".to_string(),
            ));
        }

        let (msg_type, msg_value) = obj.iter().next().unwrap();

        match msg_type.as_str() {
            "getProperties" => Self::parse_get_properties(msg_value),
            "defTextVector" => Self::parse_def_text_vector(msg_value),
            "defNumberVector" => Self::parse_def_number_vector(msg_value),
            "defSwitchVector" => Self::parse_def_switch_vector(msg_value),
            "defLightVector" => Self::parse_def_light_vector(msg_value),
            "defBLOBVector" => Self::parse_def_blob_vector(msg_value),
            "setTextVector" => Self::parse_set_text_vector(msg_value),
            "setNumberVector" => Self::parse_set_number_vector(msg_value),
            "setSwitchVector" => Self::parse_set_switch_vector(msg_value),
            "setLightVector" => Self::parse_set_light_vector(msg_value),
            "setBLOBVector" => Self::parse_set_blob_vector(msg_value),
            "newTextVector" => Self::parse_new_text_vector(msg_value),
            "newNumberVector" => Self::parse_new_number_vector(msg_value),
            "newSwitchVector" => Self::parse_new_switch_vector(msg_value),
            "newBLOBVector" => Self::parse_new_blob_vector(msg_value),
            "message" => Self::parse_message_element(msg_value),
            "deleteProperty" => Self::parse_delete_property(msg_value),
            "enableBLOB" => Self::parse_enable_blob(msg_value),
            _ => Err(IndigoError::ParseError(format!(
                "Unknown message type: {}",
                msg_type
            ))),
        }
    }

    // Helper to get required string field
    fn get_string(obj: &Map<String, Value>, key: &str) -> Result<String> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| IndigoError::ParseError(format!("Missing or invalid field: {}", key)))
    }

    // Helper to get optional string field
    fn get_opt_string(obj: &Map<String, Value>, key: &str) -> Option<String> {
        obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    // Helper to get optional number field
    fn get_opt_f64(obj: &Map<String, Value>, key: &str) -> Option<f64> {
        obj.get(key).and_then(|v| v.as_f64())
    }

    // Parse vector attributes
    fn parse_vector_attrs(obj: &Map<String, Value>) -> Result<VectorAttributes> {
        Ok(VectorAttributes {
            device: Self::get_string(obj, "device")?,
            name: Self::get_string(obj, "name")?,
            label: Self::get_opt_string(obj, "label").unwrap_or_default(),
            group: Self::get_opt_string(obj, "group").unwrap_or_default(),
            state: PropertyState::from_str(&Self::get_string(obj, "state")?)
                .map_err(|e| IndigoError::ParseError(e))?,
            timeout: Self::get_opt_f64(obj, "timeout"),
            timestamp: Self::get_opt_string(obj, "timestamp"),
            message: Self::get_opt_string(obj, "message"),
        })
    }

    // Parse set vector attributes
    fn parse_set_vector_attrs(obj: &Map<String, Value>) -> Result<SetVectorAttributes> {
        Ok(SetVectorAttributes {
            device: Self::get_string(obj, "device")?,
            name: Self::get_string(obj, "name")?,
            state: Self::get_opt_string(obj, "state")
                .map(|s| PropertyState::from_str(&s).map_err(|e| IndigoError::ParseError(e)))
                .transpose()?,
            timeout: Self::get_opt_f64(obj, "timeout"),
            timestamp: Self::get_opt_string(obj, "timestamp"),
            message: Self::get_opt_string(obj, "message"),
        })
    }

    // Parse new vector attributes
    fn parse_new_vector_attrs(obj: &Map<String, Value>) -> Result<NewVectorAttributes> {
        Ok(NewVectorAttributes {
            device: Self::get_string(obj, "device")?,
            name: Self::get_string(obj, "name")?,
            timestamp: Self::get_opt_string(obj, "timestamp"),
        })
    }

    // Parse getProperties
    fn parse_get_properties(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        // Version can be either a number (512) or a string ("2.0")
        let version = obj.get("version").and_then(|v| {
            if let Some(num) = v.as_u64() {
                Some(num.to_string())
            } else {
                v.as_str().map(|s| s.to_string())
            }
        });

        Ok(ProtocolMessage::GetProperties(GetProperties {
            version,
            device: Self::get_opt_string(obj, "device"),
            name: Self::get_opt_string(obj, "name"),
        }))
    }

    // Parse defTextVector
    fn parse_def_text_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_vector_attrs(obj)?;
        let perm = PropertyPerm::from_str(&Self::get_string(obj, "perm")?)
            .map_err(|e| IndigoError::ParseError(e))?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            elements.push(DefText {
                name: Self::get_string(item_obj, "name")?,
                label: Self::get_opt_string(item_obj, "label").unwrap_or_default(),
                value: Self::get_opt_string(item_obj, "value").unwrap_or_default(),
            });
        }

        Ok(ProtocolMessage::DefTextVector(DefTextVector {
            attrs,
            perm,
            elements,
        }))
    }

    // Parse defNumberVector
    fn parse_def_number_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_vector_attrs(obj)?;
        let perm = PropertyPerm::from_str(&Self::get_string(obj, "perm")?)
            .map_err(|e| IndigoError::ParseError(e))?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            elements.push(DefNumber {
                name: Self::get_string(item_obj, "name")?,
                label: Self::get_opt_string(item_obj, "label").unwrap_or_default(),
                format: Self::get_string(item_obj, "format")?,
                min: item_obj
                    .get("min")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| IndigoError::ParseError("Missing min".to_string()))?,
                max: item_obj
                    .get("max")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| IndigoError::ParseError("Missing max".to_string()))?,
                step: item_obj
                    .get("step")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| IndigoError::ParseError("Missing step".to_string()))?,
                value: item_obj
                    .get("value")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| IndigoError::ParseError("Missing value".to_string()))?,
            });
        }

        Ok(ProtocolMessage::DefNumberVector(DefNumberVector {
            attrs,
            perm,
            elements,
        }))
    }

    // Parse defSwitchVector
    fn parse_def_switch_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_vector_attrs(obj)?;
        let perm = PropertyPerm::from_str(&Self::get_string(obj, "perm")?)
            .map_err(|e| IndigoError::ParseError(e))?;
        let rule = SwitchRule::from_str(&Self::get_string(obj, "rule")?)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            let value_bool = item_obj
                .get("value")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| IndigoError::ParseError("Missing or invalid value".to_string()))?;
            elements.push(DefSwitch {
                name: Self::get_string(item_obj, "name")?,
                label: Self::get_opt_string(item_obj, "label").unwrap_or_default(),
                value: SwitchState::from_bool(value_bool),
            });
        }

        Ok(ProtocolMessage::DefSwitchVector(DefSwitchVector {
            attrs,
            perm,
            rule,
            elements,
        }))
    }

    // Parse defLightVector
    fn parse_def_light_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            let value_str = Self::get_string(item_obj, "value")?;
            elements.push(DefLight {
                name: Self::get_string(item_obj, "name")?,
                label: Self::get_opt_string(item_obj, "label").unwrap_or_default(),
                value: PropertyState::from_str(&value_str)
                    .map_err(|e| IndigoError::ParseError(e))?,
            });
        }

        Ok(ProtocolMessage::DefLightVector(DefLightVector {
            attrs,
            elements,
        }))
    }

    // Parse defBLOBVector
    fn parse_def_blob_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_vector_attrs(obj)?;
        let perm = PropertyPerm::from_str(&Self::get_string(obj, "perm")?)
            .map_err(|e| IndigoError::ParseError(e))?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            elements.push(DefBLOB {
                name: Self::get_string(item_obj, "name")?,
                label: Self::get_opt_string(item_obj, "label").unwrap_or_default(),
            });
        }

        Ok(ProtocolMessage::DefBLOBVector(DefBLOBVector {
            attrs,
            perm,
            elements,
        }))
    }

    // Parse setTextVector
    fn parse_set_text_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_set_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            elements.push(OneText {
                name: Self::get_string(item_obj, "name")?,
                value: Self::get_opt_string(item_obj, "value").unwrap_or_default(),
            });
        }

        Ok(ProtocolMessage::SetTextVector(SetTextVector {
            attrs,
            elements,
        }))
    }

    // Parse setNumberVector
    fn parse_set_number_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_set_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            elements.push(OneNumber {
                name: Self::get_string(item_obj, "name")?,
                value: item_obj
                    .get("value")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| IndigoError::ParseError("Missing value".to_string()))?,
            });
        }

        Ok(ProtocolMessage::SetNumberVector(SetNumberVector {
            attrs,
            elements,
        }))
    }

    // Parse setSwitchVector
    fn parse_set_switch_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_set_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            let value_bool = item_obj
                .get("value")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| IndigoError::ParseError("Missing or invalid value".to_string()))?;
            elements.push(OneSwitch {
                name: Self::get_string(item_obj, "name")?,
                value: SwitchState::from_bool(value_bool),
            });
        }

        Ok(ProtocolMessage::SetSwitchVector(SetSwitchVector {
            attrs,
            elements,
        }))
    }

    // Parse setLightVector
    fn parse_set_light_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_set_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            let value_str = Self::get_string(item_obj, "value")?;
            elements.push(OneLight {
                name: Self::get_string(item_obj, "name")?,
                value: PropertyState::from_str(&value_str)
                    .map_err(|e| IndigoError::ParseError(e))?,
            });
        }

        Ok(ProtocolMessage::SetLightVector(SetLightVector {
            attrs,
            elements,
        }))
    }

    // Parse setBLOBVector
    fn parse_set_blob_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_set_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            // In JSON protocol, BLOBs are URL references only
            elements.push(OneBLOB {
                name: Self::get_string(item_obj, "name")?,
                size: 0, // Size not used for URL references
                format: Self::get_opt_string(item_obj, "format").unwrap_or_default(),
                value: Self::get_string(item_obj, "value")?, // URL path
            });
        }

        Ok(ProtocolMessage::SetBLOBVector(SetBLOBVector {
            attrs,
            elements,
        }))
    }

    // Parse newTextVector
    fn parse_new_text_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_new_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            elements.push(OneText {
                name: Self::get_string(item_obj, "name")?,
                value: Self::get_opt_string(item_obj, "value").unwrap_or_default(),
            });
        }

        Ok(ProtocolMessage::NewTextVector(NewTextVector {
            attrs,
            elements,
        }))
    }

    // Parse newNumberVector
    fn parse_new_number_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_new_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            elements.push(OneNumber {
                name: Self::get_string(item_obj, "name")?,
                value: item_obj
                    .get("value")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| IndigoError::ParseError("Missing value".to_string()))?,
            });
        }

        Ok(ProtocolMessage::NewNumberVector(NewNumberVector {
            attrs,
            elements,
        }))
    }

    // Parse newSwitchVector
    fn parse_new_switch_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_new_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            let value_bool = item_obj
                .get("value")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| IndigoError::ParseError("Missing or invalid value".to_string()))?;
            elements.push(OneSwitch {
                name: Self::get_string(item_obj, "name")?,
                value: SwitchState::from_bool(value_bool),
            });
        }

        Ok(ProtocolMessage::NewSwitchVector(NewSwitchVector {
            attrs,
            elements,
        }))
    }

    // Parse newBLOBVector
    fn parse_new_blob_vector(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let attrs = Self::parse_new_vector_attrs(obj)?;

        let items = obj
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| IndigoError::ParseError("Missing items array".to_string()))?;

        let mut elements = Vec::new();
        for item in items {
            let item_obj = item
                .as_object()
                .ok_or_else(|| IndigoError::ParseError("Expected item object".to_string()))?;
            elements.push(OneBLOB {
                name: Self::get_string(item_obj, "name")?,
                size: 0,
                format: Self::get_string(item_obj, "format")?,
                value: String::new(), // Not used in newBLOBVector
            });
        }

        Ok(ProtocolMessage::NewBLOBVector(NewBLOBVector {
            attrs,
            elements,
        }))
    }

    // Parse message
    fn parse_message_element(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        Ok(ProtocolMessage::Message(Message {
            device: Self::get_opt_string(obj, "device"),
            timestamp: Self::get_opt_string(obj, "timestamp"),
            message: Self::get_opt_string(obj, "message"),
        }))
    }

    // Parse deleteProperty
    fn parse_delete_property(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        Ok(ProtocolMessage::DelProperty(DelProperty {
            device: Self::get_string(obj, "device")?,
            name: Self::get_opt_string(obj, "name"),
            timestamp: Self::get_opt_string(obj, "timestamp"),
            message: Self::get_opt_string(obj, "message"),
        }))
    }

    // Parse enableBLOB
    fn parse_enable_blob(value: &Value) -> Result<ProtocolMessage> {
        let obj = value
            .as_object()
            .ok_or_else(|| IndigoError::ParseError("Expected object".to_string()))?;

        let value_str = Self::get_string(obj, "value")?;

        Ok(ProtocolMessage::EnableBLOB(EnableBLOB {
            device: Self::get_string(obj, "device")?,
            name: Self::get_opt_string(obj, "name"),
            value: BLOBEnable::from_str(&value_str)?,
        }))
    }
}

// ============================================================================
// JSON Protocol Serializer
// ============================================================================

/// JSON protocol message serializer.
///
/// Serializes Rust types into INDIGO JSON protocol messages.
pub struct JsonProtocolSerializer;

impl JsonProtocolSerializer {
    /// Serializes a protocol message to JSON string.
    ///
    /// Returns compact JSON (no pretty printing) for network efficiency.
    pub fn serialize(message: &ProtocolMessage) -> Result<String> {
        let value = match message {
            ProtocolMessage::GetProperties(m) => Self::serialize_get_properties(m),
            ProtocolMessage::DefTextVector(m) => Self::serialize_def_text_vector(m),
            ProtocolMessage::DefNumberVector(m) => Self::serialize_def_number_vector(m),
            ProtocolMessage::DefSwitchVector(m) => Self::serialize_def_switch_vector(m),
            ProtocolMessage::DefLightVector(m) => Self::serialize_def_light_vector(m),
            ProtocolMessage::DefBLOBVector(m) => Self::serialize_def_blob_vector(m),
            ProtocolMessage::SetTextVector(m) => Self::serialize_set_text_vector(m),
            ProtocolMessage::SetNumberVector(m) => Self::serialize_set_number_vector(m),
            ProtocolMessage::SetSwitchVector(m) => Self::serialize_set_switch_vector(m),
            ProtocolMessage::SetLightVector(m) => Self::serialize_set_light_vector(m),
            ProtocolMessage::SetBLOBVector(m) => Self::serialize_set_blob_vector(m),
            ProtocolMessage::NewTextVector(m) => Self::serialize_new_text_vector(m),
            ProtocolMessage::NewNumberVector(m) => Self::serialize_new_number_vector(m),
            ProtocolMessage::NewSwitchVector(m) => Self::serialize_new_switch_vector(m),
            ProtocolMessage::NewBLOBVector(m) => Self::serialize_new_blob_vector(m),
            ProtocolMessage::Message(m) => Self::serialize_message(m),
            ProtocolMessage::DelProperty(m) => Self::serialize_del_property(m),
            ProtocolMessage::EnableBLOB(m) => Self::serialize_enable_blob(m),
        };

        serde_json::to_string(&value)
            .map_err(|e| IndigoError::ProtocolError(format!("JSON serialization error: {}", e)))
    }

    // Helper to add vector attributes to JSON object
    fn add_vector_attrs(obj: &mut Map<String, Value>, attrs: &VectorAttributes) {
        obj.insert("device".to_string(), json!(attrs.device));
        obj.insert("name".to_string(), json!(attrs.name));
        if !attrs.label.is_empty() {
            obj.insert("label".to_string(), json!(attrs.label));
        }
        if !attrs.group.is_empty() {
            obj.insert("group".to_string(), json!(attrs.group));
        }
        obj.insert("state".to_string(), json!(attrs.state.as_str()));
        if let Some(timeout) = attrs.timeout {
            obj.insert("timeout".to_string(), json!(timeout));
        }
        if let Some(ref timestamp) = attrs.timestamp {
            obj.insert("timestamp".to_string(), json!(timestamp));
        }
        if let Some(ref message) = attrs.message {
            obj.insert("message".to_string(), json!(message));
        }
    }

    // Helper to add set vector attributes to JSON object
    fn add_set_vector_attrs(obj: &mut Map<String, Value>, attrs: &SetVectorAttributes) {
        obj.insert("device".to_string(), json!(attrs.device));
        obj.insert("name".to_string(), json!(attrs.name));
        if let Some(state) = attrs.state {
            obj.insert("state".to_string(), json!(state.as_str()));
        }
        if let Some(timeout) = attrs.timeout {
            obj.insert("timeout".to_string(), json!(timeout));
        }
        if let Some(ref timestamp) = attrs.timestamp {
            obj.insert("timestamp".to_string(), json!(timestamp));
        }
        if let Some(ref message) = attrs.message {
            obj.insert("message".to_string(), json!(message));
        }
    }

    // Helper to add new vector attributes to JSON object
    fn add_new_vector_attrs(obj: &mut Map<String, Value>, attrs: &NewVectorAttributes) {
        obj.insert("device".to_string(), json!(attrs.device));
        obj.insert("name".to_string(), json!(attrs.name));
        if let Some(ref timestamp) = attrs.timestamp {
            obj.insert("timestamp".to_string(), json!(timestamp));
        }
    }

    // Serialize getProperties
    fn serialize_get_properties(msg: &GetProperties) -> Value {
        let mut obj = Map::new();
        obj.insert("version".to_string(), json!(JSON_PROTOCOL_VERSION));
        if let Some(ref device) = msg.device {
            obj.insert("device".to_string(), json!(device));
        }
        if let Some(ref name) = msg.name {
            obj.insert("name".to_string(), json!(name));
        }
        json!({ "getProperties": obj })
    }

    // Serialize defTextVector
    fn serialize_def_text_vector(msg: &DefTextVector) -> Value {
        let mut obj = Map::new();
        obj.insert("version".to_string(), json!(JSON_PROTOCOL_VERSION));
        Self::add_vector_attrs(&mut obj, &msg.attrs);
        obj.insert("perm".to_string(), json!(msg.perm.as_str()));

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                if !e.label.is_empty() {
                    item.insert("label".to_string(), json!(e.label));
                }
                item.insert("value".to_string(), json!(e.value));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "defTextVector": obj })
    }

    // Serialize defNumberVector
    fn serialize_def_number_vector(msg: &DefNumberVector) -> Value {
        let mut obj = Map::new();
        obj.insert("version".to_string(), json!(JSON_PROTOCOL_VERSION));
        Self::add_vector_attrs(&mut obj, &msg.attrs);
        obj.insert("perm".to_string(), json!(msg.perm.as_str()));

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                if !e.label.is_empty() {
                    item.insert("label".to_string(), json!(e.label));
                }
                item.insert("format".to_string(), json!(e.format));
                item.insert("min".to_string(), json!(e.min));
                item.insert("max".to_string(), json!(e.max));
                item.insert("step".to_string(), json!(e.step));
                item.insert("value".to_string(), json!(e.value));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "defNumberVector": obj })
    }

    // Serialize defSwitchVector
    fn serialize_def_switch_vector(msg: &DefSwitchVector) -> Value {
        let mut obj = Map::new();
        obj.insert("version".to_string(), json!(JSON_PROTOCOL_VERSION));
        Self::add_vector_attrs(&mut obj, &msg.attrs);
        obj.insert("perm".to_string(), json!(msg.perm.as_str()));
        obj.insert("rule".to_string(), json!(msg.rule.as_str()));

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                if !e.label.is_empty() {
                    item.insert("label".to_string(), json!(e.label));
                }
                item.insert("value".to_string(), json!(e.value.to_bool()));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "defSwitchVector": obj })
    }

    // Serialize defLightVector
    fn serialize_def_light_vector(msg: &DefLightVector) -> Value {
        let mut obj = Map::new();
        obj.insert("version".to_string(), json!(JSON_PROTOCOL_VERSION));
        Self::add_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                if !e.label.is_empty() {
                    item.insert("label".to_string(), json!(e.label));
                }
                item.insert("value".to_string(), json!(e.value.as_str()));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "defLightVector": obj })
    }

    // Serialize defBLOBVector
    fn serialize_def_blob_vector(msg: &DefBLOBVector) -> Value {
        let mut obj = Map::new();
        obj.insert("version".to_string(), json!(JSON_PROTOCOL_VERSION));
        Self::add_vector_attrs(&mut obj, &msg.attrs);
        obj.insert("perm".to_string(), json!(msg.perm.as_str()));

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                if !e.label.is_empty() {
                    item.insert("label".to_string(), json!(e.label));
                }
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "defBLOBVector": obj })
    }

    // Serialize setTextVector
    fn serialize_set_text_vector(msg: &SetTextVector) -> Value {
        let mut obj = Map::new();
        Self::add_set_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("value".to_string(), json!(e.value));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "setTextVector": obj })
    }

    // Serialize setNumberVector
    fn serialize_set_number_vector(msg: &SetNumberVector) -> Value {
        let mut obj = Map::new();
        Self::add_set_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("value".to_string(), json!(e.value));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "setNumberVector": obj })
    }

    // Serialize setSwitchVector
    fn serialize_set_switch_vector(msg: &SetSwitchVector) -> Value {
        let mut obj = Map::new();
        Self::add_set_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("value".to_string(), json!(e.value.to_bool()));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "setSwitchVector": obj })
    }

    // Serialize setLightVector
    fn serialize_set_light_vector(msg: &SetLightVector) -> Value {
        let mut obj = Map::new();
        Self::add_set_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("value".to_string(), json!(e.value.as_str()));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "setLightVector": obj })
    }

    // Serialize setBLOBVector
    fn serialize_set_blob_vector(msg: &SetBLOBVector) -> Value {
        let mut obj = Map::new();
        Self::add_set_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("value".to_string(), json!(e.value)); // URL path
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "setBLOBVector": obj })
    }

    // Serialize newTextVector
    fn serialize_new_text_vector(msg: &NewTextVector) -> Value {
        let mut obj = Map::new();
        Self::add_new_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("value".to_string(), json!(e.value));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "newTextVector": obj })
    }

    // Serialize newNumberVector
    fn serialize_new_number_vector(msg: &NewNumberVector) -> Value {
        let mut obj = Map::new();
        Self::add_new_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("value".to_string(), json!(e.value));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "newNumberVector": obj })
    }

    // Serialize newSwitchVector
    fn serialize_new_switch_vector(msg: &NewSwitchVector) -> Value {
        let mut obj = Map::new();
        Self::add_new_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("value".to_string(), json!(e.value.to_bool()));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "newSwitchVector": obj })
    }

    // Serialize newBLOBVector
    fn serialize_new_blob_vector(msg: &NewBLOBVector) -> Value {
        let mut obj = Map::new();
        Self::add_new_vector_attrs(&mut obj, &msg.attrs);

        let items: Vec<Value> = msg
            .elements
            .iter()
            .map(|e| {
                let mut item = Map::new();
                item.insert("name".to_string(), json!(e.name));
                item.insert("format".to_string(), json!(e.format));
                Value::Object(item)
            })
            .collect();
        obj.insert("items".to_string(), json!(items));

        json!({ "newBLOBVector": obj })
    }

    // Serialize message
    fn serialize_message(msg: &Message) -> Value {
        let mut obj = Map::new();
        if let Some(ref device) = msg.device {
            obj.insert("device".to_string(), json!(device));
        }
        if let Some(ref timestamp) = msg.timestamp {
            obj.insert("timestamp".to_string(), json!(timestamp));
        }
        if let Some(ref message) = msg.message {
            obj.insert("message".to_string(), json!(message));
        }
        json!({ "message": obj })
    }

    // Serialize delProperty
    fn serialize_del_property(msg: &DelProperty) -> Value {
        let mut obj = Map::new();
        obj.insert("device".to_string(), json!(msg.device));
        if let Some(ref name) = msg.name {
            obj.insert("name".to_string(), json!(name));
        }
        if let Some(ref timestamp) = msg.timestamp {
            obj.insert("timestamp".to_string(), json!(timestamp));
        }
        if let Some(ref message) = msg.message {
            obj.insert("message".to_string(), json!(message));
        }
        json!({ "deleteProperty": obj })
    }

    // Serialize enableBLOB
    fn serialize_enable_blob(msg: &EnableBLOB) -> Value {
        let mut obj = Map::new();
        obj.insert("device".to_string(), json!(msg.device));
        if let Some(ref name) = msg.name {
            obj.insert("name".to_string(), json!(name));
        }
        obj.insert("value".to_string(), json!(msg.value.as_str()));
        json!({ "enableBLOB": obj })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_properties() {
        let json = r#"{"getProperties": {"version": 512, "device": "Server", "name": "LOAD"}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::GetProperties(gp) => {
                assert_eq!(gp.version, Some("512".to_string()));
                assert_eq!(gp.device, Some("Server".to_string()));
                assert_eq!(gp.name, Some("LOAD".to_string()));
            }
            _ => panic!("Expected GetProperties"),
        }
    }

    #[test]
    fn test_serialize_get_properties() {
        let msg = ProtocolMessage::GetProperties(GetProperties {
            version: Some("2.0".to_string()),
            device: Some("Server".to_string()),
            name: Some("LOAD".to_string()),
        });

        let json = JsonProtocolSerializer::serialize(&msg).unwrap();
        assert!(json.contains("getProperties"));
        assert!(json.contains("\"version\":512"));
        assert!(json.contains("\"device\":\"Server\""));
        assert!(json.contains("\"name\":\"LOAD\""));
    }

    #[test]
    fn test_parse_def_text_vector() {
        let json = r#"{"defTextVector": {"version": 512, "device": "Server", "name": "LOAD", "group": "Main", "label": "Load driver", "perm": "rw", "state": "Idle", "items": [{"name": "DRIVER", "label": "Load driver", "value": ""}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(dtv) => {
                assert_eq!(dtv.attrs.device, "Server");
                assert_eq!(dtv.attrs.name, "LOAD");
                assert_eq!(dtv.attrs.state, PropertyState::Idle);
                assert_eq!(dtv.perm, PropertyPerm::ReadWrite);
                assert_eq!(dtv.elements.len(), 1);
                assert_eq!(dtv.elements[0].name, "DRIVER");
            }
            _ => panic!("Expected DefTextVector"),
        }
    }

    #[test]
    fn test_parse_set_switch_vector() {
        let json = r#"{"setSwitchVector": {"device": "CCD Imager Simulator", "name": "CONNECTION", "state": "Ok", "items": [{"name": "CONNECTED", "value": true}, {"name": "DISCONNECTED", "value": false}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetSwitchVector(ssv) => {
                assert_eq!(ssv.attrs.device, "CCD Imager Simulator");
                assert_eq!(ssv.attrs.name, "CONNECTION");
                assert_eq!(ssv.attrs.state, Some(PropertyState::Ok));
                assert_eq!(ssv.elements.len(), 2);
                assert_eq!(ssv.elements[0].name, "CONNECTED");
                assert_eq!(ssv.elements[0].value, SwitchState::On);
                assert_eq!(ssv.elements[1].name, "DISCONNECTED");
                assert_eq!(ssv.elements[1].value, SwitchState::Off);
            }
            _ => panic!("Expected SetSwitchVector"),
        }
    }

    #[test]
    fn test_parse_new_number_vector() {
        let json = r#"{"newNumberVector":{"device":"CCD Imager Simulator","name":"CCD_EXPOSURE","token":"FA0012","items":[{"name":"EXPOSURE","value":1}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::NewNumberVector(nnv) => {
                assert_eq!(nnv.attrs.device, "CCD Imager Simulator");
                assert_eq!(nnv.attrs.name, "CCD_EXPOSURE");
                assert_eq!(nnv.elements.len(), 1);
                assert_eq!(nnv.elements[0].name, "EXPOSURE");
                assert_eq!(nnv.elements[0].value, 1.0);
            }
            _ => panic!("Expected NewNumberVector"),
        }
    }

    #[test]
    fn test_roundtrip_def_switch_vector() {
        let original = ProtocolMessage::DefSwitchVector(DefSwitchVector {
            attrs: VectorAttributes {
                device: "Server".to_string(),
                name: "RESTART".to_string(),
                label: "Restart".to_string(),
                group: "Main".to_string(),
                state: PropertyState::Idle,
                timeout: None,
                timestamp: None,
                message: None,
            },
            perm: PropertyPerm::ReadWrite,
            rule: SwitchRule::AnyOfMany,
            elements: vec![DefSwitch {
                name: "RESTART".to_string(),
                label: "Restart server".to_string(),
                value: SwitchState::Off,
            }],
        });

        let json = JsonProtocolSerializer::serialize(&original).unwrap();
        let parsed = JsonProtocolParser::parse_message(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_switch_state_bool_conversion() {
        assert_eq!(SwitchState::On.to_bool(), true);
        assert_eq!(SwitchState::Off.to_bool(), false);
        assert_eq!(SwitchState::from_bool(true), SwitchState::On);
        assert_eq!(SwitchState::from_bool(false), SwitchState::Off);
    }
}
