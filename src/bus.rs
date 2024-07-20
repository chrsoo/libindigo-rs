use super::*;
use libindigo_sys::*;

pub fn log(level: LogLevel) {
    unsafe { indigo_set_log_level(level as i32) };
}

pub fn start() -> Result<(), IndigoError> {
    let r = unsafe { indigo_start() };
    map_indigo_result(r)
}

pub fn stop() -> Result<(), IndigoError> {
    let r = unsafe { indigo_stop() };
    map_indigo_result(r)
}

/// Discover INDIGO servers on the network and invoke the callback for each server found.
pub fn discover(_c: fn(ServerConnection) -> Result<(), IndigoError>) -> Result<(), IndigoError> {
    todo!("implment server discovery over bonjour et co")
}

/// Map the indigo result to `Ok(())` if result code is `0`, to `Err(IndigoError::Bus)` if the code represents
/// a known error, and to `Err(IndigoError::Other)` if the result code is not a well-known result.
pub fn map_indigo_result<'a>(result: indigo_result) -> Result<(), IndigoError> {
    if result == indigo_result_INDIGO_OK {
        return Ok(());
    }
    if let Some(result) = BusError::from_u32(result) {
        Err(IndigoError::Bus(result))
    } else {
        let msg = format!("Unknown INDIGO bus result: {}", result);
        Err(IndigoError::Other(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_indigo_result_ok() {
        assert_eq!(map_indigo_result(indigo_result_INDIGO_OK).ok(), Some(()));
        if let IndigoError::Bus(e) = map_indigo_result(indigo_result_INDIGO_FAILED).err().unwrap() {
            assert_eq!(e, BusError::Failed)
        } else {
            assert!(false, "expected IndigoError::Bus");
        }
    }
}
