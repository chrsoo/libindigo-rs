mod bus;
mod client;
mod server;
mod device;

use std::{error::Error, ffi::{CString, NulError}, fmt::{Debug, Display}};

use enum_primitive::*;
use libindigo_sys::{self, *};


#[derive(Debug)]
pub struct IndigoError<'a> {
    msg: &'a str,
    src: Option<&'a (dyn Error + 'static)>,
}

impl Display for IndigoError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for IndigoError<'_> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.src
    }

    fn description(&self) -> &str {
        todo!()
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl <'a> From<NulError> for IndigoError<'a> {
    fn from(e: NulError) -> Self {
        let s = e.source();
        let m = format!("NullError: {}",e);

        IndigoError {
            msg: "m.as_str()",
            src: None,
        }
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
pub enum IndigoResult {
    /// success
    OK = indigo_result_INDIGO_OK,
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

impl IndigoResult {
    fn sys<'a>(result: indigo_result) -> Result<Self,IndigoError<'a>> {
        if let Some(result) = IndigoResult::from_u32(result) {
            Ok(result)
        } else {
            Err(
                IndigoError {
                    msg: "unknown INDIGO result code returned from connection",
                    src: None,
            })
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

fn str_to_buf<'a>(value: &'a str) -> Result<[i8;128],IndigoError<'a>> {
    let mut buf = [0i8; 128];
    let binding = CString::new(value)?;
    let bytes = binding.as_bytes_with_nul();
    for (i,b) in bytes.iter().enumerate() {
        if i == buf.len() { // truncate if name is larger than the buffer size
            break;
        };
        buf[i] = *b as i8;
    }
    Ok(buf)
}
