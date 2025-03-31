#![allow(dead_code, unused_variables)]
#[cfg(feature = "sys")]
use std::{
    fmt::Display,
    str::{self, Utf8Error},
    usize,
};

use enum_primitive::*;
use libindigo_sys::{indigo_property_type_INDIGO_BLOB_VECTOR, indigo_property_type_INDIGO_LIGHT_VECTOR, indigo_property_type_INDIGO_NUMBER_VECTOR, indigo_property_type_INDIGO_SWITCH_VECTOR, indigo_property_type_INDIGO_TEXT_VECTOR};
use log::{debug, warn};
use number::NumberFormat;
use url_fork::Url;

use crate::number;
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
#[deprecated]
pub struct IndigoError<'s> {
    msg: &'s str,
}

impl<'s> IndigoError<'s> {

    pub fn new(msg: &'s str) -> IndigoError<'s> {
        IndigoError { msg }
    }

    pub fn msg(&self) -> &'s str {
        self.msg
    }
}

impl<'s> Error for IndigoError<'s> {
    fn description(&self) -> &str {
        self.msg
    }
}

#[test]
fn test_error() {
    let s = String::from("test");
    let e = IndigoError::new(s.as_str());
    println!("{e}")
}

#[cfg(feature = "sys")]
impl From<Utf8Error> for IndigoError<'_> {
    fn from(value: Utf8Error) -> Self {
        warn!("Could not convert from UTF8: {value}");
        IndigoError::new("could not convert UTF8 value")
    }
}

#[cfg(feature = "sys")]
impl<'a> Display for IndigoError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg)
    }
}

// TODO evaluate if sync and send are required for IndigoError
unsafe impl<'a> Sync for IndigoError<'a> {}
unsafe impl<'a> Send for IndigoError<'a> {}

pub type IndigoResult<T,E: Error> = Result<T,E>;
pub type Callback<T,E: Error> = dyn FnOnce(T) -> IndigoResult<(),E>;

pub trait NamedObject {
    /// Name used to identify the object.
    fn name(&self) -> &str;
    /// Label suitable for displaying in a human interface, defaults to the name of the object.
    fn label(&self) -> &str {
        self.name()
    }
}

pub trait BusController: AttachedObject {
    /// Detach from [Bus] and consume self.
    fn detach(self) -> IndigoResult<(),impl Error>;
}

/// Manage objects attached to the [Bus].
pub trait AttachedObject: NamedObject {
    /// Called when the [AttachedObject] has been attached to a [Bus].
    fn on_attach(&mut self) -> IndigoResult< (),impl Error> {
        debug!("attached '{}'", self.name());
        Ok::<(), IndigoError>(())
    }

    /// Called when the [AttachedObject] has been [detached](AttachedObject::detach).
    fn on_detach(&mut self) -> IndigoResult<(),impl Error> {
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
    type Item: PropertyItem;

    fn key<'a>(&'a self) -> PropertyKey<'a> {
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

    fn items<'a>(&'a self) -> impl Iterator<Item = &'a Self::Item>;

    fn get_item<'a>(&'a self, name: &str) -> Option<&'a Self::Item> {
        self.items().filter(|i| i.name() == name).nth(0)
    }

    fn get_mut_item<'a>(&'a mut self, name: &str) -> Option<&'a Self::Item> {
        self.items().filter(|i| i.name() == name).nth(0)
    }

    /// Update this [Property] with the values of the provided [Property].
    fn update<'a>(&mut self, p: &'a Self);

    /*
    #[doc = "< property version INDIGO_VERSION_NONE, INDIGO_VERSION_LEGACY or INDIGO_VERSION_2_0"]
    pub version: ::std::os::raw::c_short,
    */
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    /// Possible states of a `Property`.
    pub enum PropertyState  {
        /// Property is passive (unused by INDIGO).
        Idle,
        /// Property is in correct state or last operation was successful.
        Ok,
        /// Property is a transient state or the outcome of an operation is pending.
        Busy,
        /// Property is in incorrect state or the last operation failed.
        Alert,
    }
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    /// Possible states of a `Property`.
    pub enum PropertyPermission  {
        ReadOnly,
        ReadWrite,
        WriteOnly,
    }
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[repr(u32)]
    /// Possible property types.
    pub enum PropertyType  {
        /// Strings of limited width.
        Text = indigo_property_type_INDIGO_TEXT_VECTOR,
        /// Float numbers with defined min, max values and increment.
        Number = indigo_property_type_INDIGO_NUMBER_VECTOR,
        /// Logical values representing “on” and “off” state.
        Switch = indigo_property_type_INDIGO_SWITCH_VECTOR,
        /// Status values with four possible values Idle, Ok, Busy, and Alert.
        Light = indigo_property_type_INDIGO_LIGHT_VECTOR,
        /// Binary data of any type and any length.
        Blob = indigo_property_type_INDIGO_BLOB_VECTOR,
    }
}

