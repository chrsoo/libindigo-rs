// mod driver;
// mod property;
mod number;
mod msg;
mod indigo;
pub mod property;
#[cfg(feature = "sys")]
pub mod sys;

// mod bus;
// mod server;
#[cfg(feature = "std")]
mod client;
// mod device;
mod spike;


include!(concat!(env!("OUT_DIR"), "/interface.rs"));

pub mod name {
    include!(concat!(env!("OUT_DIR"), "/name.rs"));

    #[cfg(test)]
    mod tests {
        use crate::name;

        #[test]
        fn names() {
            assert_eq!(name::INFO_PROPERTY, "INFO");
        }
    }
}

pub use number::NumberFormat;
pub use number::FormatFlags;
pub use number::ParseError;

// pub use property::Property;
// pub use property::PropertyItem;
// pub use property::PropertyKey;
// pub use property::PropertyState;
// pub use property::PropertyType;
// pub use property::PropertyValue;

use parking_lot::RwLockWriteGuard;
use strum_macros::EnumIter;
use core::str;
use std::collections::hash_map::Values;
use std::collections::hash_map::ValuesMut;
use std::collections::HashMap;

use std::fmt::Debug;

use enum_primitive::*;
use strum::IntoEnumIterator;
use libindigo_sys::{self, *};

pub type StringMap<T> = HashMap<String, T>;

// -- re-exports

pub use crate::indigo::*;

enum_from_primitive! {
#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumIter, strum_macros::Display)]
#[non_exhaustive]
#[repr(u32)]
// sys-doc: Device interface (value should be used for INFO_DEVICE_INTERFACE_ITEM->text.value)
/// Each interface defines a set of well-known properties.
pub enum Interface  {
    Mount = indigo_device_interface::INDIGO_INTERFACE_MOUNT.0,
    CCD = indigo_device_interface::INDIGO_INTERFACE_CCD.0,
    Guider = indigo_device_interface::INDIGO_INTERFACE_GUIDER.0,
    Focuser = indigo_device_interface::INDIGO_INTERFACE_FOCUSER.0,
    Wheel = indigo_device_interface::INDIGO_INTERFACE_WHEEL.0,
    Dome = indigo_device_interface::INDIGO_INTERFACE_DOME.0,
    GPS = indigo_device_interface::INDIGO_INTERFACE_GPS.0,
    AdaptiveOptics = indigo_device_interface::INDIGO_INTERFACE_AO.0,
    Rotator = indigo_device_interface::INDIGO_INTERFACE_ROTATOR.0,
    Agent = indigo_device_interface::INDIGO_INTERFACE_AGENT.0,
    Auxiliary = indigo_device_interface::INDIGO_INTERFACE_AUX.0,
    AuxJoystic = indigo_device_interface::INDIGO_INTERFACE_AUX_JOYSTICK.0,
    Shutter = indigo_device_interface::INDIGO_INTERFACE_AUX_SHUTTER.0,
    PowerBox = indigo_device_interface::INDIGO_INTERFACE_AUX_POWERBOX.0,
    SQM = indigo_device_interface::INDIGO_INTERFACE_AUX_SQM.0,
    DustCap = indigo_device_interface::INDIGO_INTERFACE_AUX_DUSTCAP.0,
    LightBox = indigo_device_interface::INDIGO_INTERFACE_AUX_LIGHTBOX.0,
    Weather = indigo_device_interface::INDIGO_INTERFACE_AUX_WEATHER.0,
    /// General Purpose IO auxiliary interface
    GPIO = indigo_device_interface::INDIGO_INTERFACE_AUX_GPIO.0,
}
}

impl Interface {

    /// Match the [Interface] against an INDIGO string encoded bitmap.
    pub fn matches(self, ifs: &str) -> bool {
        let ifs = Interface::convert(ifs);
        self.matches_bitmap(ifs)
    }

    /// Match the [Interface] against an INDIGO bitmap.
    pub fn matches_bitmap(self, ifs: u32) -> bool {
        (self as u32 & ifs) == self as u32
    }

    /// Convert an INDIGO interface string to an u32 bitmap.
    pub fn convert(ifs: &str) -> u32 {
        unsafe { atoi(ifs.as_ptr() as *const _) as u32 }
    }

    /// Map a bitfield to the corresponding list of interfaces, returning [None]
    /// if no interface.
    pub fn map(bf: u32) -> Option<Vec<Interface>> {
        let mut vec = Vec::new();
        for i in Interface::iter() {
            if i.matches_bitmap(bf) {
                vec.push(i);
            }
        }
        if vec.is_empty() {
            None
        } else {
            Some(vec)
        }
    }
}

#[deprecated]
pub struct GuardedStringMap<'a, T> {
    lock: RwLockWriteGuard<'a, StringMap<T>>,
}


impl<'a, 'b: 'a, T: 'a> IntoIterator for &'b mut GuardedStringMap<'a, T> {
    type Item = &'a mut T;
    type IntoIter = ValuesMut<'a, String, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.lock.values_mut()
    }
}

