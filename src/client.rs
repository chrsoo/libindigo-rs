use std::{
    collections::{HashMap, VecDeque}, fmt::{Debug, Display}, marker::PhantomData, time::SystemTime
};

use crate::{indigo::{self, *}, name, Interface};
use chrono::{DateTime, Utc};
use log::{debug, trace, warn};

type DeviceHook<P: Property> = fn(&ClientDeviceModel<P>, &ClientDeviceEvent) -> IndigoResult<()>;

/// Log entry for a named object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Name of the object.
    name: String,
    /// Log message for the event.
    message: String,
    /// Timestamp of the event.
    timestamp: DateTime<Utc>,
}

/// Event for a [ClientDeviceModel].
#[derive(Debug,Clone, PartialEq, Eq)]
pub enum ClientDeviceEvent {
    /// [Device] created event.
    Created(LogEntry),
    /// [Property] defined event.
    Defined(LogEntry),
    /// [Property] updated event.
    Updated(LogEntry),
    /// [Property] deleted event.
    Deleted(LogEntry),
    /// Message received event.
    Received(LogEntry),
}

impl ClientDeviceEvent {

    fn created(device: String) -> ClientDeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = "added device to model".to_string();
        ClientDeviceEvent::Created(LogEntry { name: device, timestamp, message })
    }

    fn received(device: String, msg: &str) -> ClientDeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = msg.to_owned();
        ClientDeviceEvent::Received(LogEntry { name: device, timestamp, message })
    }

    fn defined(property: String, msg: Option<&str>) -> ClientDeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = msg.unwrap_or("property defined").to_owned();
        ClientDeviceEvent::Defined(LogEntry { name: property, timestamp, message })
    }

    fn updated(property: String, msg: Option<&str>) -> ClientDeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = msg.unwrap_or("property updated").to_owned();
        ClientDeviceEvent::Updated(LogEntry { name: property, timestamp, message })
    }
    fn deleted(property: String, msg: Option<&str>) -> ClientDeviceEvent {
        let timestamp = SystemTime::now().into();
        let message = msg.unwrap_or("property deleted").to_owned();
        ClientDeviceEvent::Deleted(LogEntry { name: property, timestamp, message })
    }
}

// -- ClientDeviceModel --------------------------------------------------------

/// A device consisting of a collection of [Properties](Property) as seen from a
/// client implementation.
#[derive(Debug,Clone)]
pub struct ClientDeviceModel<P>
where P: Property {
    name: String,
    props: HashMap<String,P>,
    events: VecDeque<ClientDeviceEvent>,
    hook: Option<DeviceHook<P>>
}

impl<P: Property> ClientDeviceModel<P> {
    
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            props: HashMap::new(),
            events: VecDeque::new(),
            hook: None,
        }
    }

    /// Callback function invoked on each [DeviceEvent].
    pub fn device_hook(&mut self, hook: DeviceHook<P>) {
        self.hook = Some(hook);
        debug!("registered hook {hook:?}");
    }

    fn upsert_property(
        &mut self,
        p: P,
        e: ClientDeviceEvent,
    ) -> IndigoResult<()> {
        self.props
            .entry(p.name().to_string())
            .and_modify(|prop| prop.update(&p))
            .or_insert(p)
            ;
        self.dispatch_event(e)
    }

    fn delete_property(
        &mut self,
        p: impl Property,
        msg: Option<&str>
    ) -> IndigoResult<P> {

        if let Some(prop) = self.props.remove(p.name()) {
            self.dispatch_event(ClientDeviceEvent::deleted(p.name().to_owned(), msg))?;
            Ok(prop)
        } else {
            Err(IndigoError::new("Trying to delete an undefined property."))
        }
    }

    fn receive_message(&mut self, msg: &str) -> IndigoResult<()> {
        self.dispatch_event(ClientDeviceEvent::received(self.name().to_owned(), msg))
    }

    fn dispatch_event(&mut self, e: ClientDeviceEvent) -> IndigoResult<()>{
        self.events.push_front(e);

        let e = self.events.front().unwrap();

        if let Some(hook) = self.hook {
            trace!("dispatching {e:?}...");
            hook(self, e)?;
            debug!("dispatched {e:?}");
        } else {
            debug!("no hook defined for {e:?}");
        }
        Ok(())
    }

}

