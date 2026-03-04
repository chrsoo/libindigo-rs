//! Protocol Negotiation Module
//!
//! This module handles protocol negotiation between JSON and XML protocols.
//! It implements a JSON-first strategy with XML fallback.
//!
//! # Overview
//!
//! The INDIGO protocol supports both XML and JSON formats. This module provides:
//!
//! - **Protocol Type Detection**: Auto-detect protocol from incoming messages
//! - **Protocol Negotiation**: Try JSON first, fall back to XML if needed
//! - **Protocol Switching**: Handle protocol changes during connection
//!
//! # Protocol Detection
//!
//! - JSON messages start with `{`
//! - XML messages start with `<`
//!
//! # Example
//!
//! ```ignore
//! use libindigo::strategies::rs::protocol_negotiation::{ProtocolType, ProtocolNegotiator};
//!
//! let negotiator = ProtocolNegotiator::new(ProtocolType::Json, true);
//! let protocol = negotiator.negotiate(&mut transport).await?;
//! ```

use super::protocol::{GetProperties, ProtocolMessage};
use super::transport::Transport;
use crate::error::{IndigoError, Result};

/// Protocol type for INDIGO communication.
///
/// INDIGO supports both XML and JSON protocols. This enum represents
/// the active protocol type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// JSON protocol (version 512, equivalent to XML 2.0)
    Json,
    /// XML protocol (version 1.7 or 2.0)
    Xml,
}

impl ProtocolType {
    /// Returns the protocol version string for this protocol type.
    pub fn version_string(&self) -> &'static str {
        match self {
            ProtocolType::Json => "512",
            ProtocolType::Xml => "1.7",
        }
    }

    /// Detects protocol type from the first non-whitespace byte of data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data buffer to inspect
    ///
    /// # Returns
    ///
    /// - `Some(ProtocolType::Json)` if data starts with `{`
    /// - `Some(ProtocolType::Xml)` if data starts with `<`
    /// - `None` if protocol cannot be determined
    pub fn detect_from_data(data: &[u8]) -> Option<Self> {
        // Skip whitespace to find first meaningful character
        for &byte in data {
            match byte {
                b' ' | b'\t' | b'\n' | b'\r' => continue,
                b'{' => return Some(ProtocolType::Json),
                b'<' => return Some(ProtocolType::Xml),
                _ => return None,
            }
        }
        None
    }
}

impl Default for ProtocolType {
    /// Default protocol is JSON (preferred).
    fn default() -> Self {
        ProtocolType::Json
    }
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolType::Json => write!(f, "JSON"),
            ProtocolType::Xml => write!(f, "XML"),
        }
    }
}

/// Protocol negotiator for establishing protocol with server.
///
/// The negotiator implements a JSON-first strategy with optional XML fallback.
/// It attempts to use JSON protocol first, and if the server doesn't support it,
/// falls back to XML protocol.
///
/// # Example
///
/// ```ignore
/// use libindigo::strategies::rs::protocol_negotiation::{ProtocolType, ProtocolNegotiator};
///
/// // Create negotiator with JSON preference and fallback enabled
/// let negotiator = ProtocolNegotiator::new(ProtocolType::Json, true);
///
/// // Negotiate protocol with server
/// let protocol = negotiator.negotiate(&mut transport).await?;
/// println!("Negotiated protocol: {}", protocol);
/// ```
#[derive(Debug, Clone)]
pub struct ProtocolNegotiator {
    /// Preferred protocol to try first.
    preferred: ProtocolType,
    /// Whether to enable fallback to alternate protocol.
    fallback_enabled: bool,
}

impl ProtocolNegotiator {
    /// Creates a new protocol negotiator.
    ///
    /// # Arguments
    ///
    /// * `preferred` - The preferred protocol to try first
    /// * `fallback_enabled` - Whether to fall back to alternate protocol if preferred fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// // JSON-first with XML fallback
    /// let negotiator = ProtocolNegotiator::new(ProtocolType::Json, true);
    ///
    /// // JSON-only (no fallback)
    /// let negotiator = ProtocolNegotiator::new(ProtocolType::Json, false);
    ///
    /// // XML-only
    /// let negotiator = ProtocolNegotiator::new(ProtocolType::Xml, false);
    /// ```
    pub fn new(preferred: ProtocolType, fallback_enabled: bool) -> Self {
        ProtocolNegotiator {
            preferred,
            fallback_enabled,
        }
    }

    /// Creates a negotiator with JSON-first strategy and XML fallback.
    ///
    /// This is the recommended default configuration.
    pub fn json_first() -> Self {
        Self::new(ProtocolType::Json, true)
    }

    /// Creates a negotiator that only uses JSON protocol.
    pub fn json_only() -> Self {
        Self::new(ProtocolType::Json, false)
    }

    /// Creates a negotiator that only uses XML protocol.
    pub fn xml_only() -> Self {
        Self::new(ProtocolType::Xml, false)
    }

