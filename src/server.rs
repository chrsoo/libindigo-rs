use std::{
    ffi::c_int,
    ptr, thread::sleep, time::Duration,
};

use super::bus::*;
use super::*;
use libindigo_sys::{self, *};
use log::{debug, info, trace};

#[derive(Debug)]
pub struct ServerConnection {
    sys: indigo_server_entry,
}

impl Display for ServerConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = buf_to_string(self.sys.name);
        let host = buf_to_string(self.sys.host);
        let port = self.sys.port;
        write!(f, "{}@{}:{}", name, host, port)
    }
}

// TODO check if the connection is valid for INDI servers...
/// Connection to a remote INDIGO server.
impl ServerConnection {
    /// Create a new, unconnected server.
    pub fn new<'a>(
        name: &'a str,
        host: &'a str,
        port: c_int,
    ) -> Result<ServerConnection, IndigoError> {
        let name = str_to_buf(name)?;
        let host = str_to_buf(host)?;

        let entry = indigo_server_entry {
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

        Ok(ServerConnection { sys: entry })
    }

    pub fn connect(&mut self)
    -> Result<(), IndigoError> {
        trace!("Connecting to {}...", self);
        let mut srv_ptr = ptr::addr_of_mut!(self.sys);
        let srv_ptr_ptr = ptr::addr_of_mut!(srv_ptr);

        let result = unsafe {
            indigo_connect_server(
                self.sys.name.as_ptr(),
                self.sys.host.as_ptr(),
                self.sys.port,
                srv_ptr_ptr,
            )
        };
        map_indigo_result(result, "indigo_connect_server").inspect(|()|
        info!("Connected to {}.", self)
    )
}

    pub fn dicsonnect(&mut self) -> Result<(), IndigoError> {
        trace!("Disconncting from {}...", self);
        let result = unsafe { indigo_disconnect_server(ptr::addr_of_mut!(self.sys)) };
        map_indigo_result(result, "indigo_disconnect_server").inspect(|()|
            info!("Disconnected from {}.", self)
        )
    }

    /// Return `true` if the server's thread is started.
    pub fn is_active(&self) -> bool {
        return self.sys.thread_started;
    }

    /// Return `true` if the server is shutting down.
    pub fn is_shutdown(&self) -> bool {
        return self.sys.shutdown;
    }

    /// Disconnect and wait for the server to shutdown.
    pub fn shutdown(&mut self) -> Result<(), IndigoError> {
        self.dicsonnect()?;
        let mut timeout = 0;
        while timeout < 10 {
            if !self.is_active() { return Ok(()) };
            debug!("Waiting for server to stop.");
            sleep(Duration::from_secs(1));
            timeout += 1;
        }
        Err(IndigoError::Other("Timed out when shutting down server".to_string()))
    }

    /// Discover INDIGO servers on the network and invoke the callback for each server found.
    pub fn discover(
        _c: fn(ServerConnection) -> Result<(), IndigoError>,
    ) -> Result<(), IndigoError> {
        todo!("implment server discovery over bonjour et co")
    }
}

// impl Drop for ServerConnection {
//     fn drop(&mut self) {
//         if self.is_active() & !self.is_shutdown() {
//             if let Err(e) = self.dicsonnect(true) {
//                 todo!("log disconnect error '{}'", e)
//             };
//         }
//     }
// }