impl<P: Property> Display for ClientDeviceModel<P> {
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
impl<P: Property> NamedObject for ClientDeviceModel<P> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<P: Property> Device for ClientDeviceModel<P> {

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

/// A default implementation of [ClientDelegate] that manages the set of all enumerated devices
/// and their properties that are defined on the [Bus](crate::Bus) .
pub struct ClientModel<P,B,C>
where P: Property, B: Bus, C: ClientController<P,B> {
    name: String,
    devices: HashMap<String,ClientDeviceModel<P>>,
    hook: Option<DeviceHook<P>>,
    _b: PhantomData<B>,
    _c: PhantomData<C>,
}

impl<P,B,C> Debug for ClientModel<P,B,C>
where P: Property, B: Bus, C: ClientController<P,B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientModel")
            .field("name", &self.name)
            .field("devices", &self.devices)
            // .field("create_device_hook", &self.create_device_hook)
            // .field("_b", &self._b)
            // .field("_c", &self._c)
            .finish()
    }
}


impl<P,B,C> NamedObject for ClientModel<P,B,C>
where P: Property, B: Bus, C: ClientController<P,B> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<P,B,C> Delegate for ClientModel<P,B,C>
where P: Property, B: Bus, C: ClientController<P,B> {
    type Bus = B;
    type BusController = C;
}

impl<P,B,C> ClientDelegate for ClientModel<P,B,C>
where P: Property, B: Bus, C: ClientController<P,B> {
    type Property = P;
    type ClientController = C;

    /// Upsert a property while creating the device if it does not already exist.
    fn on_define_property(
        &mut self,
        _c: &mut C,
        _d: &str,
        p: P,
        msg: Option<&str>,
    ) -> IndigoResult<()> {

        let device = self.get_or_create_device(p.device());
        let name = p.name().to_owned();
        device.upsert_property(p, ClientDeviceEvent::defined(name, msg))
    }

    /// Upsert a property while creating the device if it does not already exist.
    fn on_update_property(
        &mut self,
        _c: &mut C,
        _d: &str,
        p: P,
        msg: Option<&str>

    ) -> IndigoResult<()> {

        let device = self.get_or_create_device(p.device());
        let name = p.name().to_owned();
        device.upsert_property(p, ClientDeviceEvent::updated(name, msg))
    }

    fn on_delete_property(
        &mut self,
        _c: &mut C,
        _d: &str,
        p: P,
        msg: Option<&str>
    ) -> IndigoResult<()> {

        if let Some(device) = self.devices.get_mut(p.device()) {
            device.delete_property(p, msg)?;
            Ok(())
        } else {
            Err(IndigoError::new("device not found"))
        }

    }

    fn on_message_broadcast(
        &mut self,
        _c: &mut C,
        d: &str,
        msg: &str
    ) -> IndigoResult<()> {

        let device = self.get_or_create_device(d);
        device.dispatch_event(ClientDeviceEvent::received(d.to_owned(), msg))
    }

}

impl<P,B,C> ClientModel<P,B,C>
where P: Property, B: Bus, C: ClientController<P,B> {

    pub fn new(name: &str) -> ClientModel<P,B,C> {
        ClientModel {
            name: name.to_owned(),
            devices: HashMap::new(),
            hook: None,
            _b: PhantomData,
            _c: PhantomData,
        }
    }

    pub fn devices(&mut self) -> impl Iterator<Item = &mut ClientDeviceModel<P>> {
        self.devices.values_mut()
    }

    /// Callback function invoked each time a new device is added to the model.
    pub fn create_hook(&mut self, hook: DeviceHook<P>) {
        self.hook = Some(hook);
        debug!("registered hook {hook:?}");
    }

    fn get_or_create_device(&mut self, d: &str) -> &mut ClientDeviceModel<P> {
        self.devices
            .entry(d.to_owned())
            .or_insert_with(|| {
                let device = ClientDeviceModel::new(d);
                debug!("added device '{d}' to model");
                if let Some(hook) = self.hook {
                    if let Err(e) = hook(&device, &ClientDeviceEvent::created(d.to_owned())) {
                        warn!("callback failed: {e}");
                    }
                }
                device
            })
    }
}
