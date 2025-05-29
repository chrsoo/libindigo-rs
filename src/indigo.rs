#![allow(dead_code, unused_variables)]
use std::ffi::{c_char, CString, NulError};
use std::fmt::Debug;
#[cfg(feature = "sys")]
use std::{fmt::Display, str::Utf8Error};

use enum_primitive::*;

use libindigo_sys::buf_to_str;
use log::{debug, error, warn};
use number::NumberFormat;
use strum_macros::Display;
use url_fork::{ParseError, Url};

use crate::property::{PropertyItem, PropertyValue, Switch};
use crate::{name, number, Interface};
use core::error::Error;
use core::result::Result;

// -- Constants ---------------------------------------------------------------

pub const INDIGO_DEFAULT_PORT: u16 = 7624;
pub const INDIGO_DEFAULT_HOST: &str = "localhost";
// TODO use _&str_ constants generated from libindigo_sys
pub const CONNECTION_PROPERTY_NAME: &str = "CONNECTION";
pub const CONNECTION_CONNECTED_ITEM_NAME: &str = "CONNECTED";
pub const CONNECTION_DISCONNECTED_ITEM_NAME: &str = "DISCONNECTED";

// -- Utility -----------------------------------------------------------------

#[derive(Debug)]
pub struct IndigoError {
    msg: String,
}

impl IndigoError {
    pub fn new(msg: &str) -> IndigoError {
        IndigoError {
            msg: msg.to_string(),
        }
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }
}

impl Error for IndigoError {
    fn description(&self) -> &str {
        &self.msg
    }
}

// #[test]
// fn test_error() {
//     let s = String::from("test");
//     let e = IndigoError::new(s.as_str());
//     println!("{e}")
// }

impl From<Utf8Error> for IndigoError {
    fn from(value: Utf8Error) -> Self {
        warn!("Could not convert from UTF8: {value}");
        IndigoError::new("could not convert UTF8 value")
    }
}

impl From<ParseError> for IndigoError {
    fn from(value: ParseError) -> Self {
        IndigoError::new(value.to_string().as_str())
    }
}

impl From<&str> for IndigoError {
    fn from(msg: &str) -> Self {
        Self {
            msg: msg.into(),
        }
    }
}

impl From<String> for IndigoError {
    fn from(msg: String) -> Self {
        Self {
            msg
        }
    }
}

// #[cfg(feature = "sys")]
// impl<const N: usize> From<[c_char; N]> for IndigoError {
//     fn from(value: [c_char; N]) -> Self {
//         let error = buf_to_str(&value);
//         IndigoError::new(error)
//     }
// }

#[cfg(feature = "sys")]
impl From<CString> for IndigoError {
    fn from(value: CString) -> Self {
        let error = value.to_str().unwrap_or_else(|e| {
            error!("{e}");
            "could not convert C-string"
        });
        IndigoError::new(error)
    }
}

impl From<NulError> for IndigoError {
    fn from(value: NulError) -> Self {
        IndigoError::new(value.to_string().as_str())
    }
}

impl Display for IndigoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

// TODO evaluate if sync and send are required for IndigoError
unsafe impl Sync for IndigoError {}
unsafe impl Send for IndigoError {}

pub type IndigoResult<T> = Result<T, IndigoError>;
pub type Callback<T> = dyn FnOnce(T) -> IndigoResult<()>;

pub trait NamedObject: Debug {
    /// Name used to identify the object.
    fn name(&self) -> &str;
    /// Label suitable for displaying in a human interface, defaults to the name of the object.
    fn label(&self) -> &str {
        self.name()
    }
}

/// Delegate receiving basic events for a [Bus].
pub trait Delegate: NamedObject {
    type Bus: Bus;
    type BusController: Controller<Self::Bus>;

    /// Called when the [AttachedObject] has been attached to a [Bus].
    fn on_attach(&mut self, controller: &mut Self::BusController) -> IndigoResult<()> {
        debug!("attached '{}'", self.name());
        Ok::<(), IndigoError>(())
    }

