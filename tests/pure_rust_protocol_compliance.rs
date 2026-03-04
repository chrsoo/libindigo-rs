//! Protocol Compliance Tests for Pure Rust INDIGO Implementation
//!
//! This test suite verifies that the pure Rust implementation correctly handles
//! the INDIGO protocol according to the INDIGO 1.7 specification.
//!
//! # Test Coverage
//!
//! - Parsing of all INDIGO message types
//! - Serialization of all INDIGO message types
//! - Roundtrip conversion (parse → serialize → parse)
//! - Message validation and error handling
//! - Protocol state transitions
//! - XML structure compliance with INDIGO 1.7 DTD

mod harness;

use libindigo::error::IndigoError;
use libindigo::strategies::rs::protocol::*;
use libindigo::types::property::{PropertyPerm, PropertyState};

// ============================================================================
// Test Utilities
// ============================================================================

/// Helper to create sample vector attributes for testing
fn sample_vector_attrs() -> VectorAttributes {
    VectorAttributes {
        device: "Test Device".to_string(),
        name: "TEST_PROPERTY".to_string(),
        label: "Test Property".to_string(),
        group: "Test Group".to_string(),
        state: PropertyState::Idle,
        timeout: Some(60.0),
        timestamp: Some("2024-01-01T00:00:00".to_string()),
        message: Some("Test message".to_string()),
    }
}

/// Helper to create minimal vector attributes
fn minimal_vector_attrs() -> VectorAttributes {
    VectorAttributes {
        device: "Device".to_string(),
        name: "PROPERTY".to_string(),
        label: String::new(),
        group: String::new(),
        state: PropertyState::Idle,
        timeout: None,
        timestamp: None,
        message: None,
    }
}

// ============================================================================
// GetProperties Message Tests
// ============================================================================

#[cfg(test)]
mod get_properties_tests {
    use super::*;

    #[test]
    fn test_parse_get_properties_minimal() {
        let xml = b"<getProperties version=\"1.7\"/>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::GetProperties(gp) => {
                assert_eq!(gp.version, Some("1.7".to_string()));
                assert_eq!(gp.device, None);
                assert_eq!(gp.name, None);
            }
            _ => panic!("Expected GetProperties message"),
        }
    }

    #[test]
    fn test_parse_get_properties_with_device() {
        let xml = b"<getProperties version=\"1.7\" device=\"CCD Simulator\"/>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::GetProperties(gp) => {
                assert_eq!(gp.version, Some("1.7".to_string()));
                assert_eq!(gp.device, Some("CCD Simulator".to_string()));
                assert_eq!(gp.name, None);
            }
            _ => panic!("Expected GetProperties message"),
        }
    }

    #[test]
    fn test_parse_get_properties_with_device_and_name() {
        let xml = b"<getProperties version=\"1.7\" device=\"CCD Simulator\" name=\"CONNECTION\"/>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::GetProperties(gp) => {
                assert_eq!(gp.version, Some("1.7".to_string()));
                assert_eq!(gp.device, Some("CCD Simulator".to_string()));
                assert_eq!(gp.name, Some("CONNECTION".to_string()));
            }
            _ => panic!("Expected GetProperties message"),
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
    fn test_roundtrip_get_properties() {
        let original = ProtocolMessage::GetProperties(GetProperties {
            version: Some("1.7".to_string()),
            device: Some("Test Device".to_string()),
            name: Some("TEST_PROP".to_string()),
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::GetProperties(gp) => {
                assert_eq!(gp.version, Some("1.7".to_string()));
                assert_eq!(gp.device, Some("Test Device".to_string()));
                assert_eq!(gp.name, Some("TEST_PROP".to_string()));
            }
            _ => panic!("Expected GetProperties message"),
        }
    }
}

// ============================================================================
// DefTextVector Tests
// ============================================================================

#[cfg(test)]
mod def_text_vector_tests {
    use super::*;

