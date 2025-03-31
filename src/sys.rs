#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::{indigo::*, NumberFormat};
/// INDI v2 implementation enabled by the `sys` feature and based on the
/// [INDIGO](https://github.com/indigo-astronomy)system library
use core::str;
use std::{any::Any, str::Utf8Error};
use enum_primitive::*;
use function_name::named;
use libindigo_sys::*;
use log::{debug, error, info, trace, warn};
use parking_lot::{RwLock, RwLockWriteGuard};
use core::{
    ffi::{c_uint, c_void, CStr}, fmt::{Debug, Display}, marker::PhantomData, ptr, usize,error::Error
};
use url_fork::Url;

// -- Utility -----------------------------------------------------------------

fn log_result<'a,T>(manager: &str, op: &str, result: &'a IndigoResult<T,impl Error>) {
    if let Err(e) = &result {
        debug!("'{}': {} {}", manager, op, e);
    } else {
        trace!("'{}': {} OK", manager, op);
    }
}

fn log_and_return_code<'a,T>(manager: &str, op: &str, result: &'a IndigoResult<T,impl Error>) -> u32 {
    log_result(manager, op, result);
    lib_result_to_sys_code(result)
}

/// log an INDIGO callback as a trace if all went well and at debug level if there was an error.
unsafe fn log_sys_callback(client: *mut indigo_client, callback: &str, msg: *const i8) {
    let code = (*client).last_result;
    let client = buf_to_str(&(*client).name);
    let result = sys_code_to_lib_result((), callback, code);

    if !msg.is_null() {
        let msg = CStr::from_ptr(msg);
        match msg.to_str() {
            Ok(m) => debug!("'{client}': {callback} - {m}"),
            Err(e) => debug!("'{client}': {callback} - could not convert message: {e}"),
        }
    }

    let msg = &format!("callback '{}'", callback);
    log_result(client, msg, &result);
}
#[derive(Debug)]
pub struct SysError {
    msg: &'static str,
}

impl SysError {

    pub fn new(msg: &'static str) -> SysError {
        SysError { msg }
    }

    pub fn msg(&self) -> &str {
        self.msg
    }
}

impl From<BusError> for SysError {
    fn from(value: BusError) -> Self {
        match value {
            BusError::Failed => SysError::new("unspecified error"),
            BusError::TooManyElements => SysError::new("too many elements"),
            BusError::LockError => SysError::new("lock error"),
            BusError::NotFound => SysError::new("not found"),
            BusError::CantStartServer => SysError::new("network server start error"),
            BusError::Duplicated => SysError::new("duplicated objects"),
            BusError::Busy => SysError::new("resource is busy"),
            BusError::GuideError => SysError::new("guide process errror"),
            BusError::UnsupportedArchitecture => SysError::new("unsupported architecture"),
            BusError::UnresolvedDependency => SysError::new("unresolved dependency"),
        }
    }
}

impl From<Utf8Error> for SysError {
    fn from(value: Utf8Error) -> Self {
        warn!("failed UTF8 conversion: {value}");
        SysError::new("failed UTF8 conversion")
    }
}

impl Display for SysError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.msg)
    }
}

impl Error for SysError {
    fn description(&self) -> &str {
        self.msg
    }
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]  // this really should be `c_uint` to safeguard agains platform specifics.
/// Bus operation return status.
pub enum BusError {
    /// unspecified error
    Failed = indigo_result_INDIGO_FAILED,
    /// too many clients/devices/properties/items etc.
    TooManyElements = indigo_result_INDIGO_TOO_MANY_ELEMENTS,
    /// mutex lock error
    LockError = indigo_result_INDIGO_LOCK_ERROR,
    /// unknown client/device/property/item etc.
    NotFound = indigo_result_INDIGO_NOT_FOUND,
    /// network server start failure
    CantStartServer = indigo_result_INDIGO_CANT_START_SERVER,
    /// duplicated items etc.
    Duplicated = indigo_result_INDIGO_DUPLICATED,
    /// operation failed because the resourse is busy.
    Busy = indigo_result_INDIGO_BUSY,
    /// Guide process error (srar lost, SNR too low etc..).
    GuideError = indigo_result_INDIGO_GUIDE_ERROR,
    /// Unsupported architecture.
    UnsupportedArchitecture = indigo_result_INDIGO_UNSUPPORTED_ARCH,
    /// Unresolved dependencies (missing library, executable, ...).
    UnresolvedDependency = indigo_result_INDIGO_UNRESOLVED_DEPS,
}
}

impl Into<c_uint> for BusError {
    fn into(self) -> c_uint {
        self as c_uint
    }
}

pub fn sys_code_to_lib_result<'a,T>(t: T, op: &str, code: indigo_result) -> IndigoResult<T,SysError> {
    if code == indigo_result_INDIGO_OK {
        return Ok(t);
    }

    if let Some(e) = BusError::from_u32(code) {
        match e {
            BusError::Failed => Err(SysError::new("unspecified error")),
            BusError::TooManyElements => Err(SysError::new("too many elements")),
            BusError::LockError => Err(SysError::new("lock error")),
            BusError::NotFound => Err(SysError::new("not found")),
            BusError::CantStartServer => Err(SysError::new("network server start error")),
            BusError::Duplicated => Err(SysError::new("duplicated objects")),
            BusError::Busy => Err(SysError::new("resource is busy")),
            BusError::GuideError => Err(SysError::new("guide process errror")),
            BusError::UnsupportedArchitecture => Err(SysError::new("unsupported architecture")),
            BusError::UnresolvedDependency => Err(SysError::new("unresolved dependency")),
        }
    } else {
        warn!("{}: unknown bus result code {}", op, code);
        Err(SysError::new("unknown bus result code, please refer see log output!"))
    }
}

