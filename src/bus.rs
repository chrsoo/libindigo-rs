use super::*;
use libindigo_sys::*;
use log::{debug, error, info, trace, warn};

pub struct Bus {}


pub fn start() -> IndigoResult<()> {
    trace!("Starting bus...");
    let r = unsafe { indigo_start() };
    sys_to_lib((), r, "indigo_start")
        .inspect_err(|e| error!("Error starting INDIGO Bus: {}", e))
        .and_then(|()| {
            info!("Started the INDIGO Bus.");
            Ok(())
        })
}

pub fn stop() -> IndigoResult<()> {
    trace!("Stopping bus...");
    let r = unsafe { indigo_stop() };
    sys_to_lib((), r, "indigo_stop")
        .inspect_err(|e| error!("Error stopping INDIGO Bus: {}", e))
        .and_then(|()| {
            info!("Stopped the INDIGO Bus.");
            Ok(())
        })
}

pub fn log(msg: &str) -> IndigoResult<()> {
    debug!("Bus log message: '{}'.", msg);
    let buf: [c_char; 256] = str_to_buf(msg);
    unsafe { indigo_log(buf.as_ptr()) };
    Ok(())
}

// TODO find out how to map INDIGO logs to Rust's log system
pub fn set_log_level(level: LogLevel) {
    debug!("Setting log level '{:?}'.", level);
    unsafe { indigo_set_log_level(level as i32) };
}

/// Map a sys INDIGO [indigo_result] to a libindigo [IndigoResult]  where `0` returns `Ok(T)`,
/// a well-known [indigo_result] returns `Err(IndigoError::Bus)`, and any other code returns
/// `Err(IndigoError::Other)`.
pub fn sys_to_lib<'a, T>(t: T, result: indigo_result, operation: &str) -> IndigoResult<T> {
    if result == indigo_result_INDIGO_OK {
        trace!("INDIGO - '{}' Ok.", operation);
        return Ok(t);
    }

    if let Some(result) = BusError::from_u32(result) {
        warn!("INDIGO bus error: '{}'.", result);
        Err(IndigoError::Bus(result))
    } else {
        let msg = format!("INDIGO unknown bus result: {}.", result);
        warn!("{msg}");
        Err(IndigoError::Other(msg))
    }
}

/// Map a libindigo [IndigoResult] to a sys INDIGO [indigo_result].
pub(crate) fn lib_to_sys(result: IndigoResult<()>, _operation: &str) -> indigo_result {
    match result {
        Ok(_) => indigo_result_INDIGO_OK,
        Err(e) => match e {
            IndigoError::Bus(bus_error) => bus_error.into(),
            IndigoError::Sys(_) => BusError::Failed.into(),
            IndigoError::Other(_) => BusError::Failed.into(),
            IndigoError::Message(_) => BusError::Failed.into(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_indigo_result_ok() {
        assert_eq!(
            sys_to_lib((), indigo_result_INDIGO_OK, "test").ok(),
            Some(())
        );
        if let IndigoError::Bus(e) = sys_to_lib((), indigo_result_INDIGO_FAILED, "test")
            .err()
            .unwrap()
        {
            assert_eq!(e, BusError::Failed)
        } else {
            assert!(false, "expected IndigoError::Bus");
        }
    }
}
