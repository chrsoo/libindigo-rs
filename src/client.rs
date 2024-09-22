use std::{marker::PhantomData, os::raw::c_void, ptr, sync::Arc};

use crate::*;
use device::Device;
use function_name::named;
use libindigo_sys::{self, *};
use log::{debug, error, info, trace};
use parking_lot::{lock_api::RwLockWriteGuard, RawRwLock, RwLock};
use property::{Blob, BlobMode};

struct ClientState<'a, T>
where
    T: Model<'a>,
{
    model: T,
    request: Option<IndigoRequest>,
    callback: Box<dyn FnMut(&mut Client<'a, T>) -> Result<(), IndigoError> + 'a>,
    ref_count: u32,
}

/// Device as seen from the Client.
pub struct ClientDevice {
    name: String,
    props: RwLock<HashMap<String, Property>>,
}

impl Device for ClientDevice {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ClientDevice {
    pub fn define_property(property: Property) {
        todo!()
    }
}

impl From<*mut indigo_device> for ClientDevice {
    fn from(value: *mut indigo_device) -> Self {
        ClientDevice {
            name: buf_to_string((unsafe { &*value }).name),
            props: RwLock::new(HashMap::new()),
        }
    }
}

/// Client to manage devices attached to the INDIGO [Bus].
pub struct Client<'a, T>
where
    T: Model<'a>,
{
    /// System record for INDIGO clients.
    sys: &'a indigo_client,
    model: &'a PhantomData<T>,
}

impl<'a, M> Client<'a, M>
where
    M: Model<'a, M = M> + 'a,
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
        trace!("'{self}' - registering {request} request.");
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
        d: &mut DeviceImpl,
        f: impl FnOnce(Result<(), IndigoError>) + 'a,
    ) -> Result<(), IndigoError> {
        trace!("Enter '{}'", function_name!());
        let n = d.addr_of_name();
        let ptr = ptr::addr_of!(*self.sys) as *mut indigo_client;
        let result = unsafe { indigo_device_connect(ptr, n) };
        Bus::sys_to_lib((), result, "indigo_device_connect")
    }

    /// Disconnect a device from the INDIGO bus.
    #[named]
    pub fn disconnect_device(
        &mut self,
        d: &mut DeviceImpl,
        f: impl FnOnce(Result<(), IndigoError>) + 'a,
    ) -> Result<(), IndigoError> {
        trace!("Enter '{}'", function_name!());
        trace!("Disconnecting device '{}'...", d);
        let n = d.addr_of_name();
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
                warn!("Indigo callback called for a {request} request for '{name}'; expected {expected}.")
            }
            // Reset the request.
            lock.request = None;
        } else {
            warn!("INDIGO callback for '{name}' without an active request.");
        }

        let c = Self::try_from(client);
        if let Err(e) = c {
            error!("INDIGO {expected} callback for '{name}' failed as the client state could not be restored.");
            return e.into();
        }

        trace!("Notifying the client requestor...");

        if let Err(e) = (lock.callback)(&mut c.unwrap()) {
            error!("Callback notification error: '{}'.", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }
        debug!("Client '{name}' notified for {expected}.");

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
        trace!("INDIGO '{}' callback.", function_name!());
        info!("'{}' attached.", buf_to_str((*client).name));
        Self::callback(client, &IndigoRequest::Attach)
    }

    #[named]
    unsafe extern "C" fn on_detach(client: *mut indigo_client) -> indigo_result {
        trace!("INDIGO '{}' callback.", function_name!());
        info!("'{}' detached.", buf_to_str((*client).name));
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

        trace!("INDIGO '{}' callback.", function_name!());

        let name = buf_to_string((unsafe { &*client }).name);
        let mut lock = Self::write_lock(client);
        let mut c = Self::try_from(client).expect("callback failed");

        let p = Property::new(property);
        let device = buf_to_string((unsafe { &*device }).name);
        let msg = ptr_to_string(message);

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

        trace!("INDIGO '{}' callback.", function_name!());

        let name = buf_to_string((unsafe { &*client }).name);
        let mut lock = Self::write_lock(client);
        let mut c = Self::try_from(client).expect("callback failed");

        let p = Property::new(property);
        let device = buf_to_string((unsafe { &*device }).name);
        let msg = ptr_to_string(message);

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

        trace!("INDIGO '{}' callback.", function_name!());

        let name = buf_to_string((unsafe { &*client }).name);
        let mut lock = Self::write_lock(client);
        let mut c = Self::try_from(client).expect("callback failed");

        let p = Property::new(property);
        let device = buf_to_string((unsafe { &*device }).name);
        let msg = ptr_to_string(message);

        let result = lock.model.on_delete_property(&mut c, device, p, msg);
        Bus::lib_to_sys(result, function_name!())
    }

    #[named]
    unsafe extern "C" fn on_send_message(
        client: *mut indigo_client,
        device: *mut indigo_device,
        message: *const ::std::os::raw::c_char,
    ) -> indigo_result {

        trace!("INDIGO '{}' callback.", function_name!());

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

        let result = lock.model.on_send_message(&mut c, device, msg.unwrap());
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

impl<'a, T: Model<'a>> Display for Client<'a, T> {
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
    T: Model<'a>,
{
    type Error = BusError;

    fn try_from(value: *mut indigo_client) -> Result<Self, Self::Error> {
        let sys = unsafe { &*value };
        if sys.client_context == ptr::null_mut() {
            let client = buf_to_string(sys.name);
            warn!("Could not restore '{}' client state as client_context is null", client);
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
