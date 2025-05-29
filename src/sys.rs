#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::{
    indigo::*,
    property::{Number, PropertyData, PropertyItem, PropertyValue, Text},
    NumberFormat,
};
/// INDI v2 implementation enabled by the `sys` feature and based on the
/// [INDIGO](https://github.com/indigo-astronomy)system library
use core::{slice, str};
use enum_primitive::*;
use fambox::{FamBox, FamBoxOwned};
use function_name::named;
use libindigo_sys::*;
use log::{debug, error, info, trace, warn};
use parking_lot::{RwLock, RwLockWriteGuard};
use regex::Regex;
use serde_json_core::from_slice;
use std::{
    alloc::LayoutError,
    collections::HashMap,
    ffi::{c_char, c_int, c_long, CString},
    io::Read,
    marker::PhantomPinned,
    ops::DerefMut,
    str::{FromStr, Utf8Error},
};

use core::{
    error::Error,
    ffi::{c_uint, c_void, CStr},
    fmt::{Debug, Display},
    marker::PhantomData,
    ptr,
};
use url_fork::{ParseError, Url};

// -- Utility -----------------------------------------------------------------

fn log_result<T>(manager: &str, op: &str, result: &IndigoResult<T>) {
    if let Err(e) = &result {
        debug!("'{}': {} {}", manager, op, e);
    } else {
        trace!("'{}': {} OK", manager, op);
    }
}

fn log_and_return_code<'a, T>(
    object: &str,
    op: &str,
    result: &'a IndigoResult<T>,
) -> indigo_result {
    log_result(object, op, result);
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

impl From<LayoutError> for IndigoError {
    fn from(value: LayoutError) -> Self {
        IndigoError::new(value.to_string().as_str())
    }
}

impl From<BusError> for IndigoError {
    fn from(value: BusError) -> Self {
        match value {
            BusError::Failed => IndigoError::new("unspecified error"),
            BusError::TooManyElements => IndigoError::new("too many elements"),
            BusError::LockError => IndigoError::new("lock error"),
            BusError::NotFound => IndigoError::new("not found"),
            BusError::CantStartServer => IndigoError::new("network server start error"),
            BusError::Duplicated => IndigoError::new("duplicated objects"),
            BusError::Busy => IndigoError::new("resource is busy"),
            BusError::GuideError => IndigoError::new("guide process errror"),
            BusError::UnsupportedArchitecture => IndigoError::new("unsupported architecture"),
            BusError::UnresolvedDependency => IndigoError::new("unresolved dependency"),
        }
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
    Failed = indigo_result::INDIGO_FAILED.0,
    /// too many clients/devices/properties/items etc.
    TooManyElements = indigo_result::INDIGO_TOO_MANY_ELEMENTS.0,
    /// mutex lock error
    LockError = indigo_result::INDIGO_LOCK_ERROR.0,
    /// unknown client/device/property/item etc.
    NotFound = indigo_result::INDIGO_NOT_FOUND.0,
    /// network server start failure
    CantStartServer = indigo_result::INDIGO_CANT_START_SERVER.0,
    /// duplicated items etc.
    Duplicated = indigo_result::INDIGO_DUPLICATED.0,
    /// operation failed because the resourse is busy.
    Busy = indigo_result::INDIGO_BUSY.0,
    /// Guide process error (srar lost, SNR too low etc..).
    GuideError = indigo_result::INDIGO_GUIDE_ERROR.0,
    /// Unsupported architecture.
    UnsupportedArchitecture = indigo_result::INDIGO_UNSUPPORTED_ARCH.0,
    /// Unresolved dependencies (missing library, executable, ...).
    UnresolvedDependency = indigo_result::INDIGO_UNRESOLVED_DEPS.0,
}
}

impl Into<c_uint> for BusError {
    fn into(self) -> c_uint {
        self as c_uint
    }
}

pub fn sys_code_to_lib_result<'a, T>(t: T, op: &str, code: indigo_result) -> IndigoResult<T> {
    match code {
        indigo_result::INDIGO_OK => Ok(t),
        indigo_result::INDIGO_FAILED => Err(IndigoError::new("unspecified error")),
        indigo_result::INDIGO_TOO_MANY_ELEMENTS => Err(IndigoError::new("too many elements")),
        indigo_result::INDIGO_LOCK_ERROR => Err(IndigoError::new("lock error")),
        indigo_result::INDIGO_NOT_FOUND => Err(IndigoError::new("not found")),
        indigo_result::INDIGO_CANT_START_SERVER => {
            Err(IndigoError::new("network server start error"))
        }
        indigo_result::INDIGO_DUPLICATED => Err(IndigoError::new("duplicated objects")),
        indigo_result::INDIGO_BUSY => Err(IndigoError::new("resource is busy")),
        indigo_result::INDIGO_GUIDE_ERROR => Err(IndigoError::new("guide process errror")),
        indigo_result::INDIGO_UNSUPPORTED_ARCH => Err(IndigoError::new("unsupported architecture")),
        indigo_result::INDIGO_UNRESOLVED_DEPS => Err(IndigoError::new("unresolved dependency")),
        _ => {
            warn!("{}: unknown bus result code {}", op, code.0);
            Err(IndigoError::new("unknown bus result code"))
        }
    }
}

fn lib_result_to_sys_code<'a, T>(result: &'a IndigoResult<T>) -> indigo_result {
    if let Err(_) = result {
        indigo_result::INDIGO_FAILED
    } else {
        indigo_result::INDIGO_OK
    }
}

impl TryInto<indigo_item> for &'_ PropertyItem {
    type Error = IndigoError;

    fn try_into(self) -> Result<indigo_item, Self::Error> {
        todo!()
    }
}

impl TryInto<FamBoxOwned<indigo_property>> for &PropertyData {
    type Error = IndigoError;

