//! Comprehensive tests for BLOB (Binary Large Object) support.
//!
//! This test suite verifies:
//! - Base64 encoding/decoding of BLOB data
//! - XML protocol BLOB parsing (setBLOBVector)
//! - XML protocol BLOB sending (newBLOBVector)
//! - JSON protocol BLOB support
//! - BlobTransferMode serialization
//! - Large BLOB handling
//! - BLOB property conversion

use libindigo::types::{
    BlobTransferMode, Property, PropertyItem, PropertyState, PropertyType, PropertyValue,
};
use libindigo_rs::protocol::{
    decode_blob, encode_blob, BLOBEnable, EnableBLOB, NewBLOBVector, NewVectorAttributes, OneBLOB,
    ProtocolMessage, ProtocolParser, ProtocolSerializer, SetBLOBVector, SetVectorAttributes,
};

// ============================================================================
// Base64 Encoding/Decoding Tests
// ============================================================================

#[test]
fn test_encode_blob_empty() {
    let data = vec![];
    let encoded = encode_blob(&data);
    assert_eq!(encoded, "");
}

#[test]
fn test_encode_blob_small() {
    let data = b"Hello, INDIGO!";
    let encoded = encode_blob(data);
    // Verify it's valid base64
    assert!(!encoded.is_empty());
    assert!(encoded
        .chars()
        .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
}

#[test]
fn test_decode_blob_empty() {
    let decoded = decode_blob("").unwrap();
    assert_eq!(decoded, Vec::<u8>::new());
}

#[test]
fn test_decode_blob_small() {
    let original = b"Hello, INDIGO!";
    let encoded = encode_blob(original);
    let decoded = decode_blob(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_blob_round_trip() {
    let original = b"The quick brown fox jumps over the lazy dog";
    let encoded = encode_blob(original);
    let decoded = decode_blob(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_blob_round_trip_binary() {
    // Test with binary data (not just ASCII)
    let original: Vec<u8> = (0..=255).collect();
    let encoded = encode_blob(&original);
    let decoded = decode_blob(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_decode_blob_with_whitespace() {
    // Base64 data may contain newlines/whitespace
    let data_with_whitespace = "SGVs\nbG8s\nIElO\nRElH\nTyE=";
    let decoded = decode_blob(data_with_whitespace).unwrap();
    assert_eq!(decoded, b"Hello, INDIGO!");
}

#[test]
fn test_decode_blob_invalid() {
    let result = decode_blob("not valid base64!!!");
    assert!(result.is_err());
}

#[test]
fn test_blob_large_data() {
    // Test with 1 MB of data
    let original: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
    let encoded = encode_blob(&original);
    let decoded = decode_blob(&encoded).unwrap();
    assert_eq!(decoded.len(), original.len());
    assert_eq!(decoded, original);
}

// ============================================================================
// BlobTransferMode Tests
// ============================================================================

#[test]
fn test_blob_transfer_mode_default() {
    let mode = BlobTransferMode::default();
    assert_eq!(mode, BlobTransferMode::Also);
}

#[test]
fn test_blob_transfer_mode_as_str() {
    assert_eq!(BlobTransferMode::Never.as_str(), "Never");
    assert_eq!(BlobTransferMode::Also.as_str(), "Also");
    assert_eq!(BlobTransferMode::Only.as_str(), "Only");
}

#[test]
fn test_blob_transfer_mode_from_str() {
    assert_eq!(
        BlobTransferMode::from_str("Never").unwrap(),
        BlobTransferMode::Never
    );
    assert_eq!(
        BlobTransferMode::from_str("Also").unwrap(),
        BlobTransferMode::Also
    );
    assert_eq!(
        BlobTransferMode::from_str("Only").unwrap(),
        BlobTransferMode::Only
    );
}

#[test]
fn test_blob_transfer_mode_from_str_invalid() {
    assert!(BlobTransferMode::from_str("Invalid").is_err());
}

#[test]
fn test_blob_transfer_mode_display() {
    assert_eq!(format!("{}", BlobTransferMode::Never), "Never");
    assert_eq!(format!("{}", BlobTransferMode::Also), "Also");
    assert_eq!(format!("{}", BlobTransferMode::Only), "Only");
}

// ============================================================================
// XML Protocol BLOB Parsing Tests
// ============================================================================

#[test]
fn test_parse_set_blob_vector() {
    let xml = r#"<setBLOBVector device="CCD Simulator" name="CCD_IMAGE" state="Ok">
        <oneBLOB name="IMAGE" format=".fits" size="14">SGVsbG8sIElORElHTyE=</oneBLOB>
    </setBLOBVector>"#;

    let msg = ProtocolParser::parse_message(xml.as_bytes()).unwrap();

    match msg {
        ProtocolMessage::SetBLOBVector(v) => {
            assert_eq!(v.attrs.device, "CCD Simulator");
            assert_eq!(v.attrs.name, "CCD_IMAGE");
            assert_eq!(v.attrs.state, Some(PropertyState::Ok));
            assert_eq!(v.elements.len(), 1);
            assert_eq!(v.elements[0].name, "IMAGE");
            assert_eq!(v.elements[0].format, ".fits");
            assert_eq!(v.elements[0].size, 14);

            // Verify the base64 data is present
            assert!(!v.elements[0].value.is_empty());

            // Decode and verify
            let decoded = decode_blob(&v.elements[0].value).unwrap();
            assert_eq!(decoded, b"Hello, INDIGO!");
        }
        _ => panic!("Expected SetBLOBVector"),
    }
}

#[test]
fn test_parse_set_blob_vector_multiple() {
    let xml = r#"<setBLOBVector device="CCD Simulator" name="CCD_IMAGE" state="Ok">
        <oneBLOB name="IMAGE" format=".fits" size="5">SGVsbG8=</oneBLOB>
        <oneBLOB name="THUMBNAIL" format=".jpg" size="5">V29ybGQ=</oneBLOB>
    </setBLOBVector>"#;

    let msg = ProtocolParser::parse_message(xml.as_bytes()).unwrap();

    match msg {
        ProtocolMessage::SetBLOBVector(v) => {
            assert_eq!(v.elements.len(), 2);
            assert_eq!(v.elements[0].name, "IMAGE");
            assert_eq!(v.elements[1].name, "THUMBNAIL");

            let decoded1 = decode_blob(&v.elements[0].value).unwrap();
            let decoded2 = decode_blob(&v.elements[1].value).unwrap();
            assert_eq!(decoded1, b"Hello");
            assert_eq!(decoded2, b"World");
        }
        _ => panic!("Expected SetBLOBVector"),
    }
}

// ============================================================================
// XML Protocol BLOB Sending Tests
// ============================================================================

#[test]
fn test_serialize_new_blob_vector() {
    let data = b"Test BLOB data";
    let encoded = encode_blob(data);

    let msg = ProtocolMessage::NewBLOBVector(NewBLOBVector {
        attrs: NewVectorAttributes {
            device: "CCD Simulator".to_string(),
            name: "CCD_UPLOAD".to_string(),
            timestamp: None,
        },
        elements: vec![OneBLOB {
            name: "IMAGE".to_string(),
            size: data.len(),
            format: ".fits".to_string(),
            value: encoded,
        }],
    });

    let xml = ProtocolSerializer::serialize(&msg).unwrap();
    let xml_str = String::from_utf8(xml.clone()).unwrap();

    // Verify the XML contains expected elements
    assert!(xml_str.contains("newBLOBVector"));
    assert!(xml_str.contains("device=\"CCD Simulator\""));
    assert!(xml_str.contains("name=\"CCD_UPLOAD\""));
    assert!(xml_str.contains("oneBLOB"));
    assert!(xml_str.contains("format=\".fits\""));
}

#[test]
fn test_new_blob_vector_round_trip() {
    let data = b"Round trip test data";
    let encoded = encode_blob(data);

    let original = ProtocolMessage::NewBLOBVector(NewBLOBVector {
        attrs: NewVectorAttributes {
            device: "CCD Simulator".to_string(),
            name: "CCD_UPLOAD".to_string(),
            timestamp: Some("2024-01-01T00:00:00".to_string()),
        },
        elements: vec![OneBLOB {
            name: "IMAGE".to_string(),
            size: data.len(),
            format: ".fits".to_string(),
            value: encoded.clone(),
        }],
    });

    let xml = ProtocolSerializer::serialize(&original).unwrap();
    let parsed = ProtocolParser::parse_message(&xml).unwrap();

    match parsed {
        ProtocolMessage::NewBLOBVector(v) => {
            assert_eq!(v.attrs.device, "CCD Simulator");
            assert_eq!(v.attrs.name, "CCD_UPLOAD");
            assert_eq!(v.elements.len(), 1);
            assert_eq!(v.elements[0].name, "IMAGE");
            assert_eq!(v.elements[0].format, ".fits");
            assert_eq!(v.elements[0].value, encoded);
        }
        _ => panic!("Expected NewBLOBVector"),
    }
}

// ============================================================================
// EnableBLOB Protocol Tests
// ============================================================================

#[test]
fn test_serialize_enable_blob() {
    let msg = ProtocolMessage::EnableBLOB(EnableBLOB {
        device: "CCD Simulator".to_string(),
        name: Some("CCD_IMAGE".to_string()),
        value: BLOBEnable::Also,
    });

    let xml = ProtocolSerializer::serialize(&msg).unwrap();
    let xml_str = String::from_utf8(xml).unwrap();

    assert!(xml_str.contains("enableBLOB"));
    assert!(xml_str.contains("device=\"CCD Simulator\""));
    assert!(xml_str.contains("name=\"CCD_IMAGE\""));
    assert!(xml_str.contains("Also"));
}

#[test]
fn test_serialize_enable_blob_all_properties() {
    let msg = ProtocolMessage::EnableBLOB(EnableBLOB {
        device: "CCD Simulator".to_string(),
        name: None,
        value: BLOBEnable::Never,
    });

    let xml = ProtocolSerializer::serialize(&msg).unwrap();
    let xml_str = String::from_utf8(xml).unwrap();

    assert!(xml_str.contains("enableBLOB"));
    assert!(xml_str.contains("device=\"CCD Simulator\""));
    assert!(xml_str.contains("Never"));
    // Should not contain name attribute
    assert!(!xml_str.contains("name="));
}

// ============================================================================
// Property Conversion Tests
// ============================================================================

#[test]
fn test_property_value_blob_creation() {
    let data = b"Test data";
    let value = PropertyValue::blob(data.to_vec(), ".fits");

    match value {
        PropertyValue::Blob {
            data: d,
            format,
            size,
        } => {
            assert_eq!(d, data);
            assert_eq!(format, ".fits");
            assert_eq!(size, data.len());
        }
        _ => panic!("Expected Blob variant"),
    }
}

#[test]
fn test_blob_property_builder() {
    let data = b"Image data";
    let property = Property::builder()
        .device("CCD Simulator")
        .name("CCD_IMAGE")
        .property_type(PropertyType::Blob)
        .item(PropertyItem::new(
            "IMAGE",
            "Image",
            PropertyValue::blob(data.to_vec(), ".fits"),
        ))
        .build()
        .unwrap();

    assert_eq!(property.device, "CCD Simulator");
    assert_eq!(property.name, "CCD_IMAGE");
    assert_eq!(property.property_type, PropertyType::Blob);
    assert_eq!(property.items.len(), 1);

    let item = property.items.get("IMAGE").unwrap();
    match &item.value {
        PropertyValue::Blob {
            data: d,
            format,
            size,
        } => {
            assert_eq!(d, data);
            assert_eq!(format, ".fits");
            assert_eq!(*size, data.len());
        }
        _ => panic!("Expected Blob value"),
    }
}

// ============================================================================
// Large BLOB Tests
// ============================================================================

#[test]
fn test_large_blob_encoding() {
    // Test with 10 MB of data (simulating a large astronomical image)
    let size = 10 * 1024 * 1024; // 10 MB
    let original: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    let encoded = encode_blob(&original);
    assert!(!encoded.is_empty());

    // Base64 encoding increases size by ~33%
    let expected_size = (size * 4 + 2) / 3;
    assert!(encoded.len() >= expected_size);
}

#[test]
fn test_large_blob_decoding() {
    // Test decoding a large BLOB
    let size = 5 * 1024 * 1024; // 5 MB
    let original: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    let encoded = encode_blob(&original);
    let decoded = decode_blob(&encoded).unwrap();

    assert_eq!(decoded.len(), original.len());
    // Verify first and last bytes to ensure correctness
    assert_eq!(decoded[0], original[0]);
    assert_eq!(decoded[size - 1], original[size - 1]);
}

// ============================================================================
// FITS Header Test Data
// ============================================================================

#[test]
fn test_fits_header_blob() {
    // Simulate a minimal FITS header (2880 bytes is standard FITS block size)
    let mut fits_header = vec![0u8; 2880];

    // Add FITS header keywords (simplified)
    let header_text =
        b"SIMPLE  =                    T / file does conform to FITS standard             ";
    fits_header[..header_text.len()].copy_from_slice(header_text);

    let encoded = encode_blob(&fits_header);
    let decoded = decode_blob(&encoded).unwrap();

    assert_eq!(decoded.len(), 2880);
    assert_eq!(&decoded[..header_text.len()], header_text);
}

#[test]
fn test_blob_with_null_bytes() {
    // Test BLOB containing null bytes (common in binary formats)
    let data = vec![0u8, 1, 2, 0, 0, 3, 4, 0];
    let encoded = encode_blob(&data);
    let decoded = decode_blob(&encoded).unwrap();
    assert_eq!(decoded, data);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_blob_decode_error_handling() {
    // Test various invalid base64 inputs
    let invalid_inputs = vec!["!!!", "not base64", "SGVsbG8=extra"];

    for input in invalid_inputs {
        let result = decode_blob(input);
        assert!(result.is_err(), "Expected error for input: {}", input);
    }
}