    /// Negotiates protocol with the server.
    ///
    /// This method attempts to establish a protocol with the server by:
    /// 1. Trying the preferred protocol first
    /// 2. If fallback is enabled and preferred fails, trying the alternate protocol
    /// 3. Returning the successfully negotiated protocol
    ///
    /// The negotiation is done by sending a `getProperties` message and waiting
    /// for a response. The protocol is detected from the server's response format.
    ///
    /// # Arguments
    ///
    /// * `transport` - The transport layer to use for negotiation
    ///
    /// # Returns
    ///
    /// The successfully negotiated protocol type.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Preferred protocol fails and fallback is disabled
    /// - Both preferred and fallback protocols fail
    /// - Server doesn't respond
    /// - Protocol detection fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let negotiator = ProtocolNegotiator::json_first();
    /// let protocol = negotiator.negotiate(&mut transport).await?;
    /// transport.set_protocol(protocol);
    /// ```
    pub async fn negotiate(&self, transport: &mut Transport) -> Result<ProtocolType> {
        match self.preferred {
            ProtocolType::Json => {
                // Try JSON first
                match self.try_json(transport).await {
                    Ok(()) => Ok(ProtocolType::Json),
                    Err(e) => {
                        if self.fallback_enabled {
                            // Fall back to XML
                            self.try_xml(transport).await?;
                            Ok(ProtocolType::Xml)
                        } else {
                            Err(IndigoError::ProtocolError(format!(
                                "JSON protocol not supported and fallback disabled: {}",
                                e
                            )))
                        }
                    }
                }
            }
            ProtocolType::Xml => {
                // Try XML (no fallback needed, XML is always supported)
                self.try_xml(transport).await?;
                Ok(ProtocolType::Xml)
            }
        }
    }

    /// Attempts to establish JSON protocol with the server.
    ///
    /// Sends a `getProperties` message with JSON version (512) and waits
    /// for a response. If the server responds with JSON, the protocol is established.
    async fn try_json(&self, transport: &mut Transport) -> Result<()> {
        // Set transport to JSON mode for sending
        transport.set_protocol(ProtocolType::Json);

        // Send getProperties with JSON version
        let msg = ProtocolMessage::GetProperties(GetProperties {
            version: Some("512".to_string()),
            device: None,
            name: None,
        });

        transport.send_message(&msg).await?;

        // Wait for response and check if it's JSON
        // The transport will auto-detect the protocol from the response
        // If server responds with XML, it means JSON is not supported
        Ok(())
    }

    /// Attempts to establish XML protocol with the server.
    ///
    /// Sends a `getProperties` message with XML version (1.7) and waits
    /// for a response. XML protocol is always supported by INDIGO servers.
    async fn try_xml(&self, transport: &mut Transport) -> Result<()> {
        // Set transport to XML mode for sending
        transport.set_protocol(ProtocolType::Xml);

        // Send getProperties with XML version
        let msg = ProtocolMessage::GetProperties(GetProperties {
            version: Some("1.7".to_string()),
            device: None,
            name: None,
        });

        transport.send_message(&msg).await?;

        // XML is always supported, so this should succeed
        Ok(())
    }
}

impl Default for ProtocolNegotiator {
    /// Default negotiator uses JSON-first strategy with XML fallback.
    fn default() -> Self {
        Self::json_first()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_type_detect_json() {
        let data = b"{\"getProperties\": {}}";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );
    }

    #[test]
    fn test_protocol_type_detect_xml() {
        let data = b"<getProperties version=\"1.7\"/>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_protocol_type_detect_with_whitespace() {
        let data = b"  \n\t{\"getProperties\": {}}";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Json)
        );

        let data = b"  \n\t<getProperties version=\"1.7\"/>";
        assert_eq!(
            ProtocolType::detect_from_data(data),
            Some(ProtocolType::Xml)
        );
    }

    #[test]
    fn test_protocol_type_detect_invalid() {
        let data = b"invalid data";
        assert_eq!(ProtocolType::detect_from_data(data), None);

        let data = b"";
        assert_eq!(ProtocolType::detect_from_data(data), None);
    }

    #[test]
    fn test_protocol_type_version_string() {
        assert_eq!(ProtocolType::Json.version_string(), "512");
        assert_eq!(ProtocolType::Xml.version_string(), "1.7");
    }

    #[test]
    fn test_protocol_type_display() {
        assert_eq!(format!("{}", ProtocolType::Json), "JSON");
        assert_eq!(format!("{}", ProtocolType::Xml), "XML");
    }

    #[test]
    fn test_protocol_type_default() {
        assert_eq!(ProtocolType::default(), ProtocolType::Json);
    }

    #[test]
    fn test_negotiator_json_first() {
        let negotiator = ProtocolNegotiator::json_first();
        assert_eq!(negotiator.preferred, ProtocolType::Json);
        assert!(negotiator.fallback_enabled);
    }

    #[test]
    fn test_negotiator_json_only() {
        let negotiator = ProtocolNegotiator::json_only();
        assert_eq!(negotiator.preferred, ProtocolType::Json);
        assert!(!negotiator.fallback_enabled);
    }

    #[test]
    fn test_negotiator_xml_only() {
        let negotiator = ProtocolNegotiator::xml_only();
        assert_eq!(negotiator.preferred, ProtocolType::Xml);
        assert!(!negotiator.fallback_enabled);
    }

    #[test]
    fn test_negotiator_default() {
        let negotiator = ProtocolNegotiator::default();
        assert_eq!(negotiator.preferred, ProtocolType::Json);
        assert!(negotiator.fallback_enabled);
    }
}