    fn try_into(self) -> Result<FamBoxOwned<indigo_property>, Self::Error> {

        // let items: Result<Vec<indigo_item>, _> = self.items().map(|i| i.try_into()).collect();
        // let items: &[indigo_item] = &(items?);

        let count = self.len() as i32;
        let header = indigo_property {
            name: str_to_buf(self.name()),
            device: str_to_buf(self.device()),
            group: str_to_buf(self.group()),
            label: str_to_buf(self.label()),
            hints: str_to_buf(self.hints()),
            state: self.state().into(),
            type_: self.property_type().into(),
            perm: self.perm().into(),
            rule: self.rule().into(),
            access_token: 0,
            version: indigo_version::INDIGO_VERSION_NONE.0 as i16,
            hidden: false,
            defined: false,
            count,
            allocated_count: count,
            items: indigo_item_array::new(),
        };

        Ok(FamBoxOwned::from_fn(header, |i| (&self[i]).try_into().expect("valid property item")))
    }
}

impl TryFrom<indigo_property_state> for PropertyState {
    type Error = IndigoError;

    fn try_from(value: indigo_property_state) -> Result<Self, Self::Error> {
        match value {
            indigo_property_state::INDIGO_OK_STATE => Ok(PropertyState::Ok),
            indigo_property_state::INDIGO_IDLE_STATE => Ok(PropertyState::Idle),
            indigo_property_state::INDIGO_BUSY_STATE => Ok(PropertyState::Busy),
            indigo_property_state::INDIGO_ALERT_STATE => Ok(PropertyState::Alert),
            _ => {
                warn!("unknown property state: {}", value.0);
                Err(IndigoError::new("unknown property state"))
            }
        }
    }
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum LogLevel {
    Plain = indigo_log_levels::INDIGO_LOG_PLAIN.0,
    Error = indigo_log_levels::INDIGO_LOG_ERROR.0,
    Info = indigo_log_levels::INDIGO_LOG_INFO.0,
    Debug = indigo_log_levels::INDIGO_LOG_DEBUG.0,
    TraceBus = indigo_log_levels::INDIGO_LOG_TRACE_BUS.0,
    Trace = indigo_log_levels::INDIGO_LOG_TRACE.0,
}
}

impl Into<indigo_log_levels> for LogLevel {
    fn into(self) -> indigo_log_levels {
        match self {
            LogLevel::Plain => indigo_log_levels::INDIGO_LOG_PLAIN,
            LogLevel::Error => indigo_log_levels::INDIGO_LOG_ERROR,
            LogLevel::Info => indigo_log_levels::INDIGO_LOG_INFO,
            LogLevel::Debug => indigo_log_levels::INDIGO_LOG_DEBUG,
            LogLevel::TraceBus => indigo_log_levels::INDIGO_LOG_TRACE_BUS,
            LogLevel::Trace => indigo_log_levels::INDIGO_LOG_TRACE,
        }
    }
}

impl From<&PropertyState> for indigo_property_state {
    fn from(s: &PropertyState) -> Self {
        match s {
            PropertyState::Idle => indigo_property_state::INDIGO_IDLE_STATE,
            PropertyState::Ok => indigo_property_state::INDIGO_OK_STATE,
            PropertyState::Busy => indigo_property_state::INDIGO_BUSY_STATE,
            PropertyState::Alert => indigo_property_state::INDIGO_ALERT_STATE,
        }
    }
}

impl TryFrom<indigo_log_levels> for LogLevel {
    type Error = IndigoError;

    fn try_from(value: indigo_log_levels) -> Result<Self, IndigoError> {
        match value {
            indigo_log_levels::INDIGO_LOG_PLAIN => Ok(LogLevel::Plain),
            indigo_log_levels::INDIGO_LOG_TRACE => Ok(LogLevel::Trace),
            indigo_log_levels::INDIGO_LOG_TRACE_BUS => Ok(LogLevel::TraceBus),
            indigo_log_levels::INDIGO_LOG_DEBUG => Ok(LogLevel::Debug),
            indigo_log_levels::INDIGO_LOG_INFO => Ok(LogLevel::Info),
            indigo_log_levels::INDIGO_LOG_ERROR => Ok(LogLevel::Error),
            _ => Err(IndigoError::new("unknown log level")),
        }
    }
}

impl From<&PropertyType> for indigo_property_type {
    fn from(t: &PropertyType) -> Self {
        match t {
            PropertyType::Text => indigo_property_type::INDIGO_TEXT_VECTOR,
            PropertyType::Number => indigo_property_type::INDIGO_NUMBER_VECTOR,
            PropertyType::Switch => indigo_property_type::INDIGO_SWITCH_VECTOR,
            PropertyType::Light => indigo_property_type::INDIGO_LIGHT_VECTOR,
            PropertyType::Blob => indigo_property_type::INDIGO_BLOB_VECTOR,
        }
    }
}

impl From<&PropertyPermission> for indigo_property_perm {
    fn from(p: &PropertyPermission) -> Self {
        match p {
            PropertyPermission::ReadOnly => indigo_property_perm::INDIGO_RO_PERM,
            PropertyPermission::ReadWrite => indigo_property_perm::INDIGO_RW_PERM,
            PropertyPermission::WriteOnly => indigo_property_perm::INDIGO_WO_PERM,
        }
    }
}

impl From<&SwitchRule> for indigo_rule {
    fn from(r: &SwitchRule) -> Self {
        match r {
            SwitchRule::Undefined => indigo_rule::UNDEFINED,
            SwitchRule::OneOfMany => indigo_rule::INDIGO_ONE_OF_MANY_RULE,
            SwitchRule::AtMostOne => indigo_rule::INDIGO_AT_MOST_ONE_RULE,
            SwitchRule::AnyOfMany => indigo_rule::INDIGO_ANY_OF_MANY_RULE,
        }
    }
}

