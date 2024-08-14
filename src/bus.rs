use super::*;
use libindigo_sys::*;
use log::{debug, error, info, trace, warn};

pub struct Bus {}

impl Bus {

    pub fn start() -> Result<(), IndigoError> {
        trace!("Starting bus...");
        let r = unsafe { indigo_start() };
        map_indigo_result(r, "indigo_start").inspect_err(|e|
            error!("Error starting INDIGO Bus: {}", e)
        ).and_then(|()|{
            info!("Started the INDIGO Bus.");
            Ok(())
        })
    }

    pub fn stop() -> Result<(), IndigoError> {
        trace!("Stopping bus...");
        let r = unsafe { indigo_stop() };
        map_indigo_result(r, "indigo_stop").inspect_err(|e|
            error!("Error stopping INDIGO Bus: {}", e)
        ).and_then(|()| {
            info!("Stopped the INDIGO Bus.");
            Ok(())
        })
    }

    pub fn connect<'a>(name: &str, host: &str, port: i32) -> Result<ServerConnection, IndigoError> {
        let mut con = ServerConnection::new(name, host, port)?;
        if let Err(e) = con.connect() {
            Err(e)
        } else {
            Ok(con)
        }
    }

    pub fn log(msg: &str) -> Result<(),IndigoError>{
        debug!("Bus log message: '{}'.", msg);
        let buf: [c_char;256] = str_to_buf(msg)?;
        unsafe { indigo_log(buf.as_ptr()) };
        Ok(())
    }

    pub fn set_log_level(level: LogLevel) {
        debug!("Setting log level '{:?}'.", level);
        unsafe { indigo_set_log_level(level as i32) };
    }

}


/// Map the indigo result to `Ok(())` if result code is `0`, to `Err(IndigoError::Bus)` if the code represents
/// a known error, and to `Err(IndigoError::Other)` if the result code is not a well-known result.
pub fn map_indigo_result<'a>(result: indigo_result, operation: &str) -> Result<(), IndigoError> {
    if result == indigo_result_INDIGO_OK {
        trace!("OK {}.", operation);
        return Ok(());
    }
    if let Some(result) = BusError::from_u32(result) {
        warn!("Bus error: '{}'.", result);
        Err(IndigoError::Bus(result))
    } else {
        let msg = format!("Unknown bus result: {}.", result);
        warn!("{}", msg);
        Err(IndigoError::Other(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_indigo_result_ok() {
        assert_eq!(map_indigo_result(indigo_result_INDIGO_OK, "test").ok(), Some(()));
        if let IndigoError::Bus(e) = map_indigo_result(indigo_result_INDIGO_FAILED, "test")
            .err()
            .unwrap()
        {
            assert_eq!(e, BusError::Failed)
        } else {
            assert!(false, "expected IndigoError::Bus");
        }
    }
}
