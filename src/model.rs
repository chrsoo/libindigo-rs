use std::{
    collections::{HashMap, VecDeque}, fmt::Display, time::SystemTime
};

use crate::{indigo::{self, *}, name, Interface};
use chrono::{DateTime, Utc};
use log::{debug, warn};
use core::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    message: String,
}

#[derive(Debug,Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    /// [Property] defined event.
    Defined(LogEntry),
    /// [Property] updated event.
    Updated(LogEntry),
    /// [Property] deleted event.
    Deleted(LogEntry),
    /// Message received event.
    Received(LogEntry),
}

impl DeviceEvent {
    fn defined(message: Option<&str>) -> DeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = message.unwrap_or("property defined").to_owned();
        DeviceEvent::Defined(LogEntry { timestamp, message })
    }

    fn updated(message: Option<&str>) -> DeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = message.unwrap_or("property defined").to_owned();
        DeviceEvent::Updated(LogEntry { timestamp, message })
    }
    fn deleted(message: Option<&str>) -> DeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = message.unwrap_or("property defined").to_owned();
        DeviceEvent::Deleted(LogEntry { timestamp, message })
    }
    fn received(message: &str) -> DeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = message.to_owned();
        DeviceEvent::Received(LogEntry { timestamp, message })
    }
}

// -- Device ------------------------------------------------------------------

/// A collection of [Properties](Property) related to a single [Device].
pub trait Device: NamedObject {
    fn get<'a>(&'a self, property: &'a str) -> Option<&'a impl Property>;

    fn get_mut(&mut self, property: &str) -> Option<&mut impl Property>;

    fn props(&self) -> impl Iterator<Item = &impl Property>;

    fn props_mut(&mut self) -> impl Iterator<Item = &mut impl Property>;

    /// Returns `Ok(true)` if the device is connected and `Ok(false)` if the [Device] is disconnnected.
    /// * If the [Device]'s [CONNECTION](CONNECTION_PROPERTY_NAME) property is not in an [Ok](PropertyState::Ok) state,
    /// the corresponding [PropertyState] is returned as an error.
    /// * If the [Device] does not have a [CONNECTION](CONNECTION_PROPERTY_NAME),
    /// [PropertyState::Alert] is returned as an error.
    /// * If the neither the [CONNECTED](CONNECTION_CONNECTED_ITEM_NAME) nor the
    /// [DISCONNECTED](CONNECTION_DISCONNECTED_ITEM_NAME) [PropertyItem] is defined for the property,
    /// [PropertyState::Alert] is returned as an errror.
    /// ```no_run
    /// match d.connected() {
    ///     Ok(true) => info!("Device {d:?} is CONNECTED."),
    ///     Ok(false) => info!("Device {d:?} is DISCONNECTED."),
    ///     Err(state) => warn!("Device's CONNECTION property is in the {state:?}"),
    /// }
    /// ```
    fn connected(&self) -> Result<bool,&PropertyState> {
        if let Some(connection) = self.get(CONNECTION_PROPERTY_NAME) {
            if connection.state() != &PropertyState::Ok {
                return Err(connection.state());
            }
            for item in connection.items() {
                let name = item.name();
                if name ==  CONNECTION_CONNECTED_ITEM_NAME || name == CONNECTION_DISCONNECTED_ITEM_NAME {
                    return match item.property_type() {
                        PropertyType::Switch => {
                            Ok(if name == CONNECTION_CONNECTED_ITEM_NAME { item.on() } else { !item.on() })
                        }
                        _ => {
                            warn!("{}: expected switch value for item '{}', found '{:?}'",
                                connection.name(), name, item.value());
                            Err(&PropertyState::Alert)
                        }
                    };
                }
            }
            warn!(
                "{}: could not find a '{}' or '{}' item",
                connection.name(),
                CONNECTION_CONNECTED_ITEM_NAME,
                CONNECTION_DISCONNECTED_ITEM_NAME,
            );
            return Err(&PropertyState::Alert);
        }
        warn!("{}.{}: property not found", self.name(), CONNECTION_PROPERTY_NAME);
        return Err(&PropertyState::Alert)
    }

    fn info(&self) -> Option<&impl Property> {
        if let Some(p) = self.get(name::INFO_PROPERTY) {
            if p.property_type() == &PropertyType::Text {
                return Some(p);
            } else {
                warn!(
                    "Expected {:?} but found {:?} for {} property on '{}' device",
                    PropertyType::Text,
                    p.property_type(),
                    name::INFO_PROPERTY,
                    self.name()
                );
            }
        } else {
            warn!("{} property not defined for device '{}'", name::INFO_PROPERTY, self.name());
        }
        None
    }

    /// List all interfaces defined for this device, returning [None] if no
    /// interfaces can be found.
    fn list_interfaces(&self) -> Option<Vec<Interface>> {
        let p = self.info()?;
        if let Some(item) = p.get_item(name::INFO_DEVICE_INTERFACE_ITEM) {
            if PropertyType::Text == item.property_type() {
                if let Ok(code) = item.text().parse() {
                    Interface::map(code)
                } else {
                    None
                }
            } else {
                warn!(
                    "DEVICE_INTERFACE item is not a text property value; found '{:?}'",
                    item.property_type()
                );
                None
            }
        } else {
            None
        }
    }
}

// -- DeviceModel --------------------------------------------------------------

/// A device consisting of a collection of [Properties](Property) as seen from a
/// client implementation.
#[derive(Debug,Clone)]
pub struct DeviceModel<Property> {
    name: String,
    props: HashMap<String,Property>,
    events: VecDeque<DeviceEvent>,
}

