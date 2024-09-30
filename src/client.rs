use std::{
    marker::PhantomData,
    os::raw::c_void,
    ptr::{self},
};

use crate::*;
use device::Device;
use function_name::named;
use libindigo_sys::{self, *};
use log::{debug, error, info, trace};
use parking_lot::{lock_api::RwLockWriteGuard, RawRwLock, RwLock};
use property::{Blob, BlobMode};

struct ClientState<'a, T>
where
    T: ClientCallbacks<'a>,
{
    model: T,
    request: Option<IndigoRequest>,
    callback: Box<dyn FnMut(&mut Client<'a, T>) -> Result<(), IndigoError> + 'a>,
    ref_count: u32,
}

/// Data model used by a [IndigoClient] with callback methods to handle [IndigoBus] events.
pub trait ClientCallbacks<'a> {
    type M: ClientCallbacks<'a>;

    /// Called each time the property of a device is defined or its definition requested.
    fn on_define_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; DEFINED with message '{:?}'",
            d,
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time a property is updated for a device.
    fn on_update_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; UPDATED with message '{:?}'",
            d,
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time a property is deleted.
    fn on_delete_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; DELETED with message '{:?}'",
            d,
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each a device broadcasts a message. The default implementation logs the message at INFO level.
    fn on_send_message(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        msg: String,
    ) -> Result<(), IndigoError> {
        info!("Device: '{d}';  SEND  message: '{msg}'");
        Ok(())
    }
}

/// A device as seen from a client implementation
pub struct ClientDevice<'a> {
    name: &'a str,
    props: &'a mut HashMap<String, Property>,
}

// TODO move ClientDevice methods to Device trait
impl<'a> ClientDevice<'a> {
    pub(crate) fn addr_of_name(&self) -> *mut c_char {
        // addr_of!((*self.sys).name) as *const _ as *mut c_char
        todo!()
    }
}

impl<'a> Display for ClientDevice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = self.connected().map_or_else(
            |e| format!("{:?}", e),
            |s| {
                if s {
                    "connected".to_string()
                } else {
                    "disconnected".to_string()
                }
            },
        );
        write!(f, "{} ({}) [", self.name(), status)?;
        let mut sep = "";
        if let Some(ifaces) = self.list_interfaces() {
            for item in ifaces {
                write!(f, "{sep}{item}")?;
                sep = ", ";
            }
        }
        write!(f, "]")?;

        Ok(())
    }
}

impl<'a> Device for ClientDevice<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn get(&self, property: &str) -> Option<&Property> {
        self.props.get(property)
    }

    fn get_mut(&mut self, property: &str) -> Option<&mut Property> {
        self.props.get_mut(property)
    }

    fn props(&self) -> impl Iterator<Item = &Property> {
        self.props.values()
    }

    fn props_mut(&mut self) -> impl Iterator<Item = &mut Property> {
        self.props.values_mut()
    }
}

/// A default implementation of [ClientCallbacks] that manages the set of all enumerated devices
/// and their properties that are defined on the [Bus](crate::Bus) .
pub struct ClientDeviceModel {
    devices: HashMap<String, HashMap<String, Property>>,
}

impl<'a> ClientCallbacks<'a> for ClientDeviceModel {
    type M = ClientDeviceModel;

    fn on_define_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        // FIXME hanlde device 'd'
        let props = self
            .devices
            .entry(p.device())
            .or_insert_with(|| HashMap::new());
        props
            .entry(p.name())
            .and_modify(|prop| prop.update(&p))
            .or_insert(p);

        Ok(())
    }

    fn on_update_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        // FIXME handle device 'd'
        if let Some(props) = self.devices.get_mut(&p.device()) {
            if let Some(prop) = props.get_mut(&p.name()) {
                prop.update(&p);
                Ok(())
            } else {
                Err(IndigoError::Message(
                    "Trying to update an undefined property.",
                ))
            }
        } else {
            Err(IndigoError::Message("Device not found."))
        }
    }

    fn on_delete_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        if let Some(props) = self.devices.get_mut(&d) {
            if let Some(prop) = props.remove(&p.name()) {
                Ok(())
            } else {
                Err(IndigoError::Message(
                    "Trying to delete an undefined property.",
                ))
            }
        } else {
            Err(IndigoError::Message("Device not found."))
        }
    }

    fn on_send_message(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        msg: String,
    ) -> Result<(), IndigoError> {
        info!("Device: '{d}';  SEND  message: '{msg}'");
        Ok(())
    }
}

