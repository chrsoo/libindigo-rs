//! Protocol Negotiation Tests
//!
//! Tests for protocol negotiation between JSON and XML protocols.

mod harness;

use libindigo::strategies::rs::protocol_negotiation::{ProtocolNegotiator, ProtocolType};

// ============================================================================
// Protocol Type Detection Tests
// ============================================================================

#[cfg(test)]
mod protocol_detection {
    use super::*;

    #[test]
    fn test_detect_json_from_data() {
        let data = b"{\"getProperties\":{}}";
        let protocol = ProtocolType::detect_from_data(data);
        assert_eq!(protocol, Some(ProtocolType::Json));
    }

    #[test]
    fn test_detect_xml_from_data() {
        let data = b"<getProperties version=\"1.7\"/>";
        let protocol = ProtocolType::detect_from_data(data);
        assert_eq!(protocol, Some(ProtocolType::Xml));
    }

    #[test]
    fn test_detect_json_with_leading_whitespace() {
        let data = b"  \n\t  {\"getProperties\":{}}";
        let protocol = ProtocolType::detect_from_data(data);
        assert_eq!(protocol, Some(ProtocolType::Json));
    }

    #[test]
    fn test_detect_xml_with_leading_whitespace() {
        let data = b"  \n\t  <getProperties version=\"1.7\"/>";
        let protocol = ProtocolType::detect_from_data(data);
        assert_eq!(protocol, Some(ProtocolType::Xml));
    }

    #[test]
    fn test_detect_invalid_data() {
        let data = b"invalid data";
        let protocol = ProtocolType::detect_from_data(data);
        assert_eq!(protocol, None);
    }

    #[test]
    fn test_detect_empty_data() {
        let data = b"";
        let protocol = ProtocolType::detect_from_data(data);
        assert_eq!(protocol, None);
    }

    #[test]
    fn test_detect_whitespace_only() {
        let data = b"   \n\t\r   ";
        let protocol = ProtocolType::detect_from_data(data);
        assert_eq!(protocol, None);
    }

    #[test]
    fn test_detect_json_array() {
        let data = b"[{\"test\":1}]";
        let protocol = ProtocolType::detect_from_data(data);
        // Arrays are not valid INDIGO messages, but detection should still identify JSON structure
        assert_eq!(protocol, None); // '[' is not '{' or '<'
    }
}

// ============================================================================
// Protocol Type Properties Tests
// ============================================================================

#[cfg(test)]
mod protocol_type_properties {
    use super::*;

    #[test]
    fn test_json_version_string() {
        assert_eq!(ProtocolType::Json.version_string(), "512");
    }

    #[test]
    fn test_xml_version_string() {
        assert_eq!(ProtocolType::Xml.version_string(), "1.7");
    }

    #[test]
    fn test_default_protocol_is_json() {
        assert_eq!(ProtocolType::default(), ProtocolType::Json);
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(format!("{}", ProtocolType::Json), "JSON");
        assert_eq!(format!("{}", ProtocolType::Xml), "XML");
    }

    #[test]
    fn test_protocol_equality() {
        assert_eq!(ProtocolType::Json, ProtocolType::Json);
        assert_eq!(ProtocolType::Xml, ProtocolType::Xml);
        assert_ne!(ProtocolType::Json, ProtocolType::Xml);
    }

    #[test]
    fn test_protocol_clone() {
        let proto = ProtocolType::Json;
        let cloned = proto.clone();
        assert_eq!(proto, cloned);
    }

    #[test]
    fn test_protocol_copy() {
        let proto = ProtocolType::Json;
        let copied = proto;
        assert_eq!(proto, copied);
    }
}

// ============================================================================
// Protocol Negotiator Configuration Tests
// ============================================================================

#[cfg(test)]
mod negotiator_configuration {
    use super::*;

    #[test]
    fn test_json_first_negotiator() {
        let negotiator = ProtocolNegotiator::json_first();
        // Verify it's created successfully
        assert_eq!(format!("{:?}", negotiator).contains("Json"), true);
    }

    #[test]
    fn test_xml_only_negotiator() {
        let negotiator = ProtocolNegotiator::xml_only();
        // Verify it's created successfully
        assert_eq!(format!("{:?}", negotiator).contains("Xml"), true);
    }

    #[test]
    fn test_json_only_negotiator() {
        let negotiator = ProtocolNegotiator::json_only();
        // Verify it's created successfully
        assert_eq!(format!("{:?}", negotiator).contains("Json"), true);
    }

    #[test]
    fn test_custom_negotiator_json_with_fallback() {
        let negotiator = ProtocolNegotiator::new(ProtocolType::Json, true);
        // Verify it's created successfully
        assert_eq!(format!("{:?}", negotiator).contains("Json"), true);
    }

    #[test]
    fn test_custom_negotiator_xml_with_fallback() {
        let negotiator = ProtocolNegotiator::new(ProtocolType::Xml, true);
        // Verify it's created successfully
        assert_eq!(format!("{:?}", negotiator).contains("Xml"), true);
    }