    /// Called when the [AttachedObject] has been [detached](AttachedObject::detach).
    fn on_detach(&mut self, controller: &mut Self::BusController) -> IndigoResult<()> {
        debug!("detached '{}'", self.name());
        Ok::<(), IndigoError>(())
    }
}

// impl Display for IndigoError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             IndigoError::Bus(result) => Display::fmt(result, f),
//             IndigoError::Sys(error) => Display::fmt(error, f),
//             IndigoError::Other(msg) => write!(f, "{msg}"),
//             IndigoError::Message(msg) => write!(f, "{msg}"),
//         }
//     }
// }

// impl Error for IndigoError { }

// impl From<NulError> for IndigoError {
//     fn from(e: NulError) -> Self {
//         IndigoError::Sys(Box::new(e))
//     }
// }

// impl From<FromBytesUntilNulError> for IndigoError {
//     fn from(e: FromBytesUntilNulError) -> Self {
//         IndigoError::Sys(Box::new(e))
//     }
// }

// impl From<Utf8Error> for IndigoError {
//     fn from(e: Utf8Error) -> Self {
//         IndigoError::Sys(Box::new(e))
//     }
// }

// impl From<std::io::Error> for IndigoError {
//     fn from(e: std::io::Error) -> Self {
//         IndigoError::Sys(Box::new(e))
//     }
// }
// impl<T: 'static> From<PoisonError<T>> for IndigoError {
//     fn from(value: PoisonError<T>) -> Self {
//         IndigoError::Sys(Box::new(value))
//     }
// }

// /// Types of request for [Client], [ServerConnection], or [Device].
// #[derive(strum_macros::Display)]
// enum IndigoRequest<'a, T> {
//     Connect(Box<&'a mut Callback<'a, T>>),
//     Disconnect(Box<&'a mut Callback<'a, T>>),
//     Attach(Box<&'a mut Callback<'a, T>>),
//     Detach(Box<Callback<'a, T>>),
// }

// impl<'a, T> IndigoRequest<'a, T> {
//     pub fn callback(&mut self, r: IndigoResult<T>) -> IndigoResult<()> {
//         match self {
//             IndigoRequest2::Connect(c) => c(r),
//             IndigoRequest2::Disconnect(c) => c(r),
//             IndigoRequest2::Attach(c) => c(r),
//             IndigoRequest2::Detach(c) => c(r),
//         }
//     }
// }

// -- Property ----------------------------------------------------------------

/// Defines [items](PropertyItem) holding the [values](PropertyValue) of the property for
/// an INDIGO [device](crate::Device).
pub trait Property: NamedObject {
    fn key(&self) -> PropertyKey<'_> {
        PropertyKey {
            dev: self.device(),
            name: self.name(),
        }
    }

    fn device(&self) -> &str;

    fn group(&self) -> &str;

    fn hints(&self) -> &str;

    fn state(&self) -> &PropertyState;

    fn property_type(&self) -> &PropertyType;

    fn perm(&self) -> &PropertyPermission;

    /// Switch behaviour rule (for switch properties).
    fn rule(&self) -> &SwitchRule;

    /// `true`if `Property` is hidden/unused by  driver (for optional properties).
    fn hidden(&self) -> bool;

    /// `true` if `Property` is defined.
    fn defined(&self) -> bool;

    fn items(&self) -> impl Iterator<Item = &PropertyItem>;

    fn get_item<'a>(&'a self, name: &str) -> Option<&'a PropertyItem> {
        self.items().filter(|i| i.name() == name).nth(0)
    }

    fn get_mut_item<'a>(&'a mut self, name: &str) -> Option<&'a PropertyItem> {
        self.items().filter(|i| i.name() == name).nth(0)
    }

    /// Update this [Property] with the values of the provided [Property].
    fn update(&mut self, p: &impl Property);

    /*
    #[doc = "< property version INDIGO_VERSION_NONE, INDIGO_VERSION_LEGACY or INDIGO_VERSION_2_0"]
    pub version: ::std::os::raw::c_short,
    */
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq, Display)]
    /// Possible states of a `Property`.
    pub enum PropertyState  {
        /// Property is passive (unused by INDIGO).
        Idle = 0,
        /// Property is in correct state or last operation was successful.
        Ok = 1,
        /// Property is a transient state or the outcome of an operation is pending.
        Busy = 2,
        /// Property is in incorrect state or the last operation failed.
        Alert = 3,
    }
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    /// Possible states of a `Property`.
    pub enum PropertyPermission  {
        ReadOnly = 1,
        ReadWrite = 2,
        WriteOnly = 3,
    }
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Display)]
    /// Possible property types.
    pub enum PropertyType  {
        /// Strings of limited width.
        Text = 1,
        /// Float numbers with defined min, max values and increment.
        Number = 2,
        /// Logical values representing “on” and “off” state.
        Switch = 3,
        /// Status values with four possible values Idle, Ok, Busy, and Alert.
        Light = 4,
        /// Binary data of any type and any length.
        Blob = 5,
    }
}