impl ClientDeviceModel {
    pub fn new() -> ClientDeviceModel {
        ClientDeviceModel {
            devices: HashMap::new(),
        }
    }

    pub fn devices(&mut self) -> impl Iterator<Item = ClientDevice> {
        self.devices
            .iter_mut()
            .map(|(k, v)| ClientDevice { name: k, props: v })
    }
}

/// Client to manage devices attached to the INDIGO [Bus].
pub struct Client<'a, T>
where
    T: ClientCallbacks<'a>,
{
    /// System record for INDIGO clients.
    sys: &'a indigo_client,
    model: &'a PhantomData<T>,
}

impl<'a, M> Client<'a, M>
where
    M: ClientCallbacks<'a, M = M> + 'a,
{
    /// Create new client for a model. Clients running locally in the same process
    /// as the INDIGO [Bus] should set remote to `false` and clients running in a
    /// remote process should set remote to `true`.
    pub fn new(name: &str, model: M, remote: bool) -> Self {
        let state = Box::new(RwLock::new(ClientState {
            model,
            request: None,
            callback: Box::new(|c|
                // If this callback is invoked it means that we received an INDIGO callback
                // without having called invoke or connect on the client, or that the code
                // has failed to store the callback on the right client object.
                Err(IndigoError::Other(format!("Initial callback placeholder that should never be called!")))),
            ref_count: 1,
        }));

        let indigo_client = Box::new(indigo_client {
            name: str_to_buf(name).unwrap(), // TODO remove result
            is_remote: remote,
            // unbox the mutex state and create a raw ptr to the state mutex
            //client_context: (&mut *state) as *mut _ as *mut c_void,
            client_context: Box::into_raw(state) as *mut c_void,
            // last result of a bus operation, assume all is OK from the beginning
            last_result: indigo_result_INDIGO_OK,
            version: indigo_version_INDIGO_VERSION_CURRENT,
            enable_blob_mode_records: ptr::null_mut(),
            // The unsafe sys callback methods required by INDIGO
            attach: Some(Self::on_attach),
            define_property: Some(Self::on_define_property),
            update_property: Some(Self::on_update_property),
            delete_property: Some(Self::on_delete_property),
            send_message: Some(Self::on_send_message),
            detach: Some(Self::on_detach),
        });

        // get ptr reference to the indigo_client by dereferencing the Box
        let ptr = Box::into_raw(indigo_client);

        Client {
            sys: unsafe { &*ptr },
            model: &PhantomData,
        }
    }

    /// Acquire a lock on the client state held in the `client_context` of sys.
    fn aquire_write_lock(&mut self) -> RwLockWriteGuard<RawRwLock, ClientState<'a, M>> {
        Self::write_lock(ptr::addr_of!(*self.sys) as *mut _)
    }

    fn register_request<F>(&mut self, request: IndigoRequest, f: F) -> Result<(), IndigoError>
    where
        F: FnMut(&mut Client<'a, M>) -> Result<(), IndigoError> + 'a,
    {
        trace!("'{self}' - registering {request} request...");
        let name = self.name();
        // let name = buf_to_str(self.sys.name);
        let mut lock = self.aquire_write_lock();

        // check if a request is already in progress
        if let Some(request) = &lock.request {
            return Err(IndigoError::Other(format!(
                "'{name}' - {request} request already in progress."
            )));
        }

        // signal that a request is ongoing
        lock.request = Some(request);
        // store the closure in a Box to create a stable reference
        lock.callback = Box::new(f);
        debug!(
            "'{name}' - {} request registered.",
            lock.request.as_ref().unwrap()
        );
        Ok(())
    }

    /// Attach the client to the INDIGO bus and invoke the callback closure when done.
    pub fn attach(
        &mut self,
        f: impl FnMut(&mut Client<'a, M>) -> Result<(), IndigoError> + 'a,
    ) -> Result<(), IndigoError> {
        self.register_request(IndigoRequest::Attach, f)?;

        trace!("'{}' - attaching client...", self);
        // request that the client is attached to the bus
        let ptr = ptr::addr_of!(*self.sys) as *mut indigo_client;
        let result = unsafe { indigo_attach_client(ptr) };
        Bus::sys_to_lib((), result, "indigo_attach_client")
    }

    /// Detach the client from the INDIGO bus and invoke the callback closure when done.
    pub fn detach(
        &mut self,
        f: impl FnMut(&mut Client<'a, M>) -> Result<(), IndigoError> + 'a,
    ) -> Result<(), IndigoError> {
        self.register_request(IndigoRequest::Detach, f)?;

        trace!("'{}' - detaching client...", self);
        let ptr = ptr::addr_of!(*self.sys) as *mut indigo_client;
        let result = unsafe { indigo_detach_client(ptr) };
        Bus::sys_to_lib((), result, "indigo_detach_client")
    }

    /// Define all properties for devices attached to the INDIGO bus.
    pub fn define_properties(&mut self) -> Result<(), IndigoError> {
        debug!(
            "Requesting all properties for '{}'...",
            buf_to_str(self.sys.name)
        );
        unsafe {
            let p = ptr::addr_of!(INDIGO_ALL_PROPERTIES) as *const _ as *mut indigo_property;
            //let p = &mut INDIGO_ALL_PROPERTIES as &mut indigo_property;
            let ptr = ptr::addr_of!(*self.sys) as *mut indigo_client;
            let result = indigo_enumerate_properties(ptr, p);
            Bus::sys_to_lib((), result, "indigo_enumerate_properties")
        }
    }

    /// Connect a device from the INDIGO bus.

    #[named]
    pub fn connect_device(
        &mut self,
        d: &mut ClientDevice,
        f: impl FnOnce(Result<(), IndigoError>) + 'a,
    ) -> Result<(), IndigoError> {
        trace!("Enter '{}'", function_name!());
        let n = d.addr_of_name();
        let ptr = ptr::addr_of!(*self.sys) as *mut indigo_client;
        let result = unsafe { indigo_device_connect(ptr, n) };
        Bus::sys_to_lib((), result, "indigo_device_connect")
    }

    /// Disconnect a device from the INDIGO bus.
    // TODO implement the callback
    #[named]
    pub fn disconnect_device(
        &mut self,
        d: &str,
        f: impl FnOnce(Result<(), IndigoError>) + 'a,
    ) -> Result<(), IndigoError> {
        trace!("Enter '{}'", function_name!());
        trace!("Disconnecting device '{}'...", d);
        let n = d.as_ptr() as *const _ as *mut c_char;
        let ptr = ptr::addr_of!(*self.sys) as *mut indigo_client;
        let result = unsafe { indigo_device_disconnect(ptr, n) };
        Bus::sys_to_lib((), result, "indigo_device_disconnect")
    }

    // -- getters

    fn state<'b>(&'b mut self) -> &'b mut RwLock<ClientState<'a, M>> {
        unsafe { &mut *(self.sys.client_context as *mut RwLock<ClientState<'a, M>>) }
    }

    pub fn name(&self) -> String {
        buf_to_string(self.sys.name)
    }

    /// Passes the client model to the supplied function.
    pub fn model<F, R>(&mut self, f: F) -> Result<R, IndigoError>
    where
        F: FnOnce(&mut M) -> Result<R, IndigoError>,
    {
        let mut lock = self.aquire_write_lock();
        let model = &mut lock.model;
        f(model)
    }

    pub fn blobs(&self) -> Result<Vec<Blob>, IndigoError> {
        let mut blobs = Vec::new();
        unsafe {
            let mut b = (*self.sys).enable_blob_mode_records;
            while !b.is_null() {
                let p = PropertyKey {
                    dev: buf_to_string((*b).device),
                    name: buf_to_string((*b).name),
                };
                match BlobMode::from_u32((*b).mode) {
                    Some(blob) => blobs.push(Blob::new(p, blob)),
                    None => {
                        return Err(IndigoError::Other(format!(
                            "Unknown BlobMode: {}.",
                            (*b).mode
                        )))
                    }
                };
                b = (*b).next;
            }
        }
        Ok(blobs)
    }

    /// Make a callback to the `callback` method registered for this [Client].
    unsafe fn callback(client: *mut indigo_client, expected: &IndigoRequest) -> indigo_result {
        let name = buf_to_str((unsafe { &*client }).name);
        let mut lock = Self::write_lock(client);

        if let Some(request) = &lock.request {
            if request != expected {
                warn!("'{name}' - Indigo callback for {request} request; expected {expected}.")
            }
            // Reset the request.
            lock.request = None;
        } else {
            warn!("'{name}' - INDIGO callback without an active request.");
        }

        let c = Self::try_from(client);
        if let Err(e) = c {
            error!("'{name}' - INDIGO {expected} callback failed as the client state could not be restored.");
            return e.into();
        }

        trace!("Notifying the client requestor...");

        if let Err(e) = (lock.callback)(&mut c.unwrap()) {
            error!("'{name}' - callback notification error: '{}'.", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }
        debug!("'{name}' - notified for {expected}.");

        lock.callback = Box::new(|c| {
            // If this error is returned we are receiving callbacks without first calling attach,
            // i.e. we the callback from INDIGO happens more than once for any given attach request.
            Err(IndigoError::Other(format!(
                "Spurious callback notification!"
            )))
        });

        indigo_result_INDIGO_OK
    }

    // -- libindigo-sys unsafe callback methods that delegate to the CallbackHandler implementation.

    #[named]
    unsafe extern "C" fn on_attach(client: *mut indigo_client) -> indigo_result {
        log_indigo_callback(client, function_name!());
        info!("'{}' - attached.", buf_to_str((*client).name));
        Self::callback(client, &IndigoRequest::Attach)
    }

    #[named]
    unsafe extern "C" fn on_detach(client: *mut indigo_client) -> indigo_result {
        log_indigo_callback(client, function_name!());
        Self::callback(client, &IndigoRequest::Detach)
    }

    // TODO consolidate duplicate code for property define, update, and delete callbacks
    #[named]
    unsafe extern "C" fn on_define_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8,
    ) -> indigo_result {
        log_indigo_callback(client, function_name!());

        let name = buf_to_string((unsafe { &*client }).name);
        let mut lock = Self::write_lock(client);
        let mut c = Self::try_from(client).expect("callback failed");

        let p = Property::new(property);
        let device = buf_to_string((unsafe { &*device }).name);
        let msg = ptr_to_string(message);

        debug!(
            "'{}' - device: '{}' DEFINE property: '{}'",
            name,
            device,
            p.name()
        );
        let result = lock.model.on_define_property(&mut c, device, p, msg);
        Bus::lib_to_sys(result, function_name!())
    }

    #[named]
    unsafe extern "C" fn on_update_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8,
    ) -> indigo_result {
        log_indigo_callback(client, function_name!());

        let name = buf_to_string((unsafe { &*client }).name);
        let mut lock = Self::write_lock(client);
        let mut c = Self::try_from(client).expect("callback failed");

        let p = Property::new(property);
        let device = buf_to_string((unsafe { &*device }).name);
        let msg = ptr_to_string(message);

        debug!(
            "'{}' - device: '{}' UPDATE property: '{}'",
            name,
            device,
            p.name()
        );
        let result = lock.model.on_update_property(&mut c, device, p, msg);
        Bus::lib_to_sys(result, function_name!())
    }

    #[named]
    unsafe extern "C" fn on_delete_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const ::std::os::raw::c_char,
    ) -> indigo_result {
        log_indigo_callback(client, function_name!());

        let name = buf_to_string((unsafe { &*client }).name);
        let mut lock = Self::write_lock(client);
        let mut c = Self::try_from(client).expect("callback failed");

        let p = Property::new(property);
        let device = buf_to_string((unsafe { &*device }).name);
        let msg = ptr_to_string(message);

        debug!(
            "'{}' - device: '{}' DELETE property: '{}'",
            name,
            device,
            p.name()
        );
        let result = lock.model.on_delete_property(&mut c, device, p, msg);
        Bus::lib_to_sys(result, function_name!())
    }

    #[named]
    unsafe extern "C" fn on_send_message(
        client: *mut indigo_client,
        device: *mut indigo_device,
        message: *const ::std::os::raw::c_char,
    ) -> indigo_result {
        log_indigo_callback(client, function_name!());

        let name = buf_to_string((unsafe { &*client }).name);
        let mut lock = Self::write_lock(client);
        let mut c = Self::try_from(client).expect("callback failed");

        let device = buf_to_string((unsafe { &*device }).name);
        let msg = ptr_to_string(message).ok_or_else(|| BusError::NotFound);
        if let Err(e) = msg {
            warn!(
                "INDIGO '{}' callback: null message for client {name} and device {device}.",
                function_name!()
            );
            return e.into();
        }

        let msg = msg.unwrap();
        debug!("'{}' - device: '{}' SEND message: '{}'", name, device, msg);
        let result = lock.model.on_send_message(&mut c, device, msg);
        Bus::lib_to_sys(result, function_name!())
    }

    /// Acquire a lock on the client state held in the `client_context` of sys.
    fn write_lock<'b>(
        client: *mut indigo_client,
    ) -> RwLockWriteGuard<'b, RawRwLock, ClientState<'a, M>> {
        let c = unsafe { &*client };
        let name = buf_to_str(c.name);
        // https://stackoverflow.com/a/24191977/51016
        let state = unsafe { &mut *(c.client_context as *mut RwLock<ClientState<M>>) };

        trace!("'{}' - acquiring model write lock...", name);
        let lock = state.write();
        trace!("'{}' - model write lock acquired.", name);
        lock
    }

    fn aquire_write_lock2<'b>(
        c: &'b mut Client<'a, M>,
    ) -> RwLockWriteGuard<'b, RawRwLock, ClientState<'a, M>> {
        c.state().write()
    }

    fn dec_ref(&mut self) -> u32 {
        let mut lock = self.aquire_write_lock();
        lock.ref_count -= 1;
        return lock.ref_count;
    }
}

