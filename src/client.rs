use std::{
    collections::HashMap, hash::Hash, os::raw::c_void, ptr::{self, addr_of, addr_of_mut}, slice::Iter, sync::RwLock
};

use super::bus::Bus;
use crate::*;
use bus::map_indigo_result;
use device::Device;
use libindigo_sys::{self, *};
use log::{debug, error, info};
use property::{Blob, BlobMode};

/// Callback methods to handle INDIGO bus events.
pub trait CallbackHandler<'a> {
    /// Called when the client is attached.
    fn on_client_attach(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
    ) -> Result<(), IndigoError> {
        debug!("... client attached.");
        Ok(())
    }

    /// Called each time the client has been detached.
    fn on_client_detach(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
    ) -> Result<(), IndigoError> {
        debug!("... client detached.");
        Ok(())
    }

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
        d: Device,
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

/// Client to manage devices attached to the INDIGO bus.
pub struct Client<'a, T: CallbackHandler<'a>> {
    /// System record for INDIGO clients.
    sys: indigo_client,
    devices: RwLock<HashMap<String,Device<'a>>>,
    pub handler: &'a mut T,
    /*
    /// `true`` if the _client_ is a remote object
    remote: bool,

    /// Result of last bus operation.
    error: Option<IndigoError>,

    /// Client version.
    version: indigo_version,

    #[doc = "< any client specific data"]
    pub client_context: *mut ::std::os::raw::c_void,
    #[doc = "< enable blob mode"]
    pub enable_blob_mode_records: *mut indigo_enable_blob_mode_record,
     */
}