// pub trait PropertyItem:
//     NamedObject + TextItem + NumberItem + SwitchItem + LightItem + BlobItem + Display
// {
//     fn property_type(&self) -> PropertyType;
// }

// #[derive(PartialEq, Debug, Clone)]
// pub enum PropertyValue<'pv> {
//     Text(&'pv str),
//     Number {
//         /// < item format (for number properties)
//         format: NumberFormat,
//         /// < item min value (for number properties)
//         min: f64,
//         /// < item max value (for number properties)
//         max: f64,
//         /// < item increment value (for number properties)
//         step: f64,
//         /// < item value (for number properties)
//         value: f64,
//         /// < item target value (for number properties)
//         target: f64,
//     },
//     Switch(bool),
//     Light(PropertyState),
//     Blob {
//         /// < item format (for blob properties), known file type suffix like \".fits\" or \".jpeg\".
//         format: &'pv str,
//         /// < item size (for blob properties) in bytes
//         size: usize,
//         /// < item URL on source server
//         url: Option<Url>,
//         /// < item value (for blob properties)
//         data: Option<Vec<u8>>,
//     },
// }

/// An INDIGO property item identified by a name and displayed with a label.
pub trait TextItem {
    fn value(&self) -> &str;
}
pub trait LightItem {
    fn state(&self) -> PropertyState;
}
pub trait SwitchItem {
    fn on(&self) -> bool;
}
pub trait NumberItem {
    fn value(&self) -> f64;
    fn format(&self) -> &NumberFormat;
    #[cfg(feature = "sys")]
    fn formatted_number(&self) -> String {
        self.format().format(self.value())
    }
    fn min(&self) -> f64;
    fn max(&self) -> f64;
    fn step(&self) -> f64;
    fn target(&self) -> f64;
}
pub trait BlobItem {
    fn url(&self) -> Option<&Url>;
    fn data(&self) -> Option<&[u8]>;
    /// known file type suffix like \".fits\" or \".jpeg\".
    fn extension(&self) -> &str;
    /// item size in bytes
    fn size(&self) -> usize;
}

/*
impl<'a> Display for PropertyValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyValue::Text(s) => write!(f, "{s}"),
            PropertyValue::Number {
                format,
                min,
                max,
                step,
                value,
                target,
            } => {
                write!(f,
                    "format: '{format}'; min: {min}; max: {max}; step: {step};
                     value: {}; target: {target}", format.format(*value)
                )
            }
            PropertyValue::Switch(v) => write!(f, "{v}"),
            PropertyValue::Light(n) => write!(f, "{n}"),
            PropertyValue::Blob {
                format,
                url,
                size,
                value,
            } => {
                write!(
                    f,
                    "format: '{format}'; size: {size}; value: {}; url: '{:?}'",
                    value.is_some(),
                    url
                )
            }
        }
    }
}
 */

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
/// Possible property types.
pub enum SwitchRule  {
    /// Undefined button group.
    Undefined = 0,
    /// Radio button group behaviour with one switch in \"on\" state.
    OneOfMany = 1,
    /// Radio button group behaviour with none or one switch in \"on\" state.
    AtMostOne = 2,
    /// Checkbox button group behaviour.
    AnyOfMany = 3,
}
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct PropertyKey<'a> {
    pub dev: &'a str,
    pub name: &'a str,
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
/// Bus operation return status.
pub enum BlobMode {
    Also,
    Never,
    URL,
}
}

