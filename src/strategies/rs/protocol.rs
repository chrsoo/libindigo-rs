//! INDIGO XML Protocol Parser and Serializer
//!
//! This module handles parsing and serialization of INDIGO XML protocol messages.
//!
//! # Overview
//!
//! The INDIGO protocol uses XML messages over TCP to communicate between clients and servers.
//! This module provides:
//!
//! - **Parsing**: Convert incoming XML messages into Rust types
//! - **Serialization**: Convert Rust types into XML messages for transmission
//! - **Protocol Messages**: Representations of all INDIGO protocol message types
//!
//! # Protocol Message Types
//!
//! The INDIGO protocol supports various message types including:
//!
//! - `defXXXVector`: Define property vectors (text, number, switch, light, BLOB)
//! - `setXXXVector`: Set property values
//! - `newXXXVector`: New property values from client
//! - `delProperty`: Delete property
//! - `message`: Status messages
//! - `getProperties`: Request property definitions
//! - `enableBLOB`: Control BLOB transfer
//!
//! # Example
//!
//! ```ignore
//! use libindigo::strategies::rs::protocol::{ProtocolParser, ProtocolMessage};
//!
//! let xml = b"<getProperties version=\"1.7\" device=\"CCD Simulator\"/>";
//! let message = ProtocolParser::parse_message(xml)?;
//! ```

use crate::error::{IndigoError, Result};
use quick_xml::events::{attributes::Attributes, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

// Re-export types from the types module
pub use crate::types::property::{PropertyPerm, PropertyState};

// ============================================================================
// Protocol Enums
// ============================================================================

/// State of a switch element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchState {
    /// Switch is off.
    Off,
    /// Switch is on.
    On,
}

impl SwitchState {
    /// Parses a switch state from a string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "Off" => Ok(SwitchState::Off),
            "On" => Ok(SwitchState::On),
            _ => Err(IndigoError::ParseError(format!(
                "Invalid switch state: {}",
                s
            ))),
        }
    }

    /// Converts the switch state to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            SwitchState::Off => "Off",
            SwitchState::On => "On",
        }
    }
}

impl std::fmt::Display for SwitchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Rule for switch vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchRule {
    /// Only one switch can be on at a time.
    OneOfMany,
    /// At most one switch can be on.
    AtMostOne,
    /// Any number of switches can be on.
    AnyOfMany,
}

impl SwitchRule {
    /// Parses a switch rule from a string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "OneOfMany" => Ok(SwitchRule::OneOfMany),
            "AtMostOne" => Ok(SwitchRule::AtMostOne),
            "AnyOfMany" => Ok(SwitchRule::AnyOfMany),
            _ => Err(IndigoError::ParseError(format!(
                "Invalid switch rule: {}",
                s
            ))),
        }
    }

    /// Converts the switch rule to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            SwitchRule::OneOfMany => "OneOfMany",
            SwitchRule::AtMostOne => "AtMostOne",
            SwitchRule::AnyOfMany => "AnyOfMany",
        }
    }
}

impl std::fmt::Display for SwitchRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// BLOB enable mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BLOBEnable {
    /// Never send BLOBs.
    Never,
    /// Send BLOBs in addition to URLs.
    Also,
    /// Only send BLOBs, no URLs.
    Only,
}

impl BLOBEnable {
    /// Parses a BLOB enable mode from a string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "Never" => Ok(BLOBEnable::Never),
            "Also" => Ok(BLOBEnable::Also),
            "Only" => Ok(BLOBEnable::Only),
            _ => Err(IndigoError::ParseError(format!(
                "Invalid BLOB enable: {}",
                s
            ))),
        }
    }

    /// Converts the BLOB enable mode to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            BLOBEnable::Never => "Never",
            BLOBEnable::Also => "Also",
            BLOBEnable::Only => "Only",
        }
    }
}

impl std::fmt::Display for BLOBEnable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PropertyState {
    /// Parses a property state from a string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "Idle" => Ok(PropertyState::Idle),
            "Ok" => Ok(PropertyState::Ok),
            "Busy" => Ok(PropertyState::Busy),
            "Alert" => Ok(PropertyState::Alert),
            _ => Err(IndigoError::ParseError(format!(
                "Invalid property state: {}",
                s
            ))),
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

impl PropertyPerm {
    /// Parses a property permission from a string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "ro" => Ok(PropertyPerm::ReadOnly),
            "wo" => Ok(PropertyPerm::WriteOnly),
            "rw" => Ok(PropertyPerm::ReadWrite),
            _ => Err(IndigoError::ParseError(format!(
                "Invalid property permission: {}",
                s
            ))),
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

// ============================================================================
// Property Element Structs
// ============================================================================

/// A text property element definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefText {
    /// Element name.
    pub name: String,
    /// Element label.
    pub label: String,
    /// Text value.
    pub value: String,
}

/// A number property element definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefNumber {
    /// Element name.
    pub name: String,
    /// Element label.
    pub label: String,
    /// Number format string.
    pub format: String,
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Step value.
    pub step: f64,
    /// Current value.
    pub value: f64,
}

/// A switch property element definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefSwitch {
    /// Element name.
    pub name: String,
    /// Element label.
    pub label: String,
    /// Switch state.
    pub value: SwitchState,
}

/// A light property element definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefLight {
    /// Element name.
    pub name: String,
    /// Element label.
    pub label: String,
    /// Light state.
    pub value: PropertyState,
}

/// A BLOB property element definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefBLOB {
    /// Element name.
    pub name: String,
    /// Element label.
    pub label: String,
}

/// A text property element value.
#[derive(Debug, Clone, PartialEq)]
pub struct OneText {
    /// Element name.
    pub name: String,
    /// Text value.
    pub value: String,
}

