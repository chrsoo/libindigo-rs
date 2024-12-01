use std::{
    ffi::c_int, ops::Deref, ptr, rc::Rc, thread::sleep, time::Duration
};

use super::*;
use libindigo_sys::{self, *};
use log::{debug, info, trace};

pub fn connect(name: &str, host: &str, port: c_int) -> IndigoResult<ServerConnection> {
    trace!("Connecting to {host}:{port}...");
    if !hostname_validator::is_valid(host) {
        return Err(IndigoError::Other(format!("invalid hostname: '{host}'")))
    }

    let name = str_to_buf(name)?;
    let host = str_to_buf(host)?;

    let mut entry = indigo_server_entry {
        name: name,
        host: host,
        port: port,
        connection_id: 0,
        thread: unsafe { std::mem::zeroed() },
        thread_started: false,
        socket: 0,
        protocol_adapter: ptr::null_mut(),
        last_error: [0; 256],
        shutdown: false,
    };

    let mut srv_ptr = ptr::addr_of_mut!(entry);
    let srv_ptr_ptr = ptr::addr_of_mut!(srv_ptr);

    let result = unsafe {
        indigo_connect_server(
            entry.name.as_ptr(),
            entry.host.as_ptr(),
            entry.port,
            srv_ptr_ptr,
        )
    };

    let connection = ServerConnection {
        sys: Rc::new(entry),
    };

    bus::sys_to_lib(connection, result, "indigo_connect_server")
        .inspect(|c| info!("Connection to {c} successful."))
        .inspect_err(|e| warn!("Connection failed: {e}."))
}

#[derive(Debug, Clone)]
pub struct ServerConnection {
    sys: Rc<indigo_server_entry>,
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

    fn addr_of_sys(&self) -> *mut indigo_server_entry {
        ptr::from_ref(self.sys.deref()) as *mut indigo_server_entry
    }

    pub fn reconnect(&mut self)
    -> Result<(), IndigoError> {
        trace!("Reconnecting to {}...", self);

        let mut srv_ptr = self.addr_of_sys();
        let srv_ptr_ptr = ptr::addr_of_mut!(srv_ptr);

        let result = unsafe {
            indigo_connect_server(
                self.sys.name.as_ptr(),
                self.sys.host.as_ptr(),
                self.sys.port,
                srv_ptr_ptr,
            )
        };
        bus::sys_to_lib((), result, "indigo_connect_server").inspect(|()|
            info!("Reconnected to {}.", self)
        )
    }

    pub fn dicsonnect(&mut self) -> Result<(), IndigoError> {
        trace!("Disconncting from {}...", self);
        let srv_ptr = self.addr_of_sys();
        let result = unsafe { indigo_disconnect_server(srv_ptr) };
        bus::sys_to_lib((), result, "indigo_disconnect_server").inspect(|()|
            info!("Disconnected from {}.", self)
        )
    }

    /// returns `Ok(true)` if connected to the server, `Ok(false)` if disconnected, and
    /// `Err(IndigoError::Other(error))` if disconnected and there was an error.
    pub fn is_connected(&self) -> IndigoResult<bool> {
        let mut msg = [0;256];
        let msg_ptr = msg.as_mut_ptr();
        let srv_ptr = (ptr::addr_of!(self.sys)) as *mut indigo_server_entry;
        let connected = unsafe { indigo_connection_status(srv_ptr, msg_ptr) };
        let msg = buf_to_string(msg);
        if connected || msg.is_empty() {
            Ok(connected)
        } else {
            Err(IndigoError::Other(format!("Connection status failed: {msg}")))
        }
    }

    /// Return `true` if the server's thread is started.
    pub fn is_active(&self) -> bool {
        return self.sys.thread_started;
    }

    /// Return `true` if the server is shutting down.
    pub fn is_shutdown(&self) -> bool {
        return self.sys.shutdown;
    }

    /// Shutdown the server thread.
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