unsafe fn log_indigo_callback(client: *mut indigo_client, method: &str) {
    let result = (*client).last_result;
    if result > 0 {
        debug!(
            "'{}' - INDIGO callback for '{}' with result {:?}",
            buf_to_str((*client).name),
            method,
            result
        );
    } else {
        trace!(
            "'{}' - INDIGO callback for '{}' with result {:?}",
            buf_to_str((*client).name),
            method,
            result
        );
    }
}

impl<'a, T: ClientCallbacks<'a>> Display for Client<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = buf_to_str(self.sys.name);
        write!(f, "{name}")
    }
}

// /// Ensure that Client is detached from the INDIGO bus and free any resources associated with the client.
// // TODO figure out how to clean up sys only after all clients are gone, somehow use Arc?
/*
impl<'a,T> Drop for Client<'a,T>
where T: Model<'a> {

    fn drop(&mut self) {
        let ref_count = self.dec_ref();
        drop(lock);
        // TODO drop sys
    }
}
*/

impl<'a, T> TryFrom<*mut indigo_client> for Client<'a, T>
where
    T: ClientCallbacks<'a>,
{
    type Error = BusError;

    fn try_from(value: *mut indigo_client) -> Result<Self, Self::Error> {
        let sys = unsafe { &*value };
        if sys.client_context == ptr::null_mut() {
            let client = buf_to_string(sys.name);
            warn!(
                "Could not restore '{}' client state as client_context is null",
                client
            );
            Err(BusError::NotFound)
        } else {
            Ok(Client {
                sys,
                model: &PhantomData,
            })
        }
    }
}

#[cfg(test)]
mod tests {}