fn lib_result_to_sys_code<'a, T>(result: &'a IndigoResult<T,impl Error>) -> u32 {
    if let Err(_) = result {
        indigo_result_INDIGO_FAILED
    } else {
        indigo_result_INDIGO_OK
    }
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum LogLevel {
    Plain = indigo_log_levels_INDIGO_LOG_PLAIN,
    Error = indigo_log_levels_INDIGO_LOG_ERROR,
    Info = indigo_log_levels_INDIGO_LOG_INFO,
    Debug = indigo_log_levels_INDIGO_LOG_DEBUG,
    TraceBus = indigo_log_levels_INDIGO_LOG_TRACE_BUS,
    Trace = indigo_log_levels_INDIGO_LOG_TRACE,
}
}

impl From<&PropertyState> for u32 {
    fn from(s: &PropertyState) -> Self {
        match s {
            PropertyState::Idle => indigo_property_state_INDIGO_IDLE_STATE,
            PropertyState::Ok => indigo_property_state_INDIGO_OK_STATE,
            PropertyState::Busy => indigo_property_state_INDIGO_BUSY_STATE,
            PropertyState::Alert => indigo_property_state_INDIGO_ALERT_STATE,
        }
    }
}

impl From<&PropertyType> for u32 {
    fn from(t: &PropertyType) -> Self {
        match t {
            PropertyType::Text => indigo_property_type_INDIGO_TEXT_VECTOR,
            PropertyType::Number => indigo_property_type_INDIGO_NUMBER_VECTOR,
            PropertyType::Switch => indigo_property_type_INDIGO_SWITCH_VECTOR,
            PropertyType::Light => indigo_property_type_INDIGO_LIGHT_VECTOR,
            PropertyType::Blob => indigo_property_type_INDIGO_BLOB_VECTOR,
        }
    }
}

impl From<&PropertyPermission> for u32 {
    fn from(p: &PropertyPermission) -> Self {
        match p {
            PropertyPermission::ReadOnly => indigo_property_perm_INDIGO_RO_PERM,
            PropertyPermission::ReadWrite => indigo_property_perm_INDIGO_RW_PERM,
            PropertyPermission::WriteOnly => indigo_property_perm_INDIGO_WO_PERM,
        }
    }
}

impl From<&SwitchRule> for u32 {
    fn from(r: &SwitchRule) -> Self {
        match r {
            SwitchRule::OneOfMany => indigo_rule_INDIGO_ONE_OF_MANY_RULE,
            SwitchRule::AtMostOne => indigo_rule_INDIGO_AT_MOST_ONE_RULE,
            SwitchRule::AnyOfMany => indigo_rule_INDIGO_ANY_OF_MANY_RULE,
        }
    }
}

// -- Property ----------------------------------------------------------------

/// [Property] based on the [indigo_property] the `libindigo-sys` FFI binding to the [INDIGO C API](https://github.com/indigo-astronomy/indigo).
///
/// From the [INDIGO client documentation](https://github.com/indigo-astronomy/indigo/blob/master/indigo_docs/CLIENT_DEVELOPMENT_BASICS.md#properties):
/// > In case the client needs to check the values of some property item of a
/// > specified device it is always a good idea to check if the property is in OK state:
/// > ```C
/// > if (!strcmp(device->name, "CCD Imager Simulator @ indigosky") &&
/// >     !strcmp(property->name, CCD_IMAGE_PROPERTY_NAME) &&
/// >     property->state == INDIGO_OK_STATE) {
/// >     ...
/// > }
/// > ```
/// > And if the client needs to change some item value this code may help:
/// > ```C
/// > static const char * items[] = { CCD_IMAGE_FORMAT_FITS_ITEM_NAME };
/// > static bool values[] = { true };
/// > indigo_change_switch_property(
/// >   client,
/// >   CCD_SIMULATOR,
/// >   CCD_IMAGE_FORMAT_PROPERTY_NAME,
/// >   1,
/// >   items,
/// >   values
/// > );
/// > ```
pub struct SysProperty<'a> {
    sys: *mut indigo_property,
    property_type: PropertyType,
    items: Vec<SysPropertyItem<'a>>,
}

impl<'a> TryFrom<*mut indigo_property> for SysProperty<'a> {
    type Error = SysError;

    fn try_from(value: *mut indigo_property) -> Result<Self, Self::Error> {
        let sys = unsafe { &*value };
        let property_type = PropertyType::from_u32(sys.type_)
            .ok_or(SysError::new("could not convert property type"))?;

        let n = sys.count as usize;
        let items = unsafe { sys.items.as_slice(n) }
            .iter()
            .map(|i| SysPropertyItem::new(i, &property_type))
            .collect();

        Ok(SysProperty {
            sys: value,
            property_type,
            items,
        })
    }
}

impl<'a> NamedObject for SysProperty<'a> {
    fn name(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).name) }
    }

    fn label(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).label) }
    }
}

impl<'a> Property for SysProperty<'a> {
    type Item = SysPropertyItem<'a>;