// -- Bus ----------------------------------------------------------------------

pub trait Bus: NamedObject {
    // fn remote() -> impl RemoteResource<Self>;
    // fn client<D: ClientDelegate>(d: D) -> impl ClientController<P,Self>;
    // fn device<D: DeviceDelegate>(d: D) -> impl DeviceController<P,Self>;

    /// Start a [Bus] with the given `name` and return an error if the [Bus] is
    /// already started.
    ///
    /// The beahviour of starting multiple buses with different names is
    /// implementation specific; either an error is returned indicating that
    /// only one bus instance can run at any given time, or multiple instances
    /// are allowed and no error is returned.
    fn start(name: &str) -> IndigoResult<impl Bus>;

    /// Stop the [Bus].
    fn stop(&mut self) -> IndigoResult<()>;
}

// -- Controller ---------------------------------------------------------------

pub trait Controller<B>: NamedObject
where
    B: Bus,
{
    /// Attach self to [Bus].
    fn attach(&mut self, bus: &mut B) -> IndigoResult<()>;

    /// Detach from [Bus] and consume self.
    fn detach(&mut self) -> IndigoResult<()>;
}

// // -- Server ------------------------------------------------------------------

// pub trait Server<S: ServerDelegate>: Bus<'static> { }

// pub trait ServerDelegate: AgentDelegate { }

// -- RemoteResource ----------------------------------------------------------

pub trait RemoteResource<B>: Controller<B> + Clone + Sized
where
    B: Bus,
{
    /// Disconnect from a remote server returning an error if not connected or
    /// in the process of connecting to a remote server.
    fn disconnect(&mut self) -> IndigoResult<()>;

    /// Reconnect to a remote server returning an error if not disconnected or
    /// in the process of disconnecting from a remote server.
    fn reconnect(&mut self) -> IndigoResult<()>;
}

// pub enum ResourceStatus<'a,B,C>: RemoteResource<B,C>
// where B: Bus<C>, C: Controller<B> {
//     /// Resolving the hostname before conencting to a remote server.
//     Resolving(&'a str),
//     /// Esatblishing the connection to the server.
//     Connecting(&'a str),
//     /// Connected to the server.
//     Connected(R),
//     /// Disconnecting from the server.
//     Disconnecting(&'a str),
//     /// Disconnected from the server.
//     Disconnected(&'a str),
//     /// Connection is defunct due to an error.
//     Error(Box<dyn Error>),
// }

// -- ClientController --------------------------------------------------------

/// Client that forwards [Property] events to a [ClientDelegate] and
/// provides methods for publishing [Property] requests to the [Bus].
pub trait ClientController<P, B>: Controller<B>
where
    P: Property,
    B: Bus,
{
    /// Request the definition of a named [Property].
    fn request_definition(&mut self, d: &str, p: &str) -> IndigoResult<()>;

    /// Request the update of the [Property] and all its [property items](PropertyItem).
    fn request_update(&mut self, d: &str, p: &P) -> IndigoResult<()>;

    /// Request the update of a single [PropertyItem].
    fn request_update_item(&mut self, d: &str, p: &P, i: &PropertyItem) -> IndigoResult<()>;
    fn request_delete(&mut self, d: &str, p: &P) -> IndigoResult<()>;
    fn request_delete_item(&mut self, d: &str, p: &P, i: &PropertyItem) -> IndigoResult<()>;

    fn request_connect(&mut self, d: &str) -> IndigoResult<()> {
        todo!("implement connection logic")
    }

    fn request_disconnect(&mut self, d: &str) -> IndigoResult<()> {
        todo!("implement disconnection logic")
    }

    // fn enable_blobs(&self) -> Result<Vec<Blob>, IndigoError>;

    /// Request definition of all [properties](Property), optionally limited to a specific [Device].
    fn request_enumeration(&mut self, d: Option<&str>) -> IndigoResult<usize>;

    // /// Do something with the model associated with this [Client].
    // fn manage<'a,F, R>(&mut self, f: Callback<&'a mut Self>) -> IndigoResult<()>;
}