    #[test]
    fn test_parse_def_text_vector_minimal() {
        let xml = b"<defTextVector device=\"Device\" name=\"PROPERTY\" state=\"Idle\" perm=\"ro\">
            <defText name=\"TEXT1\" label=\"Text 1\">Value1</defText>
        </defTextVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(v) => {
                assert_eq!(v.attrs.device, "Device");
                assert_eq!(v.attrs.name, "PROPERTY");
                assert_eq!(v.attrs.state, PropertyState::Idle);
                assert_eq!(v.perm, PropertyPerm::ReadOnly);
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].name, "TEXT1");
                assert_eq!(v.elements[0].label, "Text 1");
                assert_eq!(v.elements[0].value, "Value1");
            }
            _ => panic!("Expected DefTextVector message"),
        }
    }

    #[test]
    fn test_parse_def_text_vector_with_all_attributes() {
        let xml = b"<defTextVector device=\"CCD\" name=\"INFO\" label=\"Information\" \
                     group=\"Main\" state=\"Ok\" perm=\"rw\" timeout=\"30\" \
                     timestamp=\"2024-01-01T00:00:00\" message=\"Ready\">
            <defText name=\"NAME\" label=\"Name\">CCD Simulator</defText>
            <defText name=\"VERSION\" label=\"Version\">1.0</defText>
        </defTextVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.attrs.name, "INFO");
                assert_eq!(v.attrs.label, "Information");
                assert_eq!(v.attrs.group, "Main");
                assert_eq!(v.attrs.state, PropertyState::Ok);
                assert_eq!(v.perm, PropertyPerm::ReadWrite);
                assert_eq!(v.attrs.timeout, Some(30.0));
                assert_eq!(v.attrs.timestamp, Some("2024-01-01T00:00:00".to_string()));
                assert_eq!(v.attrs.message, Some("Ready".to_string()));
                assert_eq!(v.elements.len(), 2);
            }
            _ => panic!("Expected DefTextVector message"),
        }
    }

    #[test]
    fn test_parse_def_text_vector_empty_value() {
        let xml = b"<defTextVector device=\"Device\" name=\"PROPERTY\" state=\"Idle\" perm=\"rw\">
            <defText name=\"EMPTY\" label=\"Empty\"></defText>
        </defTextVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(v) => {
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].value, "");
            }
            _ => panic!("Expected DefTextVector message"),
        }
    }

    #[test]
    fn test_serialize_def_text_vector() {
        let msg = ProtocolMessage::DefTextVector(DefTextVector {
            attrs: minimal_vector_attrs(),
            perm: PropertyPerm::ReadOnly,
            elements: vec![DefText {
                name: "TEXT1".to_string(),
                label: "Text 1".to_string(),
                value: "Value1".to_string(),
            }],
        });

        let xml = ProtocolSerializer::serialize(&msg).unwrap();
        let xml_str = String::from_utf8(xml).unwrap();

        assert!(xml_str.contains("defTextVector"));
        assert!(xml_str.contains("device=\"Device\""));
        assert!(xml_str.contains("perm=\"ro\""));
        assert!(xml_str.contains("defText"));
        assert!(xml_str.contains("Value1"));
    }

    #[test]
    fn test_roundtrip_def_text_vector() {
        let original = ProtocolMessage::DefTextVector(DefTextVector {
            attrs: sample_vector_attrs(),
            perm: PropertyPerm::ReadWrite,
            elements: vec![
                DefText {
                    name: "TEXT1".to_string(),
                    label: "Text 1".to_string(),
                    value: "Value1".to_string(),
                },
                DefText {
                    name: "TEXT2".to_string(),
                    label: "Text 2".to_string(),
                    value: "Value2".to_string(),
                },
            ],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::DefTextVector(v) => {
                assert_eq!(v.attrs.device, "Test Device");
                assert_eq!(v.attrs.name, "TEST_PROPERTY");
                assert_eq!(v.perm, PropertyPerm::ReadWrite);
                assert_eq!(v.elements.len(), 2);
                assert_eq!(v.elements[0].value, "Value1");
                assert_eq!(v.elements[1].value, "Value2");
            }
            _ => panic!("Expected DefTextVector message"),
        }
    }
}

// ============================================================================
// DefNumberVector Tests
// ============================================================================

#[cfg(test)]
mod def_number_vector_tests {
    use super::*;

