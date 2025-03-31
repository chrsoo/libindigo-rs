// mod driver;
mod property;
mod number;
mod msg;
mod indigo;
#[cfg(feature = "sys")]
pub mod sys;

// mod bus;
// mod server;
#[cfg(feature = "std")]
mod model;
// mod device;
mod spike;

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

pub use property::Property;
pub use property::PropertyItem;
pub use property::PropertyKey;
pub use property::PropertyState;
pub use property::PropertyType;
pub use property::PropertyValue;

use parking_lot::RwLockWriteGuard;
use strum_macros::EnumIter;
use core::str;
use std::collections::hash_map::Values;
use std::collections::hash_map::ValuesMut;
use std::collections::HashMap;
use std::ffi::c_char;
use std::ffi::c_uint;
use std::ffi::CStr;
use std::ffi::FromBytesUntilNulError;
use std::ptr;
use std::str::Utf8Error;
use std::sync::PoisonError;
use std::{
    error::Error,
    ffi::NulError,
    fmt::{Debug, Display},
};

use enum_primitive::*;
use strum::IntoEnumIterator;
use libindigo_sys::{self, *};

pub type StringMap<T> = HashMap<String, T>;


enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq, EnumIter, strum_macros::Display)]
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

    /// Map a bitfield to the corresponding list of interfaces, returning [None]
    /// if no interface.
    fn map(bf: u32) -> Option<Vec<Interface>> {
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

#[derive(Debug)]
pub enum IndigoError {
    /// All errors returned as a result code by INDIGO functions.
    Bus(BusError),
    /// Errors resulting from interacting with the `libindigo-sys`` crate.
    Sys(Box<dyn Error>),
    /// Other errors.
    Other(String),
    /// Other errors.
    Message(&'static str),
}

unsafe impl Sync for IndigoError {}
unsafe impl Send for IndigoError {}

impl Display for IndigoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndigoError::Bus(result) => Display::fmt(result, f),
            IndigoError::Sys(error) => Display::fmt(error, f),
            IndigoError::Other(msg) => write!(f, "{msg}"),
            IndigoError::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for IndigoError {
    /*
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.src
    }

    fn description(&self) -> &str {
        todo!()
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
    */
}

impl From<NulError> for IndigoError {
    fn from(e: NulError) -> Self {
        IndigoError::Sys(Box::new(e))
    }
}

impl From<FromBytesUntilNulError> for IndigoError {
    fn from(e: FromBytesUntilNulError) -> Self {
        IndigoError::Sys(Box::new(e))
    }
}

impl From<Utf8Error> for IndigoError {
    fn from(e: Utf8Error) -> Self {
        IndigoError::Sys(Box::new(e))
    }
}

impl From<std::io::Error> for IndigoError {
    fn from(e: std::io::Error) -> Self {
        IndigoError::Sys(Box::new(e))
    }
}
impl<T: 'static> From<PoisonError<T>> for IndigoError {
    fn from(value: PoisonError<T>) -> Self {
        IndigoError::Sys(Box::new(value))
    }
}

/// Opaque wrapper for the INDIGO access token.
pub struct AccessToken {
    tok: u64,
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]  // this really should be `c_uint` to safeguard agains platform specifics.
/// Bus operation return status.
pub enum BusError {
    /// unspecified error
    Failed = indigo_result_INDIGO_FAILED,
    /// too many clients/devices/properties/items etc.
    TooManyElements = indigo_result_INDIGO_TOO_MANY_ELEMENTS,
    /// mutex lock error
    LockError = indigo_result_INDIGO_LOCK_ERROR,
    /// unknown client/device/property/item etc.
    NotFound = indigo_result_INDIGO_NOT_FOUND,
    /// network server start failure
    CantStartServer = indigo_result_INDIGO_CANT_START_SERVER,
    /// duplicated items etc.
    Duplicated = indigo_result_INDIGO_DUPLICATED,
    /// operation failed because the resourse is busy.
    Busy = indigo_result_INDIGO_BUSY,
    /// Guide process error (srar lost, SNR too low etc..).
    GuideError = indigo_result_INDIGO_GUIDE_ERROR,
    /// Unsupported architecture.
    UnsupportedArchitecture = indigo_result_INDIGO_UNSUPPORTED_ARCH,
    /// Unresolved dependencies (missing library, executable, ...).
    UnresolvedDependency = indigo_result_INDIGO_UNRESOLVED_DEPS,
}
}

