//! JSON Protocol Tests
//!
//! Comprehensive tests for INDIGO JSON protocol implementation.
//! Tests all examples from PROTOCOLS.md and verifies JSON-specific features.

mod harness;

use libindigo::error::IndigoError;
use libindigo::strategies::rs::protocol::*;
use libindigo::strategies::rs::protocol_json::{
    JsonProtocolParser, JsonProtocolSerializer, JSON_PROTOCOL_VERSION,
};
use libindigo::types::property::{PropertyPerm, PropertyState};

// ============================================================================
// PROTOCOLS.md Examples Tests
// ============================================================================

#[cfg(test)]
mod protocols_md_examples {
    use super::*;

    #[test]
    fn test_protocols_md_get_properties() {
        // Exact JSON from PROTOCOLS.md
        let json = r#"{ "getProperties": { "version": 512, "client": "My Client", "device": "Server", "name": "LOAD" } }"#;

        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match &msg {
            ProtocolMessage::GetProperties(gp) => {
                assert_eq!(gp.version, Some("512".to_string()));
                assert_eq!(gp.device, Some("Server".to_string()));
                assert_eq!(gp.name, Some("LOAD".to_string()));
            }
            _ => panic!("Expected GetProperties"),
        }

        // Verify it can be serialized back
        let serialized = JsonProtocolSerializer::serialize(&msg).unwrap();
        assert!(serialized.contains("getProperties"));
        assert!(serialized.contains("\"version\":512"));
    }