    #[test]
    fn test_parse_def_number_vector() {
        let xml = b"<defNumberVector device=\"CCD\" name=\"EXPOSURE\" state=\"Idle\" perm=\"rw\">
            <defNumber name=\"DURATION\" label=\"Duration\" format=\"%.2f\" min=\"0\" max=\"3600\" step=\"0.01\">1.0</defNumber>
        </defNumberVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefNumberVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.attrs.name, "EXPOSURE");
                assert_eq!(v.perm, PropertyPerm::ReadWrite);
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].name, "DURATION");
                assert_eq!(v.elements[0].format, "%.2f");
                assert_eq!(v.elements[0].min, 0.0);
                assert_eq!(v.elements[0].max, 3600.0);
                assert_eq!(v.elements[0].step, 0.01);
                assert_eq!(v.elements[0].value, 1.0);
            }
            _ => panic!("Expected DefNumberVector message"),
        }
    }

    #[test]
    fn test_parse_def_number_vector_negative_values() {
        let xml = b"<defNumberVector device=\"Device\" name=\"TEMP\" state=\"Idle\" perm=\"ro\">
            <defNumber name=\"VALUE\" label=\"Temperature\" format=\"%.1f\" min=\"-50\" max=\"50\" step=\"0.1\">-10.5</defNumber>
        </defNumberVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefNumberVector(v) => {
                assert_eq!(v.elements[0].min, -50.0);
                assert_eq!(v.elements[0].max, 50.0);
                assert_eq!(v.elements[0].value, -10.5);
            }
            _ => panic!("Expected DefNumberVector message"),
        }
    }

    #[test]
    fn test_serialize_def_number_vector() {
        let msg = ProtocolMessage::DefNumberVector(DefNumberVector {
            attrs: minimal_vector_attrs(),
            perm: PropertyPerm::ReadWrite,
            elements: vec![DefNumber {
                name: "NUM1".to_string(),
                label: "Number 1".to_string(),
                format: "%.2f".to_string(),
                min: 0.0,
                max: 100.0,
                step: 1.0,
                value: 50.0,
            }],
        });

        let xml = ProtocolSerializer::serialize(&msg).unwrap();
        let xml_str = String::from_utf8(xml).unwrap();

        assert!(xml_str.contains("defNumberVector"));
        assert!(xml_str.contains("defNumber"));
        assert!(xml_str.contains("format=\"%.2f\""));
        assert!(xml_str.contains("min=\"0\""));
        assert!(xml_str.contains("max=\"100\""));
    }

    #[test]
    fn test_roundtrip_def_number_vector() {
        let original = ProtocolMessage::DefNumberVector(DefNumberVector {
            attrs: sample_vector_attrs(),
            perm: PropertyPerm::ReadOnly,
            elements: vec![DefNumber {
                name: "X".to_string(),
                label: "X Coordinate".to_string(),
                format: "%.3f".to_string(),
                min: -100.0,
                max: 100.0,
                step: 0.001,
                value: 42.123,
            }],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::DefNumberVector(v) => {
                assert_eq!(v.elements[0].name, "X");
                assert_eq!(v.elements[0].min, -100.0);
                assert_eq!(v.elements[0].max, 100.0);
                assert!((v.elements[0].value - 42.123).abs() < 0.001);
            }
            _ => panic!("Expected DefNumberVector message"),
        }
    }
}

// ============================================================================
// DefSwitchVector Tests
// ============================================================================

#[cfg(test)]
mod def_switch_vector_tests {
    use super::*;