impl Into<c_uint> for BusError {
    fn into(self) -> c_uint {
        self as c_uint
    }
}

impl Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum LogLevel {
    Plain = indigo_log_levels_INDIGO_LOG_PLAIN,
    Error = indigo_log_levels_INDIGO_LOG_ERROR,
    Info = indigo_log_levels_INDIGO_LOG_INFO,
    Debug = indigo_log_levels_INDIGO_LOG_DEBUG,
    TraceBus = indigo_log_levels_INDIGO_LOG_TRACE_BUS,
    Trace = indigo_log_levels_INDIGO_LOG_TRACE,
}
}

fn buf_to_string<const N: usize>(buf: [c_char; N]) -> String {
    let ptr = buf.as_ptr();
    let cstr = unsafe { CStr::from_ptr(ptr) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

fn const_to_string(buf: &[u8]) -> String {
    // if the unwrap panics we are calling it with a faulty argument and it is a bug...
    let cstr = CStr::from_bytes_with_nul(buf).unwrap();
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

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

/// Types of request for [Client], [ServerConnection], or [Device].
// TODO refactor IndigoRequest so that it takes the callback function as a value
#[derive(Debug, PartialEq, Eq, Clone, strum_macros::Display)]
enum IndigoRequest {
    Connect,
    Disconnect,
    Attach,
    Detach,
}

pub type IndigoResult<T> = Result<T, IndigoError>;
pub type Callback<'a, T> = dyn FnMut(IndigoResult<T>) -> IndigoResult<()> + 'a;

/// Types of request for [Client], [ServerConnection], or [Device].
#[derive(strum_macros::Display)]
enum IndigoRequest2<'a, T> {
    Connect(Box<&'a mut Callback<'a, T>>),
    Disconnect(Box<&'a mut Callback<'a, T>>),
    Attach(Box<&'a mut Callback<'a, T>>),
    Detach(Box<Callback<'a, T>>),
}

impl<'a, T> IndigoRequest2<'a, T> {
    pub fn callback(&mut self, r: IndigoResult<T>) -> IndigoResult<()> {
        match self {
            IndigoRequest2::Connect(c) => c(r),
            IndigoRequest2::Disconnect(c) => c(r),
            IndigoRequest2::Attach(c) => c(r),
            IndigoRequest2::Detach(c) => c(r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_indigo_result(result: indigo_result) -> Result<BusError, String> {
        if let Some(result) = BusError::from_u32(result) {
            Ok(result)
        } else {
            Err(format!("Unknown INDIGO error result: {}", result))
        }
    }

    #[test]
    fn bus_error() {
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_FAILED).unwrap(),
            BusError::Failed
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_BUSY).unwrap(),
            BusError::Busy
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_DUPLICATED).unwrap(),
            BusError::Duplicated
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_GUIDE_ERROR).unwrap(),
            BusError::GuideError
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_LOCK_ERROR).unwrap(),
            BusError::LockError
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_NOT_FOUND).unwrap(),
            BusError::NotFound
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_UNRESOLVED_DEPS).unwrap(),
            BusError::UnresolvedDependency
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_UNSUPPORTED_ARCH).unwrap(),
            BusError::UnsupportedArchitecture
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_CANT_START_SERVER).unwrap(),
            BusError::CantStartServer
        );
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_TOO_MANY_ELEMENTS).unwrap(),
            BusError::TooManyElements
        );
    }

    #[test]
    fn bus_error_unknown_code() {
        assert_eq!(
            from_indigo_result(indigo_result_INDIGO_OK).err(),
            Some("Unknown INDIGO error result: 0".to_string())
        );
    }

    #[test]
    fn sys_constants() {
        assert_eq!("INFO", name::INFO_PROPERTY);
    }
}
