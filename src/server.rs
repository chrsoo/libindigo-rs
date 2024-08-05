use std::{
    default,
    ffi::{c_int, CStr, CString},
    ptr, thread::sleep, time::Duration,
};

use super::bus::*;
use super::*;
use libindigo_sys::{self, *};
use log::{debug, info};

#[derive(Debug)]
pub struct ServerConnection {
    sys: indigo_server_entry,
}

impl Display for ServerConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = buf_to_str(self.sys.name);
        let host = buf_to_str(self.sys.host);
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

    pub fn connect(&mut self) -> Result<(), IndigoError> {
        info!("Connecting to {}...", self);
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
        Bus::map_indigo_result(result, "indigo_connect_server")
    }

    /*
    pub(crate) fn new_default_client() -> Result<indigo_client,IndigoError> {
        Ok(indigo_client {
            name: str_to_buf("@ Indigo")?,
            is_remote: false,
            client_context: std::ptr::null_mut(),
            last_result: indigo_result_INDIGO_OK,
            version: indigo_version_INDIGO_VERSION_CURRENT,
            enable_blob_mode_records: ptr::null_mut(),
            attach: None,
            define_property: None,
            update_property: None,
            delete_property: None,
            send_message: None,
            detach: None,
        })
    }

    pub fn detach(&mut self) ->  Result<(), IndigoError> {
        let mut default = ServerConnection::new_default_client().unwrap();
        let result = unsafe { indigo_detach_client(ptr::addr_of_mut!(default)) };
        Bus::map_indigo_result(result)
    }
    */
    pub fn dicsonnect(&mut self) -> Result<(), IndigoError> {
        info!("Disconncting from {}...", self);
        let result = unsafe { indigo_disconnect_server(ptr::addr_of_mut!(self.sys)) };
        Bus::map_indigo_result(result, "indigo_disconnect_server")
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