    fn device(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).device) }
    }

    fn group(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).group) }
    }

    fn hints(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).hints) }
    }

    fn state(&self) -> &PropertyState {
        match unsafe { (*self.sys).state } {
            indigo_property_state_INDIGO_IDLE_STATE => &PropertyState::Idle,
            indigo_property_state_INDIGO_OK_STATE => &PropertyState::Ok,
            indigo_property_state_INDIGO_BUSY_STATE => &PropertyState::Busy,
            indigo_property_state_INDIGO_ALERT_STATE => &PropertyState::Alert,
            s => unimplemented!("property state '{}' not implemented", s),
        }
    }

    fn property_type(&self) -> &PropertyType {
        &self.property_type
    }

    fn perm(&self) -> &PropertyPermission {
        match unsafe { (*self.sys).perm } {
            indigo_property_perm_INDIGO_RO_PERM => &PropertyPermission::ReadOnly,
            indigo_property_perm_INDIGO_RW_PERM => &PropertyPermission::ReadWrite,
            indigo_property_perm_INDIGO_WO_PERM => &PropertyPermission::WriteOnly,
            p => unimplemented!("property permission '{}' not implemented", p),
        }
    }

    fn rule(&self) -> &SwitchRule {
        match unsafe { (*self.sys).rule } {
            indigo_rule_INDIGO_ANY_OF_MANY_RULE => &SwitchRule::AnyOfMany,
            indigo_rule_INDIGO_AT_MOST_ONE_RULE => &SwitchRule::AtMostOne,
            indigo_rule_INDIGO_ONE_OF_MANY_RULE => &SwitchRule::OneOfMany,
            r => unimplemented!("switch rule '{}' not implemented", r),
        }
    }

    fn hidden(&self) -> bool {
        unsafe { (*self.sys).hidden }
    }

    fn defined(&self) -> bool {
        unsafe { (*self.sys).defined }
    }

    fn items<'b>(&'b self) -> impl Iterator<Item = &'b Self::Item> {
        self.items.iter()
    }

    fn update<'b>(&mut self, p: &'b Self) {
        if self.sys == p.sys {
            trace!("skipping update of same property instance");
            return;
        }
        trace!("updating property by copying values");
        let sys = unsafe { &mut *self.sys };

        copy_from_str(sys.device, p.device());
        copy_from_str(sys.device, p.device());
        copy_from_str(sys.group, p.group());
        copy_from_str(sys.hints, p.hints());
        sys.state = u32::from(p.state());
        sys.type_ = u32::from(p.property_type());
        sys.perm = u32::from(p.perm());
        sys.rule = u32::from(p.rule());
        sys.hidden = p.hidden();
        sys.defined = p.defined();

        // let all: HashSet<&Self::Item> = self.items().collect();
        // for item in &mut self.items {
        //     item.update()
        // }

        /* TODO items */
    }
}

impl<'a> Display for SysProperty<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Property[Name: '{}'; Device: '{}'; Group: '{}'; Label: '{}'; Hints: '{}']",
            self.name(),
            self.device(),
            self.group(),
            self.label(),
            self.hints(),
        )?;
        for item in self.items() {
            write!(f, "    {item:?}")?;
        }
        Ok(())
    }
}

pub struct SysPropertyItem<'s> {
    sys: &'s indigo_item,
    property_type: PropertyType,
}

impl<'s> SysPropertyItem<'s> {
    fn new(sys: &'s indigo_item, property_type: &PropertyType) -> Self {
        Self {
            sys,
            property_type: property_type.to_owned(),
        }
    }
}

impl<'s> Debug for SysPropertyItem<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SysPropertyItem")
            .field("name", &self.name())
            .field("label", &self.label())
            .finish()
    }
}

impl<'s> NamedObject for SysPropertyItem<'s> {
    fn name(&self) -> &str {
        buf_to_str(&self.sys.name)
    }

    fn label(&self) -> &str {
        buf_to_str(&self.sys.label)
    }
}

impl<'s> PropertyItem for SysPropertyItem<'s> {
    fn property_type(&self) -> PropertyType {
        self.property_type
    }
}

impl<'s> SwitchItem for SysPropertyItem<'s> {
    fn on(&self) -> bool {
        unsafe { self.sys.__bindgen_anon_1.sw.value }
    }
}

impl<'s> TextItem for SysPropertyItem<'s> {
    fn text(&self) -> &str {
        // let v = unsafe { sys.__bindgen_anon_1.text };
        // let text = if v.long_value.is_null() {
        //     buf_to_str(&v.value)
        // } else {
        //     ptr_to_str(v.long_value).unwrap()
        // };
        todo!()
    }
}

impl<'s> LightItem for SysPropertyItem<'s> {
    fn state(&self) -> PropertyState {
        PropertyState::from_u32(unsafe { self.sys.__bindgen_anon_1.light.value })
            .expect("unknown property state")
    }
}

impl<'s> BlobItem for SysPropertyItem<'s> {
    fn url(&self) -> Option<&Url> {
        todo!()
    }

    fn data(&self) -> Option<&[u8]> {
        todo!()
    }

    fn format(&self) -> &str {
        buf_to_str(unsafe { &self.sys.__bindgen_anon_1.blob.format })
    }

    fn size(&self) -> usize {
        unsafe { self.sys.__bindgen_anon_1.blob.size as usize }
    }
}

impl<'s> NumberItem for SysPropertyItem<'s> {
    fn value(&self) -> f64 {
        unsafe { self.sys.__bindgen_anon_1.number.value }
    }

    fn format(&self) -> NumberFormat {
        NumberFormat::new(buf_to_str(unsafe {
            &self.sys.__bindgen_anon_1.number.format
        }))
    }

    fn min(&self) -> f64 {
        unsafe { self.sys.__bindgen_anon_1.number.min }
    }

    fn max(&self) -> f64 {
        unsafe { self.sys.__bindgen_anon_1.number.max }
    }

    fn step(&self) -> f64 {
        unsafe { self.sys.__bindgen_anon_1.number.step }
    }

    fn target(&self) -> f64 {
        unsafe { self.sys.__bindgen_anon_1.number.target }
    }
}

// -- SysClientController -----------------------------------------------------

/// Client to manage devices attached to the INDIGO [Bus].
pub struct SysClientController<'s,C>
where C: ClientDelegate<SysProperty<'s>> {
    sys: *mut indigo_client,
    _phantom: &'s PhantomData<C>,
}

