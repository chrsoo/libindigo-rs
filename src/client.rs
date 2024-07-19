use std::ffi::CStr;

use libindigo_sys::{self, *};
use enum_primitive::*;
use super::*;

pub struct Client {
    sys: indigo_client,
}

impl Client {
    /// Attach the client to the INDIGO bus and invoke the `attach()` callback function with the result.
    // sys-doc: Attach client to bus.\nReturn value of attach() callback function is assigned to last_result in client structure.
    pub fn attach<'a>(&mut self) -> Result<IndigoResult,IndigoError<'a>> {
        let result = unsafe {
            indigo_attach_client(std::ptr::addr_of_mut!(self.sys))
        };
        IndigoResult::sys(result)
    }
    /// Detach the client from the INDIGO bus and invoke the `detach()` callback function with the result.
    // sys-doc: Detach client from bus.\nReturn value of detach() callback function is assigned to last_result in client structure.
    pub fn detach<'a>(&mut self) -> Result<IndigoResult,IndigoError<'a>> {
        let result = unsafe {
            indigo_detach_client(std::ptr::addr_of_mut!(self.sys))
        };
        IndigoResult::sys(result)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Err(e) = self.detach() {
            todo!("log error")
        }
    }
}
