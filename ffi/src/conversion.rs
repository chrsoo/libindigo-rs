//! Conversions between INDIGO C structures and Rust types.
//!
//! This module handles the safe conversion of C property structures from
//! the `libindigo-sys` crate into the Rust `Property` and `PropertyValue` types.
//!
//! # Safety
//!
//! All functions in this module that accept raw pointers must be called with
//! valid pointers to properly initialized C structures. The caller is responsible
//! for ensuring pointer validity and proper memory management.

use libindigo::error::{IndigoError, Result};
use libindigo::types::{
    BlobTransferMode, LightState, Property, PropertyItem, PropertyPerm, PropertyState,
    PropertyType, PropertyValue, SwitchState,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Conditional compilation based on whether sys crate types are available
#[cfg(feature = "sys-available")]
use libindigo_sys::*;

/// Placeholder type for when sys crate is not available
#[cfg(not(feature = "sys-available"))]
#[repr(C)]
pub struct indigo_property {
    _private: [u8; 0],
}

/// Placeholder type for when sys crate is not available
#[cfg(not(feature = "sys-available"))]
#[repr(C)]
pub struct indigo_item {
    _private: [u8; 0],
}

/// Convert a C INDIGO property to a Rust Property.
///
/// # Safety
///
/// The caller must ensure that `c_property` is a valid pointer to a properly
/// initialized `indigo_property` structure.
///
/// # Errors
///
/// Returns an error if:
/// - The property type is unknown or unsupported
/// - String conversion from C fails (invalid UTF-8)
/// - Property structure is malformed
#[cfg(feature = "sys-available")]
pub unsafe fn property_from_c(c_property: *const indigo_property) -> Result<Property> {
    if c_property.is_null() {
        return Err(IndigoError::InvalidParameter(
            "Null property pointer".to_string(),
        ));
    }

    let prop = &*c_property;

    // Extract basic metadata
    let device = c_str_to_string(prop.device)?;
    let name = c_str_to_string(prop.name)?;
    let group = c_str_to_string(prop.group)?;
    let label = c_str_to_string(prop.label)?;

    // Convert state
    let state = state_from_c(prop.state)?;

    // Convert permission
    let perm = perm_from_c(prop.perm)?;

    // Convert property type and items
    let (property_type, items) = items_from_c(prop)?;

    // Extract optional fields
    let timeout = if prop.timeout > 0.0 {
        Some(prop.timeout)
    } else {
        None
    };

    let timestamp = if !prop.timestamp.is_null() {
        Some(c_str_to_string(prop.timestamp)?)
    } else {
        None
    };

    let message = if !prop.message.is_null() {
        Some(c_str_to_string(prop.message)?)
    } else {
        None
    };

    Ok(Property {
        device,
        name,
        group,
        label,
        state,
        perm,
        property_type,
        items,
        timeout,
        timestamp,
        message,
    })
}

/// Stub implementation when sys crate is not available
#[cfg(not(feature = "sys-available"))]
pub unsafe fn property_from_c(_c_property: *const indigo_property) -> Result<Property> {
    Err(IndigoError::NotSupported(
        "FFI not available - sys crate not built".to_string(),
    ))
}

/// Convert a Rust Property to C INDIGO property for sending.
///
/// # Safety
///
/// The returned pointer must be freed using [`free_c_property`] when no longer needed.
///
/// # Errors
///
/// Returns an error if:
/// - String conversion to C fails (contains null bytes)
/// - Memory allocation fails
/// - Property type is unsupported
#[cfg(feature = "sys-available")]
pub unsafe fn property_to_c(property: &Property) -> Result<*mut indigo_property> {
    // This is a complex operation that would require allocating C structures
    // For now, return NotSupported - full implementation would need careful memory management
    Err(IndigoError::NotSupported(
        "property_to_c not yet implemented".to_string(),
    ))
}

/// Stub implementation when sys crate is not available
#[cfg(not(feature = "sys-available"))]
pub unsafe fn property_to_c(_property: &Property) -> Result<*mut indigo_property> {
    Err(IndigoError::NotSupported(
        "FFI not available - sys crate not built".to_string(),
    ))
}

/// Frees a C indigo_property allocated by property_to_c.
///
/// # Safety
///
/// The caller must ensure that `prop` was allocated by [`property_to_c`] and
/// has not been freed already.
#[cfg(feature = "sys-available")]
pub unsafe fn free_c_property(_prop: *mut indigo_property) {
    // TODO: Implement proper cleanup of C structures
    // This would need to free all nested allocations (strings, items, etc.)
}

/// Stub implementation when sys crate is not available
#[cfg(not(feature = "sys-available"))]
pub unsafe fn free_c_property(_prop: *mut indigo_property) {
    // No-op when sys crate not available
}

/// Convert a C property state to Rust PropertyState.
///
/// # Errors
///
/// Returns an error if the state value is not recognized.
#[cfg(feature = "sys-available")]
fn state_from_c(state: indigo_property_state) -> Result<PropertyState> {
    // The sys crate uses NewType enums, so we need to access the inner value
    match state.0 {
        0 => Ok(PropertyState::Idle),
        1 => Ok(PropertyState::Ok),
        2 => Ok(PropertyState::Busy),
        3 => Ok(PropertyState::Alert),
        _ => Err(IndigoError::ParseError(format!(
            "Unknown property state: {}",
            state.0
        ))),
    }
}

/// Convert Rust PropertyState to C property state.
#[cfg(feature = "sys-available")]
pub fn state_to_c(state: PropertyState) -> indigo_property_state {
    match state {
        PropertyState::Idle => indigo_property_state(0),
        PropertyState::Ok => indigo_property_state(1),
        PropertyState::Busy => indigo_property_state(2),
        PropertyState::Alert => indigo_property_state(3),
    }
}

/// Convert a C property permission to Rust PropertyPerm.
///
/// # Errors
///
/// Returns an error if the permission value is not recognized.
#[cfg(feature = "sys-available")]
fn perm_from_c(perm: indigo_property_perm) -> Result<PropertyPerm> {
    match perm.0 {
        0 => Ok(PropertyPerm::ReadOnly),
        1 => Ok(PropertyPerm::WriteOnly),
        2 => Ok(PropertyPerm::ReadWrite),
        _ => Err(IndigoError::ParseError(format!(
            "Unknown property permission: {}",
            perm.0
        ))),
    }
}

/// Convert Rust PropertyPerm to C property permission.
#[cfg(feature = "sys-available")]
pub fn perm_to_c(perm: PropertyPerm) -> indigo_property_perm {
    match perm {
        PropertyPerm::ReadOnly => indigo_property_perm(0),
        PropertyPerm::WriteOnly => indigo_property_perm(1),
        PropertyPerm::ReadWrite => indigo_property_perm(2),
    }
}

/// Extract property type and items from C property structure.
///
/// # Safety
///
/// The caller must ensure that `prop` is a valid pointer to a properly
/// initialized `indigo_property` structure.
///
/// # Errors
///
/// Returns an error if the property type is unknown or item conversion fails.
#[cfg(feature = "sys-available")]
unsafe fn items_from_c(
    prop: &indigo_property,
) -> Result<(PropertyType, HashMap<String, PropertyItem>)> {
    let mut items = HashMap::new();

    match prop.type_ {
        x if x.0 == 0 => {
            // INDIGO_TEXT_VECTOR
            for i in 0..prop.count as usize {
                let item = &prop.items[i];
                let name = c_str_to_string(item.name)?;
                let label = c_str_to_string(item.label)?;
                let text_value = c_str_to_string(item.text.value)?;
                items.insert(
                    name.clone(),
                    PropertyItem::new(name, label, PropertyValue::Text(text_value)),
                );
            }
            Ok((PropertyType::Text, items))
        }
        x if x.0 == 1 => {
            // INDIGO_NUMBER_VECTOR
            for i in 0..prop.count as usize {
                let item = &prop.items[i];
                let name = c_str_to_string(item.name)?;
                let label = c_str_to_string(item.label)?;
                let format = c_str_to_string(item.number.format)?;
                items.insert(
                    name.clone(),
                    PropertyItem::new(
                        name,
                        label,
                        PropertyValue::Number {
                            value: item.number.value,
                            min: item.number.min,
                            max: item.number.max,
                            step: item.number.step,
                            format,
                        },
                    ),
                );
            }
            Ok((PropertyType::Number, items))
        }
        x if x.0 == 2 => {
            // INDIGO_SWITCH_VECTOR
            for i in 0..prop.count as usize {
                let item = &prop.items[i];
                let name = c_str_to_string(item.name)?;
                let label = c_str_to_string(item.label)?;
                let state = if item.sw.value {
                    SwitchState::On
                } else {
                    SwitchState::Off
                };
                items.insert(
                    name.clone(),
                    PropertyItem::new(name, label, PropertyValue::Switch { state }),
                );
            }
            Ok((PropertyType::Switch, items))
        }
        x if x.0 == 3 => {
            // INDIGO_LIGHT_VECTOR
            for i in 0..prop.count as usize {
                let item = &prop.items[i];
                let name = c_str_to_string(item.name)?;
                let label = c_str_to_string(item.label)?;
                let state = match item.light.value.0 {
                    0 => LightState::Idle,
                    1 => LightState::Ok,
                    2 => LightState::Busy,
                    3 => LightState::Alert,
                    _ => LightState::Idle,
                };
                items.insert(
                    name.clone(),
                    PropertyItem::new(name, label, PropertyValue::Light { state }),
                );
            }
            Ok((PropertyType::Light, items))
        }
        x if x.0 == 4 => {
            // INDIGO_BLOB_VECTOR
            for i in 0..prop.count as usize {
                let item = &prop.items[i];
                let name = c_str_to_string(item.name)?;
                let label = c_str_to_string(item.label)?;
                let format = c_str_to_string(item.blob.format)?;
                let size = item.blob.size as usize;

                // Copy BLOB data
                let data = if !item.blob.value.is_null() && size > 0 {
                    std::slice::from_raw_parts(item.blob.value as *const u8, size).to_vec()
                } else {
                    Vec::new()
                };

                items.insert(
                    name.clone(),
                    PropertyItem::new(name, label, PropertyValue::Blob { data, format, size }),
                );
            }
            Ok((PropertyType::Blob, items))
        }
        _ => Err(IndigoError::ParseError(format!(
            "Unknown property type: {}",
            prop.type_.0
        ))),
    }
}

/// Convert a C string to a Rust String.
///
/// # Safety
///
/// The caller must ensure that `c_str` is a valid pointer to a null-terminated C string.
///
/// # Errors
///
/// Returns an error if the C string contains invalid UTF-8.
#[cfg(feature = "sys-available")]
unsafe fn c_str_to_string(c_str: *const c_char) -> Result<String> {
    if c_str.is_null() {
        return Ok(String::new());
    }

    CStr::from_ptr(c_str)
        .to_str()
        .map(|s| s.to_string())
        .map_err(|e| IndigoError::ParseError(format!("Invalid UTF-8 in C string: {}", e)))
}

/// Convert a Rust string to a C string.
///
/// # Errors
///
/// Returns an error if the string contains null bytes.
pub fn string_to_c_string(s: &str) -> Result<CString> {
    CString::new(s)
        .map_err(|e| IndigoError::InvalidParameter(format!("String contains null byte: {}", e)))
}

/// Convert BlobTransferMode to C string representation.
pub fn blob_mode_to_c_str(mode: BlobTransferMode) -> &'static str {
    mode.as_str()
}