impl<'s,C> NamedObject for SysClientController<'s,C>
where C: ClientDelegate<SysProperty<'s>> {
    fn name(&self) -> &str {
        buf_to_str(unsafe { &(*self.sys).name })
    }
}

impl<'s,C> AttachedObject for SysClientController<'s, C>
where C: ClientDelegate<SysProperty<'s>> {}

impl<'s,C> BusController for SysClientController<'s, C>
where C: ClientDelegate<SysProperty<'s>> {
    fn detach(self) -> std::result::Result<(), impl core::error::Error> {
        trace!("detaching '{}' client from bus...", self.name());
        let code = unsafe { indigo_detach_client(self.sys) };
        sys_code_to_lib_result((), "indigo_detach_client", code)
            .inspect(|_| info!("detached '{}' client from bus", self.name()))
            .inspect_err(|e| warn!("'{}': {}", self.name(), e))
    }
}

impl<'s, C: ClientDelegate<SysProperty<'s>>> Drop for SysClientController<'s, C> {
    fn drop(&mut self) {
        let client = unsafe { Box::from_raw(self.sys) };
        let state = unsafe { Box::from_raw(client.client_context) };
        drop(state);
        drop(client);
    }
}

impl<'s,C> SysClientController<'s,C>
where C: ClientDelegate<SysProperty<'s>> {
    /// Create new [SysClient] mapping INDGO system callbacks to a [ClientObject].
    fn new(c: C) -> Self {
        let name = str_to_buf(c.name());
        let state = Box::new(RwLock::new(c));
        let indigo_client = Box::new(indigo_client {
            name,
            is_remote: true,
            // unbox the mutex state and create a raw ptr to the state mutex
            client_context: Box::into_raw(state) as *mut c_void,
            // last result of a bus operation, assume all is OK from the beginning
            last_result: indigo_result_INDIGO_OK,
            version: indigo_version_INDIGO_VERSION_CURRENT,
            enable_blob_mode_records: ptr::null_mut(),
            attach: Some(Self::on_attach),
            detach: Some(Self::on_detach),
            define_property: Some(Self::on_define_property),
            update_property: Some(Self::on_update_property),
            delete_property: Some(Self::on_delete_property),
            send_message: Some(Self::on_send_message),
        });

        // get ptr reference to the indigo_client by dereferencing the Box
        let sys = Box::into_raw(indigo_client);
        SysClientController {
            sys,
            _phantom: &PhantomData,
        }
    }

    /// Acquire a lock on the client state held in the `client_context` of sys.
    fn write_lock<'b>(client: *mut indigo_client) -> RwLockWriteGuard<'b, C> {
        let c = unsafe { &*client };
        let name = buf_to_str(&c.name);
        // https://stackoverflow.com/a/24191977/51016
        let state = unsafe { &mut *(c.client_context as *mut RwLock<C>) };

        trace!("'{}': acquiring delegate write lock...", name);
        let lock = state.write();
        trace!("'{}': delegate write lock acquired.", name);
        lock
    }

    fn map_param<'b>(
        c: *mut indigo_client,
        d: *mut indigo_device,
        p: *mut indigo_property,
        m: *const i8,
    ) -> Result<(
        RwLockWriteGuard<'b, C>,
        &'b str,
        &'b str,
        SysProperty<'b>,
        Option<&'b str>,
    ), SysError> {
        let object = Self::write_lock(c);
        let c = buf_to_str(&(unsafe { &*c }).name);
        let d = buf_to_str(&(unsafe { &*d }).name);
        let p = SysProperty::try_from(p)?;
        let m = if m.is_null() {
            None
        } else {
            unsafe {
                Some(CStr::from_ptr(m).to_str()?)
            }
        };

        Ok((object, c, d, p, m))
    }

    // -- libindigo-sys unsafe callback methods that delegate to the CallbackHandler implementation.

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_attach(c: *mut indigo_client) -> indigo_result {
        log_sys_callback(c, function_name!(), ptr::null());

        let mut delegate = Self::write_lock(c);
        let c = buf_to_str(&(unsafe { &*c }).name);

        let result = delegate.on_attach();
        log_and_return_code(c, function_name!(), &result)
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_detach(c: *mut indigo_client) -> indigo_result {
        log_sys_callback(c, function_name!(), ptr::null());

        let mut delegate = Self::write_lock(c);
        let c = buf_to_str(&(unsafe { &*c }).name);

        let result = delegate.on_detach();
        log_and_return_code(c, function_name!(), &result)
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_define_property(
        c: *mut indigo_client,
        d: *mut indigo_device,
        p: *mut indigo_property,
        m: *const i8,
    ) -> indigo_result {
        log_sys_callback(c, function_name!(), m);
        let result = SysClientController::<C>::map_param(c, d, p, m);
        if let Ok((mut delegate, c, d, p, m)) = result {
            let result = delegate.on_define_property(d, p, m);
            log_and_return_code(c, function_name!(), &result)
        } else {
            let c = buf_to_str(&(unsafe { &*c }).name);
            log_and_return_code(c, function_name!(), &result)
        }
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_update_property(
        c: *mut indigo_client,
        d: *mut indigo_device,
        p: *mut indigo_property,
        m: *const i8,
    ) -> indigo_result {
        log_sys_callback(c, function_name!(), m);
        let result = SysClientController::<C>::map_param(c, d, p, m);
        if let Ok((mut delegate, c, d, p, m)) = result {
            let result = delegate.on_update_property(d, p, m);
            log_and_return_code(c, function_name!(), &result)
        } else {
            let c = buf_to_str(&(unsafe { &*c }).name);
            log_and_return_code(c, function_name!(), &result)
        }
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_delete_property(
        c: *mut indigo_client,
        d: *mut indigo_device,
        p: *mut indigo_property,
        m: *const i8,
    ) -> indigo_result {
        log_sys_callback(c, function_name!(), m);
        let result = SysClientController::<C>::map_param(c, d, p, m);
        if let Ok((mut delegate, c, d, p, m)) = result {
            let result = delegate.on_delete_property(d, p, m);
            log_and_return_code(c, function_name!(), &result)
        } else {
            let c = buf_to_str(&(unsafe { &*c }).name);
            log_and_return_code(c, function_name!(), &result)
        }
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_send_message(
        c: *mut indigo_client,
        d: *mut indigo_device,
        m: *const ::std::os::raw::c_char,
    ) -> indigo_result {
        log_sys_callback(c, function_name!(), m);

        let mut delegate = Self::write_lock(c);
        let c = buf_to_str(&(unsafe { &*c }).name);
        let d = buf_to_str(&(unsafe { &*d }).name);
        let m = ptr_to_str(m);

        let result = delegate.on_message_broadcast(d, m.unwrap());
        log_and_return_code(c, function_name!(), &result)
    }
}

impl<'s,C> ClientController<SysProperty<'s>, C> for SysClientController<'s, C>
where C: ClientDelegate<SysProperty<'s>> {
    fn request_definition<'a>(
        &mut self,
        _d: &'a str,
        p: &'a SysProperty
    ) -> IndigoResult<(),impl Error> {

        let code = unsafe { indigo_enumerate_properties(self.sys, p.sys) };
        sys_code_to_lib_result((), "indigo_enumerate_properties", code)
    }

    fn request_update<'a>(
        &mut self, _d:
        &'a str,
        p: &'a SysProperty
    ) -> IndigoResult<(),impl Error> {

        let code = unsafe { indigo_change_property(self.sys, p.sys) };
        sys_code_to_lib_result((), "indigo_enumerate_properties", code)
    }

    fn request_update_item<'a>(
        &mut self,
        _d: &'a str,
        _p: &'a SysProperty,
        _i: &'a impl PropertyItem,
    ) -> IndigoResult<(),SysError> {
        todo!()
    }

    fn request_delete<'a>(
        &mut self,
        _d: &'a str,
        _p: &'a SysProperty
    ) -> IndigoResult<(),SysError> {
        todo!()
    }

    fn request_delete_item<'a>(
        &mut self,
        _d: &'a str,
        _p: &'a SysProperty,
        _i: &'a impl PropertyItem,
    ) -> IndigoResult<(),SysError> {
        todo!()
    }

    fn request_enumeration(
        &mut self,
        _d: Option<&str>
    ) -> IndigoResult<usize,impl Error> {

        let code = unsafe { indigo_enumerate_properties(self.sys, &raw mut INDIGO_ALL_PROPERTIES) };
        sys_code_to_lib_result(0, "indigo_enumerate_properties", code)
    }

    // fn manage<'a,F, R>(&mut self, f: Callback<&'a mut Self>) -> IndigoResult<()> {
    //     todo!()
    // }
}