/// Convert an [indigo_property] to a [PropertyData] failing on first error.
impl TryFrom<&indigo_property> for PropertyData {
    type Error = IndigoError;
    fn try_from(sys: &indigo_property) -> IndigoResult<Self> {
        let type_ = PropertyType::try_from(sys.type_)?;
        let n = sys.count as usize;

        let items: Result<Vec<_>, _> = unsafe { sys.items.as_slice(n) }
            .iter()
            .map(|i| -> Result<PropertyItem, IndigoError> {
                let name = buf_to_str(&i.name);
                let value = unsafe {
                    match type_ {
                        PropertyType::Text => PropertyValue::try_from(i.__bindgen_anon_1.text)?,
                        PropertyType::Number => PropertyValue::try_from(i.__bindgen_anon_1.number)?,
                        PropertyType::Switch => PropertyValue::try_from(i.__bindgen_anon_1.sw)?,
                        PropertyType::Light => PropertyValue::try_from(i.__bindgen_anon_1.light)?,
                        PropertyType::Blob => PropertyValue::try_from(i.__bindgen_anon_1.blob)?,
                    }
                };
                Ok(PropertyItem::new(name, value))
            })
            .collect();

        let name = buf_to_str(&sys.name);
        let device = buf_to_str(&sys.device);
        let group = buf_to_str(&sys.group);
        let hints = buf_to_str(&sys.hints);
        let state =
            PropertyState::from_u32(sys.state.0).ok_or(IndigoError::new("property state error"))?;
        let type_ =
            PropertyType::from_u32(sys.type_.0).ok_or(IndigoError::new("property type error"))?;
        let perm = PropertyPermission::from_u32(sys.perm.0)
            .ok_or(IndigoError::new("property permission error"))?;
        let rule = SwitchRule::from_u32(sys.rule.0).ok_or_else(|| {
            IndigoError::from(format!("no rule mapped to code {}", sys.rule.0))
        })?;
        let hidden = sys.hidden;
        let defined = sys.defined;

        let items = items?;

        Ok(PropertyData::new(
            name, device, group, hints, state, type_, perm, rule, hidden, defined, items,
        ))
    }
}

/// Convert INDIGO text to [PropertyValue].
impl TryFrom<indigo_text> for PropertyValue {
    type Error = IndigoError;
    fn try_from(text: indigo_text) -> Result<Self, Self::Error> {
        let value = if text.long_value.is_null() {
            buf_to_str(&text.value)
        } else {
            let n = text.length as usize;
            let ptr = text.long_value as *const u8;
            let bytes = unsafe { slice::from_raw_parts(ptr, n) };
            CStr::from_bytes_until_nul(&bytes[0..n])
                .expect("could not read CStr")
                .to_str()
                .expect("could not convert to UTF8 str")
        };
        Ok(PropertyValue::text(value))
    }
}

/// Convert INDIGO number to [PropertyValue].
impl TryFrom<indigo_number> for PropertyValue {
    type Error = IndigoError;
    fn try_from(number: indigo_number) -> Result<Self, Self::Error> {
        let fmt = buf_to_str(&number.format);
        let fmt = NumberFormat::new(fmt);

        Ok(PropertyValue::number(
            number.value,
            number.target,
            fmt,
            number.min,
            number.max,
            number.step,
        ))
    }
}

/// Convert INDIGO switch to [PropertyValue].
impl TryFrom<indigo_switch> for PropertyValue {
    type Error = IndigoError;
    fn try_from(switch: indigo_switch) -> Result<Self, Self::Error> {
        Ok(PropertyValue::switch(switch.value))
    }
}

/// Convert INDIGO light to [PropertyValue].
impl TryFrom<indigo_light> for PropertyValue {
    type Error = IndigoError;
    fn try_from(light: indigo_light) -> Result<Self, Self::Error> {
        if let Some(value) = PropertyState::from_u32(light.value.0) {
            Ok(PropertyValue::light(value))
        } else {
            Err(IndigoError::new("missing light value"))
        }
    }
}

/// Convert INDIGO blob to [PropertyValue].
impl TryFrom<indigo_blob> for PropertyValue {
    type Error = IndigoError;
    fn try_from(blob: indigo_blob) -> Result<Self, Self::Error> {
        let s = buf_to_str(&blob.url);
        let url = if s.is_empty() {
            None
        } else {
            Some(Url::from_str(s)?)
        };

        let ext = buf_to_str(&blob.format);
        let size = blob.size as usize;
        let value = unsafe {
            if blob.value.is_null() || size == 0 {
                None
            } else {
                Some(Vec::from_raw_parts(blob.value as *mut u8, size, size))
            }
        };

        Ok(PropertyValue::blob(size, ext, value, url))
    }
}

impl TryFrom<indigo_property_type> for PropertyType {
    type Error = IndigoError;

    fn try_from(value: indigo_property_type) -> Result<Self, Self::Error> {
        match value {
            indigo_property_type::INDIGO_TEXT_VECTOR => Ok(PropertyType::Text),
            indigo_property_type::INDIGO_NUMBER_VECTOR => Ok(PropertyType::Number),
            indigo_property_type::INDIGO_LIGHT_VECTOR => Ok(PropertyType::Light),
            indigo_property_type::INDIGO_SWITCH_VECTOR => Ok(PropertyType::Switch),
            indigo_property_type::INDIGO_BLOB_VECTOR => Ok(PropertyType::Blob),
            t => {
                warn!("unknonwn property type: {}", t.0);
                Err(IndigoError::new("unknown property type"))
            }
        }
    }
}

// -- SysClientController -----------------------------------------------------
/// Client to manage devices attached to the INDIGO [Bus].
pub struct SysClientController<D>
where
    D: ClientDelegate<Property = PropertyData, Bus = SysBus>,
{
    sys: *mut indigo_client,
    delegate: PhantomData<D>,
}

impl<'s, D> NamedObject for SysClientController<D>
where
    D: ClientDelegate<
        Property = PropertyData,
        Bus = SysBus,
        BusController = Self,
        ClientController = Self,
    >,
{
    fn name(&self) -> &str {
        buf_to_str(unsafe { &(*self.sys).name })
    }
}

