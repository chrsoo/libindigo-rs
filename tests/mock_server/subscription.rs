//! Property subscription management for the mock INDIGO server.

use libindigo_rs::protocol::ProtocolMessage;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Manages property subscriptions for clients
pub struct SubscriptionManager {
    /// Subscriptions by connection ID
    subscriptions: HashMap<usize, ClientSubscription>,
}

/// A client's subscription preferences
#[derive(Debug)]
pub struct ClientSubscription {
    /// Connection ID
    pub connection_id: usize,

    /// Device filter (None = all devices)
    pub device_filter: Option<String>,

    /// Property filter (None = all properties)
    pub property_filter: Option<String>,

    /// Channel to send updates
    pub sender: mpsc::UnboundedSender<ProtocolMessage>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    /// Subscribe a client connection
    pub fn subscribe(&mut self, subscription: ClientSubscription) {
        self.subscriptions
            .insert(subscription.connection_id, subscription);
    }

    /// Unsubscribe a client connection
    pub fn unsubscribe(&mut self, connection_id: usize) {
        self.subscriptions.remove(&connection_id);
    }

    /// Notify all subscribers of a property update
    pub fn notify_property_update(&self, device: &str, property: &str, message: ProtocolMessage) {
        for subscription in self.get_subscribers(device, property) {
            // Non-blocking send (drops if channel full or closed)
            let _ = subscription.sender.send(message.clone());
        }
    }

    /// Get all subscribers interested in a specific property
    pub fn get_subscribers(&self, device: &str, property: &str) -> Vec<&ClientSubscription> {
        self.subscriptions
            .values()
            .filter(|sub| {
                // Check device filter
                if let Some(ref filter) = sub.device_filter {
                    if filter != device {
                        return false;
                    }
                }

                // Check property filter
                if let Some(ref filter) = sub.property_filter {
                    if filter != property {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get the number of active subscriptions
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libindigo_rs::protocol::{GetProperties, ProtocolMessage};

    #[test]
    fn test_subscription_manager() {
        let mut manager = SubscriptionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();

        let subscription = ClientSubscription {
            connection_id: 1,
            device_filter: None,
            property_filter: None,
            sender: tx,
        };

        manager.subscribe(subscription);
        assert_eq!(manager.subscription_count(), 1);

        manager.unsubscribe(1);
        assert_eq!(manager.subscription_count(), 0);
    }

    #[test]
    fn test_get_subscribers_with_filters() {
        let mut manager = SubscriptionManager::new();
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();
        let (tx3, _rx3) = mpsc::unbounded_channel();

        // Subscribe to all
        manager.subscribe(ClientSubscription {
            connection_id: 1,
            device_filter: None,
            property_filter: None,
            sender: tx1,
        });

        // Subscribe to specific device
        manager.subscribe(ClientSubscription {
            connection_id: 2,
            device_filter: Some("CCD Simulator".to_string()),
            property_filter: None,
            sender: tx2,
        });

        // Subscribe to specific device and property
        manager.subscribe(ClientSubscription {
            connection_id: 3,
            device_filter: Some("CCD Simulator".to_string()),
            property_filter: Some("CCD_TEMPERATURE".to_string()),
            sender: tx3,
        });

        // Test: CCD Simulator, CCD_TEMPERATURE should match all 3
        let subs = manager.get_subscribers("CCD Simulator", "CCD_TEMPERATURE");
        assert_eq!(subs.len(), 3);

        // Test: CCD Simulator, OTHER_PROP should match 1 and 2
        let subs = manager.get_subscribers("CCD Simulator", "OTHER_PROP");
        assert_eq!(subs.len(), 2);

        // Test: Other Device, PROP should match only 1
        let subs = manager.get_subscribers("Other Device", "PROP");
        assert_eq!(subs.len(), 1);
    }

    #[tokio::test]
    async fn test_notify_property_update() {
        let mut manager = SubscriptionManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel();

        manager.subscribe(ClientSubscription {
            connection_id: 1,
            device_filter: None,
            property_filter: None,
            sender: tx,
        });

        let message = ProtocolMessage::GetProperties(GetProperties {
            version: Some("512".to_string()),
            device: Some("Test".to_string()),
            name: None,
        });

        manager.notify_property_update("Test", "PROP", message.clone());

        // Should receive the message
        let received = rx.recv().await.unwrap();
        assert!(matches!(received, ProtocolMessage::GetProperties(_)));
    }
}
