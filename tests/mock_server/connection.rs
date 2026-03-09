//! Per-client connection handler for the mock INDIGO server.

use super::handler::MessageHandler;
use super::server::ServerState;
use super::subscription::ClientSubscription;
use libindigo::error::{IndigoError, Result};
use libindigo_rs::protocol::ProtocolMessage;
use libindigo_rs::protocol_json::{JsonProtocolParser, JsonProtocolSerializer};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};

/// Represents a single client connection
pub struct Connection {
    /// Unique connection ID
    id: usize,

    /// TCP stream
    stream: TcpStream,

    /// Shared server state
    state: Arc<ServerState>,

    /// Shutdown signal receiver
    shutdown_rx: broadcast::Receiver<()>,

    /// Property update receiver
    update_rx: mpsc::UnboundedReceiver<ProtocolMessage>,

    /// Property update sender (for subscription)
    update_tx: mpsc::UnboundedSender<ProtocolMessage>,

    /// Connection-specific state
    negotiated: bool,
}

impl Connection {
    /// Create a new connection handler
    pub fn new(
        id: usize,
        stream: TcpStream,
        state: Arc<ServerState>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        let (update_tx, update_rx) = mpsc::unbounded_channel();

        Self {
            id,
            stream,
            state,
            shutdown_rx,
            update_rx,
            update_tx,
            negotiated: false,
        }
    }

    /// Main connection handling loop
    pub async fn handle(mut self) -> Result<()> {
        // Split stream for reading and writing
        let (reader, mut writer) = self.stream.split();
        let mut reader = BufReader::new(reader);

        // Create message handler
        let mut handler = MessageHandler::new(self.id, self.state.clone());

        // Subscribe this connection
        {
            let mut subscriptions = self.state.subscriptions.write().await;
            subscriptions.subscribe(ClientSubscription {
                connection_id: self.id,
                device_filter: None,
                property_filter: None,
                sender: self.update_tx.clone(),
            });
        }

        let mut line = String::new();

        loop {
            tokio::select! {
                // Read messages from client
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            // EOF - client disconnected
                            break;
                        }
                        Ok(_) => {
                            // Parse and handle message
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                match JsonProtocolParser::parse_message(trimmed) {
                                    Ok(message) => {
                                        // Update stats
                                        {
                                            let mut stats = self.state.stats.write().await;
                                            stats.messages_received += 1;
                                        }

                                        // Handle message
                                        match handler.handle(message).await {
                                            Ok(responses) => {
                                                // Send responses
                                                for response in responses {
                                                    if let Err(e) = Self::send_message(&mut writer, &response, &self.state).await {
                                                        eprintln!("Connection {} send error: {}", self.id, e);
                                                        break;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Connection {} handler error: {}", self.id, e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Connection {} parse error: {}", self.id, e);
                                    }
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            eprintln!("Connection {} read error: {}", self.id, e);
                            break;
                        }
                    }
                }

                // Send property updates to client
                Some(update) = self.update_rx.recv() => {
                    if let Err(e) = Self::send_message(&mut writer, &update, &self.state).await {
                        eprintln!("Connection {} update send error: {}", self.id, e);
                        break;
                    }
                }

                // Handle shutdown
                _ = self.shutdown_rx.recv() => {
                    break;
                }
            }
        }

        // Unsubscribe on disconnect
        {
            let mut subscriptions = self.state.subscriptions.write().await;
            subscriptions.unsubscribe(self.id);
        }

        Ok(())
    }

    /// Send a protocol message to the client
    async fn send_message(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        message: &ProtocolMessage,
        state: &Arc<ServerState>,
    ) -> Result<()> {
        let json = JsonProtocolSerializer::serialize(message)
            .map_err(|e| IndigoError::ProtocolError(format!("Serialization error: {}", e)))?;

        writer
            .write_all(json.as_bytes())
            .await
            .map_err(|e| IndigoError::ConnectionError(format!("Write error: {}", e)))?;

        writer
            .write_all(b"\n")
            .await
            .map_err(|e| IndigoError::ConnectionError(format!("Write error: {}", e)))?;

        writer
            .flush()
            .await
            .map_err(|e| IndigoError::ConnectionError(format!("Flush error: {}", e)))?;

        // Update stats
        {
            let mut stats = state.stats.write().await;
            stats.messages_sent += 1;
        }

        Ok(())
    }
}