type BUS_CALLBACK<'a, D: ClientDelegate> =
    fn(delegate: &'a mut D, controller: &mut D::BusController) -> IndigoResult<()>;

impl<D> Controller<SysBus> for SysClientController<D>
where
    D: ClientDelegate<
        Property = PropertyData,
        Bus = SysBus,
        BusController = Self,
        ClientController = Self,
    >,
{
    /// Attach the self to the [Bus].
    fn attach(&mut self, _: &mut SysBus) -> IndigoResult<()> {
        trace!("attaching '{}' client from bus...", self.name());
        let code = unsafe { indigo_attach_client(self.sys) };
        let name = self.name().to_owned();

        sys_code_to_lib_result((), "indigo_attach_client", code)
            .inspect(|_| info!("detached '{}' client from bus", name))
            .inspect_err(|e| warn!("'{}': {}", name, e))
    }

    fn detach(&mut self) -> IndigoResult<()> {
        trace!("detaching '{}' client from bus...", self.name());
        let code = unsafe { indigo_detach_client(self.sys) };
        let name = self.name().to_owned();

        sys_code_to_lib_result((), "indigo_detach_client", code)
            .inspect(|_| info!("detached '{}' client from bus", name))
            .inspect_err(|e| warn!("'{}': {}", name, e))
    }
}

// impl<'s, D: ClientDelegate<PropertyData,SysBus,Self>> Drop for SysClientController<D> {
//     fn drop(&mut self) {
//         if !self.sys.is_null() {
//             let client = unsafe { Box::from_raw(self.sys) };
//             if !client.client_context.is_null() {
//                 let state = unsafe { Box::from_raw(client.client_context) };
//                 drop(state);
//             }
//             drop(client);
//         }
//     }
// }

type PROPERTY_CALLBACK<D: ClientDelegate> = fn(
    delegate: &mut D,
    controller: &mut D::ClientController,
    device: &str,
    property: D::Property,
    message: Option<&str>,
) -> IndigoResult<()>;

impl<D> SysClientController<D>
where
    D: ClientDelegate<
        Property = PropertyData,
        Bus = SysBus,
        BusController = Self,
        ClientController = Self,
    >,
{
    /// Create new [SysClient] mapping INDGO system callbacks to a [ClientObject].
    pub fn new(c: D) -> Self {
        let name = str_to_buf(c.name());
        let state = Box::new(RwLock::new(c));
        let indigo_client = Box::new(indigo_client {
            name,
            is_remote: true,
            // unbox the mutex state and create a raw ptr to the state mutex
            client_context: Box::into_raw(state) as *mut c_void,
            // last result of a bus operation, assume all is OK from the beginning
            last_result: indigo_result::INDIGO_OK,
            version: indigo_version::INDIGO_VERSION_CURRENT,
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
            delegate: PhantomData,
        }
    }

    /// Acquire a lock on the client state held in the `client_context` of sys.
    fn write_lock2<'b>(&self) -> RwLockWriteGuard<'b, D> {
        let c = unsafe { &*(self.sys) };
        let name = buf_to_str(&c.name);
        // https://stackoverflow.com/a/24191977/51016
        let state = unsafe { &mut *(c.client_context as *mut RwLock<D>) };

        trace!("'{}': acquiring delegate write lock...", name);
        let lock = state.write();
        trace!("'{}': delegate write lock acquired.", name);
        lock
    }
    /// Acquire a lock on the client state held in the `client_context` of sys.
    fn write_lock<'b>(client: *mut indigo_client) -> RwLockWriteGuard<'b, D> {
        let c = unsafe { &*client };
        let name = buf_to_str(&c.name);
        // https://stackoverflow.com/a/24191977/51016
        let state = unsafe { &mut *(c.client_context as *mut RwLock<D>) };

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
    ) -> IndigoResult<(
        RwLockWriteGuard<'b, D>,
        &'b str,
        &'b str,
        PropertyData,
        Option<&'b str>,
    )> {
        let object = Self::write_lock(c);
        let c = buf_to_str(&(unsafe { &*c }).name);
        let d = buf_to_str(&(unsafe { &*d }).name);

        let p = unsafe { &*p };
        let p = PropertyData::try_from(p)?;

        let m = if m.is_null() {
            None
        } else {
            unsafe { Some(CStr::from_ptr(m).to_str()?) }
        };

        Ok((object, c, d, p, m))
    }

    fn controller(sys: *mut indigo_client) -> SysClientController<D> {
        SysClientController {
            sys,
            delegate: PhantomData::<D>,
        }
    }

    // -- libindigo-sys unsafe callback methods that delegate to the CallbackHandler implementation.

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_attach(c: *mut indigo_client) -> indigo_result {
        log_sys_callback(c, function_name!(), ptr::null());

        let controller = &mut Self::controller(c);
        let mut delegate = Self::write_lock(controller.sys);

        let result = delegate.on_attach(controller);

        log_and_return_code(controller.name(), function_name!(), &result)
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_detach(c: *mut indigo_client) -> indigo_result {
        log_sys_callback(c, function_name!(), ptr::null());

        let controller = &mut Self::controller(c);
        let mut delegate = Self::write_lock(controller.sys);

        let result = delegate.on_detach(controller);

        log_and_return_code(controller.name(), function_name!(), &result)
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
        SysClientController::<D>::delegate_property_event(D::on_define_property, c, d, p, m)
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
        SysClientController::<D>::delegate_property_event(D::on_update_property, c, d, p, m)
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
        SysClientController::<D>::delegate_property_event(D::on_delete_property, c, d, p, m)
    }

    #[named]
    #[allow(unused)]
    unsafe extern "C" fn on_send_message(
        c: *mut indigo_client,
        d: *mut indigo_device,
        m: *const ::std::os::raw::c_char,
    ) -> indigo_result {
        log_sys_callback(c, function_name!(), m);

        let controller = &mut Self::controller(c);

        let mut delegate = Self::write_lock(c);
        let client_name = buf_to_str(&(unsafe { &*c }).name);
        let device = buf_to_str(&(unsafe { &*d }).name);
        let message = CStr::from_ptr(m)
            .to_str()
            .inspect_err(|e| warn!("could not read message: {e}"))
            .expect("valid UTF8 string");

        let result = delegate.on_message_broadcast(controller, device, message);
        log_and_return_code(client_name, function_name!(), &result)
    }

    #[named]
    fn delegate_property_event(
        f: PROPERTY_CALLBACK<D>,
        c: *mut indigo_client,
        d: *mut indigo_device,
        p: *mut indigo_property,
        m: *const i8,
    ) -> indigo_result {
        let controller = &mut Self::controller(c);
        let client_name = buf_to_str(&(unsafe { &*c }).name);

        let device = buf_to_str(&(unsafe { &*d }).name);

        let p = unsafe { &*p };
        let property = PropertyData::try_from(p)
            .inspect_err(|e| warn!("could not create property: {e}"))
            .expect("invalid system property");
        let message = if m.is_null() {
            None
        } else {
            unsafe {
                let msg = CStr::from_ptr(m)
                    .to_str()
                    .inspect_err(|e| warn!("could not read message: {e}"))
                    .expect("valid UTF8 string");
                Some(msg)
            }
        };
        // delegate property event handling to callback method
        let result = f(
            &mut Self::write_lock(c),
            controller,
            device,
            property,
            message,
        );

        log_and_return_code(client_name, function_name!(), &result)
    }

    #[named]
    fn request_text_item_update(&self, p: &PropertyData, i: &[&PropertyItem]) -> IndigoResult<()> {
        let mut items = Vec::new();
        let mut values = Vec::new();

        i.iter().try_for_each(|item| {
            let name = CString::new(item.name())?;
            items.push(name.into_raw() as *const c_char);
            if let PropertyValue::Text(text) = item.value() {
                let value = CString::new(text.value())?;
                values.push(value.into_raw() as *const c_char);
                Ok(())
            } else {
                Err(IndigoError::new("not a text item"))
            }
        })?;

        let device = CString::new(p.device())?.into_raw();
        let property = CString::new(p.name())?.into_raw();

        let code = unsafe {
            indigo_change_text_property(
                self.sys,
                device,
                property,
                items.len() as i32,
                items.as_mut_ptr(),
                values.as_mut_ptr(),
            )
        };

        // cleanup of allocated CStrings
        unsafe {
            drop(CString::from_raw(device));
            drop(CString::from_raw(property));
            items.iter().for_each(|ptr| {
                let name = CString::from_raw(*ptr as *mut i8);
                drop(name)
            });
            values.iter().for_each(|ptr| {
                let value = CString::from_raw(*ptr as *mut i8);
                drop(value)
            });
        }

        sys_code_to_lib_result((), function_name!(), code)
    }

    #[named]
    fn request_number_items_update(
        &self,
        p: &PropertyData,
        i: &[&PropertyItem],
    ) -> IndigoResult<()> {
        let mut items = Vec::new();
        let mut values = Vec::new();

        i.iter().try_for_each(|item| {
            let name = CString::new(item.name())?;
            items.push(name.into_raw() as *const c_char);
            if let PropertyValue::Number(nbr) = item.value() {
                values.push(nbr.target());
                Ok(())
            } else {
                Err(IndigoError::new("not a text item"))
            }
        })?;

        let device = CString::new(p.device())?.into_raw();
        let property = CString::new(p.name())?.into_raw();

        let code = unsafe {
            indigo_change_number_property(
                self.sys,
                device,
                property,
                items.len() as i32,
                items.as_mut_ptr(),
                values.as_mut_ptr(),
            )
        };

        // cleanup of allocated CStrings
        unsafe {
            drop(CString::from_raw(device));
            drop(CString::from_raw(property));
            items.iter().for_each(|ptr| {
                let name = CString::from_raw(*ptr as *mut i8);
                drop(name)
            });
        }
        // values assumed to be dropped on exit...
        sys_code_to_lib_result((), function_name!(), code)
    }

    #[named]
    fn request_switch_items_update(
        &self,
        p: &PropertyData,
        i: &[&PropertyItem],
    ) -> IndigoResult<()> {
        let mut items = Vec::new();
        let mut values = Vec::new();

        i.iter().try_for_each(|item| {
            let name = CString::new(item.name())?;
            items.push(name.into_raw() as *const c_char);
            if let PropertyValue::Switch(s) = item.value() {
                values.push(s.on());
                Ok(())
            } else {
                Err(IndigoError::new("not a text item"))
            }
        })?;

        let device = CString::new(p.device())?.into_raw();
        let property = CString::new(p.name())?.into_raw();

        let code = unsafe {
            indigo_change_switch_property(
                self.sys,
                device,
                property,
                items.len() as i32,
                items.as_mut_ptr(),
                values.as_mut_ptr(),
            )
        };

        // cleanup of allocated CStrings
        unsafe {
            drop(CString::from_raw(device));
            drop(CString::from_raw(property));
            items.iter().for_each(|ptr| {
                let name = CString::from_raw(*ptr as *mut c_char);
                drop(name)
            });
        }
        // values assumed to be dropped on exit...
        sys_code_to_lib_result((), function_name!(), code)
    }

    #[named]
    fn request_blob_items_update(&self, p: &PropertyData, i: &[&PropertyItem]) -> IndigoResult<()> {
        let mut items = Vec::new(); // *mut *const
        let mut values = Vec::new(); // *mut *mut *c_void
        let mut sizes = Vec::new(); // *const c_long
        let mut formats = Vec::new(); // *mut *const c_char
        let mut urls = Vec::new(); // *mut *const c_char

        i.iter().try_for_each(|item| {
            let name = CString::new(item.name())?;
            items.push(name.into_raw() as *const c_char);
            if let PropertyValue::Blob(blob) = item.value() {
                sizes.push(blob.size() as c_long);
                let format = CString::new(blob.extension())?;
                formats.push(format.into_raw() as *const c_char);
                // values
                if let Some(bytes) = blob.data() {
                    values.push(bytes.as_ptr() as *mut c_void)
                } else {
                    values.push(ptr::null_mut())
                }
                // urls
                if let Some(url) = blob.url() {
                    let url = CString::new(url.as_str())?;
                    urls.push(url.into_raw() as *const c_char)
                } else {
                    urls.push(ptr::null_mut());
                }
                Ok(())
            } else {
                Err(IndigoError::new("not a text item"))
            }
        })?;

        let device = CString::new(p.device())?.into_raw();
        let property = CString::new(p.name())?.into_raw();

        let code = unsafe {
            indigo_change_blob_property(
                self.sys,
                device,
                property,
                items.len() as i32,
                items.as_mut_ptr(),
                values.as_mut_ptr(),
                sizes.as_mut_ptr(),
                formats.as_mut_ptr(),
                urls.as_mut_ptr(),
            )
        };

        // cleanup of allocated CStrings
        unsafe {
            drop(CString::from_raw(device));
            drop(CString::from_raw(property));
            items.iter().for_each(|ptr| {
                let name = CString::from_raw(*ptr as *mut i8);
                drop(name)
            });
            formats.iter().for_each(|format| {
                let format = CString::from_raw(*format as *mut c_char);
                drop(format)
            });
            urls.iter().for_each(|url| {
                let url = CString::from_raw(*url as *mut c_char);
                drop(url)
            });
        }
        // sizes and ptrs to the value byte slices are assumed to be dropped on exit...
        sys_code_to_lib_result((), function_name!(), code)
    }
}

impl<D> ClientController<PropertyData, SysBus> for SysClientController<D>
where
    D: ClientDelegate<
        Property = PropertyData,
        Bus = SysBus,
        BusController = Self,
        ClientController = Self,
    >,
{
    fn request_definition<'a>(&mut self, d: &'a str, p: &'a str) -> IndigoResult<()> {
        let sys = &mut indigo_property::new(p, d, indigo_property_type::UNDEFINED);
        let code = unsafe { indigo_enumerate_properties(self.sys, sys) };
        sys_code_to_lib_result((), "indigo_enumerate_properties", code)
    }

    fn request_update<'a>(&mut self, _d: &'a str, p: &'a PropertyData) -> IndigoResult<()> {
        let sys: FamBoxOwned<indigo_property> = p.try_into()?;
        let code = unsafe { indigo_change_property(self.sys, sys.leak().as_mut()) };
        sys_code_to_lib_result((), "indigo_enumerate_properties", code)
    }

    fn request_update_item<'a>(
        &mut self,
        _d: &'a str,
        _p: &'a PropertyData,
        _i: &'a PropertyItem,
    ) -> IndigoResult<()> {
        match _p.property_type() {
            PropertyType::Text => self.request_text_item_update(_p, &[_i]),
            PropertyType::Number => self.request_number_items_update(_p, &[_i]),
            PropertyType::Switch => self.request_switch_items_update(_p, &[_i]),
            PropertyType::Blob => self.request_blob_items_update(_p, &[_i]),
            PropertyType::Light => Err(IndigoError::new(
                "update not supported for light property type",
            )),
        }
    }

    fn request_delete<'a>(&mut self, _d: &'a str, _pz2: &'a PropertyData) -> IndigoResult<()> {
        todo!()
    }

    fn request_delete_item<'a>(
        &mut self,
        _d: &'a str,
        _p: &'a PropertyData,
        _i: &'a PropertyItem,
    ) -> IndigoResult<()> {
        todo!()
    }

    fn request_enumeration(&mut self, _d: Option<&str>) -> IndigoResult<usize> {
        debug!("requested property enumeration");
        let code = unsafe { indigo_enumerate_properties(self.sys, &raw mut INDIGO_ALL_PROPERTIES) };
        sys_code_to_lib_result(0, "indigo_enumerate_properties", code)
    }

    // fn manage<'a,F, R>(&mut self, f: Callback<&'a mut Self>) -> IndigoResult<()> {
    //     todo!()
    // }
}