    #[test]
    fn test_protocols_md_def_text_vector() {
        // Exact JSON from PROTOCOLS.md
        let json = r#"{ "defTextVector": { "version": 512, "device": "Server", "name": "LOAD", "group": "Main", "label": "Load driver", "perm": "rw", "state": "Idle", "items": [ { "name": "DRIVER", "label": "Load driver", "value": "" } ] } }"#;

        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(dtv) => {
                assert_eq!(dtv.attrs.device, "Server");
                assert_eq!(dtv.attrs.name, "LOAD");
                assert_eq!(dtv.attrs.group, "Main");
                assert_eq!(dtv.attrs.label, "Load driver");
                assert_eq!(dtv.attrs.state, PropertyState::Idle);
                assert_eq!(dtv.perm, PropertyPerm::ReadWrite);
                assert_eq!(dtv.elements.len(), 1);
                assert_eq!(dtv.elements[0].name, "DRIVER");
                assert_eq!(dtv.elements[0].label, "Load driver");
                assert_eq!(dtv.elements[0].value, "");
            }
            _ => panic!("Expected DefTextVector"),
        }
    }

    #[test]
    fn test_protocols_md_def_switch_vector() {
        // Exact JSON from PROTOCOLS.md
        let json = r#"{ "defSwitchVector": { "version": 512, "device": "Server", "name": "RESTART", "group": "Main", "label": "Restart", "perm": "rw", "state": "Idle", "rule": "AnyOfMany", "hints": "order: 10; widget: button", "items": [ { "name": "RESTART", "label": "Restart server", "value": false } ] } }"#;

        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefSwitchVector(dsv) => {
                assert_eq!(dsv.attrs.device, "Server");
                assert_eq!(dsv.attrs.name, "RESTART");
                assert_eq!(dsv.attrs.group, "Main");
                assert_eq!(dsv.attrs.label, "Restart");
                assert_eq!(dsv.attrs.state, PropertyState::Idle);
                assert_eq!(dsv.perm, PropertyPerm::ReadWrite);
                assert_eq!(dsv.rule, SwitchRule::AnyOfMany);
                assert_eq!(dsv.elements.len(), 1);
                assert_eq!(dsv.elements[0].name, "RESTART");
                assert_eq!(dsv.elements[0].label, "Restart server");
                assert_eq!(dsv.elements[0].value, SwitchState::Off); // false -> Off
            }
            _ => panic!("Expected DefSwitchVector"),
        }
    }

    #[test]
    fn test_protocols_md_def_number_vector() {
        // Exact JSON from PROTOCOLS.md
        let json = r#"{ "defNumberVector": { "version": 512, "device": "CCD Imager Simulator", "name": "CCD_EXPOSURE", "group": "Camera", "label": "Start exposure", "perm": "rw", "state": "Idle", "hints": "order: 10; target: show", "items": [ { "name": "EXPOSURE", "label": "Start exposure", "min": 0, "max": 10000, "step": 1, "format": "%g", "target": 0, "value": 0 } ] } }"#;

        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefNumberVector(dnv) => {
                assert_eq!(dnv.attrs.device, "CCD Imager Simulator");
                assert_eq!(dnv.attrs.name, "CCD_EXPOSURE");
                assert_eq!(dnv.attrs.group, "Camera");
                assert_eq!(dnv.attrs.label, "Start exposure");
                assert_eq!(dnv.attrs.state, PropertyState::Idle);
                assert_eq!(dnv.perm, PropertyPerm::ReadWrite);
                assert_eq!(dnv.elements.len(), 1);
                assert_eq!(dnv.elements[0].name, "EXPOSURE");
                assert_eq!(dnv.elements[0].label, "Start exposure");
                assert_eq!(dnv.elements[0].min, 0.0);
                assert_eq!(dnv.elements[0].max, 10000.0);
                assert_eq!(dnv.elements[0].step, 1.0);
                assert_eq!(dnv.elements[0].format, "%g");
                assert_eq!(dnv.elements[0].value, 0.0);
            }
            _ => panic!("Expected DefNumberVector"),
        }
    }

    #[test]
    fn test_protocols_md_set_switch_vector() {
        // Exact JSON from PROTOCOLS.md
        let json = r#"{ "setSwitchVector": { "device": "CCD Imager Simulator", "name": "CONNECTION", "state": "Ok", "items": [ { "name": "CONNECTED", "value": true }, { "name": "DISCONNECTED", "value": false } ] } }"#;

        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetSwitchVector(ssv) => {
                assert_eq!(ssv.attrs.device, "CCD Imager Simulator");
                assert_eq!(ssv.attrs.name, "CONNECTION");
                assert_eq!(ssv.attrs.state, Some(PropertyState::Ok));
                assert_eq!(ssv.elements.len(), 2);
                assert_eq!(ssv.elements[0].name, "CONNECTED");
                assert_eq!(ssv.elements[0].value, SwitchState::On); // true -> On
                assert_eq!(ssv.elements[1].name, "DISCONNECTED");
                assert_eq!(ssv.elements[1].value, SwitchState::Off); // false -> Off
            }
            _ => panic!("Expected SetSwitchVector"),
        }
    }

    #[test]
    fn test_protocols_md_set_blob_vector() {
        // Exact JSON from PROTOCOLS.md
        let json = r#"{ "setBLOBVector": { "device": "CCD Imager Simulator", "name": "CCD_IMAGE", "state": "Ok", "items": [ { "name": "IMAGE", "value": "/blob/0x10381d798.fits" } ] } }"#;

        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetBLOBVector(sbv) => {
                assert_eq!(sbv.attrs.device, "CCD Imager Simulator");
                assert_eq!(sbv.attrs.name, "CCD_IMAGE");
                assert_eq!(sbv.attrs.state, Some(PropertyState::Ok));
                assert_eq!(sbv.elements.len(), 1);
                assert_eq!(sbv.elements[0].name, "IMAGE");
                assert_eq!(sbv.elements[0].value, "/blob/0x10381d798.fits");
            }
            _ => panic!("Expected SetBLOBVector"),
        }
    }

    #[test]
    fn test_protocols_md_new_number_vector() {
        // Exact JSON from PROTOCOLS.md
        let json = r#"{"newNumberVector":{"device":"CCD Imager Simulator","name":"CCD_EXPOSURE","token": "FA0012", "items":[{"name":"EXPOSURE","value":1}]}}"#;

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
    fn test_protocols_md_delete_property() {
        // Exact JSON from PROTOCOLS.md
        let json = r#"{ "deleteProperty": { "device": "Mount IEQ (guider)" } }"#;

        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DelProperty(dp) => {
                assert_eq!(dp.device, "Mount IEQ (guider)");
                assert_eq!(dp.name, None);
            }
            _ => panic!("Expected DelProperty"),
        }
    }
}

