//! Telescope mount simulator preset device.

use crate::mock_server::device::{DeviceMetadata, MockDevice};
use crate::mock_server::property::*;
use libindigo_rs::protocol::{PropertyPerm, PropertyState, SwitchRule};
use std::collections::HashMap;

/// Creates a mock telescope mount simulator device
pub fn mount_simulator() -> MockDevice {
    let mut device = MockDevice {
        name: "Mount Simulator".to_string(),
        interface: 0x02, // Mount interface
        properties: HashMap::new(),
        metadata: DeviceMetadata {
            version: Some("1.0".to_string()),
            driver_name: Some("indigo_mount_simulator".to_string()),
            driver_version: Some("2.0.300".to_string()),
            driver_interface: Some(0x02),
        },
    };

    // CONNECTION property
    device.properties.insert(
        "CONNECTION".to_string(),
        MockProperty {
            device: "Mount Simulator".to_string(),
            name: "CONNECTION".to_string(),
            group: "Main".to_string(),
            label: "Connection".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Switch,
            items: vec![
                PropertyItem {
                    name: "CONNECTED".to_string(),
                    label: "Connected".to_string(),
                    value: PropertyValue::Switch(false),
                },
                PropertyItem {
                    name: "DISCONNECTED".to_string(),
                    label: "Disconnected".to_string(),
                    value: PropertyValue::Switch(true),
                },
            ],
            timeout: None,
            timestamp: None,
            message: None,
            type_metadata: PropertyTypeMetadata::Switch {
                rule: SwitchRule::OneOfMany,
            },
        },
    );

    // MOUNT_EQUATORIAL_COORDINATES property
    device.properties.insert(
        "MOUNT_EQUATORIAL_COORDINATES".to_string(),
        MockProperty {
            device: "Mount Simulator".to_string(),
            name: "MOUNT_EQUATORIAL_COORDINATES".to_string(),
            group: "Main".to_string(),
            label: "Equatorial Coordinates".to_string(),
            state: PropertyState::Ok,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Number,
            items: vec![
                PropertyItem {
                    name: "RA".to_string(),
                    label: "Right Ascension".to_string(),
                    value: PropertyValue::Number(NumberValue {
                        value: 0.0,
                        format: "%.6f".to_string(),
                        min: 0.0,
                        max: 24.0,
                        step: 0.0001,
                    }),
                },
                PropertyItem {
                    name: "DEC".to_string(),
                    label: "Declination".to_string(),
                    value: PropertyValue::Number(NumberValue {
                        value: 0.0,
                        format: "%.6f".to_string(),
                        min: -90.0,
                        max: 90.0,
                        step: 0.0001,
                    }),
                },
            ],
            timeout: None,
            timestamp: None,
            message: None,
            type_metadata: PropertyTypeMetadata::Number,
        },
    );

    // MOUNT_PARK property
    device.properties.insert(
        "MOUNT_PARK".to_string(),
        MockProperty {
            device: "Mount Simulator".to_string(),
            name: "MOUNT_PARK".to_string(),
            group: "Main".to_string(),
            label: "Park".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Switch,
            items: vec![
                PropertyItem {
                    name: "PARKED".to_string(),
                    label: "Parked".to_string(),
                    value: PropertyValue::Switch(true),
                },
                PropertyItem {
                    name: "UNPARKED".to_string(),
                    label: "Unparked".to_string(),
                    value: PropertyValue::Switch(false),
                },
            ],
            timeout: None,
            timestamp: None,
            message: None,
            type_metadata: PropertyTypeMetadata::Switch {
                rule: SwitchRule::OneOfMany,
            },
        },
    );

    device
}