pub trait PropertyItem:
    NamedObject + TextItem + NumberItem + SwitchItem + LightItem + BlobItem
{
    fn property_type(&self) -> PropertyType;
}

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
    fn text(&self) -> &str;
}
pub trait LightItem {
    fn state(&self) -> PropertyState;
}
pub trait SwitchItem {
    fn on(&self) -> bool;
}
pub trait NumberItem {
    fn value(&self) -> f64;
    fn format(&self) -> NumberFormat;
    fn min(&self) -> f64;
    fn max(&self) -> f64;
    fn step(&self) -> f64;
    fn target(&self) -> f64;
}
pub trait BlobItem {
    fn url(&self) -> Option<&Url>;
    fn data(&self) -> Option<&[u8]>;
    /// known file type suffix like \".fits\" or \".jpeg\".
    fn format(&self) -> &str;
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
    /// Radio button group like behaviour with one switch in \"on\" state.
    OneOfMany,
    /// Radio button group like behaviour with none or one switch in \"on\" state.
    AtMostOne,
    /// Checkbox button group like behaviour.
    AnyOfMany,
}
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
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

// -- Bus ---------------------------------------------------------------------

pub trait Bus<P,C,D>: NamedObject
where P: Property, C: ClientDelegate<P>, D: DeviceDelegate<P> {
    /// Start the [Bus] and return an error if the [Bus] is already started.
    fn start(name: &str) -> IndigoResult<impl Bus<P,C,D>,impl Error>;

    /// Stop the [Bus].
    fn stop(&mut self) -> IndigoResult<(),impl Error>;

    /// Create a [ClientController], attached to the [Bus] that forwards all
    /// property events to a [ClientDelegate]
    fn attach_client(
        &mut self,
        client: C,
    ) -> IndigoResult<impl ClientController<P,C>,impl Error>;

    // /// Create a [DeviceController] attached to the [Bus] that forwards all
    // /// property requests to a [DeviceDelegate].
    // fn attach_device(
    //     &mut self,
    //     device: D,
    // ) -> IndigoResult<impl DeviceController<P,D>, impl Error>;

    // /// Create an [Agent] managing an [AgentObject] attached to the [Bus].
    // fn attach_agent<A: AgentObject>(&'o mut self, agent: A) -> IndigoResult<impl Agent<A>>;

    /// Attach and connect a [RemoteResource] returning an error if the URL is
    /// invalid or not recognised.
    fn attach_remote(&mut self, name: &str, remote: &Url) -> IndigoResult<impl RemoteResource,impl Error>;
}

// // -- Agent -------------------------------------------------------------------

// pub trait Agent<A: AgentDelegate> {
//     fn client(&mut self) -> impl ClientController<A>;
//     fn device(&mut self) -> impl DeviceController<A>;
// }

// pub trait AgentDelegate: ClientDelegate + DeviceDelegate { }

// // -- Server ------------------------------------------------------------------

// pub trait Server<S: ServerDelegate>: Bus<'static> { }

// pub trait ServerDelegate: AgentDelegate { }

// -- RemoteResource ----------------------------------------------------------

pub trait RemoteResource: BusController {
    /// Disconnect from a remote server returning an error if not connected or
    /// in the process of connecting to a remote server.
    fn disconnect(&mut self) -> IndigoResult<(),impl Error>;

    /// Reconnect to a remote server returning an error if not disconnected or
    /// in the process of disconnecting from a remote server.
    fn reconnect(&mut self) -> IndigoResult<(),impl Error>;
}

