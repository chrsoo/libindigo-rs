//! TCP Transport Layer for INDIGO Protocol
//!
//! This module handles the TCP connection and data transmission for the INDIGO protocol.
//!
//! # Overview
//!
//! The transport layer provides:
//!
//! - **Connection Management**: Establish and maintain TCP connections to INDIGO servers
//! - **Message Framing**: Handle XML and JSON message boundaries in the TCP stream
//! - **Protocol Negotiation**: Support both XML and JSON protocols with auto-detection
//! - **Buffering**: Efficient buffering of incoming and outgoing data
//! - **Error Handling**: Handle network errors and connection failures
//!
//! # Connection Lifecycle
//!
//! 1. **Connect**: Establish TCP connection to server (host:port)
//! 2. **Protocol Negotiation**: Determine protocol (JSON-first with XML fallback)
//! 3. **Send/Receive**: Exchange protocol messages
//! 4. **Disconnect**: Gracefully close the connection
//!
//! # Message Framing
//!
//! INDIGO protocol sends messages sequentially over TCP without explicit delimiters.
//! Messages are framed by their structure:
//!
//! ## XML Messages
//! - Each message is a complete XML element (e.g., `<getProperties.../> ` or `<defTextVector>...</defTextVector>`)
//! - We track XML depth to detect message boundaries
//!
//! ## JSON Messages
//! - Each message is a complete JSON object (e.g., `{"getProperties": {...}}`)
//! - We track brace depth to detect message boundaries
//! - Handle escaped braces in strings

use crate::protocol::{ProtocolMessage, ProtocolParser, ProtocolSerializer};
use crate::protocol_json::{JsonProtocolParser, JsonProtocolSerializer};
use crate::protocol_negotiation::ProtocolType;
use libindigo::error::{IndigoError, Result};
use std::time::Duration as StdDuration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

/// Default INDIGO server port.
pub const DEFAULT_INDIGO_PORT: u16 = 7624;

/// Default connection timeout in seconds.
const DEFAULT_CONNECT_TIMEOUT: u64 = 10;

/// Default read timeout in seconds.
const DEFAULT_READ_TIMEOUT: u64 = 30;

/// Initial buffer size for reading.
const INITIAL_BUFFER_SIZE: usize = 8192;

/// Maximum buffer size to prevent unbounded growth.
const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10 MB

/// Connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    /// Not connected.
    Disconnected,
    /// Connected and ready.
    Connected,
}

/// TCP transport for INDIGO protocol.
///
/// Manages the TCP connection to an INDIGO server and provides
/// methods for sending and receiving protocol messages.
///
/// # Example
///
/// ```ignore
/// use libindigo::strategies::rs::transport::Transport;
///
/// // Connect to server
/// let mut transport = Transport::connect("localhost:7624").await?;
///
/// // Send a message
/// let msg = ProtocolMessage::GetProperties(GetProperties { ... });
/// transport.send_message(&msg).await?;
///
/// // Receive messages
/// while let Ok(msg) = transport.receive_message().await {
///     // Process message
/// }
///
/// // Disconnect
/// transport.disconnect().await?;
/// ```
#[derive(Debug)]
pub struct Transport {
    /// TCP stream.
    stream: Option<TcpStream>,
    /// Read buffer for accumulating partial messages.
    read_buffer: Vec<u8>,
    /// Connection state.
    state: ConnectionState,
    /// Connection timeout.
    connect_timeout: Duration,
    /// Read timeout.
    read_timeout: Duration,
    /// Active protocol type (JSON or XML).
    protocol: ProtocolType,
}

impl Transport {
    /// Creates a new transport instance (not yet connected).
    ///
    /// Use [`connect`](Self::connect) to establish a connection.
    pub fn new() -> Self {
        Transport {
            stream: None,
            read_buffer: Vec::with_capacity(INITIAL_BUFFER_SIZE),
            state: ConnectionState::Disconnected,
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            read_timeout: Duration::from_secs(DEFAULT_READ_TIMEOUT),
            protocol: ProtocolType::default(), // Default to JSON
        }
    }

