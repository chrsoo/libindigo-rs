use bus::map_indigo_result;
use std::{
    any::Any,
    borrow::Borrow,
    collections::HashMap,
    ops::Deref,
    ptr::{self, addr_of, addr_of_mut},
    sync::{Mutex, MutexGuard, RwLock},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::*;
use enum_primitive::*;
use libindigo_sys::{self, *};
use log::warn;

pub struct Device<'a> {
    // TODO change sys into raw mutable pointer?
    sys: &'a indigo_device,
    client: *mut indigo_client,
    props: HashMap<String, Property<'a>>,
    request: Mutex<Option<IndigoRequest>>,
    callback: Option<&'a dyn Fn(Result<(), IndigoError>)>,
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq, EnumIter)]
#[repr(u32)]
// sys-doc: Device interface (value should be used for INFO_DEVICE_INTERFACE_ITEM->text.value)
/// Each interface defines a set of well-known properties.
pub enum Interface  {
    Mount = indigo_device_interface_INDIGO_INTERFACE_MOUNT,
    CCD = indigo_device_interface_INDIGO_INTERFACE_CCD,
    Guider = indigo_device_interface_INDIGO_INTERFACE_GUIDER,
    Foduser = indigo_device_interface_INDIGO_INTERFACE_FOCUSER,
    Wheel = indigo_device_interface_INDIGO_INTERFACE_WHEEL,
    Dome = indigo_device_interface_INDIGO_INTERFACE_DOME,
    GPS = indigo_device_interface_INDIGO_INTERFACE_GPS,
    AdaptiveOptics = indigo_device_interface_INDIGO_INTERFACE_AO,
    Rotator = indigo_device_interface_INDIGO_INTERFACE_ROTATOR,
    Agent = indigo_device_interface_INDIGO_INTERFACE_AGENT,
    Auxiliary = indigo_device_interface_INDIGO_INTERFACE_AUX,
    AuxJoystic = indigo_device_interface_INDIGO_INTERFACE_AUX_JOYSTICK,
    Shutter = indigo_device_interface_INDIGO_INTERFACE_AUX_SHUTTER,
    PowerBox = indigo_device_interface_INDIGO_INTERFACE_AUX_POWERBOX,
    SQM = indigo_device_interface_INDIGO_INTERFACE_AUX_SQM,
    DustCap = indigo_device_interface_INDIGO_INTERFACE_AUX_DUSTCAP,
    LightBox = indigo_device_interface_INDIGO_INTERFACE_AUX_LIGHTBOX,
    Weather = indigo_device_interface_INDIGO_INTERFACE_AUX_WEATHER,
    /// General Purpose IO auxiliary interface
    GPIO = indigo_device_interface_INDIGO_INTERFACE_AUX_GPIO,
}
}

impl Interface {
    /// map an INDIGO interface `String` value to list of `Interface` variants.
    fn map(ifs: String) -> Vec<Interface> {
        // Convert the interface string to a bitmap unsigned integer.
        let ifs = Interface::convert(ifs);
        let mut vec = Vec::new();
        for i in Interface::iter() {
            if i.matches(ifs) {
                vec.push(i);
            }
        }
        vec
    }
    /// Match the INDIGO interface bitmap against a specific interface.
    fn matches(self, ifs: u32) -> bool {
        (self as u32 & ifs) == self as u32
    }

    /// Convert an INDIGO interface `String` to an u32 bitmap.
    fn convert(ifs: String) -> u32 {
        unsafe { atoi(ptr::addr_of!(ifs) as *const _) as u32 }
    }
}

pub struct GlobalLock {
    tok: indigo_glock,
}