// ============================================================================
// JSON Message Type Coverage Tests
// ============================================================================

#[cfg(test)]
mod json_message_types {
    use super::*;

    #[test]
    fn test_def_text_vector() {
        let json = r#"{"defTextVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"ro","state":"Ok","items":[{"name":"ITEM","label":"Item","value":"test"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefTextVector(dtv) => {
                assert_eq!(dtv.attrs.device, "Dev");
                assert_eq!(dtv.perm, PropertyPerm::ReadOnly);
                assert_eq!(dtv.elements[0].value, "test");
            }
            _ => panic!("Expected DefTextVector"),
        }
    }

    #[test]
    fn test_def_number_vector() {
        let json = r#"{"defNumberVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"wo","state":"Busy","items":[{"name":"NUM","label":"Number","min":0,"max":100,"step":1,"format":"%g","value":50}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefNumberVector(dnv) => {
                assert_eq!(dnv.attrs.device, "Dev");
                assert_eq!(dnv.perm, PropertyPerm::WriteOnly);
                assert_eq!(dnv.attrs.state, PropertyState::Busy);
                assert_eq!(dnv.elements[0].value, 50.0);
            }
            _ => panic!("Expected DefNumberVector"),
        }
    }

    #[test]
    fn test_def_switch_vector_one_of_many() {
        let json = r#"{"defSwitchVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"rw","state":"Idle","rule":"OneOfMany","items":[{"name":"OPT1","label":"Option 1","value":true},{"name":"OPT2","label":"Option 2","value":false}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefSwitchVector(dsv) => {
                assert_eq!(dsv.rule, SwitchRule::OneOfMany);
                assert_eq!(dsv.elements[0].value, SwitchState::On);
                assert_eq!(dsv.elements[1].value, SwitchState::Off);
            }
            _ => panic!("Expected DefSwitchVector"),
        }
    }

    #[test]
    fn test_def_switch_vector_at_most_one() {
        let json = r#"{"defSwitchVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"rw","state":"Idle","rule":"AtMostOne","items":[{"name":"SW","label":"Switch","value":false}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefSwitchVector(dsv) => {
                assert_eq!(dsv.rule, SwitchRule::AtMostOne);
            }
            _ => panic!("Expected DefSwitchVector"),
        }
    }

    #[test]
    fn test_def_light_vector() {
        let json = r#"{"defLightVector":{"version":512,"device":"Dev","name":"LIGHTS","group":"G","label":"L","state":"Ok","items":[{"name":"RED","label":"Red","value":"Alert"},{"name":"GREEN","label":"Green","value":"Ok"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefLightVector(dlv) => {
                assert_eq!(dlv.elements.len(), 2);
                assert_eq!(dlv.elements[0].value, PropertyState::Alert);
                assert_eq!(dlv.elements[1].value, PropertyState::Ok);
            }
            _ => panic!("Expected DefLightVector"),
        }
    }

    #[test]
    fn test_def_blob_vector() {
        let json = r#"{"defBLOBVector":{"version":512,"device":"Dev","name":"IMAGE","group":"G","label":"L","perm":"ro","state":"Idle","items":[{"name":"IMG","label":"Image"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefBLOBVector(dbv) => {
                assert_eq!(dbv.elements.len(), 1);
                assert_eq!(dbv.elements[0].name, "IMG");
            }
            _ => panic!("Expected DefBLOBVector"),
        }
    }

    #[test]
    fn test_set_text_vector() {
        let json = r#"{"setTextVector":{"device":"Dev","name":"PROP","state":"Ok","items":[{"name":"TEXT","value":"hello"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetTextVector(stv) => {
                assert_eq!(stv.elements[0].value, "hello");
            }
            _ => panic!("Expected SetTextVector"),
        }
    }

    #[test]
    fn test_set_number_vector() {
        let json = r#"{"setNumberVector":{"device":"Dev","name":"PROP","state":"Ok","items":[{"name":"NUM","value":3.14159}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(snv) => {
                assert_eq!(snv.elements[0].value, 3.14159);
            }
            _ => panic!("Expected SetNumberVector"),
        }
    }

    #[test]
    fn test_set_light_vector() {
        let json = r#"{"setLightVector":{"device":"Dev","name":"LIGHTS","state":"Ok","items":[{"name":"STATUS","value":"Busy"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetLightVector(slv) => {
                assert_eq!(slv.elements[0].value, PropertyState::Busy);
            }
            _ => panic!("Expected SetLightVector"),
        }
    }

    #[test]
    fn test_new_text_vector() {
        let json = r#"{"newTextVector":{"device":"Dev","name":"PROP","items":[{"name":"TEXT","value":"new value"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::NewTextVector(ntv) => {
                assert_eq!(ntv.elements[0].value, "new value");
            }
            _ => panic!("Expected NewTextVector"),
        }
    }

    #[test]
    fn test_new_switch_vector() {
        let json = r#"{"newSwitchVector":{"device":"Dev","name":"PROP","items":[{"name":"SW","value":true}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::NewSwitchVector(nsv) => {
                assert_eq!(nsv.elements[0].value, SwitchState::On);
            }
            _ => panic!("Expected NewSwitchVector"),
        }
    }

    #[test]
    fn test_new_blob_vector() {
        let json = r#"{"newBLOBVector":{"device":"Dev","name":"IMAGE","items":[{"name":"IMG","format":".fits"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::NewBLOBVector(nbv) => {
                assert_eq!(nbv.elements[0].format, ".fits");
            }
            _ => panic!("Expected NewBLOBVector"),
        }
    }

    #[test]
    fn test_message() {
        let json = r#"{"message":{"device":"Dev","timestamp":"2024-01-01T00:00:00","message":"Test message"}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::Message(m) => {
                assert_eq!(m.device, Some("Dev".to_string()));
                assert_eq!(m.message, Some("Test message".to_string()));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_enable_blob() {
        let json = r#"{"enableBLOB":{"device":"Dev","name":"IMAGE","value":"Also"}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::EnableBLOB(eb) => {
                assert_eq!(eb.device, "Dev");
                assert_eq!(eb.value, BLOBEnable::Also);
            }
            _ => panic!("Expected EnableBLOB"),
        }
    }
}