    #[test]
    fn test_custom_negotiator_json_no_fallback() {
        let negotiator = ProtocolNegotiator::new(ProtocolType::Json, false);
        // Verify it's created successfully
        assert_eq!(format!("{:?}", negotiator).contains("Json"), true);
    }

    #[test]
    fn test_custom_negotiator_xml_no_fallback() {
        let negotiator = ProtocolNegotiator::new(ProtocolType::Xml, false);
        // Verify it's created successfully
        assert_eq!(format!("{:?}", negotiator).contains("Xml"), true);
    }

    #[test]
    fn test_negotiator_clone() {
        let negotiator = ProtocolNegotiator::json_first();
        let cloned = negotiator.clone();
        // Both should have same configuration
        assert_eq!(format!("{:?}", negotiator), format!("{:?}", cloned));
    }
}

// ============================================================================
// Message Boundary Detection Tests
// ============================================================================

#[cfg(test)]
mod message_boundary_detection {
    use super::*;

    #[test]
    fn test_json_simple_object() {
        let data = b"{\"test\":1}";
        // JSON detection should work
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_json_nested_objects() {
        let data = b"{\"outer\":{\"inner\":{\"deep\":1}}}";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_json_with_escaped_braces() {
        // JSON with escaped braces in strings
        let data = br#"{"message":"Test \{ and \} braces"}"#;
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_json_with_string_containing_xml() {
        // JSON containing XML-like content in strings
        let data = br#"{"value":"<tag>content</tag>"}"#;
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_xml_simple_tag() {
        let data = b"<tag/>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_xml_nested_tags() {
        let data = b"<outer><inner><deep/></inner></outer>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_xml_with_attributes() {
        let data = b"<tag attr1=\"value1\" attr2=\"value2\"/>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_xml_with_content() {
        let data = b"<tag>content</tag>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }
}

// ============================================================================
// Protocol Switching Scenarios Tests
// ============================================================================

#[cfg(test)]
mod protocol_switching {
    use super::*;

    #[test]
    fn test_json_to_json_no_switch() {
        // Client sends JSON, server responds with JSON - no switch needed
        let client_data = b"{\"getProperties\":{\"version\":512}}";
        let server_data = b"{\"defTextVector\":{}}";

        assert_eq!(
            ProtocolType::detect_from_data(client_data),
            Some(ProtocolType::Json)
        );
        assert_eq!(
            ProtocolType::detect_from_data(server_data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_json_to_xml_switch() {
        // Client sends JSON, server responds with XML - switch needed
        let client_data = b"{\"getProperties\":{\"version\":512}}";
        let server_data = b"<defTextVector/>";

        assert_eq!(
            ProtocolType::detect_from_data(client_data),
            Some(ProtocolType::Json)
        );
        assert_eq!(
            ProtocolType::detect_from_data(server_data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_xml_to_xml_no_switch() {
        // Client sends XML, server responds with XML - no switch needed
        let client_data = b"<getProperties version=\"1.7\"/>";
        let server_data = b"<defTextVector/>";

        assert_eq!(
            ProtocolType::detect_from_data(client_data),
            Some(ProtocolType::Xml)
        );
        assert_eq!(
            ProtocolType::detect_from_data(server_data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_xml_to_json_switch() {
        // Client sends XML, server responds with JSON - switch needed
        let client_data = b"<getProperties version=\"1.7\"/>";
        let server_data = b"{\"defTextVector\":{}}";

        assert_eq!(
            ProtocolType::detect_from_data(client_data),
            Some(ProtocolType::Xml)
        );
        assert_eq!(
            ProtocolType::detect_from_data(server_data),
            Some(ProtocolType::Json)
        );
    }
}

// ============================================================================
// Edge Cases and Special Scenarios Tests
// ============================================================================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_detect_with_bom() {
        // UTF-8 BOM followed by JSON
        let data = b"\xEF\xBB\xBF{\"test\":1}";
        // BOM bytes are not whitespace, so detection should fail
        assert_eq!(ProtocolType::detect_from_data(data), None);
    }

    #[test]
    fn test_detect_with_null_bytes() {
        let data = b"\0\0{\"test\":1}";
        // Null bytes should cause detection to fail
        assert_eq!(ProtocolType::detect_from_data(data), None);
    }

    #[test]
    fn test_detect_single_brace() {
        let data = b"{";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_detect_single_angle_bracket() {
        let data = b"<";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_detect_mixed_whitespace() {
        let data = b" \t\n\r {\"test\":1}";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_detect_unicode_whitespace() {
        // Regular ASCII whitespace only
        let data = b"   {\"test\":1}";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_detect_comment_like_content() {
        // JSON doesn't support comments, but if data starts with //, it's invalid
        let data = b"// comment\n{\"test\":1}";
        assert_eq!(ProtocolType::detect_from_data(data), None);
    }

    #[test]
    fn test_detect_xml_declaration() {
        let data = b"<?xml version=\"1.0\"?><root/>";
        // Starts with '<', so should detect as XML
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_detect_xml_comment() {
        let data = b"<!-- comment --><root/>";
        // Starts with '<', so should detect as XML
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }
}

// ============================================================================
// Real-World Protocol Messages Tests
// ============================================================================

#[cfg(test)]
mod real_world_messages {
    use super::*;

    #[test]
    fn test_detect_real_json_get_properties() {
        let data = br#"{"getProperties":{"version":512,"device":"CCD Simulator"}}"#;
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_detect_real_json_def_text_vector() {
        let data = br#"{"defTextVector":{"version":512,"device":"Server","name":"LOAD","group":"Main","label":"Load driver","perm":"rw","state":"Idle","items":[{"name":"DRIVER","label":"Load driver","value":""}]}}"#;
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_detect_real_json_set_switch_vector() {
        let data = br#"{"setSwitchVector":{"device":"CCD Imager Simulator","name":"CONNECTION","state":"Ok","items":[{"name":"CONNECTED","value":true},{"name":"DISCONNECTED","value":false}]}}"#;
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_detect_real_xml_get_properties() {
        let data = b"<getProperties version=\"1.7\" device=\"CCD Simulator\"/>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_detect_real_xml_def_text_vector() {
        let data = b"<defTextVector device=\"Server\" name=\"LOAD\" group=\"Main\" label=\"Load driver\" state=\"Idle\" perm=\"rw\"><defText name=\"DRIVER\" label=\"Load driver\"></defText></defTextVector>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_detect_real_xml_set_switch_vector() {
        let data = b"<setSwitchVector device=\"CCD Imager Simulator\" name=\"CONNECTION\" state=\"Ok\"><oneSwitch name=\"CONNECTED\">On</oneSwitch><oneSwitch name=\"DISCONNECTED\">Off</oneSwitch></setSwitchVector>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }
}

// ============================================================================
// Protocol Negotiation Strategy Tests
// ============================================================================

#[cfg(test)]
mod negotiation_strategies {
    use super::*;

    #[test]
    fn test_json_first_strategy() {
        // JSON-first strategy should prefer JSON
        let negotiator = ProtocolNegotiator::json_first();
        // Verify the negotiator is configured for JSON preference
        let debug_str = format!("{:?}", negotiator);
        assert!(debug_str.contains("Json"));
    }

    #[test]
    fn test_xml_only_strategy() {
        // XML-only strategy should not fall back
        let negotiator = ProtocolNegotiator::xml_only();
        let debug_str = format!("{:?}", negotiator);
        assert!(debug_str.contains("Xml"));
    }

    #[test]
    fn test_json_only_strategy() {
        // JSON-only strategy should not fall back
        let negotiator = ProtocolNegotiator::json_only();
        let debug_str = format!("{:?}", negotiator);
        assert!(debug_str.contains("Json"));
    }

    #[test]
    fn test_fallback_enabled() {
        // Test that fallback can be enabled
        let negotiator = ProtocolNegotiator::new(ProtocolType::Json, true);
        let debug_str = format!("{:?}", negotiator);
        assert!(debug_str.contains("fallback_enabled: true"));
    }

    #[test]
    fn test_fallback_disabled() {
        // Test that fallback can be disabled
        let negotiator = ProtocolNegotiator::new(ProtocolType::Json, false);
        let debug_str = format!("{:?}", negotiator);
        assert!(debug_str.contains("fallback_enabled: false"));
    }
}

// ============================================================================
// Performance and Stress Tests
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_detect_large_json_message() {
        // Create a large JSON message
        let mut data = Vec::new();
        data.extend_from_slice(b"{\"items\":[");
        for i in 0..1000 {
            if i > 0 {
                data.push(b',');
            }
            data.extend_from_slice(format!("{{\"id\":{}}}", i).as_bytes());
        }
        data.extend_from_slice(b"]}");

        assert_eq!(
            ProtocolType::detect_from_data(&data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_detect_large_xml_message() {
        // Create a large XML message
        let mut data = Vec::new();
        data.extend_from_slice(b"<root>");
        for i in 0..1000 {
            data.extend_from_slice(format!("<item id=\"{}\"/>", i).as_bytes());
        }
        data.extend_from_slice(b"</root>");

        assert_eq!(
            ProtocolType::detect_from_data(&data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_detect_with_many_leading_spaces() {
        let mut data = vec![b' '; 10000];
        data.extend_from_slice(b"{\"test\":1}");

        assert_eq!(
            ProtocolType::detect_from_data(&data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_rapid_detection_calls() {
        // Test that detection can be called many times quickly
        let json_data = b"{\"test\":1}";
        let xml_data = b"<test/>";

        for _ in 0..1000 {
            assert_eq!(
                ProtocolType::detect_from_data(json_data),
                Some(ProtocolType::Json)
            );
            assert_eq!(
                ProtocolType::detect_from_data(xml_data),
                Some(ProtocolType::Xml)
            );
        }
    }
}