    /// Connects to an INDIGO server.
    ///
    /// # Arguments
    ///
    /// * `url` - Server URL in format "host:port" or just "host" (uses default port 7624)
    ///
    /// # Returns
    ///
    /// A connected `Transport` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL format is invalid
    /// - Connection fails
    /// - Connection times out
    ///
    /// # Example
    ///
    /// ```ignore
    /// let transport = Transport::connect("localhost:7624").await?;
    /// let transport = Transport::connect("192.168.1.100").await?; // Uses default port
    /// ```
    pub async fn connect(url: &str) -> Result<Self> {
        let mut transport = Self::new();
        transport.connect_to(url).await?;
        Ok(transport)
    }

    /// Connects to an INDIGO server with custom timeouts.
    ///
    /// # Arguments
    ///
    /// * `url` - Server URL in format "host:port" or just "host"
    /// * `connect_timeout` - Connection timeout duration
    /// * `read_timeout` - Read timeout duration
    pub async fn connect_with_timeout(
        url: &str,
        connect_timeout: StdDuration,
        read_timeout: StdDuration,
    ) -> Result<Self> {
        let mut transport = Self::new();
        transport.connect_timeout = connect_timeout;
        transport.read_timeout = read_timeout;
        transport.connect_to(url).await?;
        Ok(transport)
    }

    /// Internal method to establish connection.
    async fn connect_to(&mut self, url: &str) -> Result<()> {
        if self.state == ConnectionState::Connected {
            return Err(IndigoError::InvalidState("Already connected".to_string()));
        }

        // Parse URL to extract host and port
        let (host, port) = Self::parse_url(url)?;
        let addr = format!("{}:{}", host, port);

        // Establish TCP connection with timeout
        let stream = timeout(self.connect_timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| {
                IndigoError::Timeout(format!(
                    "Connection to {} timed out after {:?}",
                    addr, self.connect_timeout
                ))
            })?
            .map_err(|e| {
                IndigoError::ConnectionError(format!("Failed to connect to {}: {}", addr, e))
            })?;

        self.stream = Some(stream);
        self.state = ConnectionState::Connected;
        self.read_buffer.clear();

        Ok(())
    }

    /// Parses a URL string to extract host and port.
    ///
    /// Supports formats:
    /// - "host:port" -> (host, port)
    /// - "host" -> (host, DEFAULT_INDIGO_PORT)
    fn parse_url(url: &str) -> Result<(String, u16)> {
        if url.is_empty() {
            return Err(IndigoError::InvalidParameter("Empty URL".to_string()));
        }

        if let Some(colon_pos) = url.rfind(':') {
            let host = &url[..colon_pos];
            let port_str = &url[colon_pos + 1..];

            if host.is_empty() {
                return Err(IndigoError::InvalidParameter(
                    "Empty host in URL".to_string(),
                ));
            }

            let port = port_str.parse::<u16>().map_err(|_| {
                IndigoError::InvalidParameter(format!("Invalid port: {}", port_str))
            })?;

            Ok((host.to_string(), port))
        } else {
            // No port specified, use default
            Ok((url.to_string(), DEFAULT_INDIGO_PORT))
        }
    }

    /// Disconnects from the INDIGO server.
    ///
    /// Flushes any pending writes and closes the TCP connection gracefully.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection is not established or if shutdown fails.
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.state != ConnectionState::Connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        if let Some(mut stream) = self.stream.take() {
            // Flush any pending writes (ignore errors if connection already closed)
            let _ = stream.flush().await;

            // Shutdown the connection (ignore errors if connection already closed)
            // This can fail with "Socket is not connected" if the remote end closed first
            let _ = stream.shutdown().await;
        }

        self.state = ConnectionState::Disconnected;
        self.read_buffer.clear();