// ============================================================================
// JSON-Specific Features Tests
// ============================================================================

#[cfg(test)]
mod json_specific_features {
    use super::*;
    use libindigo::strategies::rs::protocol::ProtocolMessage;

    #[test]
    fn test_boolean_switch_values() {
        // JSON uses true/false, not "On"/"Off"
        let json = r#"{"setSwitchVector":{"device":"Dev","name":"SW","state":"Ok","items":[{"name":"A","value":true},{"name":"B","value":false}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetSwitchVector(ssv) => {
                assert_eq!(ssv.elements[0].value, SwitchState::On);
                assert_eq!(ssv.elements[1].value, SwitchState::Off);
            }
            _ => panic!("Expected SetSwitchVector"),
        }
    }

    #[test]
    fn test_numeric_version() {
        // JSON uses version number 512, not string "2.0"
        let json = r#"{"getProperties":{"version":512}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::GetProperties(gp) => {
                assert_eq!(gp.version, Some("512".to_string()));
            }
            _ => panic!("Expected GetProperties"),
        }
    }

    #[test]
    fn test_items_array_structure() {
        // JSON uses "items" array for elements
        let json = r#"{"setNumberVector":{"device":"Dev","name":"NUMS","state":"Ok","items":[{"name":"N1","value":1},{"name":"N2","value":2},{"name":"N3","value":3}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(snv) => {
                assert_eq!(snv.elements.len(), 3);
                assert_eq!(snv.elements[0].value, 1.0);
                assert_eq!(snv.elements[1].value, 2.0);
                assert_eq!(snv.elements[2].value, 3.0);
            }
            _ => panic!("Expected SetNumberVector"),
        }
    }

    #[test]
    fn test_optional_fields_handling() {
        // Test with minimal required fields
        let json = r#"{"deleteProperty":{"device":"Dev"}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DelProperty(dp) => {
                assert_eq!(dp.device, "Dev");
                assert_eq!(dp.name, None);
                assert_eq!(dp.timestamp, None);
                assert_eq!(dp.message, None);
            }
            _ => panic!("Expected DelProperty"),
        }
    }

    #[test]
    fn test_hints_attribute_parsing() {
        // Hints are passed through as strings
        let json = r#"{"defSwitchVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"rw","state":"Idle","rule":"AnyOfMany","hints":"order: 10; widget: button","items":[{"name":"BTN","label":"Button","value":false}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::DefSwitchVector(_) => {
                // Hints are parsed but not validated in this test
                // Just verify the message parses correctly
            }
            _ => panic!("Expected DefSwitchVector"),
        }
    }

    #[test]
    fn test_numeric_values_not_strings() {
        // Numbers should be JSON numbers, not strings
        let json = r#"{"setNumberVector":{"device":"Dev","name":"PROP","state":"Ok","items":[{"name":"NUM","value":42.5}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(snv) => {
                assert_eq!(snv.elements[0].value, 42.5);
            }
            _ => panic!("Expected SetNumberVector"),
        }
    }

    #[test]
    fn test_blob_url_references_only() {
        // JSON protocol only supports URL-referenced BLOBs
        let json = r#"{"setBLOBVector":{"device":"Dev","name":"IMG","state":"Ok","items":[{"name":"IMAGE","value":"/blob/0x12345.fits"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetBLOBVector(sbv) => {
                assert_eq!(sbv.elements[0].value, "/blob/0x12345.fits");
                // Size should be 0 for URL references
                assert_eq!(sbv.elements[0].size, 0);
            }
            _ => panic!("Expected SetBLOBVector"),
        }
    }
}

