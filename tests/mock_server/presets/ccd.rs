//! CCD camera simulator preset device.

use crate::mock_server::device::{DeviceMetadata, MockDevice};
use crate::mock_server::property::*;
use libindigo_rs::protocol::{PropertyPerm, PropertyState, SwitchRule};
use std::collections::HashMap;

/// Creates a mock CCD camera simulator device
pub fn ccd_simulator() -> MockDevice {
    let mut device = MockDevice {
        name: "CCD Simulator".to_string(),
        interface: 0x01, // CCD interface
        properties: HashMap::new(),
        metadata: DeviceMetadata {
            version: Some("1.0".to_string()),
            driver_name: Some("indigo_ccd_simulator".to_string()),
            driver_version: Some("2.0.300".to_string()),
            driver_interface: Some(0x01),
        },
    };

    // CONNECTION property
    device.properties.insert(
        "CONNECTION".to_string(),
        MockProperty {
            device: "CCD Simulator".to_string(),
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

    // CCD_EXPOSURE property
    device.properties.insert(
        "CCD_EXPOSURE".to_string(),
        MockProperty {
            device: "CCD Simulator".to_string(),
            name: "CCD_EXPOSURE".to_string(),
            group: "Main".to_string(),
            label: "Exposure".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Number,
            items: vec![PropertyItem {
                name: "EXPOSURE".to_string(),
                label: "Exposure time".to_string(),
                value: PropertyValue::Number(NumberValue {
                    value: 1.0,
                    format: "%.2f".to_string(),
                    min: 0.001,
                    max: 3600.0,
                    step: 0.001,
                }),
            }],
            timeout: None,
            timestamp: None,
            message: None,
            type_metadata: PropertyTypeMetadata::Number,
        },
    );

    // CCD_TEMPERATURE property
    device.properties.insert(
        "CCD_TEMPERATURE".to_string(),
        MockProperty {
            device: "CCD Simulator".to_string(),
            name: "CCD_TEMPERATURE".to_string(),
            group: "Main".to_string(),
            label: "Temperature".to_string(),
            state: PropertyState::Ok,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Number,
            items: vec![PropertyItem {
                name: "CCD_TEMPERATURE_VALUE".to_string(),
                label: "Temperature".to_string(),
                value: PropertyValue::Number(NumberValue {
                    value: 20.0,
                    format: "%.2f".to_string(),
                    min: -50.0,
                    max: 50.0,
                    step: 0.1,
                }),
            }],
            timeout: None,
            timestamp: None,
            message: None,
            type_metadata: PropertyTypeMetadata::Number,
        },
    );

    // CCD_INFO property (read-only)
    device.properties.insert(
        "CCD_INFO".to_string(),
        MockProperty {
            device: "CCD Simulator".to_string(),
            name: "CCD_INFO".to_string(),
            group: "Main".to_string(),
            label: "CCD Info".to_string(),
            state: PropertyState::Ok,
            perm: PropertyPerm::ReadOnly,
            property_type: PropertyType::Number,
            items: vec![
                PropertyItem {
                    name: "CCD_WIDTH".to_string(),
                    label: "Width".to_string(),
                    value: PropertyValue::Number(NumberValue {
                        value: 1280.0,
                        format: "%.0f".to_string(),
                        min: 0.0,
                        max: 10000.0,
                        step: 1.0,
                    }),
                },
                PropertyItem {
                    name: "CCD_HEIGHT".to_string(),
                    label: "Height".to_string(),
                    value: PropertyValue::Number(NumberValue {
                        value: 1024.0,
                        format: "%.0f".to_string(),
                        min: 0.0,
                        max: 10000.0,
                        step: 1.0,
                    }),
                },
                PropertyItem {
                    name: "CCD_PIXEL_SIZE".to_string(),
                    label: "Pixel size".to_string(),
                    value: PropertyValue::Number(NumberValue {
                        value: 5.2,
                        format: "%.2f".to_string(),
                        min: 0.0,
                        max: 100.0,
                        step: 0.1,
                    }),
                },
            ],
            timeout: None,
            timestamp: None,
            message: None,
            type_metadata: PropertyTypeMetadata::Number,
        },
    );

    device
}