// -- Device ------------------------------------------------------------------

/// A collection of [Properties](Property) related to a single [Device] with
/// default implementations of INDIGO conventions for the info property,
/// interfaces, and the connection status.
pub trait Device: NamedObject {
    fn get<'a>(&'a self, property: &'a str) -> Option<&'a impl Property>;

    fn get_mut(&mut self, property: &str) -> Option<&mut impl Property>;

    fn props(&self) -> impl Iterator<Item = &impl Property>;

    fn props_mut(&mut self) -> impl Iterator<Item = &mut impl Property>;

    /// Returns `Ok(true)` if the device is connected and `Ok(false)` if the
    /// [Device] is disconnnected.
    /// * If the [Device]'s [CONNECTION](CONNECTION_PROPERTY_NAME) property is
    ///   not in an [Ok](PropertyState::Ok) state, the corresponding
    ///   [PropertyState] is returned as an error.
    /// * If the [Device] does not have a
    ///   [CONNECTION](CONNECTION_PROPERTY_NAME), [PropertyState::Alert] is
    ///   returned as an error.
    /// * If the neither the [CONNECTED](CONNECTION_CONNECTED_ITEM_NAME) nor the
    ///   [DISCONNECTED](CONNECTION_DISCONNECTED_ITEM_NAME) [PropertyItem] is
    ///   defined for the property, [PropertyState::Alert] is returned as an
    ///   errror.
    ///
    /// ```text
    ///     use log::{info, warn};
    ///     match d.connected() {
    ///         Ok(true) => info!("Device {d:?} is CONNECTED."),
    ///         Ok(false) => info!("Device {d:?} is DISCONNECTED."),
    ///         Err(state) => warn!("Device's CONNECTION property is in the {state:?}"),
    ///     }
    /// ```
    fn connected(&self) -> Result<bool, &PropertyState> {
        if let Some(connection) = self.get(CONNECTION_PROPERTY_NAME) {
            if connection.state() != &PropertyState::Ok {
                return Err(connection.state());
            }
            for item in connection.items() {
                let name = item.name();
                if name == CONNECTION_CONNECTED_ITEM_NAME
                    || name == CONNECTION_DISCONNECTED_ITEM_NAME
                {
                    return match item.value() {
                        PropertyValue::Switch(sw) => {
                            Ok(if name == CONNECTION_CONNECTED_ITEM_NAME {
                                sw.on()
                            } else {
                                !sw.on()
                            })
                        }
                        _ => {
                            warn!(
                                "{}: expected switch value for item '{}', found '{:?}'",
                                connection.name(),
                                name,
                                item.value()
                            );
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
        warn!(
            "{}.{}: property not found",
            self.name(),
            CONNECTION_PROPERTY_NAME
        );
        Err(&PropertyState::Alert)
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
            warn!(
                "{} property not defined for device '{}'",
                name::INFO_PROPERTY,
                self.name()
            );
        }
        None
    }

    /// List all interfaces defined for this device, returning [None] if no
    /// interfaces can be found.
    fn list_interfaces(&self) -> Option<Vec<Interface>> {
        let p = self.info()?;
        if let Some(item) = p.get_item(name::INFO_DEVICE_INTERFACE_ITEM) {
            if let PropertyValue::Text(text) = item.value() {
                if let Ok(code) = text.value().parse() {
                    Interface::map(code)
                } else {
                    None
                }
            } else {
                warn!(
                    "DEVICE_INTERFACE item is not a text property value; found '{:?}'",
                    item.value()
                );
                None
            }
        } else {
            None
        }
    }
}

// -- DeviceController --------------------------------------------------------

/// [Controller] that forwards [Property] requests to a [DeviceDelegate] and
/// provides methods for publishing [Property] events to the [Bus].
pub trait DeviceController<P, B>: Controller<B>
where
    P: Property,
    B: Bus,
{
    /// Publish a [Property] definition event on the [Bus].
    fn define_property<'a>(&mut self, p: &'a P) -> IndigoResult<()>;
    /// Publish a [Property] update event on the [Bus].
    fn update_property<'a>(&mut self, p: &'a P) -> IndigoResult<()>;
    /// Publish a [Property] delete event on the [Bus].
    fn delete_property<'a>(&mut self, p: &'a P) -> IndigoResult<()>;
    /// Broadcast device message on the [Bus].
    fn broadcast_message(&mut self, msg: &str) -> IndigoResult<()>;
}

// -- ClientDelegate ----------------------------------------------------------

/// Receives [Property] events forwarded by a [ClientController] attached to the [Bus].
pub trait ClientDelegate: Delegate {
    type Property: Property;
    type ClientController: ClientController<Self::Property, Self::Bus>;

