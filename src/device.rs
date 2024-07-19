use std::ffi::CStr;

use libindigo_sys::{self, *};
use enum_primitive::*;
use super::*;

pub struct Device {
    sys: indigo_device,
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
// sys-doc: Device interface (value should be used for INFO_DEVICE_INTERFACE_ITEM->text.value)
/// defines a set of well-known properties
pub enum Interface  {
    Mount = indigo_device_interface_INDIGO_INTERFACE_MOUNT,
    CCD = indigo_device_interface_INDIGO_INTERFACE_CCD,
    Guider = indigo_device_interface_INDIGO_INTERFACE_GUIDER,
    Foduser = indigo_device_interface_INDIGO_INTERFACE_FOCUSER,
    Wheel = indigo_device_interface_INDIGO_INTERFACE_WHEEL,
    Dome = indigo_device_interface_INDIGO_INTERFACE_DOME,
    GPS = indigo_device_interface_INDIGO_INTERFACE_GPS,
    /// Adaptive Optics Interface
    AdaptiveOptics = indigo_device_interface_INDIGO_INTERFACE_AO,
    Rotator = indigo_device_interface_INDIGO_INTERFACE_ROTATOR,
    Agent = indigo_device_interface_INDIGO_INTERFACE_AGENT,
    /// Auxiliary interface
    Auxiliary = indigo_device_interface_INDIGO_INTERFACE_AUX,
    AuxJoystic = indigo_device_interface_INDIGO_INTERFACE_AUX_JOYSTICK,
    Shutter = indigo_device_interface_INDIGO_INTERFACE_AUX_SHUTTER,
    PowerBox = indigo_device_interface_INDIGO_INTERFACE_AUX_POWERBOX,
    SQM = indigo_device_interface_INDIGO_INTERFACE_AUX_SQM,
    DustCap = indigo_device_interface_INDIGO_INTERFACE_AUX_DUSTCAP,
    LightBox = indigo_device_interface_INDIGO_INTERFACE_AUX_LIGHTBOX,
    Weather = indigo_device_interface_INDIGO_INTERFACE_AUX_WEATHER,
    /// General purpose IO Aux interface
    GPIO = indigo_device_interface_INDIGO_INTERFACE_AUX_GPIO,
}
}

pub struct IndigoGLock {
    tok: indigo_glock,
}

impl Device {

    // -- getters

    /// device name
    pub fn name(&self) -> &str {
        let ptr = self.sys.name.as_ptr();
        let p = unsafe { CStr::from_ptr(ptr) };
        // let p = p.to_owned();
        p.to_str().unwrap()     // name must be set
    }

    /// `true` if the device is remote
    pub fn is_remote(&self) -> bool {
        self.sys.is_remote
    }

    /// return the device lock
    pub fn lock(&self) -> IndigoGLock {
        IndigoGLock {
            tok: self.sys.lock
        }
    }

    /// return the device lock
    pub fn last_result(&self) -> Option<IndigoResult> {
        IndigoResult::from_u32(self.sys.last_result)
    }

    /// return an AccessToken for synchronized property change
    pub fn access_token(&self) -> AccessToken {
        AccessToken{ tok: self.sys.access_token }
    }

    // -- methods

    pub fn change_property<'a>(&self) -> Result<(),IndigoError<'a>> {
        // self.sys.change_property();
        todo!()
    }
}

impl TryFrom<indigo_device> for Device {
    type Error = IndigoError<'static>; // TODO constrain the lifetime...

    fn try_from(device: indigo_device) -> Result<Self, Self::Error> {
        Ok(Device {
            sys: device,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interface() {
        assert_eq!(Interface::Mount as u32, indigo_device_interface_INDIGO_INTERFACE_MOUNT);
    }
}