    #[test]
    fn test_parse_def_switch_vector() {
        let xml = b"<defSwitchVector device=\"CCD\" name=\"CONNECTION\" state=\"Idle\" perm=\"rw\" rule=\"OneOfMany\">
            <defSwitch name=\"CONNECT\" label=\"Connect\">On</defSwitch>
            <defSwitch name=\"DISCONNECT\" label=\"Disconnect\">Off</defSwitch>
        </defSwitchVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefSwitchVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.attrs.name, "CONNECTION");
                assert_eq!(v.perm, PropertyPerm::ReadWrite);
                assert_eq!(v.rule, SwitchRule::OneOfMany);
                assert_eq!(v.elements.len(), 2);
                assert_eq!(v.elements[0].name, "CONNECT");
                assert_eq!(v.elements[0].value, SwitchState::On);
                assert_eq!(v.elements[1].name, "DISCONNECT");
                assert_eq!(v.elements[1].value, SwitchState::Off);
            }
            _ => panic!("Expected DefSwitchVector message"),
        }
    }

    #[test]
    fn test_parse_def_switch_vector_all_rules() {
        // Test OneOfMany
        let xml = b"<defSwitchVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\" rule=\"OneOfMany\">
            <defSwitch name=\"S1\">On</defSwitch>
        </defSwitchVector>";
        let msg = ProtocolParser::parse_message(xml).unwrap();
        match msg {
            ProtocolMessage::DefSwitchVector(v) => assert_eq!(v.rule, SwitchRule::OneOfMany),
            _ => panic!("Expected DefSwitchVector"),
        }

        // Test AtMostOne
        let xml = b"<defSwitchVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\" rule=\"AtMostOne\">
            <defSwitch name=\"S1\">Off</defSwitch>
        </defSwitchVector>";
        let msg = ProtocolParser::parse_message(xml).unwrap();
        match msg {
            ProtocolMessage::DefSwitchVector(v) => assert_eq!(v.rule, SwitchRule::AtMostOne),
            _ => panic!("Expected DefSwitchVector"),
        }

        // Test AnyOfMany
        let xml = b"<defSwitchVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\" rule=\"AnyOfMany\">
            <defSwitch name=\"S1\">On</defSwitch>
        </defSwitchVector>";
        let msg = ProtocolParser::parse_message(xml).unwrap();
        match msg {
            ProtocolMessage::DefSwitchVector(v) => assert_eq!(v.rule, SwitchRule::AnyOfMany),
            _ => panic!("Expected DefSwitchVector"),
        }
    }

    #[test]
    fn test_serialize_def_switch_vector() {
        let msg = ProtocolMessage::DefSwitchVector(DefSwitchVector {
            attrs: minimal_vector_attrs(),
            perm: PropertyPerm::ReadWrite,
            rule: SwitchRule::OneOfMany,
            elements: vec![
                DefSwitch {
                    name: "ON".to_string(),
                    label: "On".to_string(),
                    value: SwitchState::On,
                },
                DefSwitch {
                    name: "OFF".to_string(),
                    label: "Off".to_string(),
                    value: SwitchState::Off,
                },
            ],
        });

        let xml = ProtocolSerializer::serialize(&msg).unwrap();
        let xml_str = String::from_utf8(xml).unwrap();

        assert!(xml_str.contains("defSwitchVector"));
        assert!(xml_str.contains("rule=\"OneOfMany\""));
        assert!(xml_str.contains("defSwitch"));
        assert!(xml_str.contains("On"));
        assert!(xml_str.contains("Off"));
    }

    #[test]
    fn test_roundtrip_def_switch_vector() {
        let original = ProtocolMessage::DefSwitchVector(DefSwitchVector {
            attrs: sample_vector_attrs(),
            perm: PropertyPerm::ReadWrite,
            rule: SwitchRule::AnyOfMany,
            elements: vec![
                DefSwitch {
                    name: "OPTION1".to_string(),
                    label: "Option 1".to_string(),
                    value: SwitchState::On,
                },
                DefSwitch {
                    name: "OPTION2".to_string(),
                    label: "Option 2".to_string(),
                    value: SwitchState::Off,
                },
            ],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::DefSwitchVector(v) => {
                assert_eq!(v.rule, SwitchRule::AnyOfMany);
                assert_eq!(v.elements.len(), 2);
                assert_eq!(v.elements[0].value, SwitchState::On);
                assert_eq!(v.elements[1].value, SwitchState::Off);
            }
            _ => panic!("Expected DefSwitchVector message"),
        }
    }
}

// ============================================================================
// DefLightVector Tests
// ============================================================================

#[cfg(test)]
mod def_light_vector_tests {
    use super::*;

    #[test]
    fn test_parse_def_light_vector() {
        let xml = b"<defLightVector device=\"CCD\" name=\"STATUS\" state=\"Ok\">
            <defLight name=\"READY\" label=\"Ready\">Ok</defLight>
            <defLight name=\"BUSY\" label=\"Busy\">Idle</defLight>
        </defLightVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefLightVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.attrs.name, "STATUS");
                assert_eq!(v.elements.len(), 2);
                assert_eq!(v.elements[0].name, "READY");
                assert_eq!(v.elements[0].value, PropertyState::Ok);
                assert_eq!(v.elements[1].name, "BUSY");
                assert_eq!(v.elements[1].value, PropertyState::Idle);
            }
            _ => panic!("Expected DefLightVector message"),
        }
    }

    #[test]
    fn test_parse_def_light_vector_all_states() {
        let xml = b"<defLightVector device=\"D\" name=\"P\" state=\"Idle\">
            <defLight name=\"L1\">Idle</defLight>
            <defLight name=\"L2\">Ok</defLight>
            <defLight name=\"L3\">Busy</defLight>
            <defLight name=\"L4\">Alert</defLight>
        </defLightVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefLightVector(v) => {
                assert_eq!(v.elements[0].value, PropertyState::Idle);
                assert_eq!(v.elements[1].value, PropertyState::Ok);
                assert_eq!(v.elements[2].value, PropertyState::Busy);
                assert_eq!(v.elements[3].value, PropertyState::Alert);
            }
            _ => panic!("Expected DefLightVector message"),
        }
    }

    #[test]
    fn test_serialize_def_light_vector() {
        let msg = ProtocolMessage::DefLightVector(DefLightVector {
            attrs: minimal_vector_attrs(),
            elements: vec![DefLight {
                name: "STATUS".to_string(),
                label: "Status".to_string(),
                value: PropertyState::Ok,
            }],
        });

        let xml = ProtocolSerializer::serialize(&msg).unwrap();
        let xml_str = String::from_utf8(xml).unwrap();

        assert!(xml_str.contains("defLightVector"));
        assert!(xml_str.contains("defLight"));
        assert!(xml_str.contains("Ok"));
    }

    #[test]
    fn test_roundtrip_def_light_vector() {
        let original = ProtocolMessage::DefLightVector(DefLightVector {
            attrs: sample_vector_attrs(),
            elements: vec![DefLight {
                name: "LIGHT1".to_string(),
                label: "Light 1".to_string(),
                value: PropertyState::Alert,
            }],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::DefLightVector(v) => {
                assert_eq!(v.elements[0].value, PropertyState::Alert);
            }
            _ => panic!("Expected DefLightVector message"),
        }
    }
}

