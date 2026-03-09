//! Message handler for INDIGO protocol messages.

use super::device::DeviceRegistry;
use super::property::{MockProperty, PropertyType, PropertyUpdate, PropertyValue};
use super::server::ServerState;
use super::subscription::ClientSubscription;
use libindigo::error::{IndigoError, Result};
use libindigo_rs::protocol::{
    DefBLOB, DefBLOBVector, DefLight, DefLightVector, DefNumber, DefNumberVector, DefSwitch,
    DefSwitchVector, DefText, DefTextVector, EnableBLOB, GetProperties, NewNumberVector,
    NewSwitchVector, NewTextVector, NewVectorAttributes, OneBLOB, OneLight, OneNumber, OneSwitch,
    OneText, PropertyPerm, PropertyState, ProtocolMessage, SetBLOBVector, SetLightVector,
    SetNumberVector, SetSwitchVector, SetTextVector, SetVectorAttributes, SwitchRule, SwitchState,
    VectorAttributes,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handles INDIGO protocol messages
pub struct MessageHandler {
    /// Connection ID
    connection_id: usize,

    /// Shared server state
    state: Arc<ServerState>,
}

impl MessageHandler {
    /// Create a new message handler
    pub fn new(connection_id: usize, state: Arc<ServerState>) -> Self {
        Self {
            connection_id,
            state,
        }
    }

    /// Handle a protocol message and return response messages
    pub async fn handle(&mut self, message: ProtocolMessage) -> Result<Vec<ProtocolMessage>> {
        match message {
            ProtocolMessage::GetProperties(msg) => self.handle_get_properties(msg).await,
            ProtocolMessage::NewTextVector(msg) => self.handle_new_text_vector(msg).await,
            ProtocolMessage::NewNumberVector(msg) => self.handle_new_number_vector(msg).await,
            ProtocolMessage::NewSwitchVector(msg) => self.handle_new_switch_vector(msg).await,
            ProtocolMessage::EnableBLOB(msg) => self.handle_enable_blob(msg).await,
            _ => {
                // Unsupported message type - just ignore
                Ok(vec![])
            }
        }
    }

    /// Handle getProperties request
    async fn handle_get_properties(&mut self, msg: GetProperties) -> Result<Vec<ProtocolMessage>> {
        let devices = self.state.devices.read().await;
        let mut responses = Vec::new();

        // List properties based on filters
        let properties = devices.list_properties(msg.device.as_deref());

        for (device_name, property) in properties {
            // Apply property filter
            if let Some(ref name_filter) = msg.name {
                if &property.name != name_filter {
                    continue;
                }
            }

            // Convert property to def*Vector message
            if let Ok(def_msg) = property_to_def_message(property) {
                responses.push(def_msg);
            }
        }

        Ok(responses)
    }

    /// Handle newTextVector request
    async fn handle_new_text_vector(&mut self, msg: NewTextVector) -> Result<Vec<ProtocolMessage>> {
        let mut devices = self.state.devices.write().await;

        // Build property update
        let items: Vec<(String, PropertyValue)> = msg
            .elements
            .iter()
            .map(|e| (e.name.clone(), PropertyValue::Text(e.value.clone())))
            .collect();

        let update = PropertyUpdate {
            state: Some(PropertyState::Ok),
            items,
            message: None,
        };

        // Apply update
        devices.update_property(&msg.attrs.device, &msg.attrs.name, update)?;

        // Get updated property
        let property = devices
            .get_property(&msg.attrs.device, &msg.attrs.name)
            .ok_or_else(|| {
                IndigoError::ProtocolError(format!(
                    "Property '{}.{}' not found",
                    msg.attrs.device, msg.attrs.name
                ))
            })?
            .clone();

        drop(devices);

        // Notify subscribers
        let set_msg = property_to_set_message(&property)?;
        let subscriptions = self.state.subscriptions.read().await;
        subscriptions.notify_property_update(&property.device, &property.name, set_msg.clone());

        Ok(vec![set_msg])
    }

    /// Handle newNumberVector request
    async fn handle_new_number_vector(
        &mut self,
        msg: NewNumberVector,
    ) -> Result<Vec<ProtocolMessage>> {
        let mut devices = self.state.devices.write().await;

        // Get the property to extract format/min/max/step
        let property = devices
            .get_property(&msg.attrs.device, &msg.attrs.name)
            .ok_or_else(|| {
                IndigoError::ProtocolError(format!(
                    "Property '{}.{}' not found",
                    msg.attrs.device, msg.attrs.name
                ))
            })?;

        // Build property update with proper NumberValue
        let items: Vec<(String, PropertyValue)> = msg
            .elements
            .iter()
            .map(|e| {
                // Find the corresponding item in the property to get format/min/max/step
                let item = property.items.iter().find(|i| i.name == e.name);
                if let Some(item) = item {
                    if let PropertyValue::Number(ref num_val) = item.value {
                        return (
                            e.name.clone(),
                            PropertyValue::Number(super::property::NumberValue {
                                value: e.value,
                                format: num_val.format.clone(),
                                min: num_val.min,
                                max: num_val.max,
                                step: num_val.step,
                            }),
                        );
                    }
                }
                // Fallback if not found
                (
                    e.name.clone(),
                    PropertyValue::Number(super::property::NumberValue {
                        value: e.value,
                        format: "%.2f".to_string(),
                        min: 0.0,
                        max: 100.0,
                        step: 1.0,
                    }),
                )
            })
            .collect();

        let update = PropertyUpdate {
            state: Some(PropertyState::Ok),
            items,
            message: None,
        };

        // Apply update
        devices.update_property(&msg.attrs.device, &msg.attrs.name, update)?;

        // Get updated property
        let property = devices
            .get_property(&msg.attrs.device, &msg.attrs.name)
            .ok_or_else(|| {
                IndigoError::ProtocolError(format!(
                    "Property '{}.{}' not found",
                    msg.attrs.device, msg.attrs.name
                ))
            })?
            .clone();

        drop(devices);

        // Notify subscribers
        let set_msg = property_to_set_message(&property)?;
        let subscriptions = self.state.subscriptions.read().await;
        subscriptions.notify_property_update(&property.device, &property.name, set_msg.clone());

        Ok(vec![set_msg])
    }

    /// Handle newSwitchVector request
    async fn handle_new_switch_vector(
        &mut self,
        msg: NewSwitchVector,
    ) -> Result<Vec<ProtocolMessage>> {
        let mut devices = self.state.devices.write().await;

        // Build property update
        let items: Vec<(String, PropertyValue)> = msg
            .elements
            .iter()
            .map(|e| (e.name.clone(), PropertyValue::Switch(e.value.to_bool())))
            .collect();

        let update = PropertyUpdate {
            state: Some(PropertyState::Ok),
            items,
            message: None,
        };

        // Apply update
        devices.update_property(&msg.attrs.device, &msg.attrs.name, update)?;

        // Get updated property
        let property = devices
            .get_property(&msg.attrs.device, &msg.attrs.name)
            .ok_or_else(|| {
                IndigoError::ProtocolError(format!(
                    "Property '{}.{}' not found",
                    msg.attrs.device, msg.attrs.name
                ))
            })?
            .clone();

        drop(devices);

        // Notify subscribers
        let set_msg = property_to_set_message(&property)?;
        let subscriptions = self.state.subscriptions.read().await;
        subscriptions.notify_property_update(&property.device, &property.name, set_msg.clone());

        Ok(vec![set_msg])
    }

    /// Handle enableBLOB request
    async fn handle_enable_blob(&mut self, _msg: EnableBLOB) -> Result<Vec<ProtocolMessage>> {
        // For now, just acknowledge - we don't actually handle BLOBs in the mock server
        Ok(vec![])
    }
}

/// Convert a MockProperty to a def*Vector message
pub fn property_to_def_message(property: &MockProperty) -> Result<ProtocolMessage> {
    let attrs = VectorAttributes {
        device: property.device.clone(),
        name: property.name.clone(),
        label: property.label.clone(),
        group: property.group.clone(),
        state: property.state,
        timeout: property.timeout,
        timestamp: property.timestamp.clone(),
        message: property.message.clone(),
    };

    match property.property_type {
        PropertyType::Text => Ok(ProtocolMessage::DefTextVector(DefTextVector {
            attrs,
            perm: property.perm,
            elements: property
                .items
                .iter()
                .map(|item| DefText {
                    name: item.name.clone(),
                    label: item.label.clone(),
                    value: match &item.value {
                        PropertyValue::Text(s) => s.clone(),
                        _ => String::new(),
                    },
                })
                .collect(),
        })),
        PropertyType::Number => Ok(ProtocolMessage::DefNumberVector(DefNumberVector {
            attrs,
            perm: property.perm,
            elements: property
                .items
                .iter()
                .map(|item| {
                    if let PropertyValue::Number(ref num) = item.value {
                        DefNumber {
                            name: item.name.clone(),
                            label: item.label.clone(),
                            format: num.format.clone(),
                            min: num.min,
                            max: num.max,
                            step: num.step,
                            value: num.value,
                        }
                    } else {
                        DefNumber {
                            name: item.name.clone(),
                            label: item.label.clone(),
                            format: "%.2f".to_string(),
                            min: 0.0,
                            max: 100.0,
                            step: 1.0,
                            value: 0.0,
                        }
                    }
                })
                .collect(),
        })),
        PropertyType::Switch => {
            let rule = if let super::property::PropertyTypeMetadata::Switch { rule } =
                &property.type_metadata
            {
                *rule
            } else {
                SwitchRule::OneOfMany
            };

            Ok(ProtocolMessage::DefSwitchVector(DefSwitchVector {
                attrs,
                perm: property.perm,
                rule,
                elements: property
                    .items
                    .iter()
                    .map(|item| DefSwitch {
                        name: item.name.clone(),
                        label: item.label.clone(),
                        value: match &item.value {
                            PropertyValue::Switch(b) => SwitchState::from_bool(*b),
                            _ => SwitchState::Off,
                        },
                    })
                    .collect(),
            }))
        }
        PropertyType::Light => Ok(ProtocolMessage::DefLightVector(DefLightVector {
            attrs,
            elements: property
                .items
                .iter()
                .map(|item| DefLight {
                    name: item.name.clone(),
                    label: item.label.clone(),
                    value: match &item.value {
                        PropertyValue::Light(state) => *state,
                        _ => PropertyState::Idle,
                    },
                })
                .collect(),
        })),
        PropertyType::Blob => Ok(ProtocolMessage::DefBLOBVector(DefBLOBVector {
            attrs,
            perm: property.perm,
            elements: property
                .items
                .iter()
                .map(|item| DefBLOB {
                    name: item.name.clone(),
                    label: item.label.clone(),
                })
                .collect(),
        })),
    }
}

/// Convert a MockProperty to a set*Vector message
pub fn property_to_set_message(property: &MockProperty) -> Result<ProtocolMessage> {
    let attrs = SetVectorAttributes {
        device: property.device.clone(),
        name: property.name.clone(),
        state: Some(property.state),
        timeout: property.timeout,
        timestamp: property.timestamp.clone(),
        message: property.message.clone(),
    };

    match property.property_type {
        PropertyType::Text => Ok(ProtocolMessage::SetTextVector(SetTextVector {
            attrs,
            elements: property
                .items
                .iter()
                .map(|item| OneText {
                    name: item.name.clone(),
                    value: match &item.value {
                        PropertyValue::Text(s) => s.clone(),
                        _ => String::new(),
                    },
                })
                .collect(),
        })),
        PropertyType::Number => Ok(ProtocolMessage::SetNumberVector(SetNumberVector {
            attrs,
            elements: property
                .items
                .iter()
                .map(|item| OneNumber {
                    name: item.name.clone(),
                    value: match &item.value {
                        PropertyValue::Number(num) => num.value,
                        _ => 0.0,
                    },
                })
                .collect(),
        })),
        PropertyType::Switch => Ok(ProtocolMessage::SetSwitchVector(SetSwitchVector {
            attrs,
            elements: property
                .items
                .iter()
                .map(|item| OneSwitch {
                    name: item.name.clone(),
                    value: match &item.value {
                        PropertyValue::Switch(b) => SwitchState::from_bool(*b),
                        _ => SwitchState::Off,
                    },
                })
                .collect(),
        })),
        PropertyType::Light => Ok(ProtocolMessage::SetLightVector(SetLightVector {
            attrs,
            elements: property
                .items
                .iter()
                .map(|item| OneLight {
                    name: item.name.clone(),
                    value: match &item.value {
                        PropertyValue::Light(state) => *state,
                        _ => PropertyState::Idle,
                    },
                })
                .collect(),
        })),
        PropertyType::Blob => Ok(ProtocolMessage::SetBLOBVector(SetBLOBVector {
            attrs,
            elements: property
                .items
                .iter()
                .map(|item| OneBLOB {
                    name: item.name.clone(),
                    size: match &item.value {
                        PropertyValue::Blob(blob) => blob.size,
                        _ => 0,
                    },
                    format: match &item.value {
                        PropertyValue::Blob(blob) => blob.format.clone(),
                        _ => String::new(),
                    },
                    value: match &item.value {
                        PropertyValue::Blob(blob) => blob.url.clone(),
                        _ => String::new(),
                    },
                })
                .collect(),
        })),
    }
}