// -- SysDeviceController -----------------------------------------------------

struct SysDeviceController<'s, D: DeviceDelegate<SysProperty<'s>>> {
    sys: *mut indigo_device,
    _phantom: &'s PhantomData<D>,
}

// impl<'s,D> Display for SysDeviceController<'s,D>
// where D: DeviceDelegate<SysProperty<'s>> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let status = self.connected().map_or_else(
//             |e| format!("{:?}", e),
//             |s| {
//                 if s {
//                     "connected".to_string()
//                 } else {
//                     "disconnected".to_string()
//                 }
//             },
//         );
//         write!(f, "{} ({}) [", self.name(), status)?;
//         let mut sep = "";
//         if let Some(ifaces) = self.list_interfaces() {
//             for item in ifaces {
//                 write!(f, "{sep}{item}")?;
//                 sep = ", ";
//             }
//         }
//         write!(f, "]")?;

//         Ok(())
//     }
// }

impl<'s, D> SysDeviceController<'s, D>
where D: DeviceDelegate<SysProperty<'s>> {
    fn new(d: D) -> Self {
        let sys = Box::new(indigo_device {
            name: str_to_buf(d.name()),
            lock: 0,
            is_remote: true,
            gp_bits: 0,
            device_context: ptr::null_mut(),
            private_data: ptr::null_mut(),
            master_device: ptr::null_mut(),
            last_result: indigo_result_INDIGO_OK,
            version: indigo_version_INDIGO_VERSION_CURRENT,
            access_token: 0,
            match_patterns: ptr::null_mut(),
            match_patterns_count: 0,
            attach: Some(Self::on_attach),
            detach: Some(Self::on_detach),
            enumerate_properties: None,
            change_property: None,
            enable_blob: None,
        });
        let sys = Box::into_raw(sys);
        Self {
            sys,
            _phantom: &PhantomData,
        }
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_attach(c: *mut indigo_device) -> indigo_result {
        // log_sys_callback(c, function_name!(), ptr::null());

        // let mut delegate = Self::write_lock(c);
        // let c = buf_to_str(&(unsafe { &*c }).name);

        // let result = delegate.on_attach();
        // log_and_return_code(c, function_name!(), &result)
        info!("device attached");
        indigo_result_INDIGO_OK
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_detach(c: *mut indigo_device) -> indigo_result {
        info!("device detached");
        // log_sys_callback(c, function_name!(), ptr::null());

        // let mut delegate = Self::write_lock(c);
        // let c = buf_to_str(&(unsafe { &*c }).name);

        // let result = delegate.on_attach();
        // log_and_return_code(c, function_name!(), &result)
        indigo_result_INDIGO_OK
    }
}

impl<'s,D> NamedObject for SysDeviceController<'s, D>
where D: DeviceDelegate<SysProperty<'s>> {
    fn name(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).name) }
    }
}

