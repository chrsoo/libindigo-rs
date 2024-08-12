#![allow(dead_code, unused_variables)]
mod bus;
mod client;
mod device;
mod model;
mod property;
mod server;

pub use bus::Bus;
pub use client::CallbackHandler;
pub use client::Client;
pub use device::Device;
use log::warn;
pub use model::IndigoModel;
pub use property::Property;
pub use property::PropertyKey;
pub use property::PropertyItem;
pub use property::PropertyType;
pub use property::PropertyState;
pub use property::PropertyValue;
pub use server::ServerConnection;

use std::borrow::Cow;
use std::ffi::c_char;
use std::ffi::c_uint;
use std::ffi::CStr;
use std::ffi::FromBytesUntilNulError;
use std::ptr;
use std::str::Utf8Error;
use std::sync::PoisonError;
use std::{
    error::Error,
    ffi::{CString, NulError},
    fmt::{Debug, Display},
};

use enum_primitive::*;
use libindigo_sys::{self, *};

#[derive(Debug)]
pub enum IndigoError {
    /// All errors returned as a result code by INDIGO functions.
    Bus(BusError),
    /// Errors resulting from interacting with the `libindigo-sys`` crate.
    Sys(Box<dyn Error>),
    /// Other errors.
    Other(String),
}

unsafe impl Sync for IndigoError {}
unsafe impl Send for IndigoError {}

impl Display for IndigoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndigoError::Bus(result) => Display::fmt(result, f),
            IndigoError::Sys(error) => Display::fmt(error, f),
            IndigoError::Other(msg) => write!(f, "{}", msg),
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

impl<T: 'static> From<PoisonError<T>> for IndigoError {
    fn from(value: PoisonError<T>) -> Self {
        IndigoError::Sys(Box::new(value))
    }
}

/// opaque wrapper for the INDIGO access token
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

impl BusError {
    fn from_indigo_result(result: indigo_result) -> Result<Self, String> {
        if let Some(result) = BusError::from_u32(result) {
            Ok(result)
        } else {
            Err(format!("Unknown INDIGO error result: {}", result))
        }
    }
}

fn str_to_buf<const N: usize>(s: &str) -> Result<[c_char; N], IndigoError> {
    let s = CString::new(s).expect("a string without \\0 bytes");
    let mut buf = [0; N];
    let bytes = s.as_bytes_with_nul();
    if bytes.len() > N {
        Err(IndigoError::Other(format!(
            "The string's byte length + 1 must be less or equal to {}",
            N
        )))
    } else {
        for (i, b) in bytes.iter().enumerate() {
            buf[i] = *b as i8;
        }
        Ok(buf)
    }
}

fn buf_to_string<const N: usize>(buf: [c_char; N]) -> String {
    let ptr = buf.as_ptr();
    let cstr = unsafe { CStr::from_ptr(ptr) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

fn buf_to_str<'a, const N: usize>(buf: [c_char; N]) -> &'a str {
    let ptr = buf.as_ptr();
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().inspect_err(|e| warn!("{}", e)).unwrap_or("<invalid>")
}

fn const_to_string(name: &[u8]) -> Cow<str> {
    // if the unwrap panics we are calling it with a faulty argument and it is a bug...
    let name = CStr::from_bytes_with_nul(name).unwrap();
    name.to_string_lossy()
}

fn ptr_to_string(message: *const c_char) -> Option<String> {
    if message == ptr::null() {
        None
    } else {
        let cstr = unsafe { CStr::from_ptr(message) };
        let s = String::from_utf8_lossy(cstr.to_bytes()).to_string();
        Some(s)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_error() {
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_FAILED).unwrap(),
            BusError::Failed
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_BUSY).unwrap(),
            BusError::Busy
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_DUPLICATED).unwrap(),
            BusError::Duplicated
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_GUIDE_ERROR).unwrap(),
            BusError::GuideError
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_LOCK_ERROR).unwrap(),
            BusError::LockError
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_NOT_FOUND).unwrap(),
            BusError::NotFound
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_UNRESOLVED_DEPS).unwrap(),
            BusError::UnresolvedDependency
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_UNSUPPORTED_ARCH).unwrap(),
            BusError::UnsupportedArchitecture
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_CANT_START_SERVER).unwrap(),
            BusError::CantStartServer
        );
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_TOO_MANY_ELEMENTS).unwrap(),
            BusError::TooManyElements
        );
    }

    #[test]
    fn bus_error_unknown_code() {
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_OK).err(),
            Some("Unknown INDIGO error result: 0".to_string())
        );
    }
}