// ============================================================================
// DefBLOBVector Tests
// ============================================================================

#[cfg(test)]
mod def_blob_vector_tests {
    use super::*;

    #[test]
    fn test_parse_def_blob_vector() {
        let xml = b"<defBLOBVector device=\"CCD\" name=\"IMAGE\" state=\"Idle\" perm=\"ro\">
            <defBLOB name=\"IMAGE\" label=\"Image\"/>
        </defBLOBVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefBLOBVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.attrs.name, "IMAGE");
                assert_eq!(v.perm, PropertyPerm::ReadOnly);
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].name, "IMAGE");
            }
            _ => panic!("Expected DefBLOBVector message"),
        }
    }

    #[test]
    fn test_serialize_def_blob_vector() {
        let msg = ProtocolMessage::DefBLOBVector(DefBLOBVector {
            attrs: minimal_vector_attrs(),
            perm: PropertyPerm::ReadOnly,
            elements: vec![DefBLOB {
                name: "BLOB1".to_string(),
                label: "Blob 1".to_string(),
            }],
        });

        let xml = ProtocolSerializer::serialize(&msg).unwrap();
        let xml_str = String::from_utf8(xml).unwrap();

        assert!(xml_str.contains("defBLOBVector"));
        assert!(xml_str.contains("defBLOB"));
    }

    #[test]
    fn test_roundtrip_def_blob_vector() {
        let original = ProtocolMessage::DefBLOBVector(DefBLOBVector {
            attrs: sample_vector_attrs(),
            perm: PropertyPerm::ReadWrite,
            elements: vec![DefBLOB {
                name: "DATA".to_string(),
                label: "Data".to_string(),
            }],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::DefBLOBVector(v) => {
                assert_eq!(v.elements[0].name, "DATA");
            }
            _ => panic!("Expected DefBLOBVector message"),
        }
    }
}

// ============================================================================
// Set Vector Tests
// ============================================================================

#[cfg(test)]
mod set_vector_tests {
    use super::*;

    #[test]
    fn test_parse_set_text_vector() {
        let xml = b"<setTextVector device=\"CCD\" name=\"INFO\" state=\"Ok\">
            <oneText name=\"NAME\">Updated Name</oneText>
        </setTextVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::SetTextVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.attrs.name, "INFO");
                assert_eq!(v.attrs.state, Some(PropertyState::Ok));
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].value, "Updated Name");
            }
            _ => panic!("Expected SetTextVector message"),
        }
    }

    #[test]
    fn test_parse_set_number_vector() {
        let xml = b"<setNumberVector device=\"CCD\" name=\"EXPOSURE\" state=\"Busy\">
            <oneNumber name=\"DURATION\">5.0</oneNumber>
        </setNumberVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.attrs.state, Some(PropertyState::Busy));
                assert_eq!(v.elements[0].value, 5.0);
            }
            _ => panic!("Expected SetNumberVector message"),
        }
    }

    #[test]
    fn test_parse_set_switch_vector() {
        let xml = b"<setSwitchVector device=\"CCD\" name=\"CONNECTION\" state=\"Ok\">
            <oneSwitch name=\"CONNECT\">On</oneSwitch>
            <oneSwitch name=\"DISCONNECT\">Off</oneSwitch>
        </setSwitchVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::SetSwitchVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.elements.len(), 2);
                assert_eq!(v.elements[0].value, SwitchState::On);
                assert_eq!(v.elements[1].value, SwitchState::Off);
            }
            _ => panic!("Expected SetSwitchVector message"),
        }
    }

    #[test]
    fn test_roundtrip_set_text_vector() {
        let original = ProtocolMessage::SetTextVector(SetTextVector {
            attrs: SetVectorAttributes {
                device: "Device".to_string(),
                name: "PROPERTY".to_string(),
                state: Some(PropertyState::Ok),
                timeout: None,
                timestamp: None,
                message: None,
            },
            elements: vec![OneText {
                name: "TEXT1".to_string(),
                value: "Value".to_string(),
            }],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::SetTextVector(v) => {
                assert_eq!(v.elements[0].value, "Value");
            }
            _ => panic!("Expected SetTextVector message"),
        }
    }
}