impl<'s,D> Drop for SysDeviceController<'s, D>
where D: DeviceDelegate<SysProperty<'s>>{
    fn drop(&mut self) {
        let sys = unsafe { Box::from_raw(self.sys) };
        drop(sys);
    }
}

impl<'s,D> AttachedObject for SysDeviceController<'s, D>
where D: DeviceDelegate<SysProperty<'s>> {}

impl<'s,D> BusController for SysDeviceController<'s, D>
where D: DeviceDelegate<SysProperty<'s>> {
    fn detach(self) -> IndigoResult<(),impl Error> {
        Err(SysError::new("FIXME: not yet implemented"))
    }
}

impl<'s,D> DeviceController<'s,SysProperty<'s>, D> for SysDeviceController<'s, D>
where D: DeviceDelegate<SysProperty<'s>> {
    fn define_property<'a>(
        &mut self,
        p: &'a SysProperty
    ) -> IndigoResult<(),impl Error> {

        let code = unsafe { indigo_update_property(self.sys, p.sys, ptr::null()) };
        sys_code_to_lib_result((), "indigo_enumerate_properties", code)
    }

    fn update_property<'a>(
        &mut self,
        p: &'a SysProperty
    ) -> IndigoResult<(),impl Error> {

        let code = unsafe { indigo_update_property(self.sys, p.sys, ptr::null()) };
        sys_code_to_lib_result((), "indigo_update_property", code)
    }

    fn delete_property<'a>(
        &mut self,
        p: &'a SysProperty
    ) -> IndigoResult<(),impl Error> {

        let code = unsafe { indigo_update_property(self.sys, p.sys, ptr::null()) };
        sys_code_to_lib_result((), "indigo_update_property", code)
    }

    fn broadcast_message(
        &mut self,
        msg: &str
    ) -> IndigoResult<(),impl Error> {

        let msg: [i8; 256] = str_to_buf(msg);
        let code = unsafe { indigo_send_message(self.sys, &raw const msg as *const i8) };
        sys_code_to_lib_result((), "indigo_send_message", code)
    }
}

// -- SysAgent ----------------------------------------------------------------

// pub struct SysAgent<A: AgentObject> {
//     object: Arc<A>
// }

// impl<A: AgentObject> Agent<A> for SysAgent<A> {

//     fn client(&mut self) -> impl Client<A> {
//         SysClient::new("TODO", self.object.as_ref())
//     }

//     fn device(&mut self) -> impl Device<A> {
//         SysDevice {
//             _o: PhantomData,
//         }
//     }
// }

// -- SysRemoteResource -------------------------------------------------------

pub struct SysRemoteResource {
    name: String,
    url: Url,
    sys: *mut indigo_server_entry,
}

// impl Default for SysRemoteResource {
//     fn default() -> Self {
//         SysRemoteResource::new("INDIGO", "localhost", INDIGO_DEFAULT_PORT).expect("expected valid defaults")
//     }
// }

impl Drop for SysRemoteResource {
    fn drop(&mut self) {
        if self.sys != ptr::null_mut() {
            let entry = unsafe { Box::from_raw(self.sys) };
            drop(entry);
        }
    }
}

impl SysRemoteResource {
    fn host(&self) -> &str {
        self.url.host_str().unwrap_or(DEFAULT_HOST)
    }

    fn port(&self) -> u16 {
        self.url.port().unwrap_or(DEFAULT_PORT)
    }

    fn new<'a>(
        name: &str,
        url: Url
    ) -> IndigoResult<Self,impl Error> {

        if url.scheme() != "tcp" {
            return Err(SysError::new("the url scheme must be 'tcp'"));
        }

        let name = name.trim().to_owned();
        if name.is_empty() {
            return Err(SysError::new(
                "name of remote resource must not be empty or blank",
            ));
        }

        let host = url
            .host_str()
            .ok_or(SysError::new("indigo url does not have a hostname"))?;
        if !hostname_validator::is_valid(&host) {
            // is this necessary, are URL hostnames always valid? Probably...
            return Err(SysError::new("invalid hostname"));
        }

        Ok(Self {
            name,
            url,
            sys: ptr::null_mut(),
        })
    }

    fn is_connected<'a>(&self) -> IndigoResult<bool,SysError> {

        let msg_buf = [0i8; 256].as_mut_ptr();
        let connected = unsafe { indigo_connection_status(self.sys, msg_buf) };
        if connected {
            return Ok(true);
        }

        let s = unsafe { CStr::from_ptr(msg_buf) };
        if s.is_empty() {
            Ok(false)
        } else {
            Err(SysError::new(s.to_str()?))
        }
    }
}

fn server_entry(name: &str, host: &str, port: u16) -> *mut indigo_server_entry {
    let _ = host;
    let _ = name;
    let entry = Box::new(indigo_server_entry {
        // name: str_to_buf(name),
        // host: str_to_buf(host),
        name: [0i8; 128],
        host: [0i8; 128],
        port: port as i32,
        connection_id: 0,
        thread: unsafe { std::mem::zeroed() },
        thread_started: false,
        socket: 0,
        protocol_adapter: ptr::null_mut(),
        last_error: [0; 256],
        shutdown: false,
    });
    Box::into_raw(entry)
}

impl Debug for SysRemoteResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SysRemoteResource")
            .field("name", &self.name())
            .field("host", &self.host())
            .field("port", &self.port())
            .finish()
    }
}

impl NamedObject for SysRemoteResource {
    fn name(&self) -> &str {
        &self.name
    }
}

impl AttachedObject for SysRemoteResource {}

