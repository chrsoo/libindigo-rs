use std::{
    collections::HashMap, mem::forget, ops::Deref, os::raw::c_void, ptr::{self}, sync::{Arc, Mutex, MutexGuard, RwLock}
};

use crate::*;
use bus::map_indigo_result;
use device::Device;
use libindigo_sys::{self, *};
use log::{debug, error, info, trace};
use property::{Blob, BlobMode};

/// Callback methods to handle INDIGO bus events.
pub trait CallbackHandler<'a> {
    /// Called each time the property of a device is defined or its definition requested.
    fn on_define_property(
        &'a mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
        d: Device<'a>,
        p: Property<'a>,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; DEFINED with message '{:?}' defined for ",
            d.name(),
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time a property is updated for a device.
    fn on_update_property(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
        d: Arc<Device>,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; UPDATED with message '{:?}' defined for ",
            d.name(),
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time a property is deleted.
    fn on_delete_property(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
        d: Device,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; DELETED with message '{:?}' defined for ",
            d.name(),
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time message has been sent.
    // TODO Move to client and use closure instead.
    fn on_send_message(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
        d: Device,
        msg: String,
    ) -> Result<(), IndigoError> {
        debug!("Message '{:?}' SENT", msg);
        Ok(())
    }
}


type Callback<'a,T> = Box<dyn FnMut(&mut Client<'a,T>) -> Result<(),IndigoError> + 'a>;

/// Client to manage devices attached to the INDIGO bus.
pub struct Client<'a, T: CallbackHandler<'a>> {
    /// System record for INDIGO clients.
    sys: *mut indigo_client,
    request: Mutex<Option<IndigoRequest>>,
    callback: Box<dyn FnMut(&mut Client<'a,T>) -> Result<(),IndigoError> + 'a>,
    devices: RwLock<HashMap<String, Arc<Device<'a>>>>,
    pub handler: &'a mut T,
}

impl<'a, T> Client<'a, T>
where
    T: CallbackHandler<'a>,
{
    /// Create a new, detached client.
    pub fn new(name: &str, handler: &'a mut T) -> Result<Box<Self>, IndigoError> {
        // Put the indigo_client in a Box to ensure a stable reference, cf.
        // https://users.rust-lang.org/t/unwanted-copies-of-values-when-using-unsafe-pointers-for-ffi-bindings/115443
        let mut indigo_client = Box::new(indigo_client {
            name: str_to_buf(name)?,
            // this client is running locally
            is_remote: false,
            // initially null pointer to handle the circular reference
            client_context: std::ptr::null_mut(),
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

        // https://users.rust-lang.org/t/unwanted-copies-of-values-when-using-unsafe-pointers-for-ffi-bindings/115443
        // put the Client in a Box to ensure a stable reference.
        let mut client = Box::new(Client {
            // dereference the Box
            sys: (&mut *indigo_client) as *mut _ as *mut indigo_client,
            request: Mutex::new(None),
            callback: Box::new(|c|
                // If this callback is invoked it means that we received an INDIGO callback
                // without having called invoke or connect on the client, or that the code
                // has failed to store the callback on the right client object.
                Err(IndigoError::Other(format!("Initial callback placeholder should never be called!")))
            ),
            // sys: addr_of_mut!(indigo_client),
            devices: RwLock::new(HashMap::new()),
            handler: handler,
        });
        // store a raw pointer to the safe Client on the unsafe indigo_client's context.
        indigo_client.client_context = (&mut *client) as *mut _ as *mut c_void;

        // prevent rust from reclaiming the memory of indigo_client when dropping the reference on exit.
        forget(indigo_client);

        Ok(client)
    }

    /// Attach the client to the INDIGO bus.
    // pub fn attach(&self) -> Result<(), IndigoError> {
    //     let name =  buf_to_str(unsafe { &*self.sys }.name);
    //     info!("Attaching client '{}'...", name);
    //     let result = unsafe { indigo_attach_client(self.sys) };
    //     map_indigo_result(result, "indigo_attach_client")
    // }

    /// Attach the client to the INDIGO bus and invoke the callback closure when done. In case
    /// of a callback error, the result will be logged.
    // TODO Define what happens if an error is returned to the INDIGO bus and adapt the code.
    pub fn attach(&mut self, f: impl FnMut(&mut Client<'a,T>) -> Result<(),IndigoError> + 'a)
    -> Result<(), IndigoError> {
        trace!("Attaching client '{}'...", self);

        let mut r = self.request.lock().unwrap();
        if let Some(request) = &mut *r {
            return Err(IndigoError::Other(format!(
                "{} request in progress for client '{}'.", request, self,
            )));
        }
        *r = Some(IndigoRequest::Attach);
        self.callback = Box::new(f);
        drop(r);

        let result = unsafe { indigo_attach_client(self.sys) };
        map_indigo_result(result, "indigo_attach_client")
    }

    /// Detach the client from the INDIGO bus.
    pub fn detach(&mut self, f: impl FnMut(&mut Client<'a,T>) -> Result<(),IndigoError> + 'a)
    -> Result<(), IndigoError> {
        trace!("Detaching all devices '{}'...", self);
        // TODO detach all attached devices

        trace!("Detaching client '{}'...", self);
        let mut r = self.request.lock().unwrap();
        if let Some(request) = &mut *r {
            return Err(IndigoError::Other(format!(
                "{} request in progress for client '{}'.", request, self,
            )));
        }
        *r = Some(IndigoRequest::Detach);
        self.callback = Box::new(f);
        drop(r);

        let result = unsafe { indigo_detach_client(self.sys) };
        map_indigo_result(result, "indigo_detach_client")
    }

    /// Get all properties from the devices attached to the INDIGO bus.
    pub fn get_all_properties(&mut self) -> Result<(), IndigoError> {
        let name = buf_to_str(unsafe { &*self.sys }.name);
        debug!("Getting all properties for '{}'...", name);
        unsafe {
            let p = std::ptr::addr_of!(INDIGO_ALL_PROPERTIES) as *const _ as *mut indigo_property;
            //let p = &mut INDIGO_ALL_PROPERTIES as &mut indigo_property;
            let result = indigo_enumerate_properties(self.sys, p);
            map_indigo_result(result, "indigo_enumerate_properties")
        }
    }

    // -- getters

    pub fn name(self) -> &'a str {
        buf_to_str(unsafe { &*self.sys }.name)
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

    // -- libindigo-sys unsafe callback methods that delegate to the CallbackHandler implementation.

    unsafe extern "C" fn on_attach(client: *mut indigo_client) -> indigo_result {
        trace!("INDIGO 'on_attach' callback.");

        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);

        info!("Client '{}' attached.", c1);

        // Lock and unwrap the request object.
        trace!("Obtaining request lock...");
        let mut request = c1.request.lock().unwrap();
        trace!("Request lock obtained.");
        if let Some(r) = request.deref() {
            match r {
                IndigoRequest::Attach => (),
                _ => warn!("Indigo callback 'on_attach' called for {} request; expected {}.", r, IndigoRequest::Attach),
            }
            // Reset the request.
            *request = None;
        } else {
            warn!("INDIGO 'on_attach' called without an active request.");
        }

        trace!("Notifying the client attach requestor...");
        if let Err(e) = (c1.callback)(c2) {
            error!("Callback notification error: '{}'.", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }
        debug!("Client attach requestor notified.");

        // If this error is called we are receiving callbacks without first calling attach,
        // i.e. we the callback from INDIGO happens more than once for any given attach request.
        c1.callback = Box::new(|c1|
            Err(IndigoError::Other(format!("Spurious callback notification!")))
        );

        indigo_result_INDIGO_OK
    }

    unsafe extern "C" fn on_detach(client: *mut indigo_client) -> indigo_result {
        trace!("INDIGO 'on_detach' callback.");

        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);

        info!("Client '{}' detached.", &c1);

        // Lock and unwrap the request object.
        trace!("Obtaining request lock...");
        let mut request = c1.request.lock().unwrap();
        trace!("Request lock obtained.");
        if let Some(r) = request.deref() {
            match r {
                IndigoRequest::Detach => (),
                _ => warn!("Indigo callback 'on_detach' called for {} request; expected {}.", r, IndigoRequest::Detach),
            }
            // Reset the request.
            *request = None;
        } else {
            warn!("INDIGO 'on_detach' called without an active request");
        }

        trace!("Notifying the client detach requestor...");
        if let Err(e) = (c1.callback)(c2) {
            error!("Callback notification error: '{}'.", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }
        debug!("Client detach requestor notified.");

        // If this error is called we are receiving callbacks without first calling attach,
        // i.e. we the callback from INDIGO happens more than once for any given attach request.
        c1.callback = Box::new(|c1|
            Err(IndigoError::Other(format!("Spurious callback notification!")))
        );

        indigo_result_INDIGO_OK
    }

    // TODO consolidate duplicate code for property define, update, and delete callbacks
    unsafe extern "C" fn on_define_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8,
    ) -> indigo_result {
        trace!("INDIGO 'on_define_property' callback.");

        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);
        let d = Device::new(c1.sys, device);
        let p = Property::new(property);
        let msg = ptr_to_string(message);

        if let Err(e) = c1.handler.on_define_property(c2, d, p, msg) {
            error!("Handler error: '{}'.", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }

        indigo_result_INDIGO_OK
    }

    fn upsert_device(&self, d: *mut indigo_device) -> Arc<Device<'a>> {
        let mut devices = self.devices.write().unwrap();
        let name = buf_to_string((unsafe { &*d }).name);
        devices.entry(name).or_insert(Arc::new(Device::new(self.sys, d))).clone()
    }

    unsafe extern "C" fn on_update_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8,
    ) -> indigo_result {
        trace!("INDIGO 'on_update_property' callback.");

        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);
        let d = c1.upsert_device(device);
        let p = Property::new(property);
        let msg = ptr_to_string(message);

        if let Err(e) = c1.handler.on_update_property(c2, d, p, msg) {
            error!("Handler error: '{}'.", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }

        indigo_result_INDIGO_OK
    }

    unsafe extern "C" fn on_delete_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const ::std::os::raw::c_char,
    ) -> indigo_result {
        trace!("INDIGO 'on_delete_property' callback.");

        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);
        let d = Device::new(c1.sys, device);
        let p = Property::new(property);
        let msg = ptr_to_string(message);

        if let Err(e) = c1.handler.on_delete_property(c2, d, p, msg) {
            error!("Handler error: '{}'.", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }

        indigo_result_INDIGO_OK
    }

    unsafe extern "C" fn on_send_message(
        client: *mut indigo_client,
        device: *mut indigo_device,
        message: *const ::std::os::raw::c_char,
    ) -> indigo_result {
        trace!("INDIGO 'on_send_message' callback.");

        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);
        let d = Device::new(c1.sys, device);
        let msg = ptr_to_string(message).unwrap();

        if let Err(e) = c1.handler.on_send_message(c2, d, msg) {
            error!("Handler error: '{}'.", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }

        indigo_result_INDIGO_OK
    }

    fn request(&self, request: IndigoRequest) -> MutexGuard<Option<IndigoRequest>>{
        // Lock and unwrap the request object.
        trace!("Obtaining request lock...");
        let mut request = self.request.lock().unwrap();
        trace!("Request lock obtained.");
        if let Some(r) = request.deref() {
            match r {
                IndigoRequest::Detach => (),
                _ => warn!("Indigo callback 'on_detach' called for {} request; expected {}.", r, IndigoRequest::Detach),
            }
            // Reset the request.
            *request = None;
        } else {
            warn!("INDIGO 'on_attach' called without an active request.");
        }
        request
    }
}

/// Return a mutable reference to `Client` from the pointer stored in the `context` field of `indigo_client`.
unsafe fn get_client<'a, T>(client: *mut indigo_client) -> &'a mut Client<'a, T>
where
    T: CallbackHandler<'a>,
{
    // https://stackoverflow.com/a/24191977/51016
    let ptr = (*client).client_context;
    let c: &mut Client<T> = &mut *(ptr as *mut Client<T>);
    c
}

impl<'a,T: CallbackHandler<'a>> Display for Client<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name =  buf_to_str(unsafe { &*self.sys }.name);
        write!(f, "{name}")
    }
}

// /// Ensure that Client is detached from the INDIGO bus and free any resources associated with the client.
// // TODO find out if detaching is a good idea and add any missing resources that needs to be cleaned up.
impl<'a, T: CallbackHandler<'a>> Drop for Client<'a, T> {
    fn drop(&mut self) {
        unsafe { free(self.sys as *mut c_void) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        visited: bool,
    }
    impl<'a> CallbackHandler<'a> for Test { }

    #[test]
    fn test_callback() -> Result<(), IndigoError> {
        let handler = &mut Test { visited: false };
        let client = Client::new("test", handler)?;
        // client.attach()?;

        let ptr = client.sys;
        unsafe {
            let r = Client::<Test>::on_attach(ptr);
            map_indigo_result(r, "on_attach")?;
        };

        assert!(client.handler.visited);
        Ok(())
    }
}