// ============================================================================
// Roundtrip Tests
// ============================================================================

#[cfg(test)]
mod roundtrip_tests {
    use super::*;
    use libindigo::strategies::rs::protocol::ProtocolMessage;

    #[test]
    fn test_roundtrip_get_properties() {
        // Note: JSON protocol always uses version 512, so roundtrip will normalize version
        let original = ProtocolMessage::GetProperties(GetProperties {
            version: Some("2.0".to_string()),
            device: Some("CCD Simulator".to_string()),
            name: Some("CONNECTION".to_string()),
        });

        let json = JsonProtocolSerializer::serialize(&original).unwrap();
        let parsed = JsonProtocolParser::parse_message(&json).unwrap();

        // After roundtrip, version will be "512" (JSON protocol version)
        let expected = ProtocolMessage::GetProperties(GetProperties {
            version: Some("512".to_string()),
            device: Some("CCD Simulator".to_string()),
            name: Some("CONNECTION".to_string()),
        });

        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_roundtrip_def_text_vector() {
        let original = ProtocolMessage::DefTextVector(DefTextVector {
            attrs: VectorAttributes {
                device: "Server".to_string(),
                name: "LOAD".to_string(),
                label: "Load driver".to_string(),
                group: "Main".to_string(),
                state: PropertyState::Idle,
                timeout: None,
                timestamp: None,
                message: None,
            },
            perm: PropertyPerm::ReadWrite,
            elements: vec![DefText {
                name: "DRIVER".to_string(),
                label: "Load driver".to_string(),
                value: "".to_string(),
            }],
        });

        let json = JsonProtocolSerializer::serialize(&original).unwrap();
        let parsed = JsonProtocolParser::parse_message(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_roundtrip_def_number_vector() {
        let original = ProtocolMessage::DefNumberVector(DefNumberVector {
            attrs: VectorAttributes {
                device: "CCD Simulator".to_string(),
                name: "CCD_EXPOSURE".to_string(),
                label: "Exposure".to_string(),
                group: "Camera".to_string(),
                state: PropertyState::Idle,
                timeout: None,
                timestamp: None,
                message: None,
            },
            perm: PropertyPerm::ReadWrite,
            elements: vec![DefNumber {
                name: "EXPOSURE".to_string(),
                label: "Duration".to_string(),
                format: "%g".to_string(),
                min: 0.0,
                max: 10000.0,
                step: 1.0,
                value: 1.0,
            }],
        });

        let json = JsonProtocolSerializer::serialize(&original).unwrap();
        let parsed = JsonProtocolParser::parse_message(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_roundtrip_def_switch_vector() {
        let original = ProtocolMessage::DefSwitchVector(DefSwitchVector {
            attrs: VectorAttributes {
                device: "CCD Simulator".to_string(),
                name: "CONNECTION".to_string(),
                label: "Connection".to_string(),
                group: "Main".to_string(),
                state: PropertyState::Idle,
                timeout: None,
                timestamp: None,
                message: None,
            },
            perm: PropertyPerm::ReadWrite,
            rule: SwitchRule::OneOfMany,
            elements: vec![
                DefSwitch {
                    name: "CONNECTED".to_string(),
                    label: "Connected".to_string(),
                    value: SwitchState::Off,
                },
                DefSwitch {
                    name: "DISCONNECTED".to_string(),
                    label: "Disconnected".to_string(),
                    value: SwitchState::On,
                },
            ],
        });

        let json = JsonProtocolSerializer::serialize(&original).unwrap();
        let parsed = JsonProtocolParser::parse_message(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_roundtrip_set_switch_vector() {
        let original = ProtocolMessage::SetSwitchVector(SetSwitchVector {
            attrs: SetVectorAttributes {
                device: "CCD Simulator".to_string(),
                name: "CONNECTION".to_string(),
                state: Some(PropertyState::Ok),
                timeout: None,
                timestamp: None,
                message: None,
            },
            elements: vec![
                OneSwitch {
                    name: "CONNECTED".to_string(),
                    value: SwitchState::On,
                },
                OneSwitch {
                    name: "DISCONNECTED".to_string(),
                    value: SwitchState::Off,
                },
            ],
        });

        let json = JsonProtocolSerializer::serialize(&original).unwrap();
        let parsed = JsonProtocolParser::parse_message(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_roundtrip_new_number_vector() {
        let original = ProtocolMessage::NewNumberVector(NewNumberVector {
            attrs: NewVectorAttributes {
                device: "CCD Simulator".to_string(),
                name: "CCD_EXPOSURE".to_string(),
                timestamp: None,
            },
            elements: vec![OneNumber {
                name: "EXPOSURE".to_string(),
                value: 5.0,
            }],
        });

        let json = JsonProtocolSerializer::serialize(&original).unwrap();
        let parsed = JsonProtocolParser::parse_message(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_roundtrip_delete_property() {
        let original = ProtocolMessage::DelProperty(DelProperty {
            device: "Mount Simulator".to_string(),
            name: Some("EQUATORIAL_EOD_COORD".to_string()),
            timestamp: None,
            message: None,
        });

        let json = JsonProtocolSerializer::serialize(&original).unwrap();
        let parsed = JsonProtocolParser::parse_message(&json).unwrap();

        assert_eq!(original, parsed);
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[cfg(test)]
mod error_handling {
    use super::*;
    use libindigo::strategies::rs::protocol_json::JsonProtocolParser;

    #[test]
    fn test_malformed_json() {
        let json = r#"{"getProperties": { invalid json }"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_field_device() {
        let json = r#"{"defTextVector":{"version":512,"name":"PROP","group":"G","label":"L","perm":"rw","state":"Idle","items":[]}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_field_name() {
        let json = r#"{"defTextVector":{"version":512,"device":"Dev","group":"G","label":"L","perm":"rw","state":"Idle","items":[]}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_state_value() {
        let json = r#"{"defTextVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"rw","state":"InvalidState","items":[]}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_perm_value() {
        let json = r#"{"defTextVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"invalid","state":"Idle","items":[]}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_rule_value() {
        let json = r#"{"defSwitchVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"rw","state":"Idle","rule":"InvalidRule","items":[]}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_mismatch_boolean_as_string() {
        let json = r#"{"setSwitchVector":{"device":"Dev","name":"SW","state":"Ok","items":[{"name":"A","value":"true"}]}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_mismatch_number_as_string() {
        let json = r#"{"setNumberVector":{"device":"Dev","name":"NUM","state":"Ok","items":[{"name":"N","value":"42"}]}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_items_array() {
        let json = r#"{"defTextVector":{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"rw","state":"Idle"}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_message_type() {
        let json = r#"{"unknownMessage":{"device":"Dev"}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_root_keys() {
        let json = r#"{"getProperties":{},"defTextVector":{}}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_json_object() {
        let json = r#"{}"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_array_instead_of_object() {
        let json = r#"[{"getProperties":{}}]"#;
        let result = JsonProtocolParser::parse_message(json);
        assert!(result.is_err());
    }
}

// ============================================================================
// Version Constant Tests
// ============================================================================

#[cfg(test)]
mod version_tests {
    use super::*;
    use libindigo::strategies::rs::protocol::{GetProperties, ProtocolMessage};
    use libindigo::strategies::rs::protocol_json::{JsonProtocolSerializer, JSON_PROTOCOL_VERSION};

    #[test]
    fn test_json_protocol_version_constant() {
        assert_eq!(JSON_PROTOCOL_VERSION, 512);
    }

    #[test]
    fn test_serialized_version_matches_constant() {
        let msg = ProtocolMessage::GetProperties(GetProperties {
            version: Some("2.0".to_string()),
            device: None,
            name: None,
        });

        let json = JsonProtocolSerializer::serialize(&msg).unwrap();
        assert!(json.contains("\"version\":512"));
    }
}

// ============================================================================
// Complex Scenarios Tests
// ============================================================================

#[cfg(test)]
mod complex_scenarios {
    use super::*;
    use libindigo::strategies::rs::protocol::ProtocolMessage;
    use libindigo::strategies::rs::protocol_json::JsonProtocolParser;
    use libindigo::types::property::{PropertyPerm, PropertyState};

    #[test]
    fn test_multiple_items_in_vector() {
        let json = r#"{"setNumberVector":{"device":"Dev","name":"COORDS","state":"Ok","items":[{"name":"RA","value":12.5},{"name":"DEC","value":45.3},{"name":"ALT","value":30.0},{"name":"AZ","value":180.0}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(snv) => {
                assert_eq!(snv.elements.len(), 4);
                assert_eq!(snv.elements[0].name, "RA");
                assert_eq!(snv.elements[1].name, "DEC");
                assert_eq!(snv.elements[2].name, "ALT");
                assert_eq!(snv.elements[3].name, "AZ");
            }
            _ => panic!("Expected SetNumberVector"),
        }
    }

    #[test]
    fn test_empty_string_values() {
        let json = r#"{"setTextVector":{"device":"Dev","name":"TEXT","state":"Ok","items":[{"name":"EMPTY","value":""}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetTextVector(stv) => {
                assert_eq!(stv.elements[0].value, "");
            }
            _ => panic!("Expected SetTextVector"),
        }
    }

    #[test]
    fn test_special_characters_in_strings() {
        let json = r#"{"setTextVector":{"device":"Dev","name":"TEXT","state":"Ok","items":[{"name":"SPECIAL","value":"Test \"quoted\" and \n newline"}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetTextVector(stv) => {
                assert!(stv.elements[0].value.contains("quoted"));
                assert!(stv.elements[0].value.contains("\n"));
            }
            _ => panic!("Expected SetTextVector"),
        }
    }

    #[test]
    fn test_very_large_numbers() {
        let json = r#"{"setNumberVector":{"device":"Dev","name":"BIG","state":"Ok","items":[{"name":"NUM","value":1.7976931348623157e308}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(snv) => {
                assert!(snv.elements[0].value > 1e308);
            }
            _ => panic!("Expected SetNumberVector"),
        }
    }

    #[test]
    fn test_very_small_numbers() {
        let json = r#"{"setNumberVector":{"device":"Dev","name":"SMALL","state":"Ok","items":[{"name":"NUM","value":2.2250738585072014e-308}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(snv) => {
                assert!(snv.elements[0].value < 1e-307);
            }
            _ => panic!("Expected SetNumberVector"),
        }
    }

    #[test]
    fn test_negative_numbers() {
        let json = r#"{"setNumberVector":{"device":"Dev","name":"NEG","state":"Ok","items":[{"name":"NUM","value":-42.5}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(snv) => {
                assert_eq!(snv.elements[0].value, -42.5);
            }
            _ => panic!("Expected SetNumberVector"),
        }
    }

    #[test]
    fn test_zero_values() {
        let json = r#"{"setNumberVector":{"device":"Dev","name":"ZERO","state":"Ok","items":[{"name":"NUM","value":0}]}}"#;
        let msg = JsonProtocolParser::parse_message(json).unwrap();

        match msg {
            ProtocolMessage::SetNumberVector(snv) => {
                assert_eq!(snv.elements[0].value, 0.0);
            }
            _ => panic!("Expected SetNumberVector"),
        }
    }

    #[test]
    fn test_all_property_states() {
        let states = vec![
            ("Idle", PropertyState::Idle),
            ("Ok", PropertyState::Ok),
            ("Busy", PropertyState::Busy),
            ("Alert", PropertyState::Alert),
        ];

        for (state_str, expected_state) in states {
            let json = format!(
                r#"{{"defTextVector":{{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"rw","state":"{}","items":[]}}}}"#,
                state_str
            );
            let msg = JsonProtocolParser::parse_message(&json).unwrap();

            match msg {
                ProtocolMessage::DefTextVector(dtv) => {
                    assert_eq!(dtv.attrs.state, expected_state);
                }
                _ => panic!("Expected DefTextVector"),
            }
        }
    }

    #[test]
    fn test_all_permission_types() {
        let perms = vec![
            ("ro", PropertyPerm::ReadOnly),
            ("wo", PropertyPerm::WriteOnly),
            ("rw", PropertyPerm::ReadWrite),
        ];

        for (perm_str, expected_perm) in perms {
            let json = format!(
                r#"{{"defTextVector":{{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"{}","state":"Idle","items":[]}}}}"#,
                perm_str
            );
            let msg = JsonProtocolParser::parse_message(&json).unwrap();

            match msg {
                ProtocolMessage::DefTextVector(dtv) => {
                    assert_eq!(dtv.perm, expected_perm);
                }
                _ => panic!("Expected DefTextVector"),
            }
        }
    }

    #[test]
    fn test_all_switch_rules() {
        let rules = vec![
            ("OneOfMany", SwitchRule::OneOfMany),
            ("AtMostOne", SwitchRule::AtMostOne),
            ("AnyOfMany", SwitchRule::AnyOfMany),
        ];

        for (rule_str, expected_rule) in rules {
            let json = format!(
                r#"{{"defSwitchVector":{{"version":512,"device":"Dev","name":"PROP","group":"G","label":"L","perm":"rw","state":"Idle","rule":"{}","items":[]}}}}"#,
                rule_str
            );
            let msg = JsonProtocolParser::parse_message(&json).unwrap();

            match msg {
                ProtocolMessage::DefSwitchVector(dsv) => {
                    assert_eq!(dsv.rule, expected_rule);
                }
                _ => panic!("Expected DefSwitchVector"),
            }
        }
    }
}