pub enum ResourceStatus<'a, R: RemoteResource> {
    /// Resolving the hostname before conencting to a remote server.
    Resolving(&'a str),
    /// Esatblishing the connection to the server.
    Connecting(&'a str),
    /// Connected to the server.
    Connected(R),
    /// Disconnecting from the server.
    Disconnecting(&'a str),
    /// Disconnected from the server.
    Disconnected(&'a str),
    /// Connection is defunct due to an error.
    Error(Box<dyn Error>),
}

// -- ClientController --------------------------------------------------------

/// [BusController] that forwards [Property] events to a [ClientDelegate] and
/// provides methods for publishing [Property] requests to the [Bus].
pub trait ClientController<P: Property, C: ClientDelegate<P>>: BusController {
    fn request_definition<'a>(
        &mut self,
        d: &'a str,
        p: &'a P
    ) -> IndigoResult<(),impl Error>;

    /// Request the update of the [Property] and all its [property items](PropertyItem).
    fn request_update<'a>(
        &mut self,
        d: &'a str,
        p: &'a P
    ) -> IndigoResult<(),impl Error>;

    /// Request the update of a single [PropertyItem].
    fn request_update_item<'a>(
        &mut self,
        d: &'a str,
        p: &'a P,
        i: &'a impl PropertyItem,
    ) -> IndigoResult<(),impl Error>;
    fn request_delete<'a>(&mut self, d: &'a str, p: &'a P) -> IndigoResult<(),impl Error>;
    fn request_delete_item<'a>(
        &mut self,
        d: &'a str,
        p: &'a P,
        i: &'a impl PropertyItem,
    ) -> IndigoResult<(),impl Error>;

    // fn enable_blobs(&self) -> Result<Vec<Blob>, IndigoError>;

    /// Request definition of all [properties](Property), optionally limited to a specific [Device].
    fn request_enumeration(&mut self, d: Option<&str>) -> IndigoResult<usize,impl Error>;

    // /// Do something with the model associated with this [Client].
    // fn manage<'a,F, R>(&mut self, f: Callback<&'a mut Self>) -> IndigoResult<()>;
}

// -- DeviceController --------------------------------------------------------

/// [BusController] that forwards [Property] requests to a [DeviceDelegate] and
/// provides methods for publishing [Property] events to the [Bus].
pub trait DeviceController<'d, P: Property, D: DeviceDelegate<P>>: BusController {
    /// Publish a [Property] definition event on the [Bus].
    fn define_property<'a>(&mut self, p: &'a P) -> IndigoResult<(),impl Error>;
    /// Publish a [Property] update event on the [Bus].
    fn update_property<'a>(&mut self, p: &'a P) -> IndigoResult<(),impl Error>;
    /// Publish a [Property] delete event on the [Bus].
    fn delete_property<'a>(&mut self, p: &'a P) -> IndigoResult<(),impl Error>;
    /// Broadcast device message on the [Bus].
    fn broadcast_message(&mut self, msg: &str) -> IndigoResult<(),impl Error>;
}

// -- ClientDelegate ----------------------------------------------------------

/// Receive [Property] events forwarded by a [ClientController] attached to the [Bus].
pub trait ClientDelegate<P: Property>: AttachedObject {
    /// Called each time the [Property] of a [Device] is defined.
    fn on_define_property<'a>(
        &'a mut self,
        d: &'a str,
        p: P,
        msg: Option<&'a str>,
    ) -> IndigoResult<(),impl Error> {
        debug!("'{}': '{}' property defined ", p.device(), p.name());
        Ok::<(),IndigoError>(())
    }

    /// Called each time a [Property] is updated for a device.
    fn on_update_property<'a>(
        &mut self,
        d: &'a str,
        p: P,
        msg: Option<&'a str>,
    ) -> IndigoResult<(),impl Error> {
        debug!("'{}': '{}' property updated ", p.device(), p.name());
        Ok::<(), IndigoError>(())
    }

    /// Called each time a [Property] of a [Device] is deleted.
    fn on_delete_property<'a>(
        &mut self,
        d: &'a str,
        p: P,
        msg: Option<&'a str>,
    ) -> IndigoResult<(),impl Error> {
        debug!("'{}': '{}' property deleted ", p.device(), p.name());
        Ok::<(), IndigoError>(())
    }

    /// Called each time a [Device] broadcasts a message.
    fn on_message_broadcast<'a>(&mut self, d: &'a str, msg: &'a str) -> IndigoResult<(),impl Error> {
        debug!("'{}': '{}'", d, msg);
        Ok::<(), IndigoError>(())
    }
}

// -- DeviceDelegate ----------------------------------------------------------

/// Receive [Property] requests forwarded by a [DeviceController] attached to the [Bus].
pub trait DeviceDelegate<P: Property>: AttachedObject {
    /// Called when a request to enumerate all properties is made, returning the [Result::Ok] on success.
    fn on_enumeration_request<'a>(&mut self, p: &'a P) -> IndigoResult<(),impl Error>;

    /// Called when a request to define a property is made, returning the [PropertyState].
    fn on_definition_request<'a>(&mut self, p: &'a P) -> IndigoResult<(),impl Error>;

    /// Called when a request to update a property is made, returning [Result::Ok] on success.
    fn on_update_request<'a>(&mut self, p: &'a P) -> IndigoResult<(),impl Error>;

    /// Called when a request to delete a property is made, returning [Result::Ok] on success.
    fn on_deletion_request<'a>(&mut self, p: &'a P) -> IndigoResult<(),impl Error>;
}