impl Display for Device<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> Device<'a> {
    pub(crate) fn new(c: *mut indigo_client, d: *mut indigo_device) -> Device<'a> {
        Device {
            sys: unsafe { &*d },
            client: c,
            props: HashMap::new(),
            request: Mutex::new(None),
            callback: None,
        }
    }

    // -- getters

    /// device name
    pub fn name(&self) -> String {
        buf_to_string(self.sys.name)
    }

    /// `true` if the device is remote
    pub fn is_remote(&self) -> bool {
        self.sys.is_remote
    }

    /// Return the device lock.
    pub fn lock(&self) -> GlobalLock {
        GlobalLock { tok: self.sys.lock }
    }

    /// Return the last result.
    pub fn last_result(&self) -> Option<BusError> {
        BusError::from_u32(self.sys.last_result)
    }

    /// Return an AccessToken for synchronized property change.
    pub fn access_token(&self) -> AccessToken {
        AccessToken {
            tok: self.sys.access_token,
        }
    }

    pub fn property(&self, name: &str) -> Result<&Property, IndigoError> {
        self.props.get(name).ok_or(IndigoError::Other(format!(
            "Property '{}' not found.",
            name
        )))
    }

    /// Return a propety using an libindigo-sys constant property name, e.g. [CONNECTION_PROPERTY_NAME].
    pub(crate) fn property_lib(&self, name: &[u8]) -> Result<&Property, IndigoError> {
        let name = const_to_string(name);
        self.property(name.as_ref())
    }

    /// Returns `Ok(true)` or `Ok(false)` if the device is [Ok](PropertyState::Ok),
    /// else the corresponding [PropertyState] is returned as an error.
    pub fn connected(&self) -> Result<bool, IndigoError> {
        let property_name = const_to_string(CONNECTION_PROPERTY_NAME);
        let connection = self.property(property_name.as_ref())?;
        if connection.state() != PropertyState::Ok {
            return Err(IndigoError::Other(format!("{:?} ", connection.state())));
        }

        let disconnected_item = const_to_string(CONNECTION_DISCONNECTED_ITEM_NAME);
        let connected_item = const_to_string(CONNECTION_CONNECTED_ITEM_NAME);
        for i in connection {
            if i.name == connected_item || i.name == disconnected_item {
                return match i.value {
                    PropertyValue::Switch(b) => {
                        //
                        Ok(if i.name == connected_item { b } else { !b })
                    }
                    _ => {
                        let msg = format!(
                            "Illegal '{}' property value, expected a switch for item '{}' and not '{:?}'",
                            property_name, i.name, i.value
                        );
                        warn!("{}", msg);
                        return Err(IndigoError::Other(msg));
                    }
                };
            }
        }
        let msg = format!(
            "Illegal '{}' property definition, could not find a '{}' or '{}' item",
            property_name, connected_item, disconnected_item
        );
        warn!("{}", msg);
        return Err(IndigoError::Other(msg));
    }

    pub(crate) fn addr_of_name(&self) -> *mut c_char {
        addr_of!(self.sys.name) as *const _ as *mut c_char
    }

    /*
    pub fn connect(&mut self, d: &mut Device, f: &dyn Fn(Result<(),IndigoError>) ) -> Result<(),IndigoError> {
        let c = addr_of_mut!(self.sys);
        let d = d.addr_of_name();
        let result = unsafe { indigo_device_connect(c, d) };
        map_indigo_result(result, "indigo_device_connect")
    }
    */
    /// Connect the device
    pub(crate) fn connect(
        &mut self,
        f: &'a dyn Fn(Result<(), IndigoError>),
    ) -> Result<(), IndigoError> {
        let mut r = self.request.lock().unwrap();
        if let Some(request) = &mut *r {
            return Err(IndigoError::Other(format!(
                "{} request in progress for device '{}'",
                request,
                self.name()
            )));
        }
        *r = Some(IndigoRequest::Connect);
        self.callback = Some(f);

        let d = self.addr_of_name();
        let result = unsafe { indigo_device_connect(self.client, d) };
        map_indigo_result(result, "indigo_device_connect")
    }

    pub fn interfaces(&self) -> Vec<Interface> {
        let info = const_to_string(INFO_PROPERTY_NAME); // TODO make this a constant
        if let Some(p) = self.props.get(info.as_ref()) {
            if let Some(ifs) = p
                .items()
                .filter_map(|i| {
                    // only look for info device driver items
                    if i.name == const_to_string(INFO_DEVICE_DRIVER_ITEM_NAME) {
                        // ensure that a text property value
                        if let PropertyValue::Text(v) = i.value.clone() {
                            Some(v) // heureka!
                        } else {
                            warn!("INFO_DEVICE_DRIVER_ITEM does not contain a text property value");
                            None
                        }
                    } else {
                        None // not an info device driver item
                    }
                })
                .nth(0)
            {
                // map the info device driver string to a list of interfaces
                return Interface::map(ifs);
            }
        }
        Vec::new()
    }

    // -- methods

    pub fn change_property(&self) -> Result<(), IndigoError> {
        // self.sys.change_property();
        todo!()
    }

    /// Returns `IndigoError::Other`if the source and target devices do not share
    /// the same name or if they refer to different `indigo_device` objects.
    pub(crate) fn check_ref_eq(&self, d: Device<'a>) -> Result<(), IndigoError> {
        if self.name() != d.name() {
            return Err(IndigoError::Other(
                "Source and target do not share the same name.".to_string(),
            ));
        }

        if ptr::eq(self.sys, d.sys) {
            Ok(())
        } else {
            Err(IndigoError::Other(
                "Indigo Device uses same name but different indigo_device objects".to_string(),
            ))
        }
    }
}

// impl<'a> TryFrom<&indigo_device> for Device<'a> {
//     type Error = IndigoError; // TODO constrain the lifetime...
//     fn try_from(device: &indigo_device) -> Result<Self, Self::Error> {
//         Ok(Device { sys: device })
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interface() {
        assert_eq!(
            Interface::Mount as u32,
            indigo_device_interface_INDIGO_INTERFACE_MOUNT
        );
    }
}