        Ok(())
    }

    /// Checks if the transport is currently connected.
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    /// Gets the current protocol type.
    pub fn protocol(&self) -> ProtocolType {
        self.protocol
    }

    /// Sets the protocol type.
    ///
    /// This should be called after protocol negotiation to set the active protocol.
    ///
    /// # Arguments
    ///
    /// * `protocol` - The protocol type to use
    ///
    /// # Example
    ///
    /// ```ignore
    /// transport.set_protocol(ProtocolType::Json);
    /// ```
    pub fn set_protocol(&mut self, protocol: ProtocolType) {
        self.protocol = protocol;
    }

    /// Detects protocol type from data buffer.
    ///
    /// This is a convenience wrapper around [`ProtocolType::detect_from_data`].
    ///
    /// # Arguments
    ///
    /// * `data` - The data buffer to inspect
    ///
    /// # Returns
    ///
    /// - `Some(ProtocolType)` if protocol can be detected
    /// - `None` if protocol cannot be determined
    pub fn detect_protocol(data: &[u8]) -> Option<ProtocolType> {
        ProtocolType::detect_from_data(data)
    }

    /// Sends a protocol message over the TCP connection.
    ///
    /// The message is serialized to XML and written to the TCP stream.
    ///
    /// # Arguments
    ///
    /// * `message` - The protocol message to send
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not connected
    /// - Serialization fails
    /// - Write operation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let msg = ProtocolMessage::GetProperties(GetProperties {
    ///     version: Some("1.7".to_string()),
    ///     device: None,
    ///     name: None,
    /// });
    /// transport.send_message(&msg).await?;
    /// ```
    pub async fn send_message(&mut self, message: &ProtocolMessage) -> Result<()> {
        if self.state != ConnectionState::Connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // Serialize the message based on active protocol
        let message_bytes: Vec<u8> = match self.protocol {
            ProtocolType::Json => JsonProtocolSerializer::serialize(message)?.into_bytes(),
            ProtocolType::Xml => ProtocolSerializer::serialize(message)?,
        };

        // Get mutable reference to stream
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| IndigoError::InvalidState("Stream not available".to_string()))?;

        // Write the message bytes to the stream
        stream
            .write_all(&message_bytes)
            .await
            .map_err(|e| IndigoError::ConnectionError(format!("Failed to write message: {}", e)))?;

        // Flush to ensure data is sent
        stream
            .flush()
            .await
            .map_err(|e| IndigoError::ConnectionError(format!("Failed to flush stream: {}", e)))?;

        Ok(())
    }

    /// Receives a single protocol message from the TCP connection.
    ///
    /// This method reads from the TCP stream, accumulates data in a buffer,
    /// and parses complete XML messages. It handles partial messages and
    /// multiple messages in a single read.
    ///
    /// # Returns
    ///
    /// The next complete protocol message.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not connected
    /// - Read operation fails
    /// - Connection is closed by peer
    /// - XML parsing fails
    /// - Read times out
    ///
    /// # Example
    ///
    /// ```ignore
    /// while let Ok(msg) = transport.receive_message().await {
    ///     match msg {
    ///         ProtocolMessage::DefTextVector(v) => { /* handle */ },
    ///         _ => { /* handle other types */ }
    ///     }
    /// }
    /// ```
    pub async fn receive_message(&mut self) -> Result<ProtocolMessage> {
        if self.state != ConnectionState::Connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        loop {
            // Try to parse a complete message from the buffer
            if let Some(message) = self.try_parse_message()? {
                return Ok(message);
            }

            // Need more data - read from stream
            self.read_more_data().await?;
        }
    }

    /// Attempts to parse a complete XML message from the read buffer.
    ///
    /// Returns `Some(message)` if a complete message is found and parsed,
    /// `None` if more data is needed.
    fn try_parse_message(&mut self) -> Result<Option<ProtocolMessage>> {
        if self.read_buffer.is_empty() {
            return Ok(None);
        }

        // Auto-detect protocol from buffer if we see data
        if let Some(detected_protocol) = Self::detect_protocol(&self.read_buffer) {
            // If detected protocol differs from current, switch to it
            if detected_protocol != self.protocol {
                self.protocol = detected_protocol;
            }
        }

        // Find the end of the first complete message
        if let Some(end_pos) = self.find_message_boundary()? {
            // Extract the message bytes
            let message_bytes = self.read_buffer[..=end_pos].to_vec();

            // Remove the message from the buffer
            self.read_buffer.drain(..=end_pos);

            // Parse the message based on active protocol
            let message = match self.protocol {
                ProtocolType::Json => {
                    let json_str = std::str::from_utf8(&message_bytes)
                        .map_err(|e| IndigoError::ParseError(format!("Invalid UTF-8: {}", e)))?;
                    JsonProtocolParser::parse_message(json_str)?
                }
                ProtocolType::Xml => ProtocolParser::parse_message(&message_bytes)?,
            };

            return Ok(Some(message));
        }

        Ok(None)
    }

    /// Finds the boundary of the first complete message in the buffer.
    ///
    /// Returns the index of the last byte of the complete message, or `None`
    /// if no complete message is found.
    ///
    /// This uses protocol-specific depth-tracking algorithms:
    /// - For XML: Track XML tag depth
    /// - For JSON: Track brace depth
    fn find_message_boundary(&self) -> Result<Option<usize>> {
        match self.protocol {
            ProtocolType::Xml => self.find_xml_boundary(),
            ProtocolType::Json => self.find_json_boundary(),
        }
    }

    /// Finds the boundary of a complete XML message.
    fn find_xml_boundary(&self) -> Result<Option<usize>> {
        let mut depth = 0;
        let mut in_tag = false;
        let mut in_string = false;
        let mut is_closing_tag = false;
        let mut is_self_closing = false;
        let mut i = 0;

        while i < self.read_buffer.len() {
            let byte = self.read_buffer[i];

            match byte {
                b'"' if in_tag && !in_string => {
                    in_string = true;
                }
                b'"' if in_tag && in_string => {
                    in_string = false;
                }
                b'<' if !in_string => {
                    in_tag = true;
                    is_closing_tag = false;
                    is_self_closing = false;

                    // Check if this is a closing tag
                    if i + 1 < self.read_buffer.len() && self.read_buffer[i + 1] == b'/' {
                        is_closing_tag = true;
                    }
                }
                b'/' if in_tag && !in_string => {
                    // Check if this is a self-closing tag
                    if i + 1 < self.read_buffer.len() && self.read_buffer[i + 1] == b'>' {
                        is_self_closing = true;
                    }
                }
                b'>' if in_tag && !in_string => {
                    in_tag = false;

                    if is_closing_tag {
                        depth -= 1;
                    } else if !is_self_closing {
                        depth += 1;
                    }

                    // If we've closed all tags, we have a complete message
                    if depth == 0 && i > 0 {
                        return Ok(Some(i));
                    }
                }
                _ => {}
            }

            i += 1;
        }

        Ok(None)
    }

    /// Finds the boundary of a complete JSON message.
    ///
    /// JSON messages are complete JSON objects. We track brace depth
    /// to find the end of the object, handling escaped characters in strings.
    fn find_json_boundary(&self) -> Result<Option<usize>> {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut started = false;

        for (i, &byte) in self.read_buffer.iter().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match byte {
                b'\\' if in_string => {
                    escape_next = true;
                }
                b'"' if !in_string => {
                    in_string = true;
                }
                b'"' if in_string => {
                    in_string = false;
                }
                b'{' if !in_string => {
                    depth += 1;
                    started = true;
                }
                b'}' if !in_string => {
                    depth -= 1;
                    // If we've closed all braces, we have a complete message
                    if depth == 0 && started {
                        return Ok(Some(i));
                    }
                }
                _ => {}
            }
        }

        Ok(None)
    }

    /// Reads more data from the TCP stream into the read buffer.
    async fn read_more_data(&mut self) -> Result<()> {
        // Check buffer size to prevent unbounded growth
        if self.read_buffer.len() >= MAX_BUFFER_SIZE {
            return Err(IndigoError::ProtocolError(
                "Read buffer exceeded maximum size".to_string(),
            ));
        }

        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| IndigoError::InvalidState("Stream not available".to_string()))?;

        // Create a temporary buffer for reading
        let mut temp_buffer = vec![0u8; 4096];

        // Read with timeout
        let bytes_read = timeout(self.read_timeout, stream.read(&mut temp_buffer))
            .await
            .map_err(|_| {
                IndigoError::Timeout(format!("Read timed out after {:?}", self.read_timeout))
            })?
            .map_err(|e| {
                IndigoError::ConnectionError(format!("Failed to read from stream: {}", e))
            })?;

        if bytes_read == 0 {
            // Connection closed by peer
            self.state = ConnectionState::Disconnected;
            return Err(IndigoError::ConnectionError(
                "Connection closed by peer".to_string(),
            ));
        }

        // Append the read data to our buffer
        self.read_buffer
            .extend_from_slice(&temp_buffer[..bytes_read]);

        Ok(())
    }

    /// Returns a stream of incoming protocol messages.
    ///
    /// This is a convenience method that continuously receives messages
    /// until an error occurs or the connection is closed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use futures::StreamExt;
    ///
    /// let mut stream = transport.receive_stream();
    /// while let Some(result) = stream.next().await {
    ///     match result {
    ///         Ok(msg) => { /* handle message */ },
    ///         Err(e) => { /* handle error */ break; }
    ///     }
    /// }
    /// ```
    pub fn receive_stream(&mut self) -> MessageStream<'_> {
        MessageStream { transport: self }
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::new()
    }
}

