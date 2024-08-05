use std::ffi::CStr;

use super::*;
use enum_primitive::*;
use libindigo_sys::{self, *};

#[derive(Debug)]
pub struct Device<'a> {
    sys: &'a indigo_device,
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
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

pub struct GlobalLock {
    tok: indigo_glock,
}

impl Display for Device<'static> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path())
    }
}

impl<'a> Device<'a> {

    pub(super) fn new(device: *mut indigo_device) -> Device<'a> {
        Device { sys: unsafe { &*device } }
    }

    fn sys_path(sys: &indigo_device) -> Vec<String> {
        let mut v = if sys.master_device.is_null() {
            vec![buf_to_str(sys.name)]
        } else {
            Device::sys_path(unsafe { &*sys.master_device } as &indigo_device)
        };
        v.push(buf_to_str(sys.name));
        v
    }

    // -- getters

    pub fn path(&self) -> String {
        Device::sys_path(self.sys).join("/")
    }

    /// device name
    pub fn name(&self) -> &str {
        buf_to_str2(self.sys.name)
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

    // -- methods

    pub fn change_property(&self) -> Result<(),IndigoError> {
        // self.sys.change_property();
        todo!()
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