// ============================================================================
// New Vector Tests (Client to Server)
// ============================================================================

#[cfg(test)]
mod new_vector_tests {
    use super::*;

    #[test]
    fn test_parse_new_text_vector() {
        let xml = b"<newTextVector device=\"CCD\" name=\"INFO\">
            <oneText name=\"NAME\">New Value</oneText>
        </newTextVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::NewTextVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.attrs.name, "INFO");
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].value, "New Value");
            }
            _ => panic!("Expected NewTextVector message"),
        }
    }

    #[test]
    fn test_parse_new_number_vector() {
        let xml = b"<newNumberVector device=\"CCD\" name=\"EXPOSURE\">
            <oneNumber name=\"DURATION\">10.5</oneNumber>
        </newNumberVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::NewNumberVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.elements[0].value, 10.5);
            }
            _ => panic!("Expected NewNumberVector message"),
        }
    }

    #[test]
    fn test_parse_new_switch_vector() {
        let xml = b"<newSwitchVector device=\"CCD\" name=\"CONNECTION\">
            <oneSwitch name=\"CONNECT\">On</oneSwitch>
        </newSwitchVector>";

        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::NewSwitchVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.elements[0].value, SwitchState::On);
            }
            _ => panic!("Expected NewSwitchVector message"),
        }
    }

    #[test]
    fn test_serialize_new_text_vector() {
        let msg = ProtocolMessage::NewTextVector(NewTextVector {
            attrs: NewVectorAttributes {
                device: "Device".to_string(),
                name: "PROPERTY".to_string(),
                timestamp: None,
            },
            elements: vec![OneText {
                name: "TEXT1".to_string(),
                value: "Value".to_string(),
            }],
        });

        let xml = ProtocolSerializer::serialize(&msg).unwrap();
        let xml_str = String::from_utf8(xml).unwrap();

        assert!(xml_str.contains("newTextVector"));
        assert!(xml_str.contains("device=\"Device\""));
        assert!(xml_str.contains("oneText"));
    }

    #[test]
    fn test_roundtrip_new_switch_vector() {
        let original = ProtocolMessage::NewSwitchVector(NewSwitchVector {
            attrs: NewVectorAttributes {
                device: "CCD".to_string(),
                name: "CONNECTION".to_string(),
                timestamp: Some("2024-01-01T00:00:00".to_string()),
            },
            elements: vec![OneSwitch {
                name: "CONNECT".to_string(),
                value: SwitchState::On,
            }],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::NewSwitchVector(v) => {
                assert_eq!(v.attrs.device, "CCD");
                assert_eq!(v.elements[0].value, SwitchState::On);
            }
            _ => panic!("Expected NewSwitchVector message"),
        }
    }
}

// ============================================================================
// Control Message Tests
// ============================================================================

#[cfg(test)]
mod control_message_tests {
    use super::*;