impl BusController for SysRemoteResource {
    fn detach(mut self) -> Result<(),impl Error> {

        trace!("detaching remote resource '{}'", self.name);
        if let Err(e) = self.disconnect() {
            error!("could not detach remote resource: {e}");
            Err(SysError::new("could not detach remote resource"))
        } else {
            Ok(())
        }

        // let name = format!("@ {}", self.name);
        // let name = "@ Server";
        // let d = raw_device(&name);
        // let code = unsafe { indigo_detach_device(d) };
        // sys_code_to_lib_result((), "indigo_detach_device", code)
        //     .inspect(|_| info!("detached remote resource"))
        //     .inspect_err(|e| warn!("could not detach resource: {}", e))

        // let code = unsafe { indigo_detach_device((*self.sys).protocol_adapter) };
        // sys_code_to_lib_result((), "indigo_detach_device", code)
        //     .inspect(|_| info!("detached remote resource"))
        //     .inspect_err(|e| warn!("could not detach resource: {}", e))
    }
}

impl RemoteResource for SysRemoteResource {

    fn disconnect(&mut self) -> IndigoResult<(), impl Error> {

        if !self.is_connected()? {
            return Err(SysError::new("not connected"));
        }

        trace!("discconnecting from {}...", self.url);
        let code = unsafe { indigo_disconnect_server(self.sys) };

        sys_code_to_lib_result((), "indigo_disconnect_server", code)
            .inspect(|_| info!("disconnected from {}", self.url))
            .inspect_err(|e| info!("failed to disconnect {}: {}", self.url, e))
    }

    fn reconnect(&mut self) -> IndigoResult<(), impl Error> {
        if self.is_connected()? {
            return Err(SysError::new("already connected"));
        }

        trace!("connecting to {:?}...", self.url);
        let mut sys = server_entry(self.name(), self.host(), self.port());
        self.sys = sys;
        let code = unsafe {
            indigo_connect_server(
                str_to_buf::<128>(self.name()).as_ptr(),
                str_to_buf::<128>(self.host()).as_ptr(),
                self.port() as i32,
                &raw mut sys,
            )
        };

        sys_code_to_lib_result((), "indigo_connect_server", code)
            .inspect(|_| info!("connection to {} successful", self.url))
            .inspect_err(|e| warn!("connection to {} failed: {e}", self.url))
    }
}

// pub fn connect(name: &str, host: &str, port: c_int) -> IndigoResult<ServerConnection> {
//     trace!("Connecting to {host}:{port}...");

//     let name = str_to_buf(name)?;
//     let host = str_to_buf(host)?;

//     let mut entry = indigo_server_entry {
//         name: name,
//         host: host,
//         port: port,
//         connection_id: 0,
//         thread: unsafe { std::mem::zeroed() },
//         thread_started: false,
//         socket: 0,
//         protocol_adapter: ptr::null_mut(),
//         last_error: [0; 256],
//         shutdown: false,
//     };

//     let mut srv_ptr = ptr::addr_of_mut!(entry);
//     let srv_ptr_ptr = ptr::addr_of_mut!(srv_ptr);

//     let result = unsafe {
//         indigo_connect_server(
//             entry.name.as_ptr(),
//             entry.host.as_ptr(),
//             entry.port,
//             srv_ptr_ptr,
//         )
//     };

//     let connection = ServerConnection {
//         sys: Rc::new(entry),
//     };

//     bus::sys_to_lib(connection, result, "indigo_connect_server")
//         .inspect(|c| info!("Connection to {c} successful."))
//         .inspect_err(|e| warn!("Connection failed: {e}."))
// }

// -- SysBus ------------------------------------------------------------------

pub struct SysBus {
    name: String,
}

impl NamedObject for SysBus {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'s,C,D> Bus<SysProperty<'s>,C,D> for SysBus
where C: ClientDelegate<SysProperty<'s>> +'s, D: DeviceDelegate<SysProperty<'s>> +'s
{

    #[named]
    #[allow(refining_impl_trait)]
    fn start(name: &str) -> IndigoResult<SysBus,impl Error> {
        trace!("configuring INDIGO log message handler...");
        unsafe {
            indigo_log_message_handler = Some(log_message_handler);
        }

        trace!("starting INDIGO system bus...");
        let code = unsafe { indigo_start() };

        let sys_bus = SysBus {
            name: name.to_owned(),
        };
        sys_code_to_lib_result(sys_bus, function_name!(), code)
            .inspect(|_| info!("started INDIGO system bus"))
            .inspect_err(|e| error!("could not start INDIGO system bus: {}", e))
    }

    #[named]
    fn stop(&mut self) -> IndigoResult<(),impl Error> {
        trace!("stopping INDIGO system bus...");
        let code = unsafe { indigo_stop() };
        sys_code_to_lib_result((), function_name!(), code)
            .inspect(|_| info!("stoped INDIGO system bus"))
            .inspect_err(|e| error!("could not stop INDIGO system bus: {}", e))
    }

    // fn attach_device<'a,D: DeviceDelegate>(&self, d: D) -> IndigoResult<impl DeviceController<'a,D>> {
    //     Ok(SysDeviceController::new(d))
    // }

    fn attach_remote(
        &mut self,
        name: &str,
        url: &Url
    ) -> IndigoResult<impl RemoteResource,impl Error> {

        match SysRemoteResource::new(name, url.clone()) {
            Ok(mut remote) => {
                if let Err(e) = remote.reconnect() {
                    error!("connection to '{name}' failed: {e}");
                    return Err(SysError::new("could not attach to remote resource"))
                }
                Ok(remote)
            },
            Err(e) => {
                error!("could not create resource '{name}' for {url}: {e}");
                Err(SysError::new("could not attach remote resource"))
            }
        }
    }

    fn attach_client(
        &mut self,
        delegate: C,
    ) -> IndigoResult<impl ClientController<SysProperty<'s>, C>, impl Error> {
        let client = SysClientController::new(delegate);
        Ok::<SysClientController<'s,C>,SysError>(client)
    }