impl<'d,P: Property> DeviceModel<P> {

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            props: HashMap::new(),
            events: VecDeque::new(),
        }
    }

    pub fn upsert_property(
        &mut self,
        p: P,
        e: DeviceEvent
    ) -> IndigoResult<(),IndigoError> {

        match e {
            DeviceEvent::Updated(_) => (),
            DeviceEvent::Defined(_) => (),
            DeviceEvent::Deleted(_) => return Err(IndigoError::new("deleted event sent to upsert")),
            DeviceEvent::Received(_) => return Err(IndigoError::new("received event sent to upsert")),
        }

        self.props
        .entry(p.name().to_string())
        .and_modify(|prop| prop.update(&p))
        .or_insert(p);

        self.events.push_front(e);
        Ok::<(),IndigoError>(())
    }

    pub fn delete_property<'a>(
        &mut self,
        p: impl Property,
        msg: Option<&str>,
    ) -> IndigoResult<P,IndigoError> {

        if let Some(prop) = self.props.remove(p.name()) {
            self.events.push_front(DeviceEvent::deleted(msg));
            Ok(prop)
        } else {
            Err(IndigoError::new("Trying to delete an undefined property."))
        }
    }

    fn append_message(
        &mut self,
        msg: &str,) {

        self.events.push_front(DeviceEvent::received(msg));
    }

}

impl<'d,P: Property> Display for DeviceModel<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = self.connected().map_or_else(
            |e| format!("{:?}", e),
            |s| {
                if s {
                    "connected".to_string()
                } else {
                    "disconnected".to_string()
                }
            },
        );
        write!(f, "{} ({}) [", self.name(), status)?;
        let mut sep = "";
        if let Some(ifaces) = self.list_interfaces() {
            for item in ifaces {
                write!(f, "{sep}{item}")?;
                sep = ", ";
            }
        }
        write!(f, "]")?;

        Ok(())
    }
}
impl<'d,P: Property> NamedObject for DeviceModel<P> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<P: Property> Device for DeviceModel<P> {

    fn get(&self, property: &str) -> enum_primitive::Option<&impl Property> {
        self.props.get(property)
    }

    fn get_mut(&mut self, property: &str) -> enum_primitive::Option<&mut impl indigo::Property> {
        self.props.get_mut(property)
    }

    fn props(&self) -> impl std::iter::Iterator<Item = &impl indigo::Property> {
        self.props.values()
    }

    fn props_mut(&mut self) -> impl std::iter::Iterator<Item = &mut impl indigo::Property> {
        self.props.values_mut()
    }
}

/// A default implementation of [ClientCallbacks] that manages the set of all enumerated devices
/// and their properties that are defined on the [Bus](crate::Bus) .
pub struct ClientModel<Property> {
    // devices: HashMap<String, HashMap<String, Property>>,
    name: String,
    devices: HashMap<String,DeviceModel<Property>>,
    create_device_hook: Option<Box<dyn FnMut(&DeviceModel<Property>)>>,
}

impl<'m,P: Property> NamedObject for ClientModel<P> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'m,P: Property> AttachedObject for ClientModel<P> {
    /* TODO implement on_attach & on_detach */
}

impl<P: Property> ClientDelegate<P> for ClientModel<P> {

    /// Upsert a property while creating the device if it does not already exist.
    fn on_define_property<'a>(
        &'a mut self,
        d: &'a str,
        p: P,
        msg: Option<&'a str>,
    ) -> std::result::Result<(), impl Error> {

        let device = self.get_or_create_device(d);
        device.upsert_property(p, DeviceEvent::defined(msg))
    }

    /// Upsert a property while creating the device if it does not already exist.
    fn on_update_property<'a>(
        &mut self,
        d: &'a str,
        p: P,
        msg: Option<&'a str>

    ) -> std::result::Result<(), impl Error> {

        let device = self.get_or_create_device(d);
        device.upsert_property(p, DeviceEvent::updated(msg))
    }

    fn on_delete_property<'a>(
        &mut self,
        d: &'a str,
        p: P,
        msg: Option<&'a str>
    ) -> std::result::Result<(), impl Error> {

        if let Some(device) = self.devices.get_mut(d) {
            device.delete_property(p, msg)?;
            Ok(())
        } else {
            Err(IndigoError::new("device not found"))
        }
    }

    fn on_message_broadcast<'a>(
        &mut self,
        d: &'a str,
        msg: &'a str
    ) -> std::result::Result<(), impl Error> {

        debug!("'{}': '{}'", d, msg);
        if let Some(device) = self.devices.get_mut(d) {
            device.append_message(msg);
            Ok(())
        } else {
            Err(IndigoError::new("Device not found."))
        }
    }

}

impl<'m,P: Property> ClientModel<P> {
    pub fn new(name: &str) -> ClientModel<P> {
        ClientModel {
            name: name.to_owned(),
            devices: HashMap::new(),
            create_device_hook: None,
        }
    }

    pub fn devices(&mut self) -> impl Iterator<Item = &mut DeviceModel<P>> {
        self.devices.values_mut()
    }

    // client device hooks
    pub fn create_device_hook(&mut self, hook: impl Fn(&DeviceModel<P>) + 'static) {
        self.create_device_hook = Some(Box::new(hook));
    }

    fn get_or_create_device<'a>(&'a mut self, d: &str) -> &'a mut DeviceModel<P> {
        let device = self
            .devices
            .entry(d.to_owned())
            .or_insert_with(|| {
                let device = DeviceModel::new(d);
                if let Some(hook) = self.create_device_hook.as_deref_mut() {
                    hook(&device)
                }
                device
            });
        device
    }
}