/// A number property element value.
#[derive(Debug, Clone, PartialEq)]
pub struct OneNumber {
    /// Element name.
    pub name: String,
    /// Number value.
    pub value: f64,
}

/// A switch property element value.
#[derive(Debug, Clone, PartialEq)]
pub struct OneSwitch {
    /// Element name.
    pub name: String,
    /// Switch state.
    pub value: SwitchState,
}

/// A light property element value.
#[derive(Debug, Clone, PartialEq)]
pub struct OneLight {
    /// Element name.
    pub name: String,
    /// Light state.
    pub value: PropertyState,
}

/// A BLOB property element value.
#[derive(Debug, Clone, PartialEq)]
pub struct OneBLOB {
    /// Element name.
    pub name: String,
    /// BLOB size in bytes.
    pub size: usize,
    /// BLOB format/extension.
    pub format: String,
    /// Base64-encoded BLOB data.
    pub value: String,
}

// ============================================================================
// Property Vector Structs
// ============================================================================

/// Common attributes for property vectors.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorAttributes {
    /// Device name.
    pub device: String,
    /// Property name.
    pub name: String,
    /// Property label.
    pub label: String,
    /// Property group.
    pub group: String,
    /// Property state.
    pub state: PropertyState,
    /// Timeout in seconds.
    pub timeout: Option<f64>,
    /// Timestamp.
    pub timestamp: Option<String>,
    /// Message.
    pub message: Option<String>,
}

/// Text property vector definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefTextVector {
    /// Common attributes.
    pub attrs: VectorAttributes,
    /// Permission.
    pub perm: PropertyPerm,
    /// Text elements.
    pub elements: Vec<DefText>,
}

/// Number property vector definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefNumberVector {
    /// Common attributes.
    pub attrs: VectorAttributes,
    /// Permission.
    pub perm: PropertyPerm,
    /// Number elements.
    pub elements: Vec<DefNumber>,
}

/// Switch property vector definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefSwitchVector {
    /// Common attributes.
    pub attrs: VectorAttributes,
    /// Permission.
    pub perm: PropertyPerm,
    /// Switch rule.
    pub rule: SwitchRule,
    /// Switch elements.
    pub elements: Vec<DefSwitch>,
}

/// Light property vector definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefLightVector {
    /// Common attributes (no perm for lights).
    pub attrs: VectorAttributes,
    /// Light elements.
    pub elements: Vec<DefLight>,
}

/// BLOB property vector definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefBLOBVector {
    /// Common attributes.
    pub attrs: VectorAttributes,
    /// Permission.
    pub perm: PropertyPerm,
    /// BLOB elements.
    pub elements: Vec<DefBLOB>,
}

// ============================================================================
// Set Vector Structs
// ============================================================================

/// Common attributes for set vectors.
#[derive(Debug, Clone, PartialEq)]
pub struct SetVectorAttributes {
    /// Device name.
    pub device: String,
    /// Property name.
    pub name: String,
    /// Property state.
    pub state: Option<PropertyState>,
    /// Timeout in seconds.
    pub timeout: Option<f64>,
    /// Timestamp.
    pub timestamp: Option<String>,
    /// Message.
    pub message: Option<String>,
}

/// Text property vector update.
#[derive(Debug, Clone, PartialEq)]
pub struct SetTextVector {
    /// Common attributes.
    pub attrs: SetVectorAttributes,
    /// Text elements.
    pub elements: Vec<OneText>,
}

/// Number property vector update.
#[derive(Debug, Clone, PartialEq)]
pub struct SetNumberVector {
    /// Common attributes.
    pub attrs: SetVectorAttributes,
    /// Number elements.
    pub elements: Vec<OneNumber>,
}

/// Switch property vector update.
#[derive(Debug, Clone, PartialEq)]
pub struct SetSwitchVector {
    /// Common attributes.
    pub attrs: SetVectorAttributes,
    /// Switch elements.
    pub elements: Vec<OneSwitch>,
}

/// Light property vector update.
#[derive(Debug, Clone, PartialEq)]
pub struct SetLightVector {
    /// Common attributes.
    pub attrs: SetVectorAttributes,
    /// Light elements.
    pub elements: Vec<OneLight>,
}

/// BLOB property vector update.
#[derive(Debug, Clone, PartialEq)]
pub struct SetBLOBVector {
    /// Common attributes.
    pub attrs: SetVectorAttributes,
    /// BLOB elements.
    pub elements: Vec<OneBLOB>,
}

// ============================================================================
// New Vector Structs (Client to Server)
// ============================================================================

/// Common attributes for new vectors.
#[derive(Debug, Clone, PartialEq)]
pub struct NewVectorAttributes {
    /// Device name.
    pub device: String,
    /// Property name.
    pub name: String,
    /// Timestamp.
    pub timestamp: Option<String>,
}

/// Text property vector from client.
#[derive(Debug, Clone, PartialEq)]
pub struct NewTextVector {
    /// Common attributes.
    pub attrs: NewVectorAttributes,
    /// Text elements.
    pub elements: Vec<OneText>,
}

/// Number property vector from client.
#[derive(Debug, Clone, PartialEq)]
pub struct NewNumberVector {
    /// Common attributes.
    pub attrs: NewVectorAttributes,
    /// Number elements.
    pub elements: Vec<OneNumber>,
}

/// Switch property vector from client.
#[derive(Debug, Clone, PartialEq)]
pub struct NewSwitchVector {
    /// Common attributes.
    pub attrs: NewVectorAttributes,
    /// Switch elements.
    pub elements: Vec<OneSwitch>,
}

/// BLOB property vector from client.
#[derive(Debug, Clone, PartialEq)]
pub struct NewBLOBVector {
    /// Common attributes.
    pub attrs: NewVectorAttributes,
    /// BLOB elements.
    pub elements: Vec<OneBLOB>,
}