    #[test]
    fn test_parse_enable_blob() {
        let xml = b"<enableBLOB device=\"CCD\" name=\"IMAGE\">Also</enableBLOB>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::EnableBLOB(eb) => {
                assert_eq!(eb.device, "CCD");
                assert_eq!(eb.name, Some("IMAGE".to_string()));
                assert_eq!(eb.value, BLOBEnable::Also);
            }
            _ => panic!("Expected EnableBLOB message"),
        }
    }

    #[test]
    fn test_parse_enable_blob_all_modes() {
        // Test Never
        let xml = b"<enableBLOB device=\"CCD\">Never</enableBLOB>";
        let msg = ProtocolParser::parse_message(xml).unwrap();
        match msg {
            ProtocolMessage::EnableBLOB(eb) => assert_eq!(eb.value, BLOBEnable::Never),
            _ => panic!("Expected EnableBLOB"),
        }

        // Test Also
        let xml = b"<enableBLOB device=\"CCD\">Also</enableBLOB>";
        let msg = ProtocolParser::parse_message(xml).unwrap();
        match msg {
            ProtocolMessage::EnableBLOB(eb) => assert_eq!(eb.value, BLOBEnable::Also),
            _ => panic!("Expected EnableBLOB"),
        }

        // Test Only
        let xml = b"<enableBLOB device=\"CCD\">Only</enableBLOB>";
        let msg = ProtocolParser::parse_message(xml).unwrap();
        match msg {
            ProtocolMessage::EnableBLOB(eb) => assert_eq!(eb.value, BLOBEnable::Only),
            _ => panic!("Expected EnableBLOB"),
        }
    }

    #[test]
    fn test_parse_message_element() {
        let xml =
            b"<message device=\"CCD\" timestamp=\"2024-01-01T00:00:00\" message=\"Test message\"/>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::Message(m) => {
                assert_eq!(m.device, Some("CCD".to_string()));
                assert_eq!(m.timestamp, Some("2024-01-01T00:00:00".to_string()));
                assert_eq!(m.message, Some("Test message".to_string()));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_parse_del_property() {
        let xml =
            b"<delProperty device=\"CCD\" name=\"EXPOSURE\" timestamp=\"2024-01-01T00:00:00\"/>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DelProperty(dp) => {
                assert_eq!(dp.device, "CCD");
                assert_eq!(dp.name, Some("EXPOSURE".to_string()));
                assert_eq!(dp.timestamp, Some("2024-01-01T00:00:00".to_string()));
            }
            _ => panic!("Expected DelProperty message"),
        }
    }

    #[test]
    fn test_parse_del_property_all() {
        // Delete all properties for a device
        let xml = b"<delProperty device=\"CCD\"/>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DelProperty(dp) => {
                assert_eq!(dp.device, "CCD");
                assert_eq!(dp.name, None);
            }
            _ => panic!("Expected DelProperty message"),
        }
    }

    #[test]
    fn test_serialize_enable_blob() {
        let msg = ProtocolMessage::EnableBLOB(EnableBLOB {
            device: "CCD".to_string(),
            name: Some("IMAGE".to_string()),
            value: BLOBEnable::Also,
        });

        let xml = ProtocolSerializer::serialize(&msg).unwrap();
        let xml_str = String::from_utf8(xml).unwrap();

        assert!(xml_str.contains("enableBLOB"));
        assert!(xml_str.contains("device=\"CCD\""));
        assert!(xml_str.contains("Also"));
    }

    #[test]
    fn test_roundtrip_del_property() {
        let original = ProtocolMessage::DelProperty(DelProperty {
            device: "CCD".to_string(),
            name: Some("EXPOSURE".to_string()),
            timestamp: Some("2024-01-01T00:00:00".to_string()),
            message: Some("Deleted".to_string()),
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::DelProperty(dp) => {
                assert_eq!(dp.device, "CCD");
                assert_eq!(dp.name, Some("EXPOSURE".to_string()));
            }
            _ => panic!("Expected DelProperty message"),
        }
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_parse_invalid_xml() {
        let xml = b"<invalid>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_message_type() {
        let xml = b"<unknownMessage/>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_attribute() {
        // Missing device attribute
        let xml = b"<getProperties version=\"1.7\" name=\"PROPERTY\"/>";
        // This should still parse (device is optional for getProperties)
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_ok());

        // Missing required device for defTextVector
        let xml = b"<defTextVector name=\"PROPERTY\" state=\"Idle\" perm=\"ro\"></defTextVector>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_state() {
        let xml = b"<defTextVector device=\"D\" name=\"P\" state=\"InvalidState\" perm=\"ro\"></defTextVector>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_permission() {
        let xml = b"<defTextVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"invalid\"></defTextVector>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_switch_state() {
        let xml = b"<defSwitchVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\" rule=\"OneOfMany\">
            <defSwitch name=\"S1\">Invalid</defSwitch>
        </defSwitchVector>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_switch_rule() {
        let xml = b"<defSwitchVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\" rule=\"InvalidRule\">
            <defSwitch name=\"S1\">On</defSwitch>
        </defSwitchVector>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_number() {
        let xml = b"<defNumberVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\">
            <defNumber name=\"N1\" format=\"%.2f\" min=\"0\" max=\"100\" step=\"1\">not_a_number</defNumber>
        </defNumberVector>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_blob_enable() {
        let xml = b"<enableBLOB device=\"CCD\">InvalidMode</enableBLOB>";
        let result = ProtocolParser::parse_message(xml);
        assert!(result.is_err());
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_parse_empty_text_value() {
        let xml = b"<defTextVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\">
            <defText name=\"EMPTY\"></defText>
        </defTextVector>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(v) => {
                assert_eq!(v.elements[0].value, "");
            }
            _ => panic!("Expected DefTextVector"),
        }
    }

    #[test]
    fn test_parse_text_with_special_characters() {
        let xml = b"<defTextVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\">
            <defText name=\"TEXT\">Value with &lt;special&gt; &amp; chars</defText>
        </defTextVector>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(v) => {
                assert!(v.elements[0].value.contains("<special>"));
                assert!(v.elements[0].value.contains("&"));
            }
            _ => panic!("Expected DefTextVector"),
        }
    }

    #[test]
    fn test_parse_very_large_number() {
        let xml = b"<defNumberVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\">
            <defNumber name=\"BIG\" format=\"%.2f\" min=\"0\" max=\"1e100\" step=\"1\">1.23e50</defNumber>
        </defNumberVector>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefNumberVector(v) => {
                assert!(v.elements[0].value > 1e49);
            }
            _ => panic!("Expected DefNumberVector"),
        }
    }

    #[test]
    fn test_parse_vector_with_many_elements() {
        let mut xml =
            String::from("<defTextVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\">");
        for i in 0..100 {
            xml.push_str(&format!("<defText name=\"TEXT{}\">Value{}</defText>", i, i));
        }
        xml.push_str("</defTextVector>");

        let msg = ProtocolParser::parse_message(xml.as_bytes()).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(v) => {
                assert_eq!(v.elements.len(), 100);
            }
            _ => panic!("Expected DefTextVector"),
        }
    }

    #[test]
    fn test_parse_minimal_attributes() {
        // Test with only required attributes
        let xml = b"<defTextVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"rw\">
            <defText name=\"T\">V</defText>
        </defTextVector>";
        let msg = ProtocolParser::parse_message(xml).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(v) => {
                assert_eq!(v.attrs.label, "");
                assert_eq!(v.attrs.group, "");
                assert_eq!(v.attrs.timeout, None);
                assert_eq!(v.attrs.timestamp, None);
                assert_eq!(v.attrs.message, None);
            }
            _ => panic!("Expected DefTextVector"),
        }
    }

    #[test]
    fn test_roundtrip_with_unicode() {
        let original = ProtocolMessage::DefTextVector(DefTextVector {
            attrs: VectorAttributes {
                device: "Device".to_string(),
                name: "PROPERTY".to_string(),
                label: "Propriété".to_string(), // French
                group: "グループ".to_string(),  // Japanese
                state: PropertyState::Idle,
                timeout: None,
                timestamp: None,
                message: Some("Сообщение".to_string()), // Russian
            },
            perm: PropertyPerm::ReadOnly,
            elements: vec![DefText {
                name: "TEXT1".to_string(),
                label: "文本".to_string(),      // Chinese
                value: "🚀 Rocket".to_string(), // Emoji
            }],
        });

        let xml = ProtocolSerializer::serialize(&original).unwrap();
        let parsed = ProtocolParser::parse_message(&xml).unwrap();

        match parsed {
            ProtocolMessage::DefTextVector(v) => {
                assert_eq!(v.attrs.label, "Propriété");
                assert_eq!(v.attrs.group, "グループ");
                assert_eq!(v.attrs.message, Some("Сообщение".to_string()));
                assert_eq!(v.elements[0].label, "文本");
                assert_eq!(v.elements[0].value, "🚀 Rocket");
            }
            _ => panic!("Expected DefTextVector"),
        }
    }
}