    /// Called each time the [Property] of a [Device] is defined.
    fn on_define_property<'a>(
        &'a mut self,
        c: &mut Self::ClientController,
        d: &'a str,
        p: Self::Property,
        msg: Option<&'a str>,
    ) -> IndigoResult<()> {
        debug!("'{}': '{}' property defined ", p.device(), p.name());
        Ok::<(), IndigoError>(())
    }

    /// Called each time a [Property] is updated for a device.
    fn on_update_property<'a>(
        &mut self,
        c: &mut Self::ClientController,
        d: &'a str,
        p: Self::Property,
        msg: Option<&'a str>,
    ) -> IndigoResult<()> {
        debug!("'{}': '{}' property updated ", p.device(), p.name());
        Ok::<(), IndigoError>(())
    }

    /// Called each time a [Property] of a [Device] is deleted.
    fn on_delete_property<'a>(
        &mut self,
        c: &mut Self::ClientController,
        d: &'a str,
        p: Self::Property,
        msg: Option<&'a str>,
    ) -> IndigoResult<()> {
        debug!("'{}': '{}' property deleted ", p.device(), p.name());
        Ok::<(), IndigoError>(())
    }

    /// Called each time a [Device] broadcasts a message.
    fn on_message_broadcast<'a>(
        &mut self,
        c: &mut Self::ClientController,
        d: &'a str,
        msg: &'a str,
    ) -> IndigoResult<()> {
        debug!("'{}': '{}'", d, msg);
        Ok::<(), IndigoError>(())
    }
}

// -- DeviceDelegate ----------------------------------------------------------

/// Receive [Property] requests forwarded by a [DeviceController] attached to the [Bus].
pub trait DeviceDelegate: Delegate {
    type Property: Property;
    type DeviceController: DeviceController<Self::Property, Self::Bus>;

    /// Called when a request to enumerate all properties is made, returning the [Result::Ok] on success.
    fn on_enumeration_request<'a>(
        &mut self,
        c: Self::DeviceController,
        p: &'a Self::Property,
    ) -> IndigoResult<()>;

    /// Called when a request to define a property is made, returning the [PropertyState].
    fn on_definition_request<'a>(&mut self, p: &'a Self::Property) -> IndigoResult<()>;

    /// Called when a request to update a property is made, returning [Result::Ok] on success.
    fn on_update_request<'a>(&mut self, p: &'a Self::Property) -> IndigoResult<()>;

    /// Called when a request to delete a property is made, returning [Result::Ok] on success.
    fn on_deletion_request<'a>(&mut self, p: &'a Self::Property) -> IndigoResult<()>;
}