/// A stream of incoming protocol messages.
///
/// Created by [`Transport::receive_stream`].
pub struct MessageStream<'a> {
    transport: &'a mut Transport,
}

impl<'a> MessageStream<'a> {
    /// Receives the next message from the stream.
    ///
    /// Returns `None` when the stream ends (connection closed or error).
    pub async fn next(&mut self) -> Option<Result<ProtocolMessage>> {
        match self.transport.receive_message().await {
            Ok(msg) => Some(Ok(msg)),
            Err(e) => {
                // Check if this is a connection closed error
                if matches!(e, IndigoError::ConnectionError(_)) {
                    None
                } else {
                    Some(Err(e))
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        // Test with port
        let (host, port) = Transport::parse_url("localhost:7624").unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 7624);

        // Test without port (should use default)
        let (host, port) = Transport::parse_url("192.168.1.100").unwrap();
        assert_eq!(host, "192.168.1.100");
        assert_eq!(port, DEFAULT_INDIGO_PORT);

        // Test with IPv6
        let (host, port) = Transport::parse_url("[::1]:7624").unwrap();
        assert_eq!(host, "[::1]");
        assert_eq!(port, 7624);

        // Test invalid port
        assert!(Transport::parse_url("localhost:invalid").is_err());

        // Test empty URL
        assert!(Transport::parse_url("").is_err());
    }

    #[test]
    fn test_find_message_boundary() {
        let mut transport = Transport::new();
        // Set protocol to XML for this test
        transport.protocol = ProtocolType::Xml;

        // Test simple self-closing tag
        // "<getProperties version=\"1.7\"/>" is 31 bytes, last char '>' is at index 29
        transport.read_buffer = b"<getProperties version=\"1.7\"/>".to_vec();
        let boundary = transport.find_message_boundary().unwrap();
        assert_eq!(boundary, Some(29)); // Index of last '>'

        // Test element with content
        // "<message device=\"CCD\">Test</message>" is 37 bytes, last '>' at index 35
        transport.read_buffer = b"<message device=\"CCD\">Test</message>".to_vec();
        let boundary = transport.find_message_boundary().unwrap();
        assert_eq!(boundary, Some(35)); // Index of last '>'

        // Test incomplete message
        transport.read_buffer = b"<defTextVector device=\"CCD\"".to_vec();
        let boundary = transport.find_message_boundary().unwrap();
        assert_eq!(boundary, None);

        // Test multiple messages
        // "<getProperties/>" is 16 bytes, last '>' at index 15
        transport.read_buffer = b"<getProperties/><message>Test</message>".to_vec();
        let boundary = transport.find_message_boundary().unwrap();
        assert_eq!(boundary, Some(15)); // Should find first message

        // Test nested elements
        // Full string is 56 bytes, last '>' at index 54
        transport.read_buffer = b"<defTextVector><defText>Value</defText></defTextVector>".to_vec();
        let boundary = transport.find_message_boundary().unwrap();
        assert_eq!(boundary, Some(54)); // Index of last '>'
    }

    #[test]
    fn test_connection_state() {
        let transport = Transport::new();
        assert_eq!(transport.state, ConnectionState::Disconnected);
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_send_message_not_connected() {
        use super::super::protocol::GetProperties;

        let mut transport = Transport::new();
        let msg = ProtocolMessage::GetProperties(GetProperties {
            version: Some("1.7".to_string()),
            device: None,
            name: None,
        });

        let result = transport.send_message(&msg).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IndigoError::InvalidState(_)));
    }

    #[tokio::test]
    async fn test_receive_message_not_connected() {
        let mut transport = Transport::new();
        let result = transport.receive_message().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IndigoError::InvalidState(_)));
    }

    #[tokio::test]
    async fn test_disconnect_not_connected() {
        let mut transport = Transport::new();
        let result = transport.disconnect().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IndigoError::InvalidState(_)));
    }
}