// ============================================================================
// Property State Tests
// ============================================================================

#[cfg(test)]
mod property_state_tests {
    use super::*;

    #[test]
    fn test_all_property_states() {
        let states = vec![
            ("Idle", PropertyState::Idle),
            ("Ok", PropertyState::Ok),
            ("Busy", PropertyState::Busy),
            ("Alert", PropertyState::Alert),
        ];

        for (state_str, expected_state) in states {
            let xml = format!(
                "<defTextVector device=\"D\" name=\"P\" state=\"{}\" perm=\"ro\"></defTextVector>",
                state_str
            );
            let msg = ProtocolParser::parse_message(xml.as_bytes()).unwrap();

            match msg {
                ProtocolMessage::DefTextVector(v) => {
                    assert_eq!(v.attrs.state, expected_state);
                }
                _ => panic!("Expected DefTextVector"),
            }
        }
    }

    #[test]
    fn test_all_permissions() {
        let perms = vec![
            ("ro", PropertyPerm::ReadOnly),
            ("wo", PropertyPerm::WriteOnly),
            ("rw", PropertyPerm::ReadWrite),
        ];

        for (perm_str, expected_perm) in perms {
            let xml = format!(
                "<defTextVector device=\"D\" name=\"P\" state=\"Idle\" perm=\"{}\"></defTextVector>",
                perm_str
            );
            let msg = ProtocolParser::parse_message(xml.as_bytes()).unwrap();

            match msg {
                ProtocolMessage::DefTextVector(v) => {
                    assert_eq!(v.perm, expected_perm);
                }
                _ => panic!("Expected DefTextVector"),
            }
        }
    }
}