impl<D> Debug for SysClientController<D>
where
    D: ClientDelegate<
        Property = PropertyData,
        Bus = SysBus,
        BusController = Self,
        ClientController = Self,
    >,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SysClientController")
            .field("sys", &self.sys)
            .field("delegate", &self.delegate)
            .finish()
    }
}

// -- SysDeviceController -----------------------------------------------------

struct SysDeviceController<D>
where
    D: DeviceDelegate<Property = PropertyData, Bus = SysBus>,
{
    sys: *mut indigo_device,
    _phantom: PhantomData<D>,
}

// impl<'s,D> Display for SysDeviceController<'s,D>
// where D: DeviceDelegate<PropertyData> {
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

impl<'s, D> SysDeviceController<D>
where
    D: DeviceDelegate<Property = PropertyData, Bus = SysBus>,
{
    fn new(d: D) -> Self {
        let sys = Box::new(indigo_device {
            name: str_to_buf(d.name()),
            lock: 0,
            is_remote: true,
            gp_bits: 0,
            device_context: ptr::null_mut(),
            private_data: ptr::null_mut(),
            master_device: ptr::null_mut(),
            last_result: indigo_result::INDIGO_OK,
            version: indigo_version::INDIGO_VERSION_CURRENT,
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
            _phantom: PhantomData,
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
        indigo_result::INDIGO_OK
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
        indigo_result::INDIGO_OK
    }
}

impl<'s, D> Debug for SysDeviceController<D>
where
    D: DeviceDelegate<Property = PropertyData, Bus = SysBus>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SysDeviceController")
            .field("sys", &self.sys)
            // .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<'s, D> NamedObject for SysDeviceController<D>
where
    D: DeviceDelegate<Property = PropertyData, Bus = SysBus>,
{
    fn name(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).name) }
    }
}

