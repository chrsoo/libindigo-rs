use std::{collections::HashMap, ffi::c_void, fmt::Display, ptr};

use enum_primitive::FromPrimitive;
use function_name::named;
use libindigo_sys::{indigo_detach_device, indigo_device, indigo_device_context, indigo_glock, indigo_result, indigo_result_INDIGO_OK, indigo_version_INDIGO_VERSION_CURRENT};
use log::{debug, error, trace, warn};
use parking_lot::{MappedRwLockWriteGuard, RwLock, RwLockWriteGuard};

use crate::{buf_to_str, buf_to_string, const_to_string, device::GlobalLock, str_to_buf, AccessToken, Bus, BusError, GuardedStringMap, IndigoError, IndigoRequest2, IndigoResult, Property, StringMap};

struct DeviceState<'a> {
    props: StringMap<Property>,
    request: Option<IndigoRequest2<'a, DeviceDriver<'a>>>,
}

impl<'a> DeviceState<'a> {
    fn new_lock_ptr() -> *mut c_void {
        let state = Box::new(RwLock::new(DeviceState {
            props: HashMap::new(),
            request: None,
        }));
        Box::into_raw(state) as *mut _
    }
}

pub struct DeviceDriver<'a> {
    // TODO refactor indigo_device ptr to &'mut ref
    sys: *mut indigo_device,
    // sys_context: &'a mut DeviceContext<'a>,
    state: &'a mut RwLock<DeviceState<'a>>,
}

impl<'a> Display for DeviceDriver<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> DeviceDriver<'a> {
    pub fn new(name: &str) -> DeviceDriver {
        let state = Box::new(RwLock::new(DeviceState {
            props: HashMap::new(),
            request: None,
        }));
        let state_ptr = Box::into_raw(state);

        let sys = Box::new(indigo_device {
            name: str_to_buf(name).unwrap(),
            lock: 0 as indigo_glock,
            is_remote: false,
            gp_bits: 0,
            device_context: ptr::null_mut(),
            private_data: DeviceState::new_lock_ptr(),
            master_device: ptr::null_mut(),
            last_result: indigo_result_INDIGO_OK,
            version: indigo_version_INDIGO_VERSION_CURRENT,
            access_token: 0,
            attach: None,
            enumerate_properties: None,
            change_property: None,
            enable_blob: None,
            detach: Some(DeviceDriver::on_detach),
        });
        let sys_ptr = Box::into_raw(sys);

        DeviceDriver {
            sys: sys_ptr,
            // sys_context: &mut sys_conetxt,
            state: unsafe { &mut *state_ptr },
        }
    }

    // -- getters

    /// device name
    pub fn name(&self) -> String {
        buf_to_string(unsafe { &*self.sys }.name)
    }

    /// `true` if the device is remote
    pub fn is_remote(&self) -> bool {
        unsafe { &*self.sys }.is_remote
    }

    /// Return the device lock.
    pub fn lock(&self) -> GlobalLock {
        GlobalLock {
            tok: unsafe { &*self.sys }.lock,
        }
    }

    /// Return the last result.
    pub fn last_result(&self) -> Option<BusError> {
        BusError::from_u32(unsafe { &*self.sys }.last_result)
    }

    /// Return an AccessToken for synchronized property change.
    pub fn access_token(&self) -> AccessToken {
        AccessToken {
            tok: unsafe { &*self.sys }.access_token,
        }
    }

    #[named]
    pub fn define_property(&mut self, p: Property) -> Result<(), IndigoError> {
        trace!("Enter '{}'", function_name!());
        let mut lock = self.state.write();
        let p = lock.props.entry(p.name()).or_insert(p);
        // TODO notify device listeners
        debug!("Property '{}' defined for '{}'", p.name(), self.name());
        Ok(())
    }

    pub fn property(&self, name: &str) -> Result<MappedRwLockWriteGuard<Property>, IndigoError> {
        let lock = self.state.write();
        if lock.props.contains_key(name) {
            let p = RwLockWriteGuard::map(
                lock,
                // should not panic as we checked that the entry exists
                |s: &mut DeviceState| s.props.get_mut(name).unwrap(),
            );
            Ok(p)
        } else {
            Err(IndigoError::Other(format!(
                "Property '{}' not found.",
                name
            )))
        }
    }