    // fn attach_device(
    //     &mut self,
    //     delegate: D,
    // ) -> IndigoResult<impl DeviceController<SysProperty<'s>, D>, impl Error> {
    //     let device = SysDeviceController::new(delegate);
    //     Ok::<SysDeviceController<'s,D>,SysError>(device)
    // }

    // fn attach_client<'a,C: ClientDelegate<SysProperty<'b>>>(&self, c: C) ->
    // IndigoResult<SysClientController<'a,C>> {

    //     trace!("'{}': attaching client...", c.name());
    //     let c = SysClientController::new(c);
    //     let code = unsafe { indigo_attach_client(c.sys) };

    //     sys_code_to_lib_result(c, "indigo_attach_client", code)
    // }
}

impl SysBus {
    /// Enable INDIGO sys bus logging at the provided level.
    pub fn enable_bus_log(level: LogLevel) {
        unsafe { indigo_set_log_level(level as i32) };
    }

    /// Return the INDIGO sys bus log level.
    pub fn bus_log() -> LogLevel {
        let level = unsafe { indigo_get_log_level() };
        LogLevel::from_i32(level).expect("expected a valid log level")
    }
}

impl Drop for SysBus {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            warn!("could not stop bus: {}", e)
        }
    }
}

unsafe extern "C" fn log_message_handler(
    level: indigo_log_levels,
    message: *const ::std::os::raw::c_char,
) {
    let mut msg = ptr_to_str(message);
    let msg = msg.get_or_insert("<emtpy log message>");

    match level {
        indigo_log_levels_INDIGO_LOG_PLAIN => info!("{}", msg),
        indigo_log_levels_INDIGO_LOG_ERROR => error!("{}", msg),
        indigo_log_levels_INDIGO_LOG_INFO => info!("{}", msg),
        indigo_log_levels_INDIGO_LOG_DEBUG => debug!("{}", msg),
        indigo_log_levels_INDIGO_LOG_TRACE_BUS => trace!("{}", msg),
        indigo_log_levels_INDIGO_LOG_TRACE => trace!("{}", msg),
        _ => warn!("[UNKNOWN LOG LEVEL: {}]: {}", level, msg),
    }
}

// indigo_log (log at INFO level)
// indigo_log_base
// indigo_log_name
// indigo_log_message
// indigo_log_message_handler
// indigo_set_log_level
// indigo_get_log_level

// indigo_use_syslog

// -- Tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use std::thread::sleep;
    use std::time::Duration;

    use libindigo_sys::{__IncompleteArrayField, indigo_property};
    use log::LevelFilter;

    use crate::indigo::*;
    use crate::sys::*;

    use crate::sys::{NamedObject, Property};

    use super::{str_to_buf, SysBus, SysProperty};

    fn init_test(level: LevelFilter) {
        let _ = env_logger::builder()
            .target(env_logger::Target::Stdout)
            .filter_level(level)
            .is_test(true)
            .try_init();
    }

    fn create_prop() -> *mut indigo_property {
        let mut prop = Box::new(indigo_property {
            device: str_to_buf("device"),
            name: str_to_buf("prop"),
            group: str_to_buf("group"),
            label: str_to_buf("label"),
            hints: str_to_buf("clueless"),
            state: 42,
            type_: 64,
            perm: 888,
            rule: 666,
            access_token: 32000,
            version: 1,
            hidden: true,
            defined: true,
            allocated_count: 3,
            count: 4,
            items: __IncompleteArrayField::new(),
        });
        prop.as_mut()
    }

    #[test]
    fn test_sys_property<'a>() -> IndigoResult<(),SysError>{
        let sys = create_prop();
        let prop: SysProperty = SysProperty::try_from(sys)?;
        assert_eq!(prop.name(), "prop");
        assert_eq!(prop.device(), "device");
        assert_eq!(prop.group(), "group");
        assert_eq!(prop.label(), "label");
        assert_eq!(prop.hints(), "clueless");
        prop.items().for_each(|p| {
            println!("name '{}'", p.name());
            // let v1 = super::TextItem::value(p);
            // let v2 = super::NumberItem::value(p);
        });
        Ok(())
    }

    struct TestClient {
        name: String,
    }
    impl NamedObject for TestClient {
        fn name(&self) -> &str {
            &self.name
        }
    }

    impl AttachedObject for TestClient {}

    impl<'s> ClientDelegate<SysProperty<'s>> for TestClient {}

    type TestBus<'s> = SysBus<'s,TestClient,TestDevice>;

    #[test]
    fn test_sys_bus() {
        init_test(LevelFilter::Debug);
        SysBus::enable_bus_log(LogLevel::Debug);
        assert_eq!(LogLevel::Debug, SysBus::bus_log());

        let mut bus = SysBus::start("Bus").expect("bus instance");
        let client = TestClient {
            name: "Client".to_owned(),
        };
        let client = bus
            .attach_client(client)
            .expect("expected a client instance");

        let url = Url::parse("tcp://localhost").expect("url to be valid");
        let server = bus
            .attach_remote("Server", &url)
            .expect("attached server");

        // client.request_enumeration(None).expect("expected bus to enumerate the properties");

        sleep(Duration::new(1, 0));
        server
            .detach()
            .expect("expected server to be detached from the bus");
        client
            .detach()
            .expect("expected client to be detached from the bus");
        //sleep(Duration::new(1,0));

        bus.stop().expect("expected bus to stop");
    }
}