impl<'s, D> Drop for SysDeviceController<D>
where
    D: DeviceDelegate<Property = PropertyData, Bus = SysBus>,
{
    fn drop(&mut self) {
        let sys = unsafe { Box::from_raw(self.sys) };
        drop(sys);
    }
}

impl<'s, D> Controller<SysBus> for SysDeviceController<D>
where
    D: DeviceDelegate<Property = PropertyData, Bus = SysBus>,
{
    fn attach<'a>(&mut self, _: &'a mut SysBus) -> IndigoResult<()> {
        todo!()
    }

    fn detach(&mut self) -> IndigoResult<()> {
        todo!()
    }
}

impl<'s, D> DeviceController<PropertyData, SysBus> for SysDeviceController<D>
where
    D: DeviceDelegate<Property = PropertyData, Bus = SysBus>,
{
    fn define_property<'a>(&mut self, _p: &'a PropertyData) -> IndigoResult<()> {
        todo!("implement device define_property")
    }

    fn update_property<'a>(&mut self, _p: &'a PropertyData) -> IndigoResult<()> {
        todo!("implement device update_property")
    }

    fn delete_property<'a>(&mut self, _p: &'a PropertyData) -> IndigoResult<()> {
        todo!("implement device delete_property")
    }

    fn broadcast_message(&mut self, msg: &str) -> IndigoResult<()> {
        let msg: [i8; 256] = str_to_buf(msg);
        let code = unsafe { indigo_send_message(self.sys, &raw const msg as *const c_char) };
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

#[derive(Clone)]
pub struct SysRemoteResource {
    name: String,
    url: Url,
    sys: *mut indigo_server_entry,
}

impl Default for SysRemoteResource {
    fn default() -> Self {
        let url = format!("tcp://{INDIGO_DEFAULT_PORT}:{INDIGO_DEFAULT_PORT}");
        let url = Url::parse(&url).expect("indigo url");
        SysRemoteResource::new("INDIGO", url).expect("valid defaults")
    }
}

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

    pub fn new<'a>(name: &str, url: Url) -> IndigoResult<Self> {
        if url.scheme() != "tcp" {
            return Err(IndigoError::new("the url scheme must be 'tcp'"));
        }

        let name = name.trim().to_owned();
        if name.is_empty() {
            return Err(IndigoError::new(
                "name of remote resource must not be empty or blank",
            ));
        }

        let host = url
            .host_str()
            .ok_or(IndigoError::new("indigo url does not have a hostname"))?;
        if !hostname_validator::is_valid(&host) {
            // is this necessary, are URL hostnames always valid? Probably...
            return Err(IndigoError::new("invalid hostname"));
        }

        Ok(Self {
            name,
            url,
            sys: ptr::null_mut(),
        })
    }

    fn is_connected(&self) -> IndigoResult<bool> {
        let msg_buf = [0i8; 256].as_mut_ptr();
        let connected = unsafe { indigo_connection_status(self.sys, msg_buf) };
        if connected {
            return Ok(true);
        }

        let s = unsafe { CStr::from_ptr(msg_buf) };
        if s.is_empty() {
            Ok(false)
        } else {
            Err(IndigoError::new(s.to_str()?))
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

impl Controller<SysBus> for SysRemoteResource {
    fn attach<'a>(&mut self, _: &'a mut SysBus) -> IndigoResult<()> {
        trace!("attaching remote resource '{}'", self.name);

        if let Err(e) = self.reconnect() {
            error!("could not attach remote resource: {e}");
            return Err(IndigoError::new("could not attach remote resource"));
        }

        Ok(())
    }

    fn detach(&mut self) -> IndigoResult<()> {
        trace!("detaching remote resource '{}'", self.name);

        if let Err(e) = self.disconnect() {
            error!("could not detach remote resource: {e}");
            return Err(IndigoError::new("could not detach remote resource"));
        }

        trace!("detached remote resource '{}'", self.name);
        Ok(())

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

impl RemoteResource<SysBus> for SysRemoteResource {
    fn disconnect(&mut self) -> IndigoResult<()> {
        if !self.is_connected()? {
            return Err(IndigoError::new("not connected"));
        }

        trace!("discconnecting from {}...", self.url);

        let code = unsafe {
            (*self.sys).shutdown = true;
            indigo_disconnect_server(self.sys)
        };

        sys_code_to_lib_result((), "indigo_disconnect_server", code)
            .inspect(|_| info!("disconnected from {}", self.url))
            .inspect_err(|e| info!("failed to disconnect {}: {}", self.url, e))
    }

    fn reconnect(&mut self) -> IndigoResult<()> {
        if self.is_connected()? {
            return Err(IndigoError::new("already connected"));
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

// -- SysBus -------------------------------------------------------------------

pub struct SysBus {
    name: String,
}

impl SysBus {
    /// Enable INDIGO sys bus logging at the provided level.
    pub fn enable_bus_log(level: LogLevel) {
        unsafe { indigo_set_log_level(level.into()) };
    }

    /// Return the INDIGO sys bus log level.
    pub fn bus_log() -> LogLevel {
        let level = unsafe { indigo_get_log_level() };
        LogLevel::try_from(level).expect("expected a valid log level")
    }
}

impl NamedObject for SysBus {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Bus for SysBus {
    #[named]
    #[allow(refining_impl_trait)]
    fn start(name: &str) -> IndigoResult<SysBus> {
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
    fn stop(&mut self) -> IndigoResult<()> {
        trace!("stopping INDIGO system bus...");
        let code = unsafe { indigo_stop() };
        sys_code_to_lib_result((), function_name!(), code)
            .inspect(|_| info!("stoped INDIGO system bus"))
            .inspect_err(|e| error!("could not stop INDIGO system bus: {}", e))
    }

    // fn attach_device<'a,D: DeviceDelegate>(&self, d: D) -> IndigoResult<impl DeviceController<'a,D>> {
    //     Ok(SysDeviceController::new(d))
    // }

    // fn attach_remote(
    //     &mut self,
    //     name: &str,
    //     url: &Url
    // ) -> IndigoResult<impl RemoteResource,impl Error> {

    //     match SysRemoteResource::new(name, url.clone()) {
    //         Ok(mut remote) => {
    //             if let Err(e) = remote.reconnect() {
    //                 error!("connection to '{name}' failed: {e}");
    //                 return Err(IndigoError::new("could not attach to remote resource"))
    //             }
    //             Ok(remote)
    //         },
    //         Err(e) => {
    //             error!("could not create resource '{name}' for {url}: {e}");
    //             Err(IndigoError::new("could not attach remote resource"))
    //         }
    //     }
    // }

    // fn attach_device(
    //     &mut self,
    //     delegate: D,
    // ) -> IndigoResult<impl DeviceController<PropertyData, D>, impl Error> {
    //     let device = SysDeviceController::new(delegate);
    //     Ok::<SysDeviceController<'s,D>,SysError>(device)
    // }

    // fn attach_client<'a,C: ClientDelegate<PropertyData<'b>>>(&self, c: C) ->
    // IndigoResult<SysClientController<'a,C>> {

    //     trace!("'{}': attaching client...", c.name());
    //     let c = SysClientController::new(c);
    //     let code = unsafe { indigo_attach_client(c.sys) };

    //     sys_code_to_lib_result(c, "indigo_attach_client", code)
    // }
}

impl Debug for SysBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SysBus").field("name", &self.name).finish()
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
        indigo_log_levels::INDIGO_LOG_PLAIN => info!("{}", msg),
        indigo_log_levels::INDIGO_LOG_ERROR => error!("{}", msg),
        indigo_log_levels::INDIGO_LOG_INFO => info!("{}", msg),
        indigo_log_levels::INDIGO_LOG_DEBUG => debug!("{}", msg),
        indigo_log_levels::INDIGO_LOG_TRACE_BUS => trace!("{}", msg),
        indigo_log_levels::INDIGO_LOG_TRACE => trace!("{}", msg),
        _ => warn!("[UNKNOWN LOG LEVEL: {}]: {}", level.0, msg),
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

    use std::sync::Arc;

    use std::time::Duration;

    use libindigo_sys::{__IncompleteArrayField, indigo_property};
    use log::LevelFilter;
    use parking_lot::Condvar;
    use parking_lot::Mutex;

    use crate::indigo::*;
    use crate::sys::*;

    use crate::sys::{NamedObject, Property};

    use super::{str_to_buf, PropertyData, SysBus};

    fn init_test(level: LevelFilter) {
        let _ = env_logger::builder()
            // .target(env_logger::Target::Stdout)
            .filter_level(level)
            .is_test(true)
            .try_init();
    }

    fn dummy_property() -> Result<FamBoxOwned<indigo_property>, IndigoError> {
        let items = [
            indigo_item::text("F1", "Fruit 1", "green and round", "Apple ")?,
            indigo_item::text("F2", "Fruit 2", "yellow and oblong", "Banana")?,
            indigo_item::text("F3", "Fruit 3", "red and small", "Strawberry")?,
        ];

        let prop = indigo_property {
            device: str_to_buf("device"),
            name: str_to_buf("prop"),
            group: str_to_buf("group"),
            label: str_to_buf("label"),
            hints: str_to_buf("clueless"),
            state: indigo_property_state::INDIGO_OK_STATE,
            type_: indigo_property_type::INDIGO_TEXT_VECTOR,
            perm: indigo_property_perm::INDIGO_RW_PERM,
            rule: indigo_rule::INDIGO_ANY_OF_MANY_RULE,
            access_token: 32000,
            version: 1,
            hidden: true,
            defined: true,
            allocated_count: 3,
            count: 3,
            items: __IncompleteArrayField::new(),
        };
        
        let fam = FamBoxOwned::from_fn(prop, |i| items[i]);
        Ok(fam)
    }

    #[test]
    fn test_sys_property() -> IndigoResult<()> {
        let sys = dummy_property()?;
        let prop: PropertyData = PropertyData::try_from(sys.as_ref())?;
        assert_eq!(prop.name(), "prop");
        assert_eq!(prop.device(), "device");
        assert_eq!(prop.group(), "group");
        assert_eq!(prop.label(), "prop");
        assert_eq!(prop.hints(), "clueless");

        assert_eq!(prop.len(), 3);
        assert_eq!(prop[0].name(), "F1");
        assert_eq!(prop[1].name(), "F2");
        assert_eq!(prop[2].name(), "F3");

        Ok(())
    }

    struct TestMonitor {
        condvar: Condvar,
        lock: Mutex<u8>,
    }
    impl TestMonitor {
        fn new() -> TestMonitor {
            TestMonitor {
                condvar: Condvar::new(),
                lock: Mutex::new(0),
            }
        }
        /// wait until done or a timeout occurs
        fn wait(&self, limit: u8, timeout: Duration) -> u8 {
            let mut pcount = self.lock.lock();
            while *pcount < limit {
                self.condvar.wait_for(&mut pcount, timeout);
            }
            *pcount
        }
        fn inc(&self) -> IndigoResult<()> {
            let mut pcount = self.lock.lock();
            *pcount += 1;
            self.condvar.notify_all();
            Ok(())
        }
    }

    struct TestDelegate {
        name: String,
        monitor: Arc<TestMonitor>,
        _phantom: PhantomData<PropertyData>,
    }

    impl Debug for TestDelegate {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TestDelegate")
                .field("name", &self.name)
                // .field("monitor", &self.monitor)
                // .field("_phantom", &self._phantom)
                .finish()
        }
    }

    impl NamedObject for TestDelegate {
        fn name(&self) -> &str {
            &self.name
        }
    }

    impl TestDelegate {
        /// Creates a [TestDelegate] from the [Arc<Condvar>] instance.
        fn new(name: &str, monitor: Arc<TestMonitor>) -> TestDelegate {
            TestDelegate {
                name: name.to_owned(),
                monitor: monitor.clone(),
                _phantom: PhantomData,
            }
        }
    }

    impl<'s> Delegate for TestDelegate {
        type Bus = SysBus;
        type BusController = SysClientController<Self>;

        fn on_attach(&mut self, c: &mut SysClientController<Self>) -> IndigoResult<()> {
            debug!("attached '{}'", self.name());
            Ok(())
        }
    }

    impl<'s> ClientDelegate for TestDelegate {
        type Property = PropertyData;
        type ClientController = SysClientController<Self>;

        fn on_define_property<'a>(
            &'a mut self,
            _c: &mut SysClientController<Self>,
            _d: &'a str,
            p: PropertyData,
            _msg: Option<&'a str>,
        ) -> IndigoResult<()> {
            debug!("'{}': '{}' property defined ", p.device(), p.name());
            for item in p.items() {
                debug!("'{}': '{}'  ", item.name(), item);
            }
            self.monitor.inc()?;
            Ok(())
        }
    }

    /// Test assumes an INDIGO server running on localhost on the default port.
    #[test]
    fn test_sys_bus() -> IndigoResult<()> {
        init_test(LevelFilter::Trace);

        SysBus::enable_bus_log(LogLevel::Debug);
        // assert_eq!(LogLevel::Trace, SysBus::bus_log());

        let monitor = Arc::new(TestMonitor::new());

        let bus = &mut SysBus::start("TestBus").expect("bus instance");
        let delegate_delegate = TestDelegate::new("TestClient", monitor.clone());
        let client_controller = &mut SysClientController::new(delegate_delegate);

        let url = Url::parse("tcp://localhost").expect("valid url");
        let remote = &mut SysRemoteResource::new("TestRemote", url).expect("remote instance");

        remote.attach(bus).expect("attached remote");
        client_controller.attach(bus).expect("attached client");

        client_controller
            .request_enumeration(None)
            .expect("enumeration request");
        assert_eq!(monitor.wait(5, Duration::from_secs(10)), 5);

        remote.detach().expect("detached remote");
        client_controller.detach().expect("detached controller");

        info!("testing, testing");

        bus.stop(); // FIXME ensure that the remote is detached
                    // bus.stop().expect("stopped bus");

        // while let Err(e) = bus.stop() {
        //     info!("failed stopping bus: {e}");
        //     log::logger().flush();
        //     sleep(Duration::from_secs(1));
        // }

        Ok(())
    }
}