    /// Return an iterator over all properties for this device.
    pub fn properties<'b>(&'b self) -> GuardedStringMap<'b, Property> {
        todo!()
        // GuardedStringMap {
        //     lock: self.state.write(),
        // }
    }

    /// Return a propety using an libindigo-sys constant property name, e.g. [CONNECTION_PROPERTY_NAME].
    pub(crate) fn property_lib(
        &self,
        name: &[u8],
    ) -> Result<MappedRwLockWriteGuard<Property>, IndigoError> {
        let name = const_to_string(name);
        self.property(&name)
    }

    // -- methods

    /// Detach the device  from the INDIGO bus.
    #[named]
    pub fn detach(
        &self,
        f: impl FnMut(IndigoResult<DeviceDriver<'a>>) -> IndigoResult<()> + 'a,
    ) -> Result<(), IndigoError>
// where F: FnOnce(Result<(), IndigoError>) + 'a, // TODO find out if the lifetime specifier really is needed!
    {
        trace!("Enter '{}'", function_name!());
        let r = self.request(IndigoRequest2::Detach(Box::new(f)))?;
        trace!("Disconnecting device '{}'...", self);
        let result = unsafe {
            let ptr = ptr::addr_of!(*self.sys) as *mut indigo_device;
            indigo_detach_device(ptr)
        };
        Bus::sys_to_lib((), result, "indigo_detach_device")
    }

    pub fn change_property(&self) -> Result<(), IndigoError> {
        // self.sys.change_property();
        todo!()
    }

    /// Returns `IndigoError::Other`if the source and target devices do not share
    /// the same name or if they refer to different `indigo_device` objects.
    pub(crate) fn assert_same(&self, d: DeviceDriver) -> Result<(), IndigoError> {
        if self.name() != d.name() {
            return Err(IndigoError::Other(
                "Source and target do not share the same name.".to_string(),
            ));
        }

        if ptr::eq(self.sys, d.sys) {
            Ok(())
        } else {
            Err(IndigoError::Other(
                "Indigo Device uses same name but different indigo_device objects".to_string(),
            ))
        }
    }

    fn request(&self, request: IndigoRequest2<'a, DeviceDriver<'a>>) -> Result<(), IndigoError> {
        let mut lock = self.state.write();
        if let Some(request) = &lock.request {
            return Err(IndigoError::Other(format!(
                "{} request in progress for device '{}'",
                request,
                self.name(),
            )));
        }
        lock.request = Some(request);
        Ok(())
    }

    // -- INDIGO unsafe callback methods

    #[named]
    unsafe extern "C" fn on_detach(device: *mut indigo_device) -> indigo_result {
        trace!("INDIGO callback '{}'", function_name!());

        // lock the device state
        let state: RwLock<DeviceState> =
            ptr::read((*device).private_data as *mut RwLock<DeviceState>);
        let mut lock = state.write();

        // restore the device
        let d = DeviceDriver::try_from(device);
        if let Err(e) = d {
            error!(
                "Could not restore the Device private_data in indigo_device: {}",
                e
            );
            return BusError::Failed as indigo_result;
        }
        let d = d.unwrap();

        // presumable the last result is that related to the INDIGO on_detach callback...
        let result = Bus::sys_to_lib(d, (*device).last_result, function_name!());

        // invoke the callback method provided it has been set previously
        if let Some(request) = &mut lock.request {
            let result = request.callback(result);
            Bus::lib_to_sys(result, function_name!())
        } else {
            warn!(
                "Spurius callback without a registered request for device '{}'",
                buf_to_str((*device).name)
            );
            BusError::Failed as indigo_result
        }
    }
}

impl<'a> TryFrom<*mut indigo_device> for DeviceDriver<'a> {
    type Error = IndigoError;

    fn try_from(value: *mut indigo_device) -> Result<Self, Self::Error> {
        if value == ptr::null_mut() {
            return Err(IndigoError::Other("indigo_device is null".to_string()));
        }
        debug!("device addr {:p}", value);

        let mut device = unsafe { ptr::read(value) };

        // connect callbacks
        if let None = device.detach {
            device.detach = Some(DeviceDriver::on_detach);
        }
        // TODO set remaining callbacks

        // TODO restore the DeviceContext
        // read the device context from the raw pointer
        //let ptr = device.device_context;
        // let context = DeviceContext::try_from(ptr)?;

        // create the device state if it has not yet been defined
        if device.private_data == ptr::null_mut() {
            trace!(
                "creating new state for device '{}'",
                buf_to_str(device.name)
            );
            device.private_data = DeviceState::new_lock_ptr();
        }

        Ok(DeviceDriver {
            sys: value,
            state: unsafe { &mut *(device.private_data as *mut _) },
        })
    }
}

pub struct DeviceContext<'a> {
    sys: &'a mut indigo_device_context,
}

impl<'a> TryFrom<*mut c_void> for DeviceContext<'a> {
    type Error = IndigoError;

    fn try_from(value: *mut c_void) -> Result<Self, Self::Error> {
        if value == ptr::null_mut() {
            return Err(IndigoError::Other(
                "indigo_device_contetxt pointer is null".to_string(),
            ));
        }
        // let sys = unsafe { &mut ptr::read(value as *mut indigo_device_context) };
        let sys = unsafe { &mut *(value as *mut _) };
        Ok(DeviceContext { sys })
    }
}