// ============================================================================
// Control Message Structs
// ============================================================================

/// Get properties request.
#[derive(Debug, Clone, PartialEq)]
pub struct GetProperties {
    /// Protocol version.
    pub version: Option<String>,
    /// Device name filter (optional).
    pub device: Option<String>,
    /// Property name filter (optional).
    pub name: Option<String>,
}

/// Enable BLOB transfer control.
#[derive(Debug, Clone, PartialEq)]
pub struct EnableBLOB {
    /// Device name.
    pub device: String,
    /// Property name filter (optional).
    pub name: Option<String>,
    /// BLOB enable mode.
    pub value: BLOBEnable,
}

/// Status message.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Device name (optional).
    pub device: Option<String>,
    /// Timestamp.
    pub timestamp: Option<String>,
    /// Message text.
    pub message: Option<String>,
}

/// Delete property command.
#[derive(Debug, Clone, PartialEq)]
pub struct DelProperty {
    /// Device name.
    pub device: String,
    /// Property name (optional, if None deletes all properties).
    pub name: Option<String>,
    /// Timestamp.
    pub timestamp: Option<String>,
    /// Message.
    pub message: Option<String>,
}

// ============================================================================
// Protocol Message Enum
// ============================================================================

/// Represents an INDIGO protocol message.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolMessage {
    // Definition messages (server to client)
    /// Define text property vector.
    DefTextVector(DefTextVector),
    /// Define number property vector.
    DefNumberVector(DefNumberVector),
    /// Define switch property vector.
    DefSwitchVector(DefSwitchVector),
    /// Define light property vector.
    DefLightVector(DefLightVector),
    /// Define BLOB property vector.
    DefBLOBVector(DefBLOBVector),

    // Set messages (server to client)
    /// Set text property values.
    SetTextVector(SetTextVector),
    /// Set number property values.
    SetNumberVector(SetNumberVector),
    /// Set switch property values.
    SetSwitchVector(SetSwitchVector),
    /// Set light property values.
    SetLightVector(SetLightVector),
    /// Set BLOB property values.
    SetBLOBVector(SetBLOBVector),

    // New messages (client to server)
    /// New text property values from client.
    NewTextVector(NewTextVector),
    /// New number property values from client.
    NewNumberVector(NewNumberVector),
    /// New switch property values from client.
    NewSwitchVector(NewSwitchVector),
    /// New BLOB property values from client.
    NewBLOBVector(NewBLOBVector),

    // Control messages
    /// Get properties request.
    GetProperties(GetProperties),
    /// Enable BLOB transfer.
    EnableBLOB(EnableBLOB),
    /// Status message.
    Message(Message),
    /// Delete property.
    DelProperty(DelProperty),
}

// ============================================================================
// Protocol Parser
// ============================================================================

/// Protocol message parser.
///
/// Parses INDIGO XML protocol messages from a byte stream.
pub struct ProtocolParser;

impl ProtocolParser {
    /// Parses a single protocol message from XML bytes.
    ///
    /// The XML should contain a single message element (not wrapped in `<INDI>`).
    pub fn parse_message(xml: &[u8]) -> Result<ProtocolMessage> {
        let mut reader = Reader::from_reader(xml);
        reader.trim_text(true);

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| IndigoError::ParseError(format!("Invalid UTF-8: {}", e)))?;