/// Convert C string to BlobTransferMode.
///
/// # Errors
///
/// Returns an error if the string is not a valid BLOB transfer mode.
pub fn blob_mode_from_c_str(s: &str) -> Result<BlobTransferMode> {
    BlobTransferMode::from_str(s)
        .map_err(|e| IndigoError::ParseError(format!("Invalid BLOB mode: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_mode_conversion() {
        assert_eq!(blob_mode_to_c_str(BlobTransferMode::Never), "Never");
        assert_eq!(blob_mode_to_c_str(BlobTransferMode::Also), "Also");
        assert_eq!(blob_mode_to_c_str(BlobTransferMode::Only), "Only");

        assert_eq!(
            blob_mode_from_c_str("Never").unwrap(),
            BlobTransferMode::Never
        );
        assert_eq!(
            blob_mode_from_c_str("Also").unwrap(),
            BlobTransferMode::Also
        );
        assert_eq!(
            blob_mode_from_c_str("Only").unwrap(),
            BlobTransferMode::Only
        );
    }

    #[test]
    fn test_string_to_c_string() {
        let s = "test string";
        let c_str = string_to_c_string(s).unwrap();
        assert_eq!(c_str.to_str().unwrap(), s);
    }

    #[test]
    fn test_string_with_null_byte_fails() {
        let s = "test\0string";
        assert!(string_to_c_string(s).is_err());
    }

    #[cfg(feature = "sys-available")]
    #[test]
    fn test_state_conversions() {
        assert_eq!(state_to_c(PropertyState::Idle).0, 0);
        assert_eq!(state_to_c(PropertyState::Ok).0, 1);
        assert_eq!(state_to_c(PropertyState::Busy).0, 2);
        assert_eq!(state_to_c(PropertyState::Alert).0, 3);
    }

    #[cfg(feature = "sys-available")]
    #[test]
    fn test_perm_conversions() {
        assert_eq!(perm_to_c(PropertyPerm::ReadOnly).0, 0);
        assert_eq!(perm_to_c(PropertyPerm::WriteOnly).0, 1);
        assert_eq!(perm_to_c(PropertyPerm::ReadWrite).0, 2);
    }

    #[cfg(not(feature = "sys-available"))]
    #[test]
    fn test_ffi_not_available() {
        let result = unsafe { property_from_c(std::ptr::null()) };
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }
}