impl<'a,T> Client<'a, T>
where T: CallbackHandler<'a> {

    /// Create a new, detached client.
    pub fn new(name: &str, handler: &'a mut T) -> Result<Box<Self>, IndigoError> {
        let indigo_client = indigo_client {
            name: str_to_buf(name)?,              // client name
            is_remote: false,                     // is this a remote client "no" - this is us
            client_context: std::ptr::null_mut(), // points to the client, initially null pointer...
            // ...to handle circular reference
            last_result: indigo_result_INDIGO_OK, // result of last bus operation
            // - we just initialize it with ok
            version: indigo_version_INDIGO_VERSION_CURRENT, // the client speaks current indigo version
            enable_blob_mode_records: ptr::null_mut(),      // BLOB mode records -> Set this to NULL
            // The unsafe callback methods on the Client delegate to safe CallbackHandler implementation.
            attach: Some(Self::on_attach),
            define_property: Some(Self::on_define_property),
            update_property: Some(Self::on_update_property),
            delete_property: Some(Self::on_delete_property),
            send_message: Some(Self::on_send_message),
            detach: Some(Self::on_detach),
        };

        // https://users.rust-lang.org/t/unwanted-copies-of-values-when-using-unsafe-pointers-for-ffi-bindings/115443
        // Put the Client in a Box to ensure a stable reference.
        let mut client = Box::new(Client {
            sys: indigo_client,
            devices: RwLock::new(HashMap::new()),
            handler: handler,
        });
        // store a raw pointer to the safe Client on the unsafe indigo_client's context.
        client.sys.client_context = (&mut *client) as *mut _ as *mut c_void;

        Ok(client)
    }

    /// Attach the client to the INDIGO bus.
    pub fn attach(&mut self) -> Result<(), IndigoError> {
        info!("Attaching client '{}'...", buf_to_string(self.sys.name));
        let c = addr_of_mut!(self.sys);
        let result = unsafe { indigo_attach_client(c) };
        map_indigo_result(result, "indigo_attach_client")
    }

    /// Detach the client from the INDIGO bus.
    pub fn detach(&mut self) -> Result<(), IndigoError> {
        info!("Detaching client '{}'...", buf_to_string(self.sys.name));
        let c = addr_of_mut!(self.sys);
        let result = unsafe { indigo_detach_client(c) };
        map_indigo_result(result, "indigo_detach_client")
    }

    /// Get all properties from the devices attached to the INDIGO bus.
    pub fn get_all_properties(&mut self) -> Result<(), IndigoError> {
        debug!(
            "Getting all properties for '{}'...",
            buf_to_string(self.sys.name)
        );
        unsafe {
            let c = addr_of_mut!(self.sys);
            let p = std::ptr::addr_of!(INDIGO_ALL_PROPERTIES) as *const _ as *mut indigo_property;
            //let p = &mut INDIGO_ALL_PROPERTIES as &mut indigo_property;
            let result = indigo_enumerate_properties(c, p);
            map_indigo_result(result, "indigo_enumerate_properties")
        }
    }

    // -- getters

    pub fn name(self) -> &'a str {
        buf_to_str(self.sys.name)
    }

    pub fn blobs(&self) -> Result<Vec<Blob>, IndigoError> {
        let mut blobs = Vec::new();
        unsafe{
            let mut b = self.sys.enable_blob_mode_records;
            while !b.is_null() {
                let p = PropertyKey {
                    dev: buf_to_string((*b).device),
                    name: buf_to_string((*b).name),

                };
                match BlobMode::from_u32((*b).mode) {
                    Some(blob) => blobs.push(Blob::new(p, blob)),
                    None => return Err(IndigoError::Other(format!("Unknown BlobMode: {}", (*b).mode)))
                };
                b = (*b).next;
            }
        }
        Ok(blobs)
    }

    // -- libindigo-sys unsafe callback methods that delegate to the CallbackHandler implementation.

    unsafe extern "C" fn on_attach(client: *mut indigo_client) -> indigo_result {
        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);

        if let Err(e) = c1.handler.on_client_attach(c2) {
            error!("Handler error: '{}'", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }
        indigo_result_INDIGO_OK
    }

    // TODO consolidate duplicate code for property define, update, and delete callbacks
    unsafe extern "C" fn on_define_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8,
    ) -> indigo_result {
        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);
        let d = Device::new(&mut c1.sys, device);
        let p = Property::new(property);
        let msg = ptr_to_string(message);

        if let Err(e) = c1.handler.on_define_property(c2, d, p, msg) {
            error!("Handler error: '{}'", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }

        indigo_result_INDIGO_OK
    }

    fn upsert_device(&self, d: *mut indigo_device) -> Option<&'a Device<'a>> {
        let mut devices = self.devices.write().unwrap();
        let name = buf_to_string((unsafe { &*d }).name);
        let entry = devices.entry(name).or_insert(Device::new(self.sys, d));

        todo!()
    }

    unsafe extern "C" fn on_update_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8,
    ) -> indigo_result {
        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);
        let d = c1.upsert_device(device);
        let d = Device::new(&mut c1.sys, device);
        let p = Property::new(property);
        let msg = ptr_to_string(message);

        if let Err(e) = c1.handler.on_update_property(c2, d, p, msg) {
            error!("Handler error: '{}'", e);
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
        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);
        let d = Device::new(&mut c1.sys, device);
        let p = Property::new(property);
        let msg = ptr_to_string(message);

        if let Err(e) = c1.handler.on_delete_property(c2, d, p, msg) {
            error!("Handler error: '{}'", e);
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
        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);
        let d = Device::new(&mut c1.sys, device);
        let msg = ptr_to_string(message).unwrap();

        if let Err(e) = c1.handler.on_send_message(c2, d, msg) {
            error!("Handler error: '{}'", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }

        indigo_result_INDIGO_OK
    }

    unsafe extern "C" fn on_detach(client: *mut indigo_client) -> indigo_result {
        let c1: &mut Client<T> = get_client(client);
        let c2: &mut Client<T> = get_client(client);

        if let Err(e) = c1.handler.on_client_detach(c2) {
            error!("Handler error: '{}'", e);
            if let IndigoError::Bus(b) = e {
                return b.into();
            }
            return BusError::Failed.into();
        }

        indigo_result_INDIGO_OK
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

// /// Ensure that Client is detached from the INDIGO bus and free any resources associated with the client.
// // TODO find out if detaching is a good idea and add any missing resources that needs to be cleaned up.
// impl<T:CallbackHandler> Drop for Client<T> {
//     fn drop(&mut self) {
//         if let Err(e) = self.detach() {
//             warn!("Could not drop Client: {}.", e)
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        visited: bool,
    }
    impl<'a> CallbackHandler<'a> for Test {
        fn on_client_attach(
            &mut self,
            c: &mut Client<'a, impl CallbackHandler<'a>>,
        ) -> Result<(), IndigoError> {
            self.visited = true;
            Ok(())
        }
    }

    #[test]
    fn test_callback() -> Result<(), IndigoError> {
        let handler = &mut Test { visited: false };
        let mut client = Client::new("test", handler)?;
        // client.attach()?;

        let ptr = core::ptr::addr_of_mut!(client.sys);
        unsafe {
            let r = Client::<Test>::on_attach(ptr);
            map_indigo_result(r, "on_attach")?;
        };

        assert!(client.handler.visited);
        Ok(())
    }
}
