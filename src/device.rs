use function_name::named;
use parking_lot::{MappedRwLockWriteGuard, Mutex, RwLock, RwLockWriteGuard};
use std::{
    collections::HashMap,
    ffi::c_void,
    ptr::{self, addr_of},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::*;
use enum_primitive::*;
use libindigo_sys::{self, *};
use log::{debug, error, trace, warn};

pub trait Device {
    fn name(&self) -> &str;
}

struct DeviceState<'a> {
    props: StringMap<Property>,
    request: Option<IndigoRequest2<'a, DeviceImpl<'a>>>,
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

pub struct DeviceImpl<'a> {
    // TODO refactor indigo_device ptr to &'mut ref
    sys: *mut indigo_device,
    // sys_context: &'a mut DeviceContext<'a>,
    state: &'a mut RwLock<DeviceState<'a>>,
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq, EnumIter)]
#[repr(u32)]
// sys-doc: Device interface (value should be used for INFO_DEVICE_INTERFACE_ITEM->text.value)
/// Each interface defines a set of well-known properties.
pub enum Interface  {
    Mount = indigo_device_interface_INDIGO_INTERFACE_MOUNT,
    CCD = indigo_device_interface_INDIGO_INTERFACE_CCD,
    Guider = indigo_device_interface_INDIGO_INTERFACE_GUIDER,
    Foduser = indigo_device_interface_INDIGO_INTERFACE_FOCUSER,
    Wheel = indigo_device_interface_INDIGO_INTERFACE_WHEEL,
    Dome = indigo_device_interface_INDIGO_INTERFACE_DOME,
    GPS = indigo_device_interface_INDIGO_INTERFACE_GPS,
    AdaptiveOptics = indigo_device_interface_INDIGO_INTERFACE_AO,
    Rotator = indigo_device_interface_INDIGO_INTERFACE_ROTATOR,
    Agent = indigo_device_interface_INDIGO_INTERFACE_AGENT,
    Auxiliary = indigo_device_interface_INDIGO_INTERFACE_AUX,
    AuxJoystic = indigo_device_interface_INDIGO_INTERFACE_AUX_JOYSTICK,
    Shutter = indigo_device_interface_INDIGO_INTERFACE_AUX_SHUTTER,
    PowerBox = indigo_device_interface_INDIGO_INTERFACE_AUX_POWERBOX,
    SQM = indigo_device_interface_INDIGO_INTERFACE_AUX_SQM,
    DustCap = indigo_device_interface_INDIGO_INTERFACE_AUX_DUSTCAP,
    LightBox = indigo_device_interface_INDIGO_INTERFACE_AUX_LIGHTBOX,
    Weather = indigo_device_interface_INDIGO_INTERFACE_AUX_WEATHER,
    /// General Purpose IO auxiliary interface
    GPIO = indigo_device_interface_INDIGO_INTERFACE_AUX_GPIO,
}
}

impl Interface {
    /// map an INDIGO interface `String` value to list of `Interface` variants.
    fn map(ifs: String) -> Vec<Interface> {
        // Convert the interface string to a bitmap unsigned integer.
        let ifs = Interface::convert(ifs);
        let mut vec = Vec::new();
        for i in Interface::iter() {
            if i.matches(ifs) {
                vec.push(i);
            }
        }
        vec
    }
    /// Match the INDIGO interface bitmap against a specific interface.
    fn matches(self, ifs: u32) -> bool {
        (self as u32 & ifs) == self as u32
    }

    /// Convert an INDIGO interface `String` to an u32 bitmap.
    fn convert(ifs: String) -> u32 {
        unsafe { atoi(ptr::addr_of!(ifs) as *const _) as u32 }
    }
}

pub struct GlobalLock {
    tok: indigo_glock,
}

