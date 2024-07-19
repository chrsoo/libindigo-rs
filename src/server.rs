use std::{ffi::{c_int, CStr, CString}, ptr};

use libindigo_sys::{self, *};
use enum_primitive::*;
use super::*;

pub struct ServerConnection {
    sys: indigo_server_entry,
}

// TODO check if the connection is valid for INDI servers...
/// Connection to a remote INDIGO server.
impl ServerConnection {
    /// Create a new, unconnected server.
    pub fn new<'a>(name: &'a str, host: &'a str, port: c_int) -> Result<ServerConnection,IndigoError<'a>> {
        let name = str_to_buf(name)?;
        let host = str_to_buf(host)?;

        let mut entry = indigo_server_entry {
            name: name,
            host: host,
            port: port,
            connection_id: 0,
            thread: ptr::null_mut(),
            thread_started: false,
            socket: 0,
            protocol_adapter: ptr::null_mut(),
            last_error: [0i8; 256],
            shutdown: false,
        };

        Ok(ServerConnection {
            sys: entry
        })
    }

    pub fn connect(&mut self) -> Result<IndigoResult,IndigoError> {
        let mut srv_ptr = ptr::addr_of_mut!(self.sys);
        let srv_ptr_ptr = ptr::addr_of_mut!(srv_ptr);

        bus::start()?; // TODO should we return an error if not started?

        let result = unsafe {
            indigo_connect_server(self.sys.name.as_ptr(), self.sys.host.as_ptr(), self.sys.port, srv_ptr_ptr)
        };
        IndigoResult::sys(result)
    }

    pub fn dicsonnect(&mut self) -> Result<IndigoResult, IndigoError> {
        let result = unsafe {
            indigo_disconnect_server(ptr::addr_of_mut!(self.sys))
        };
        IndigoResult::sys(result)
    }

    /// Return `true` if the server's thread is started.
    pub fn is_active(&self) -> bool {
        return self.sys.thread_started;
    }

    /// Return `true` if the server is shutting down.
    pub fn is_shutdown(&self) -> bool {
        return self.sys.shutdown;
    }
}

impl Drop for ServerConnection {
    fn drop(&mut self) {
        if self.is_active() & !self.is_shutdown() {
            if let Err(e) = self.dicsonnect() {
                todo!("log disconnect error '{}'", e)
            };
        }
    }
}
