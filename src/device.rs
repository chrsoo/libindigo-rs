use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use super::*;
use enum_primitive::*;
use libindigo_sys::{self, *};
use log::warn;

pub trait Device {
    fn name(&self) -> &str;

    fn get(&self, property: &str) -> Option<&Property>;
    fn get_mut(&mut self, property: &str) -> Option<&mut Property>;

    fn props(&self) -> impl Iterator<Item = &Property>;
    fn props_mut(&mut self) -> impl Iterator<Item = &mut Property>;

    fn info(&self) -> Option<&Property> {
        // let info = const_to_str(INFO_PROPERTY_NAME)?; // TODO make this a constant
        let info = &const_to_string(INFO_PROPERTY_NAME); // TODO make this a constant

        if let Some(p) = self.get(info) {
            if p.property_type() == &PropertyType::Text {
                return Some(p);
            } else {
                warn!(
                    "Expected {:?} but found {:?} for {} property on '{}' device",
                    PropertyType::Text,
                    p.property_type(),
                    info,
                    self.name()
                );
            }
        } else {
            warn!("{} property not defined for device '{}'", info, self.name());
        }
        None
    }

    /// Returns [Ok](PropertyState::Ok) if the device is in a usable state, else the corresponding [PropertyState]
    /// is returned as an error. If the device is [Ok](PropertyState::Ok), the returned value can be used
    /// to determine the device's connection status.
    /// ```
    /// let d = Device::new("TestDevice");
    /// match d.connection_status() {
    ///     Ok(true) => info!("Device {d} is CONNECTED."),
    ///     Ok(false) => info!("Device {d} is DISCONNECTED."),
    ///     Err(state) => warn!("Device {d} is in the {state}"),
    /// }
    /// ```
    fn connected(&self) -> Result<bool,&PropertyState> {
        let property_name = const_to_string(CONNECTION_PROPERTY_NAME);
        if let Some(connection) = self.get(&property_name) {
            if connection.state() != &PropertyState::Ok {
                return Err(connection.state());
            }
            let disconnected_item = const_to_string(CONNECTION_DISCONNECTED_ITEM_NAME);
            let connected_item = const_to_string(CONNECTION_CONNECTED_ITEM_NAME);
            for item in connection {
                if item.name == connected_item || item.name == disconnected_item {
                    return match item.value {
                        PropertyValue::Switch(b) => {
                            Ok(if item.name == connected_item { b } else { !b })
                        }
                        _ => {
                            let msg = format!(
                                "{}.{}: expected switch value for item '{}', found '{:?}'",
                                self.name(), property_name, item.name, item.value
                            );
                            warn!("{}", msg);
                            return Err(&PropertyState::Alert);
                        }
                    };
                }
            }
            warn!(
                "{}.{}: could not find a '{}' or '{}' item",
                self.name(), property_name, connected_item, disconnected_item
            );
            return Err(&PropertyState::Alert);

        }

        warn!("{}.{}: property not found", self.name(), property_name);
        return Err(&PropertyState::Alert)
    }

    fn is_interface(&self, iface: &Interface) -> bool {
        if let Some(p) = self.info() {
            //let device_interface_item = const_to_str(INFO_DEVICE_INTERFACE_ITEM_NAME)?;
            let device_interface_item = &const_to_string(INFO_DEVICE_INTERFACE_ITEM_NAME);
            if let Some(item) = p.get_item(device_interface_item) {
                if let PropertyValue::Text(ifs) = &item.value {
                    return iface.matches(&ifs);
                } else {
                    warn!(
                        "DEVICE_INTERFACE item is not a text property value; found '{}'",
                        item.value
                    );
                }
            }
        }
        false
    }

    fn list_interfaces(&self) -> Option<Vec<Interface>> {
        let p = self.info()?;
        // let device_interface_item = const_to_str(INFO_DEVICE_INTERFACE_ITEM_NAME)?;
        let device_interface_item = &const_to_string(INFO_DEVICE_INTERFACE_ITEM_NAME);
        if let Some(item) = p.get_item(device_interface_item) {
            if let PropertyValue::Text(ifs) = &item.value {
                return Some(Interface::map(&ifs));
            } else {
                warn!(
                    "DEVICE_INTERFACE item is not a text property value; found '{}'",
                    item.value
                );
            }
        }
        None
    }
}

enum_from_primitive! {
#[derive(Display, Debug, Copy, Clone, PartialEq, EnumIter)]
#[repr(u32)]
// sys-doc: Device interface (value should be used for INFO_DEVICE_INTERFACE_ITEM->text.value)
/// Each interface defines a set of well-known properties.
pub enum Interface  {
    Mount = indigo_device_interface_INDIGO_INTERFACE_MOUNT,
    CCD = indigo_device_interface_INDIGO_INTERFACE_CCD,
    Guider = indigo_device_interface_INDIGO_INTERFACE_GUIDER,
    Focuser = indigo_device_interface_INDIGO_INTERFACE_FOCUSER,
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
    /// Parse an INDIGO interface `String` value to the corrsponding list of `Interface` variants.
    pub(crate) fn map(ifs: &str) -> Vec<Interface> {
        // Convert the interface string to a bitmap unsigned integer.
        let ifs = Interface::convert(ifs);
        let mut vec = Vec::new();
        for i in Interface::iter() {
            if i.matches_bitmap(ifs) {
                vec.push(i);
            }
        }
        vec
    }

    /// Match the [Interface] against an INDIGO string encoded bitmap.
    pub(crate) fn matches(self, ifs: &str) -> bool {
        let ifs = Interface::convert(ifs);
        self.matches_bitmap(ifs)
    }

    /// Match the [Interface] against an INDIGO bitmap.
    pub(crate) fn matches_bitmap(self, ifs: u32) -> bool {
        (self as u32 & ifs) == self as u32
    }

    /// Convert an INDIGO interface `String` to an u32 bitmap.
    fn convert(ifs: &str) -> u32 {
        unsafe { atoi(ifs.as_ptr() as *const _) as u32 }
    }
}

pub struct GlobalLock {
    pub(crate) tok: indigo_glock,
}

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

    #[test]
    fn map_interface_string() {
        let ifs = format!("{}", Interface::Agent as u32 | Interface::CCD as u32);
        let ifs = Interface::map(&ifs);
        assert!(ifs.contains(&Interface::Agent));
        assert!(ifs.contains(&Interface::Agent));
    }

    #[test]
    fn convert_interface_string() {
        let ifs = format!("{}", Interface::Agent as u32);
        assert_eq!(Interface::convert(&ifs), Interface::Agent as u32)
    }
}
