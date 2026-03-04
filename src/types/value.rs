//! Value types for INDIGO properties.
//!
//! This module defines the various value types that can be contained
//! in INDIGO properties (text, number, switch, light, BLOB).

use std::fmt;

/// Represents the different types of values that can be stored in a property.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// Text value.
    Text(String),

    /// Numeric value with optional format information.
    Number {
        /// The numeric value.
        value: f64,
        /// Minimum allowed value.
        min: f64,
        /// Maximum allowed value.
        max: f64,
        /// Step size for increments.
        step: f64,
        /// Display format string (printf-style).
        format: String,
    },

    /// Switch (boolean) value.
    Switch {
        /// Current state of the switch.
        state: SwitchState,
    },

    /// Light indicator value (read-only status).
    Light {
        /// Current state of the light.
        state: LightState,
    },

    /// Binary Large Object (BLOB) value.
    Blob {
        /// BLOB data.
        data: Vec<u8>,
        /// MIME type of the data.
        format: String,
        /// Size of the data in bytes.
        size: usize,
    },
}

/// State of a switch property item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchState {
    /// Switch is off.
    Off,
    /// Switch is on.
    On,
}

impl fmt::Display for SwitchState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwitchState::Off => write!(f, "Off"),
            SwitchState::On => write!(f, "On"),
        }
    }
}

/// Rule for switch property behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchRule {
    /// At most one switch can be on.
    OneOfMany,
    /// At most one switch can be on, but all can be off.
    AtMostOne,
    /// Any number of switches can be on.
    AnyOfMany,
}

impl fmt::Display for SwitchRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwitchRule::OneOfMany => write!(f, "OneOfMany"),
            SwitchRule::AtMostOne => write!(f, "AtMostOne"),
            SwitchRule::AnyOfMany => write!(f, "AnyOfMany"),
        }
    }
}

/// State of a light property item (read-only indicator).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightState {
    /// Idle state.
    Idle,
    /// OK/success state.
    Ok,
    /// Busy/in-progress state.
    Busy,
    /// Alert/error state.
    Alert,
}

impl fmt::Display for LightState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LightState::Idle => write!(f, "Idle"),
            LightState::Ok => write!(f, "Ok"),
            LightState::Busy => write!(f, "Busy"),
            LightState::Alert => write!(f, "Alert"),
        }
    }
}

impl PropertyValue {
    /// Creates a new text value.
    pub fn text(value: impl Into<String>) -> Self {
        PropertyValue::Text(value.into())
    }

    /// Creates a new number value with default range and format.
    pub fn number(value: f64) -> Self {
        PropertyValue::Number {
            value,
            min: f64::MIN,
            max: f64::MAX,
            step: 0.0,
            format: "%.2f".to_string(),
        }
    }

    /// Creates a new switch value.
    pub fn switch(state: SwitchState) -> Self {
        PropertyValue::Switch { state }
    }

    /// Creates a new light value.
    pub fn light(state: LightState) -> Self {
        PropertyValue::Light { state }
    }

    /// Creates a new BLOB value.
    pub fn blob(data: Vec<u8>, format: impl Into<String>) -> Self {
        let size = data.len();
        PropertyValue::Blob {
            data,
            format: format.into(),
            size,
        }
    }
}
