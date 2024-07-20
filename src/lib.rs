mod bus;
mod client;
mod device;
mod server;

pub use bus::discover;
pub use bus::log;
pub use bus::start;
pub use bus::stop;
pub use server::ServerConnection;

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

/// opaque wrapper for the INDIGO access token
pub struct AccessToken {
    tok: u64,
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
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
    fn from_indigo_result(result: indigo_result) -> Result<Self,String> {
        if let Some(result) = BusError::from_u32(result) {
            Ok(result)
        } else {
            Err(format!("Unknown INDIGO error result: {}", result))
        }
    }
}

pub struct Property {
    sys: indigo_property,
}

impl Property {
    pub fn name(&self) -> &str {
        todo!();
    }
}

pub struct Item {
    sys: indigo_item,
}

impl Item {
    pub fn name(&self) -> &str {
        todo!();
    }
}

fn str_to_buf<'a>(value: &'a str) -> Result<[i8; 128], IndigoError> {
    let mut buf = [0i8; 128];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_error() {
        assert_eq!(BusError::from_indigo_result(indigo_result_INDIGO_FAILED).unwrap(), BusError::Failed);
        assert_eq!(BusError::from_indigo_result(indigo_result_INDIGO_BUSY).unwrap(), BusError::Busy);
        assert_eq!(BusError::from_indigo_result(indigo_result_INDIGO_DUPLICATED).unwrap(), BusError::Duplicated);
        assert_eq!(BusError::from_indigo_result(indigo_result_INDIGO_GUIDE_ERROR).unwrap(), BusError::GuideError);
        assert_eq!(BusError::from_indigo_result(indigo_result_INDIGO_LOCK_ERROR).unwrap(), BusError::LockError);
        assert_eq!(BusError::from_indigo_result(indigo_result_INDIGO_NOT_FOUND).unwrap(), BusError::NotFound);
        assert_eq!(BusError::from_indigo_result(
            indigo_result_INDIGO_UNRESOLVED_DEPS).unwrap(), BusError::UnresolvedDependency);
        assert_eq!(BusError::from_indigo_result(
            indigo_result_INDIGO_UNSUPPORTED_ARCH).unwrap(), BusError::UnsupportedArchitecture);
        assert_eq!(BusError::from_indigo_result(
            indigo_result_INDIGO_CANT_START_SERVER).unwrap(), BusError::CantStartServer);
        assert_eq!(BusError::from_indigo_result(
            indigo_result_INDIGO_TOO_MANY_ELEMENTS).unwrap(), BusError::TooManyElements);
    }

    #[test]
    fn bus_error_unknown_code() {
        assert_eq!(
            BusError::from_indigo_result(indigo_result_INDIGO_OK).err(),
            Some("Unknown INDIGO error result: 0".to_string()));
    }
}