impl<'a> Display for DeviceImpl<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> DeviceImpl<'a> {
    pub fn new(name: &str) -> DeviceImpl {
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
            detach: Some(DeviceImpl::on_detach),
        });
        let sys_ptr = Box::into_raw(sys);

        DeviceImpl {
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

    /// Returns [Ok](PropertyState::Ok) if the device is in a usable state, else the corresponding [PropertyState]
    /// is returned as an error. If the device is [Ok](PropertyState::Ok), the returned value can be used
    /// to determine the device's connection status.
    /// ```
    /// let d = Device::new("TestDevice");
    /// map d.connection_status() {
    ///     Ok(true) => info!("Device {d} is CONNECTED."),
    ///     Ok(false) => info!("Device {d} is DISCONNECTED."),
    ///     Err(state) => warn!("Device {d} is in the {state}"),
    /// }
    /// ```
    pub fn connection_status(&self) -> Result<bool, IndigoError> {
        let property_name = const_to_string(CONNECTION_PROPERTY_NAME);
        let connection = self.property(property_name.as_ref())?;
        if connection.state() != PropertyState::Ok {
            return Err(IndigoError::Other(format!("{:?} ", connection.state())));
        }

        let disconnected_item = const_to_string(CONNECTION_DISCONNECTED_ITEM_NAME);
        let connected_item = const_to_string(CONNECTION_CONNECTED_ITEM_NAME);
        for i in connection.items() {
            if i.name == connected_item || i.name == disconnected_item {
                return match i.value {
                    PropertyValue::Switch(b) => {
                        //
                        Ok(if i.name == connected_item { b } else { !b })
                    }
                    _ => {
                        let msg = format!(
                            "Illegal '{}' property value, expected a switch for item '{}' and not '{:?}'",
                            property_name, i.name, i.value
                        );
                        warn!("{}", msg);
                        return Err(IndigoError::Other(msg));
                    }
                };
            }
        }
        let msg = format!(
            "Illegal '{}' property definition, could not find a '{}' or '{}' item",
            property_name, connected_item, disconnected_item
        );
        warn!("{}", msg);
        return Err(IndigoError::Other(msg));
    }

    pub(crate) fn addr_of_name(&self) -> *mut c_char {
        unsafe { addr_of!((*self.sys).name) as *const _ as *mut c_char }
    }

    pub fn interfaces(&self) -> Vec<Interface> {
        let info = const_to_string(INFO_PROPERTY_NAME); // TODO make this a constant
        if let Some(p) = self.state.read().props.get(&info) {
            trace!("found INFO property");
            if let Some(ifs) = p
                .items()
                .filter_map(|i| {
                    // only look for info device driver items
                    if i.name == const_to_string(INFO_DEVICE_DRIVER_ITEM_NAME) {
                        trace!("found DEVICE_DRIVER item");
                        // ensure that a text property value
                        if let PropertyValue::Text(v) = i.value.clone() {
                            Some(v) // heureka!
                        } else {
                            warn!("INFO_DEVICE_DRIVER_ITEM does not contain a text property value");
                            None
                        }
                    } else {
                        None // not an info device driver item
                    }
                })
                .nth(0)
            {
                // map the info device driver string to a list of interfaces
                return Interface::map(ifs);
            }
        } else {
            warn!(
                "Could not find an INFO property for the '{}' device-",
                self.name()
            )
        }
        Vec::new()
    }

    // -- methods

    /// Detach the device  from the INDIGO bus.
    #[named]
    pub fn detach(
        &self,
        f: impl FnMut(IndigoResult<DeviceImpl<'a>>) -> IndigoResult<()> + 'a,
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
    pub(crate) fn assert_same(&self, d: DeviceImpl) -> Result<(), IndigoError> {
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

    fn request(&self, request: IndigoRequest2<'a, DeviceImpl<'a>>) -> Result<(), IndigoError> {
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
        let d = DeviceImpl::try_from(device);
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

impl<'a> TryFrom<*mut indigo_device> for DeviceImpl<'a> {
    type Error = IndigoError;

    fn try_from(value: *mut indigo_device) -> Result<Self, Self::Error> {
        if value == ptr::null_mut() {
            return Err(IndigoError::Other("indigo_device is null".to_string()));
        }
        debug!("device addr {:p}", value);

        let mut device = unsafe { ptr::read(value) };

        // connect callbacks
        if let None = device.detach {
            device.detach = Some(DeviceImpl::on_detach);
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

        Ok(DeviceImpl {
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

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;
    #[test]
    fn interface() {
        assert_eq!(
            Interface::Mount as u32,
            indigo_device_interface_INDIGO_INTERFACE_MOUNT
        );
    }

    fn test_connection_status() -> Result<(), IndigoError> {
        let m = FlatPropertyModel::new();
        let c = Client::new("TestClient", m, false);
        let d = DeviceImpl::new("TestDevice");
        match d.connection_status() {
            Ok(true) => info!("Device {d} is CONNECTED."),
            Ok(false) => info!("Device {d} is DISCONNECTED."),
            Err(state) => warn!("Device {d} is in the {state} state"),
        }
        Ok(())
    }
}
