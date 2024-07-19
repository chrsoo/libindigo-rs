use std::{ffi::{c_int, CString}, ptr};
use libindigo_sys::*;
use super::*;

pub fn set_log_level<'a>(level: LogLevel) {
    unsafe { indigo_set_log_level(level as i32) };
}

pub fn start<'a>() -> Result<IndigoResult,IndigoError<'a>>{
    let result = unsafe { indigo_start() };
    IndigoResult::sys(result)
}

pub fn stop<'a>() -> Result<IndigoResult,IndigoError<'a>>{
    let result = unsafe { indigo_stop() };
    IndigoResult::sys(result)
}