impl<'a, 'b: 'a, T: 'a> IntoIterator for &'b GuardedStringMap<'a, T> {
    type Item = &'a T;
    type IntoIter = Values<'a, String, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.lock.values()
    }
}

// #[derive(Debug)]
// pub enum IndigoError {
//     /// All errors returned as a result code by INDIGO functions.
//     Bus(BusError),
//     /// Errors resulting from interacting with the `libindigo-sys`` crate.
//     Sys(Box<dyn Error>),
//     /// Other errors.
//     Other(String),
//     /// Other errors.
//     Message(&'static str),
// }

// unsafe impl Sync for IndigoError {}
// unsafe impl Send for IndigoError {}

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

// impl Error for IndigoError {
//     /*
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         self.src
//     }

//     fn description(&self) -> &str {
//         todo!()
//     }

//     fn cause(&self) -> Option<&dyn Error> {
//         self.source()
//     }
//     */
// }

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

/// Opaque wrapper for the INDIGO access token.
pub struct AccessToken {
    tok: u64,
}

// enum_from_primitive! {
// #[derive(Debug, Copy, Clone, PartialEq)]
// #[repr(u32)]  // this really should be `c_uint` to safeguard agains platform specifics.
// /// Bus operation return status.
// pub enum BusError {
//     /// unspecified error
//     Failed = indigo_result_INDIGO_FAILED,
//     /// too many clients/devices/properties/items etc.
//     TooManyElements = indigo_result_INDIGO_TOO_MANY_ELEMENTS,
//     /// mutex lock error
//     LockError = indigo_result_INDIGO_LOCK_ERROR,
//     /// unknown client/device/property/item etc.
//     NotFound = indigo_result_INDIGO_NOT_FOUND,
//     /// network server start failure
//     CantStartServer = indigo_result_INDIGO_CANT_START_SERVER,
//     /// duplicated items etc.
//     Duplicated = indigo_result_INDIGO_DUPLICATED,
//     /// operation failed because the resourse is busy.
//     Busy = indigo_result_INDIGO_BUSY,
//     /// Guide process error (srar lost, SNR too low etc..).
//     GuideError = indigo_result_INDIGO_GUIDE_ERROR,
//     /// Unsupported architecture.
//     UnsupportedArchitecture = indigo_result_INDIGO_UNSUPPORTED_ARCH,
//     /// Unresolved dependencies (missing library, executable, ...).
//     UnresolvedDependency = indigo_result_INDIGO_UNRESOLVED_DEPS,
// }
// }

// impl Display for BusError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Debug::fmt(self, f)
//     }
// }

// enum_from_primitive! {
// #[derive(Debug, Copy, Clone, PartialEq)]
// #[repr(i32)]
// pub enum LogLevel {
//     Plain = indigo_log_levels_INDIGO_LOG_PLAIN,
//     Error = indigo_log_levels_INDIGO_LOG_ERROR,
//     Info = indigo_log_levels_INDIGO_LOG_INFO,
//     Debug = indigo_log_levels_INDIGO_LOG_DEBUG,
//     TraceBus = indigo_log_levels_INDIGO_LOG_TRACE_BUS,
//     Trace = indigo_log_levels_INDIGO_LOG_TRACE,
// }
// }

/*
fn str_to_buf<'a,T>(value: &'a str, _len: u16) -> Result<[i8; 128], IndigoError> {
    let mut buf = [T; 128];
    let binding = CString::new(value)?;
    let bytes = binding.as_bytes_with_nul();
    for (i, b) in bytes.iter().enumerate() {
        if i == buf.len() {
            // truncate if name is larger than the buffer size
            break;
        };
        buf[i] = *b as i8;
    }
    Ok(buf)
}
*/

// /// Types of request for [Client], [ServerConnection], or [Device].
// // TODO refactor IndigoRequest so that it takes the callback function as a value
// #[derive(Debug, PartialEq, Eq, Clone, strum_macros::Display)]
// enum IndigoRequest {
//     Connect,
//     Disconnect,
//     Attach,
//     Detach,
// }

// pub type IndigoResult<T> = Result<T, IndigoError>;
// pub type Callback<'a, T> = dyn FnMut(IndigoResult<T>) -> IndigoResult<()> + 'a;

// /// Types of request for [Client], [ServerConnection], or [Device].
// #[derive(strum_macros::Display)]
// enum IndigoRequest2<'a, T> {
//     Connect(Box<&'a mut Callback<'a, T>>),
//     Disconnect(Box<&'a mut Callback<'a, T>>),
//     Attach(Box<&'a mut Callback<'a, T>>),
//     Detach(Box<Callback<'a, T>>),
// }

// impl<'a, T> IndigoRequest2<'a, T> {
//     pub fn callback(&mut self, r: IndigoResult<T>) -> IndigoResult<()> {
//         match self {
//             IndigoRequest2::Connect(c) => c(r),
//             IndigoRequest2::Disconnect(c) => c(r),
//             IndigoRequest2::Attach(c) => c(r),
//             IndigoRequest2::Detach(c) => c(r),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    // use super::*;

}
