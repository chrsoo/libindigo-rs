//! Core types for INDIGO properties, devices, and values.
//!
//! This module provides the fundamental types used throughout the libindigo API,
//! including property definitions, device information, and value types.

pub mod device;
pub mod property;
pub mod value;

pub use device::{Device, DeviceInfo};
pub use property::{Property, PropertyItem, PropertyPerm, PropertyState, PropertyType};
pub use value::{BlobTransferMode, LightState, PropertyValue, SwitchRule, SwitchState};