                    return match name_str {
                        "defTextVector" => Ok(ProtocolMessage::DefTextVector(
                            Self::parse_def_text_vector(&mut reader, e.attributes())?,
                        )),
                        "defNumberVector" => Ok(ProtocolMessage::DefNumberVector(
                            Self::parse_def_number_vector(&mut reader, e.attributes())?,
                        )),
                        "defSwitchVector" => Ok(ProtocolMessage::DefSwitchVector(
                            Self::parse_def_switch_vector(&mut reader, e.attributes())?,
                        )),
                        "defLightVector" => Ok(ProtocolMessage::DefLightVector(
                            Self::parse_def_light_vector(&mut reader, e.attributes())?,
                        )),
                        "defBLOBVector" => Ok(ProtocolMessage::DefBLOBVector(
                            Self::parse_def_blob_vector(&mut reader, e.attributes())?,
                        )),
                        "setTextVector" => Ok(ProtocolMessage::SetTextVector(
                            Self::parse_set_text_vector(&mut reader, e.attributes())?,
                        )),
                        "setNumberVector" => Ok(ProtocolMessage::SetNumberVector(
                            Self::parse_set_number_vector(&mut reader, e.attributes())?,
                        )),
                        "setSwitchVector" => Ok(ProtocolMessage::SetSwitchVector(
                            Self::parse_set_switch_vector(&mut reader, e.attributes())?,
                        )),
                        "setLightVector" => Ok(ProtocolMessage::SetLightVector(
                            Self::parse_set_light_vector(&mut reader, e.attributes())?,
                        )),
                        "setBLOBVector" => Ok(ProtocolMessage::SetBLOBVector(
                            Self::parse_set_blob_vector(&mut reader, e.attributes())?,
                        )),
                        "newTextVector" => Ok(ProtocolMessage::NewTextVector(
                            Self::parse_new_text_vector(&mut reader, e.attributes())?,
                        )),
                        "newNumberVector" => Ok(ProtocolMessage::NewNumberVector(
                            Self::parse_new_number_vector(&mut reader, e.attributes())?,
                        )),
                        "newSwitchVector" => Ok(ProtocolMessage::NewSwitchVector(
                            Self::parse_new_switch_vector(&mut reader, e.attributes())?,
                        )),
                        "newBLOBVector" => Ok(ProtocolMessage::NewBLOBVector(
                            Self::parse_new_blob_vector(&mut reader, e.attributes())?,
                        )),
                        "getProperties" => Ok(ProtocolMessage::GetProperties(
                            Self::parse_get_properties(e.attributes())?,
                        )),
                        "enableBLOB" => Ok(ProtocolMessage::EnableBLOB(Self::parse_enable_blob(
                            &mut reader,
                            e.attributes(),
                        )?)),
                        "message" => Ok(ProtocolMessage::Message(Self::parse_message_element(
                            e.attributes(),
                        )?)),
                        "delProperty" => Ok(ProtocolMessage::DelProperty(
                            Self::parse_del_property(e.attributes())?,
                        )),
                        _ => Err(IndigoError::ParseError(format!(
                            "Unknown message type: {}",
                            name_str
                        ))),
                    };
                }
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }
    }

    // Helper to get required attribute
    fn get_attr(attrs: &Attributes, name: &str) -> Result<String> {
        for attr in attrs.clone() {
            let attr =
                attr.map_err(|e| IndigoError::ParseError(format!("Attribute error: {}", e)))?;
            if attr.key.as_ref() == name.as_bytes() {
                return Ok(std::str::from_utf8(&attr.value)
                    .map_err(|e| IndigoError::ParseError(format!("Invalid UTF-8: {}", e)))?
                    .to_string());
            }
        }
        Err(IndigoError::ParseError(format!(
            "Missing required attribute: {}",
            name
        )))
    }

    // Helper to get optional attribute
    fn get_opt_attr(attrs: &Attributes, name: &str) -> Result<Option<String>> {
        for attr in attrs.clone() {
            let attr =
                attr.map_err(|e| IndigoError::ParseError(format!("Attribute error: {}", e)))?;
            if attr.key.as_ref() == name.as_bytes() {
                return Ok(Some(
                    std::str::from_utf8(&attr.value)
                        .map_err(|e| IndigoError::ParseError(format!("Invalid UTF-8: {}", e)))?
                        .to_string(),
                ));
            }
        }
        Ok(None)
    }

    // Parse common vector attributes
    fn parse_vector_attrs(attrs: Attributes) -> Result<VectorAttributes> {
        Ok(VectorAttributes {
            device: Self::get_attr(&attrs, "device")?,
            name: Self::get_attr(&attrs, "name")?,
            label: Self::get_opt_attr(&attrs, "label")?.unwrap_or_default(),
            group: Self::get_opt_attr(&attrs, "group")?.unwrap_or_default(),
            state: PropertyState::from_str(&Self::get_attr(&attrs, "state")?)?,
            timeout: Self::get_opt_attr(&attrs, "timeout")?.and_then(|s| s.parse().ok()),
            timestamp: Self::get_opt_attr(&attrs, "timestamp")?,
            message: Self::get_opt_attr(&attrs, "message")?,
        })
    }

    // Parse set vector attributes
    fn parse_set_vector_attrs(attrs: Attributes) -> Result<SetVectorAttributes> {
        Ok(SetVectorAttributes {
            device: Self::get_attr(&attrs, "device")?,
            name: Self::get_attr(&attrs, "name")?,
            state: Self::get_opt_attr(&attrs, "state")?
                .map(|s| PropertyState::from_str(&s))
                .transpose()?,
            timeout: Self::get_opt_attr(&attrs, "timeout")?.and_then(|s| s.parse().ok()),
            timestamp: Self::get_opt_attr(&attrs, "timestamp")?,
            message: Self::get_opt_attr(&attrs, "message")?,
        })
    }

    // Parse new vector attributes
    fn parse_new_vector_attrs(attrs: Attributes) -> Result<NewVectorAttributes> {
        Ok(NewVectorAttributes {
            device: Self::get_attr(&attrs, "device")?,
            name: Self::get_attr(&attrs, "name")?,
            timestamp: Self::get_opt_attr(&attrs, "timestamp")?,
        })
    }

    // Read text content from an element
    fn read_text_content<R: std::io::BufRead>(reader: &mut Reader<R>) -> Result<String> {
        let mut buf = Vec::new();
        let mut content = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    content.push_str(
                        &e.unescape().map_err(|e| {
                            IndigoError::ParseError(format!("Unescape error: {}", e))
                        })?,
                    );
                }
                Ok(Event::End(_)) | Ok(Event::Eof) => break,
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(content)
    }

    // Parse defTextVector
    fn parse_def_text_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<DefTextVector> {
        let vector_attrs = Self::parse_vector_attrs(attrs.clone())?;
        let perm = PropertyPerm::from_str(&Self::get_attr(&attrs, "perm")?)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"defText" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let label = Self::get_opt_attr(&e.attributes(), "label")?.unwrap_or_default();
                    let value = Self::read_text_content(reader)?;
                    elements.push(DefText { name, label, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"defTextVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(DefTextVector {
            attrs: vector_attrs,
            perm,
            elements,
        })
    }

    // Parse defNumberVector
    fn parse_def_number_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<DefNumberVector> {
        let vector_attrs = Self::parse_vector_attrs(attrs.clone())?;
        let perm = PropertyPerm::from_str(&Self::get_attr(&attrs, "perm")?)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"defNumber" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let label = Self::get_opt_attr(&e.attributes(), "label")?.unwrap_or_default();
                    let format = Self::get_attr(&e.attributes(), "format")?;
                    let min = Self::get_attr(&e.attributes(), "min")?
                        .parse()
                        .map_err(|e| IndigoError::ParseError(format!("Invalid min: {}", e)))?;
                    let max = Self::get_attr(&e.attributes(), "max")?
                        .parse()
                        .map_err(|e| IndigoError::ParseError(format!("Invalid max: {}", e)))?;
                    let step = Self::get_attr(&e.attributes(), "step")?
                        .parse()
                        .map_err(|e| IndigoError::ParseError(format!("Invalid step: {}", e)))?;
                    let value_str = Self::read_text_content(reader)?;
                    let value = value_str
                        .trim()
                        .parse()
                        .map_err(|e| IndigoError::ParseError(format!("Invalid value: {}", e)))?;
                    elements.push(DefNumber {
                        name,
                        label,
                        format,
                        min,
                        max,
                        step,
                        value,
                    });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"defNumberVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(DefNumberVector {
            attrs: vector_attrs,
            perm,
            elements,
        })
    }

    // Parse defSwitchVector
    fn parse_def_switch_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<DefSwitchVector> {
        let vector_attrs = Self::parse_vector_attrs(attrs.clone())?;
        let perm = PropertyPerm::from_str(&Self::get_attr(&attrs, "perm")?)?;
        let rule = SwitchRule::from_str(&Self::get_attr(&attrs, "rule")?)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"defSwitch" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let label = Self::get_opt_attr(&e.attributes(), "label")?.unwrap_or_default();
                    let value_str = Self::read_text_content(reader)?;
                    let value = SwitchState::from_str(value_str.trim())?;
                    elements.push(DefSwitch { name, label, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"defSwitchVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(DefSwitchVector {
            attrs: vector_attrs,
            perm,
            rule,
            elements,
        })
    }

    // Parse defLightVector
    fn parse_def_light_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<DefLightVector> {
        let vector_attrs = Self::parse_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"defLight" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let label = Self::get_opt_attr(&e.attributes(), "label")?.unwrap_or_default();
                    let value_str = Self::read_text_content(reader)?;
                    let value = PropertyState::from_str(value_str.trim())?;
                    elements.push(DefLight { name, label, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"defLightVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(DefLightVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse defBLOBVector
    fn parse_def_blob_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<DefBLOBVector> {
        let vector_attrs = Self::parse_vector_attrs(attrs.clone())?;
        let perm = PropertyPerm::from_str(&Self::get_attr(&attrs, "perm")?)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(e)) if e.name().as_ref() == b"defBLOB" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let label = Self::get_opt_attr(&e.attributes(), "label")?.unwrap_or_default();
                    elements.push(DefBLOB { name, label });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"defBLOBVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(DefBLOBVector {
            attrs: vector_attrs,
            perm,
            elements,
        })
    }

    // Parse setTextVector
    fn parse_set_text_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<SetTextVector> {
        let vector_attrs = Self::parse_set_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneText" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let value = Self::read_text_content(reader)?;
                    elements.push(OneText { name, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"setTextVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(SetTextVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse setNumberVector
    fn parse_set_number_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<SetNumberVector> {
        let vector_attrs = Self::parse_set_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneNumber" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let value_str = Self::read_text_content(reader)?;
                    let value = value_str
                        .trim()
                        .parse()
                        .map_err(|e| IndigoError::ParseError(format!("Invalid value: {}", e)))?;
                    elements.push(OneNumber { name, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"setNumberVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(SetNumberVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse setSwitchVector
    fn parse_set_switch_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<SetSwitchVector> {
        let vector_attrs = Self::parse_set_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneSwitch" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let value_str = Self::read_text_content(reader)?;
                    let value = SwitchState::from_str(value_str.trim())?;
                    elements.push(OneSwitch { name, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"setSwitchVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(SetSwitchVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse setLightVector
    fn parse_set_light_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<SetLightVector> {
        let vector_attrs = Self::parse_set_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneLight" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let value_str = Self::read_text_content(reader)?;
                    let value = PropertyState::from_str(value_str.trim())?;
                    elements.push(OneLight { name, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"setLightVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(SetLightVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse setBLOBVector
    fn parse_set_blob_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<SetBLOBVector> {
        let vector_attrs = Self::parse_set_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneBLOB" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let size = Self::get_attr(&e.attributes(), "size")?
                        .parse()
                        .map_err(|e| IndigoError::ParseError(format!("Invalid size: {}", e)))?;
                    let format = Self::get_attr(&e.attributes(), "format")?;
                    let value = Self::read_text_content(reader)?;
                    elements.push(OneBLOB {
                        name,
                        size,
                        format,
                        value,
                    });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"setBLOBVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(SetBLOBVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse newTextVector
    fn parse_new_text_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<NewTextVector> {
        let vector_attrs = Self::parse_new_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneText" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let value = Self::read_text_content(reader)?;
                    elements.push(OneText { name, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"newTextVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(NewTextVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse newNumberVector
    fn parse_new_number_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<NewNumberVector> {
        let vector_attrs = Self::parse_new_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneNumber" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let value_str = Self::read_text_content(reader)?;
                    let value = value_str
                        .trim()
                        .parse()
                        .map_err(|e| IndigoError::ParseError(format!("Invalid value: {}", e)))?;
                    elements.push(OneNumber { name, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"newNumberVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(NewNumberVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse newSwitchVector
    fn parse_new_switch_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<NewSwitchVector> {
        let vector_attrs = Self::parse_new_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneSwitch" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let value_str = Self::read_text_content(reader)?;
                    let value = SwitchState::from_str(value_str.trim())?;
                    elements.push(OneSwitch { name, value });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"newSwitchVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(NewSwitchVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse newBLOBVector
    fn parse_new_blob_vector<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<NewBLOBVector> {
        let vector_attrs = Self::parse_new_vector_attrs(attrs)?;
        let mut elements = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"oneBLOB" => {
                    let name = Self::get_attr(&e.attributes(), "name")?;
                    let size = Self::get_attr(&e.attributes(), "size")?
                        .parse()
                        .map_err(|e| IndigoError::ParseError(format!("Invalid size: {}", e)))?;
                    let format = Self::get_attr(&e.attributes(), "format")?;
                    let value = Self::read_text_content(reader)?;
                    elements.push(OneBLOB {
                        name,
                        size,
                        format,
                        value,
                    });
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"newBLOBVector" => break,
                Ok(Event::Eof) => {
                    return Err(IndigoError::ParseError("Unexpected EOF".to_string()))
                }
                Err(e) => return Err(IndigoError::ParseError(format!("XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(NewBLOBVector {
            attrs: vector_attrs,
            elements,
        })
    }

    // Parse getProperties
    fn parse_get_properties(attrs: Attributes) -> Result<GetProperties> {
        Ok(GetProperties {
            version: Self::get_opt_attr(&attrs, "version")?,
            device: Self::get_opt_attr(&attrs, "device")?,
            name: Self::get_opt_attr(&attrs, "name")?,
        })
    }

    // Parse enableBLOB
    fn parse_enable_blob<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attrs: Attributes,
    ) -> Result<EnableBLOB> {
        let device = Self::get_attr(&attrs, "device")?;
        let name = Self::get_opt_attr(&attrs, "name")?;
        let value_str = Self::read_text_content(reader)?;
        let value = BLOBEnable::from_str(value_str.trim())?;

        Ok(EnableBLOB {
            device,
            name,
            value,
        })
    }

    // Parse message element
    fn parse_message_element(attrs: Attributes) -> Result<Message> {
        Ok(Message {
            device: Self::get_opt_attr(&attrs, "device")?,
            timestamp: Self::get_opt_attr(&attrs, "timestamp")?,
            message: Self::get_opt_attr(&attrs, "message")?,
        })
    }

    // Parse delProperty
    fn parse_del_property(attrs: Attributes) -> Result<DelProperty> {
        Ok(DelProperty {
            device: Self::get_attr(&attrs, "device")?,
            name: Self::get_opt_attr(&attrs, "name")?,
            timestamp: Self::get_opt_attr(&attrs, "timestamp")?,
            message: Self::get_opt_attr(&attrs, "message")?,
        })
    }
}

// ============================================================================
// Protocol Serializer
// ============================================================================

/// Protocol message serializer.
///
/// Serializes Rust types into INDIGO XML protocol messages.
pub struct ProtocolSerializer;

impl ProtocolSerializer {
    /// Serializes a protocol message to XML bytes.
    pub fn serialize(message: &ProtocolMessage) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        match message {
            ProtocolMessage::DefTextVector(v) => Self::write_def_text_vector(&mut writer, v)?,
            ProtocolMessage::DefNumberVector(v) => Self::write_def_number_vector(&mut writer, v)?,
            ProtocolMessage::DefSwitchVector(v) => Self::write_def_switch_vector(&mut writer, v)?,
            ProtocolMessage::DefLightVector(v) => Self::write_def_light_vector(&mut writer, v)?,
            ProtocolMessage::DefBLOBVector(v) => Self::write_def_blob_vector(&mut writer, v)?,
            ProtocolMessage::SetTextVector(v) => Self::write_set_text_vector(&mut writer, v)?,
            ProtocolMessage::SetNumberVector(v) => Self::write_set_number_vector(&mut writer, v)?,
            ProtocolMessage::SetSwitchVector(v) => Self::write_set_switch_vector(&mut writer, v)?,
            ProtocolMessage::SetLightVector(v) => Self::write_set_light_vector(&mut writer, v)?,
            ProtocolMessage::SetBLOBVector(v) => Self::write_set_blob_vector(&mut writer, v)?,
            ProtocolMessage::NewTextVector(v) => Self::write_new_text_vector(&mut writer, v)?,
            ProtocolMessage::NewNumberVector(v) => Self::write_new_number_vector(&mut writer, v)?,
            ProtocolMessage::NewSwitchVector(v) => Self::write_new_switch_vector(&mut writer, v)?,
            ProtocolMessage::NewBLOBVector(v) => Self::write_new_blob_vector(&mut writer, v)?,
            ProtocolMessage::GetProperties(v) => Self::write_get_properties(&mut writer, v)?,
            ProtocolMessage::EnableBLOB(v) => Self::write_enable_blob(&mut writer, v)?,
            ProtocolMessage::Message(v) => Self::write_message(&mut writer, v)?,
            ProtocolMessage::DelProperty(v) => Self::write_del_property(&mut writer, v)?,
        }

        Ok(writer.into_inner().into_inner())
    }

    // Helper to write attributes
    fn write_vector_attrs(elem: &mut BytesStart, attrs: &VectorAttributes) {
        elem.push_attribute(("device", attrs.device.as_str()));
        elem.push_attribute(("name", attrs.name.as_str()));
        if !attrs.label.is_empty() {
            elem.push_attribute(("label", attrs.label.as_str()));
        }
        if !attrs.group.is_empty() {
            elem.push_attribute(("group", attrs.group.as_str()));
        }
        elem.push_attribute(("state", attrs.state.as_str()));
        if let Some(timeout) = attrs.timeout {
            elem.push_attribute(("timeout", timeout.to_string().as_str()));
        }
        if let Some(ref timestamp) = attrs.timestamp {
            elem.push_attribute(("timestamp", timestamp.as_str()));
        }
        if let Some(ref message) = attrs.message {
            elem.push_attribute(("message", message.as_str()));
        }
    }

    fn write_set_vector_attrs(elem: &mut BytesStart, attrs: &SetVectorAttributes) {
        elem.push_attribute(("device", attrs.device.as_str()));
        elem.push_attribute(("name", attrs.name.as_str()));
        if let Some(state) = attrs.state {
            elem.push_attribute(("state", state.as_str()));
        }
        if let Some(timeout) = attrs.timeout {
            elem.push_attribute(("timeout", timeout.to_string().as_str()));
        }
        if let Some(ref timestamp) = attrs.timestamp {
            elem.push_attribute(("timestamp", timestamp.as_str()));
        }
        if let Some(ref message) = attrs.message {
            elem.push_attribute(("message", message.as_str()));
        }
    }

    fn write_new_vector_attrs(elem: &mut BytesStart, attrs: &NewVectorAttributes) {
        elem.push_attribute(("device", attrs.device.as_str()));
        elem.push_attribute(("name", attrs.name.as_str()));
        if let Some(ref timestamp) = attrs.timestamp {
            elem.push_attribute(("timestamp", timestamp.as_str()));
        }
    }

    // Write defTextVector
    fn write_def_text_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &DefTextVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("defTextVector");
        Self::write_vector_attrs(&mut elem, &vector.attrs);
        elem.push_attribute(("perm", vector.perm.as_str()));

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("defText");
            item_elem.push_attribute(("name", item.name.as_str()));
            if !item.label.is_empty() {
                item_elem.push_attribute(("label", item.label.as_str()));
            }

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(&item.value)))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("defText")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("defTextVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write defNumberVector
    fn write_def_number_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &DefNumberVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("defNumberVector");
        Self::write_vector_attrs(&mut elem, &vector.attrs);
        elem.push_attribute(("perm", vector.perm.as_str()));

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("defNumber");
            item_elem.push_attribute(("name", item.name.as_str()));
            if !item.label.is_empty() {
                item_elem.push_attribute(("label", item.label.as_str()));
            }
            item_elem.push_attribute(("format", item.format.as_str()));
            item_elem.push_attribute(("min", item.min.to_string().as_str()));
            item_elem.push_attribute(("max", item.max.to_string().as_str()));
            item_elem.push_attribute(("step", item.step.to_string().as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(&item.value.to_string())))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("defNumber")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("defNumberVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write defSwitchVector
    fn write_def_switch_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &DefSwitchVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("defSwitchVector");
        Self::write_vector_attrs(&mut elem, &vector.attrs);
        elem.push_attribute(("perm", vector.perm.as_str()));
        elem.push_attribute(("rule", vector.rule.as_str()));

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("defSwitch");
            item_elem.push_attribute(("name", item.name.as_str()));
            if !item.label.is_empty() {
                item_elem.push_attribute(("label", item.label.as_str()));
            }

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(item.value.as_str())))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("defSwitch")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("defSwitchVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write defLightVector
    fn write_def_light_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &DefLightVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("defLightVector");
        Self::write_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("defLight");
            item_elem.push_attribute(("name", item.name.as_str()));
            if !item.label.is_empty() {
                item_elem.push_attribute(("label", item.label.as_str()));
            }

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(item.value.as_str())))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("defLight")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("defLightVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write defBLOBVector
    fn write_def_blob_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &DefBLOBVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("defBLOBVector");
        Self::write_vector_attrs(&mut elem, &vector.attrs);
        elem.push_attribute(("perm", vector.perm.as_str()));

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("defBLOB");
            item_elem.push_attribute(("name", item.name.as_str()));
            if !item.label.is_empty() {
                item_elem.push_attribute(("label", item.label.as_str()));
            }

            writer
                .write_event(Event::Empty(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("defBLOBVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write setTextVector
    fn write_set_text_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &SetTextVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("setTextVector");
        Self::write_set_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneText");
            item_elem.push_attribute(("name", item.name.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(&item.value)))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneText")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("setTextVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write setNumberVector
    fn write_set_number_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &SetNumberVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("setNumberVector");
        Self::write_set_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneNumber");
            item_elem.push_attribute(("name", item.name.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(&item.value.to_string())))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneNumber")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("setNumberVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write setSwitchVector
    fn write_set_switch_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &SetSwitchVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("setSwitchVector");
        Self::write_set_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneSwitch");
            item_elem.push_attribute(("name", item.name.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(item.value.as_str())))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneSwitch")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("setSwitchVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write setLightVector
    fn write_set_light_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &SetLightVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("setLightVector");
        Self::write_set_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneLight");
            item_elem.push_attribute(("name", item.name.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(item.value.as_str())))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneLight")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("setLightVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write setBLOBVector
    fn write_set_blob_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &SetBLOBVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("setBLOBVector");
        Self::write_set_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneBLOB");
            item_elem.push_attribute(("name", item.name.as_str()));
            item_elem.push_attribute(("size", item.size.to_string().as_str()));
            item_elem.push_attribute(("format", item.format.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(&item.value)))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneBLOB")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("setBLOBVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write newTextVector
    fn write_new_text_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &NewTextVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("newTextVector");
        Self::write_new_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneText");
            item_elem.push_attribute(("name", item.name.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(&item.value)))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneText")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("newTextVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write newNumberVector
    fn write_new_number_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &NewNumberVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("newNumberVector");
        Self::write_new_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneNumber");
            item_elem.push_attribute(("name", item.name.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(&item.value.to_string())))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneNumber")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("newNumberVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write newSwitchVector
    fn write_new_switch_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &NewSwitchVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("newSwitchVector");
        Self::write_new_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneSwitch");
            item_elem.push_attribute(("name", item.name.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(item.value.as_str())))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneSwitch")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("newSwitchVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write newBLOBVector
    fn write_new_blob_vector<W: std::io::Write>(
        writer: &mut Writer<W>,
        vector: &NewBLOBVector,
    ) -> Result<()> {
        let mut elem = BytesStart::new("newBLOBVector");
        Self::write_new_vector_attrs(&mut elem, &vector.attrs);

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        for item in &vector.elements {
            let mut item_elem = BytesStart::new("oneBLOB");
            item_elem.push_attribute(("name", item.name.as_str()));
            item_elem.push_attribute(("size", item.size.to_string().as_str()));
            item_elem.push_attribute(("format", item.format.as_str()));

            writer
                .write_event(Event::Start(item_elem))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::Text(BytesText::new(&item.value)))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
            writer
                .write_event(Event::End(BytesEnd::new("oneBLOB")))
                .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("newBLOBVector")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write getProperties
    fn write_get_properties<W: std::io::Write>(
        writer: &mut Writer<W>,
        msg: &GetProperties,
    ) -> Result<()> {
        let mut elem = BytesStart::new("getProperties");
        if let Some(ref version) = msg.version {
            elem.push_attribute(("version", version.as_str()));
        }
        if let Some(ref device) = msg.device {
            elem.push_attribute(("device", device.as_str()));
        }
        if let Some(ref name) = msg.name {
            elem.push_attribute(("name", name.as_str()));
        }

        writer
            .write_event(Event::Empty(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write enableBLOB
    fn write_enable_blob<W: std::io::Write>(
        writer: &mut Writer<W>,
        msg: &EnableBLOB,
    ) -> Result<()> {
        let mut elem = BytesStart::new("enableBLOB");
        elem.push_attribute(("device", msg.device.as_str()));
        if let Some(ref name) = msg.name {
            elem.push_attribute(("name", name.as_str()));
        }

        writer
            .write_event(Event::Start(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        writer
            .write_event(Event::Text(BytesText::new(msg.value.as_str())))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;
        writer
            .write_event(Event::End(BytesEnd::new("enableBLOB")))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write message
    fn write_message<W: std::io::Write>(writer: &mut Writer<W>, msg: &Message) -> Result<()> {
        let mut elem = BytesStart::new("message");
        if let Some(ref device) = msg.device {
            elem.push_attribute(("device", device.as_str()));
        }
        if let Some(ref timestamp) = msg.timestamp {
            elem.push_attribute(("timestamp", timestamp.as_str()));
        }
        if let Some(ref message) = msg.message {
            elem.push_attribute(("message", message.as_str()));
        }

        writer
            .write_event(Event::Empty(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
    }

    // Write delProperty
    fn write_del_property<W: std::io::Write>(
        writer: &mut Writer<W>,
        msg: &DelProperty,
    ) -> Result<()> {
        let mut elem = BytesStart::new("delProperty");
        elem.push_attribute(("device", msg.device.as_str()));
        if let Some(ref name) = msg.name {
            elem.push_attribute(("name", name.as_str()));
        }
        if let Some(ref timestamp) = msg.timestamp {
            elem.push_attribute(("timestamp", timestamp.as_str()));
        }
        if let Some(ref message) = msg.message {
            elem.push_attribute(("message", message.as_str()));
        }

        writer
            .write_event(Event::Empty(elem))
            .map_err(|e| IndigoError::ProtocolError(format!("Write error: {}", e)))?;

        Ok(())
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
        let xml = b"<getProperties version=\"1.7\" device=\"CCD Simulator\"/>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::GetProperties(gp) => {
                assert_eq!(gp.version, Some("1.7".to_string()));
                assert_eq!(gp.device, Some("CCD Simulator".to_string()));
                assert_eq!(gp.name, None);
            }
            _ => panic!("Expected GetProperties"),
        }
    }

    #[test]
    fn test_serialize_get_properties() {
        let msg = ProtocolMessage::GetProperties(GetProperties {
            version: Some("1.7".to_string()),
            device: Some("CCD Simulator".to_string()),
            name: None,
        });

        let xml = ProtocolSerializer::serialize(&msg).unwrap();
        let xml_str = String::from_utf8(xml).unwrap();

        assert!(xml_str.contains("getProperties"));
        assert!(xml_str.contains("version=\"1.7\""));
        assert!(xml_str.contains("device=\"CCD Simulator\""));
    }

    #[test]
    fn test_parse_def_text_vector() {
        let xml =
            b"<defTextVector device=\"CCD Simulator\" name=\"INFO\" state=\"Idle\" perm=\"ro\">
            <defText name=\"MAKE\">Acme</defText>
        </defTextVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(dtv) => {
                assert_eq!(dtv.attrs.device, "CCD Simulator");
                assert_eq!(dtv.attrs.name, "INFO");
                assert_eq!(dtv.attrs.state, PropertyState::Idle);
                assert_eq!(dtv.perm, PropertyPerm::ReadOnly);
                assert_eq!(dtv.elements.len(), 1);
                assert_eq!(dtv.elements[0].name, "MAKE");
                assert_eq!(dtv.elements[0].value, "Acme");
            }
            _ => panic!("Expected DefTextVector"),
        }
    }

    #[test]
    fn test_parse_new_switch_vector() {
        let xml = b"<newSwitchVector device=\"Telescope Simulator\" name=\"POWER\">
            <oneSwitch name=\"ON\">On</oneSwitch>
        </newSwitchVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::NewSwitchVector(nsv) => {
                assert_eq!(nsv.attrs.device, "Telescope Simulator");
                assert_eq!(nsv.attrs.name, "POWER");
                assert_eq!(nsv.elements.len(), 1);
                assert_eq!(nsv.elements[0].name, "ON");
                assert_eq!(nsv.elements[0].value, SwitchState::On);
            }
            _ => panic!("Expected NewSwitchVector"),
        }
    }

    #[test]
    fn test_roundtrip_def_number_vector() {
        let original = ProtocolMessage::DefNumberVector(DefNumberVector {
            attrs: VectorAttributes {
                device: "Mount".to_string(),
                name: "EQUATORIAL_EOD_COORD".to_string(),
                label: "Coordinates".to_string(),
                group: "Main".to_string(),
                state: PropertyState::Ok,
                timeout: Some(60.0),
                timestamp: None,
                message: None,
            },
            perm: PropertyPerm::ReadWrite,
            elements: vec![DefNumber {
                name: "RA".to_string(),
                label: "RA (hh:mm:ss)".to_string(),
                format: "%010.6m".to_string(),
                min: 0.0,
                max: 24.0,
                step: 0.0,
                value: 12.5,
            }],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        assert_eq!(original, parsed);
    }
}
