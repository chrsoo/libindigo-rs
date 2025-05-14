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
use function_name::named;
use libindigo_sys::*;
use log::{debug, error, info, trace, warn};
use parking_lot::{RwLock, RwLockWriteGuard};
use regex::Regex;
use serde_json_core::from_slice;
use std::{
    collections::HashMap,
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

impl Into<indigo_property_state> for &PropertyState {
    fn into(self) -> indigo_property_state {
        match self {
            PropertyState::Idle => indigo_property_state::INDIGO_IDLE_STATE,
            PropertyState::Ok => indigo_property_state::INDIGO_OK_STATE,
            PropertyState::Busy => indigo_property_state::INDIGO_BUSY_STATE,
            PropertyState::Alert => indigo_property_state::INDIGO_ALERT_STATE,
        }
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

impl Into<indigo_property_perm> for &PropertyPermission {
    fn into(self) -> indigo_property_perm {
        match self {
            PropertyPermission::ReadOnly => indigo_property_perm::INDIGO_RO_PERM,
            PropertyPermission::ReadWrite => indigo_property_perm::INDIGO_RW_PERM,
            PropertyPermission::WriteOnly => indigo_property_perm::INDIGO_WO_PERM,
        }
    }
}

impl Into<indigo_rule> for &SwitchRule {
    fn into(self) -> indigo_rule {
        match self {
            SwitchRule::OneOfMany => indigo_rule::INDIGO_ONE_OF_MANY_RULE,
            SwitchRule::AtMostOne => indigo_rule::INDIGO_AT_MOST_ONE_RULE,
            SwitchRule::AnyOfMany => indigo_rule::INDIGO_ANY_OF_MANY_RULE,
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

impl From<&PropertyState> for u32 {
    fn from(s: &PropertyState) -> Self {
        match s {
            PropertyState::Idle => indigo_property_state::INDIGO_IDLE_STATE.0,
            PropertyState::Ok => indigo_property_state::INDIGO_OK_STATE.0,
            PropertyState::Busy => indigo_property_state::INDIGO_BUSY_STATE.0,
            PropertyState::Alert => indigo_property_state::INDIGO_ALERT_STATE.0,
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

impl From<&PropertyType> for u32 {
    fn from(t: &PropertyType) -> Self {
        match t {
            PropertyType::Text => indigo_property_type::INDIGO_TEXT_VECTOR.0,
            PropertyType::Number => indigo_property_type::INDIGO_NUMBER_VECTOR.0,
            PropertyType::Switch => indigo_property_type::INDIGO_SWITCH_VECTOR.0,
            PropertyType::Light => indigo_property_type::INDIGO_LIGHT_VECTOR.0,
            PropertyType::Blob => indigo_property_type::INDIGO_BLOB_VECTOR.0,
        }
    }
}

impl From<&PropertyPermission> for u32 {
    fn from(p: &PropertyPermission) -> Self {
        match p {
            PropertyPermission::ReadOnly => indigo_property_perm::INDIGO_RO_PERM.0,
            PropertyPermission::ReadWrite => indigo_property_perm::INDIGO_RW_PERM.0,
            PropertyPermission::WriteOnly => indigo_property_perm::INDIGO_WO_PERM.0,
        }
    }
}

impl From<&SwitchRule> for u32 {
    fn from(r: &SwitchRule) -> Self {
        match r {
            SwitchRule::OneOfMany => indigo_rule::INDIGO_ONE_OF_MANY_RULE.0,
            SwitchRule::AtMostOne => indigo_rule::INDIGO_AT_MOST_ONE_RULE.0,
            SwitchRule::AnyOfMany => indigo_rule::INDIGO_ANY_OF_MANY_RULE.0,
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
pub struct SysProperty {
    sys: *mut indigo_property,
    type_: PropertyType,
    items: Vec<PropertyItem>,
}

impl SysProperty {
    fn new(name: &str, ptype: PropertyType, device: &str) -> SysProperty {
        let sys = Box::new(indigo_property::new(name, device, (&ptype).into()));

        // let n = sys.count as usize;
        // let items = unsafe { sys.items.as_slice(n) }
        //     .iter()
        //     .map(|i| SysPropertyItem::new(i, &ptype))
        //     .collect();

        SysProperty {
            sys: Box::into_raw(sys),
            type_: ptype,
            items: Vec::new(),
        }
    }
}

impl Debug for SysProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SysProperty")
            .field("sys", &self.sys)
            .field("property_type", &self.type_)
            // .field("items", &self.items())
            .finish()
    }
}
/// Convert an [indigo_property] to a [SysProperty] failing on first error.
impl TryFrom<*mut indigo_property> for SysProperty {
    type Error = IndigoError;
    fn try_from(value: *mut indigo_property) -> IndigoResult<Self> {
        let sys = unsafe { &*value };
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

        Ok(SysProperty {
            sys: value,
            type_,
            items: items?,
        })
    }
}

/// Convert INDIGO text to [PropertyValue].
impl TryFrom<indigo_item__bindgen_ty_1__bindgen_ty_1> for PropertyValue {
    type Error = IndigoError;
    fn try_from(text: indigo_item__bindgen_ty_1__bindgen_ty_1) -> Result<Self, Self::Error> {
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
impl TryFrom<indigo_item__bindgen_ty_1__bindgen_ty_2> for PropertyValue {
    type Error = IndigoError;
    fn try_from(number: indigo_item__bindgen_ty_1__bindgen_ty_2) -> Result<Self, Self::Error> {
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
impl TryFrom<indigo_item__bindgen_ty_1__bindgen_ty_3> for PropertyValue {
    type Error = IndigoError;
    fn try_from(switch: indigo_item__bindgen_ty_1__bindgen_ty_3) -> Result<Self, Self::Error> {
        Ok(PropertyValue::switch(switch.value))
    }
}

/// Convert INDIGO light to [PropertyValue].
impl TryFrom<indigo_item__bindgen_ty_1__bindgen_ty_4> for PropertyValue {
    type Error = IndigoError;
    fn try_from(light: indigo_item__bindgen_ty_1__bindgen_ty_4) -> Result<Self, Self::Error> {
        if let Some(value) = PropertyState::from_u32(light.value.0) {
            Ok(PropertyValue::light(value))
        } else {
            Err(IndigoError::new("missing light value"))
        }
    }
}

impl From<ParseError> for IndigoError {
    fn from(value: ParseError) -> Self {
        IndigoError::new(value.to_string().as_str())
    }
}

/// Convert INDIGO blob to [PropertyValue].
impl TryFrom<indigo_item__bindgen_ty_1__bindgen_ty_5> for PropertyValue {
    type Error = IndigoError;
    fn try_from(blob: indigo_item__bindgen_ty_1__bindgen_ty_5) -> Result<Self, Self::Error> {
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
    // fn url(&self) -> Option<&Url> {
    //     Option::as_ref(&self.url)
    // }

    // fn data(&self) -> Option<&[u8]> {
    //     let v = unsafe { self.sys.__bindgen_anon_1.blob };
    //     if v.value.is_null() {
    //         None
    //     } else {
    //         Some(unsafe { slice::from_raw_parts(v.value as *const u8, v.size as usize) })
    //     }
    // }

    // fn extension(&self) -> &str {
    //     buf_to_str(unsafe { &self.sys.__bindgen_anon_1.blob.format })
    // }

    // fn size(&self) -> usize {
    //     unsafe { self.sys.__bindgen_anon_1.blob.size as usize }
    // }
}

impl NamedObject for SysProperty {
    fn name(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).name) }
    }

    fn label(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).label) }
    }
}

impl Property for SysProperty {
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
            indigo_property_state::INDIGO_IDLE_STATE => &PropertyState::Idle,
            indigo_property_state::INDIGO_OK_STATE => &PropertyState::Ok,
            indigo_property_state::INDIGO_BUSY_STATE => &PropertyState::Busy,
            indigo_property_state::INDIGO_ALERT_STATE => &PropertyState::Alert,
            s => unimplemented!("property state '{}' not implemented", s.0),
        }
    }

    fn property_type(&self) -> &PropertyType {
        &self.type_
    }

    fn perm(&self) -> &PropertyPermission {
        match unsafe { (*self.sys).perm } {
            indigo_property_perm::INDIGO_RO_PERM => &PropertyPermission::ReadOnly,
            indigo_property_perm::INDIGO_RW_PERM => &PropertyPermission::ReadWrite,
            indigo_property_perm::INDIGO_WO_PERM => &PropertyPermission::WriteOnly,
            p => unimplemented!("property permission '{}' not implemented", p.0),
        }
    }

    fn rule(&self) -> &SwitchRule {
        match unsafe { (*self.sys).rule } {
            indigo_rule::INDIGO_ANY_OF_MANY_RULE => &SwitchRule::AnyOfMany,
            indigo_rule::INDIGO_AT_MOST_ONE_RULE => &SwitchRule::AtMostOne,
            indigo_rule::INDIGO_ONE_OF_MANY_RULE => &SwitchRule::OneOfMany,
            r => unimplemented!("switch rule '{}' not implemented", r.0),
        }
    }

    fn hidden(&self) -> bool {
        unsafe { (*self.sys).hidden }
    }

    fn defined(&self) -> bool {
        unsafe { (*self.sys).defined }
    }

    fn items(&self) -> impl Iterator<Item = &PropertyItem> {
        self.items.iter()
    }

    // fn update(&mut self, p: &Self) {
    //     if self.sys == p.sys {
    //         trace!("skipping update of same property instance");
    //         return;
    //     }
    //     trace!("updating property by copying values");
    //     let sys = unsafe { &mut *self.sys };

    //     copy_from_str(sys.device, p.device());
    //     copy_from_str(sys.device, p.device());
    //     copy_from_str(sys.group, p.group());
    //     copy_from_str(sys.hints, p.hints());
    //     sys.state = p.state().into();
    //     sys.type_ = p.property_type().into();
    //     sys.perm = p.perm().into();
    //     sys.rule = p.rule().into();
    //     sys.hidden = p.hidden();
    //     sys.defined = p.defined();

    //     // let all: HashSet<&Self::Item> = self.items().collect();
    //     // for item in &mut self.items {
    //     //     item.update()
    //     // }

    //     /* TODO items */
    // }
}

impl Display for SysProperty {
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

impl From<SysProperty> for PropertyData {
    fn from(p: SysProperty) -> Self {
        let items = p.items()
            .map(|i| i.to_owned())
            .collect()
            ;
        PropertyData::new(
            p.name(),
            p.device(),
            p.group(),
            p.hints(),
            p.state().to_owned(),
            p.property_type().to_owned(),
            p.perm().to_owned(),
            p.rule().to_owned(),
            p.hidden(),
            p.defined(),
            items,
        )
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

impl Into<indigo_property_type> for &PropertyType {
    fn into(self) -> indigo_property_type {
        match self {
            PropertyType::Text => indigo_property_type::INDIGO_TEXT_VECTOR,
            PropertyType::Number => indigo_property_type::INDIGO_NUMBER_VECTOR,
            PropertyType::Switch => indigo_property_type::INDIGO_SWITCH_VECTOR,
            PropertyType::Light => indigo_property_type::INDIGO_LIGHT_VECTOR,
            PropertyType::Blob => indigo_property_type::INDIGO_BLOB_VECTOR,
        }
    }
}

// -- SysClientController -----------------------------------------------------
/// Client to manage devices attached to the INDIGO [Bus].
pub struct SysClientController<D>
where
    D: ClientDelegate<Property = SysProperty, Bus = SysBus>,
{
    sys: *mut indigo_client,
    delegate: PhantomData<D>,
}

impl<'s, D> NamedObject for SysClientController<D>
where
    D: ClientDelegate<
        Property = SysProperty,
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
        Property = SysProperty,
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

// impl<'s, D: ClientDelegate<SysProperty,SysBus,Self>> Drop for SysClientController<D> {
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
        Property = SysProperty,
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
        SysProperty,
        Option<&'b str>,
    )> {
        let object = Self::write_lock(c);
        let c = buf_to_str(&(unsafe { &*c }).name);
        let d = buf_to_str(&(unsafe { &*d }).name);
        let p = SysProperty::try_from(p)?;
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
        let property = SysProperty::try_from(p)
            .inspect_err(|e| warn!("could not create property: {e}"))
            .expect("valid system property");
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
}

impl<D> ClientController<SysProperty, SysBus> for SysClientController<D>
where
    D: ClientDelegate<
        Property = SysProperty,
        Bus = SysBus,
        BusController = Self,
        ClientController = Self,
    >,
{
    fn request_definition<'a>(&mut self, d: &'a str, p: &'a str) -> IndigoResult<()> {
        let sys = &mut indigo_property::new(p, d, indigo_property_type::INDIGO_TEXT_VECTOR);
        let code = unsafe { indigo_enumerate_properties(self.sys, sys) };
        sys_code_to_lib_result((), "indigo_enumerate_properties", code)
    }

    fn request_update<'a>(&mut self, _d: &'a str, p: &'a SysProperty) -> IndigoResult<()> {
        let code = unsafe { indigo_change_property(self.sys, p.sys) };
        sys_code_to_lib_result((), "indigo_enumerate_properties", code)
    }

    fn request_update_item<'a>(
        &mut self,
        _d: &'a str,
        _p: &'a SysProperty,
        _i: &'a PropertyItem,
    ) -> IndigoResult<()> {
        todo!()
    }

    fn request_delete<'a>(&mut self, _d: &'a str, _p: &'a SysProperty) -> IndigoResult<()> {
        todo!()
    }

    fn request_delete_item<'a>(
        &mut self,
        _d: &'a str,
        _p: &'a SysProperty,
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
        Property = SysProperty,
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
    D: DeviceDelegate<Property = SysProperty, Bus = SysBus>,
{
    sys: *mut indigo_device,
    _phantom: PhantomData<D>,
}

// impl<'s,D> Display for SysDeviceController<'s,D>
// where D: DeviceDelegate<SysProperty> {
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
    D: DeviceDelegate<Property = SysProperty, Bus = SysBus>,
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
    D: DeviceDelegate<Property = SysProperty, Bus = SysBus>,
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
    D: DeviceDelegate<Property = SysProperty, Bus = SysBus>,
{
    fn name(&self) -> &str {
        unsafe { buf_to_str(&(*self.sys).name) }
    }
}

impl<'s, D> Drop for SysDeviceController<D>
where
    D: DeviceDelegate<Property = SysProperty, Bus = SysBus>,
{
    fn drop(&mut self) {
        let sys = unsafe { Box::from_raw(self.sys) };
        drop(sys);
    }
}

impl<'s, D> Controller<SysBus> for SysDeviceController<D>
where
    D: DeviceDelegate<Property = SysProperty, Bus = SysBus>,
{
    fn attach<'a>(&mut self, _: &'a mut SysBus) -> IndigoResult<()> {
        todo!()
    }

    fn detach(&mut self) -> IndigoResult<()> {
        todo!()
    }
}

impl<'s, D> DeviceController<SysProperty, SysBus> for SysDeviceController<D>
where
    D: DeviceDelegate<Property = SysProperty, Bus = SysBus>,
{
    fn define_property<'a>(&mut self, p: &'a SysProperty) -> IndigoResult<()> {
        let code = unsafe { indigo_update_property(self.sys, p.sys, ptr::null()) };
        sys_code_to_lib_result((), "indigo_enumerate_properties", code)
    }

    fn update_property<'a>(&mut self, p: &'a SysProperty) -> IndigoResult<()> {
        let code = unsafe { indigo_update_property(self.sys, p.sys, ptr::null()) };
        sys_code_to_lib_result((), "indigo_update_property", code)
    }

    fn delete_property<'a>(&mut self, p: &'a SysProperty) -> IndigoResult<()> {
        let code = unsafe { indigo_update_property(self.sys, p.sys, ptr::null()) };
        sys_code_to_lib_result((), "indigo_update_property", code)
    }

    fn broadcast_message(&mut self, msg: &str) -> IndigoResult<()> {
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

// -- SysBus ------------------------------------------------------------------

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
    // ) -> IndigoResult<impl DeviceController<SysProperty, D>, impl Error> {
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

    use super::{str_to_buf, SysBus, SysProperty};

    fn init_test(level: LevelFilter) {
        let _ = env_logger::builder()
            // .target(env_logger::Target::Stdout)
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
            state: indigo_property_state::INDIGO_OK_STATE,
            type_: indigo_property_type::INDIGO_TEXT_VECTOR,
            perm: indigo_property_perm::INDIGO_RW_PERM,
            rule: indigo_rule::INDIGO_ANY_OF_MANY_RULE,
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
    fn test_sys_property() -> IndigoResult<()> {
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
        _phantom: PhantomData<SysProperty>,
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
        type Property = SysProperty;
        type ClientController = SysClientController<Self>;

        fn on_define_property<'a>(
            &'a mut self,
            _c: &mut SysClientController<Self>,
            _d: &'a str,
            p: SysProperty,
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
