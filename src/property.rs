use std::{
    cmp::Ordering, ffi::{c_long, CStr}, fmt::Display, ptr
};

use enum_primitive::*;
use libindigo_sys::{self, *};
use log::warn;
use url::Url;

use crate::{buf_to_str, buf_to_string};

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[repr(u32)]
    /// Possible states of a `Property`.
    pub enum PropertyState  {
        /// Property is passive (unused by INDIGO).
        Idle = indigo_property_state_INDIGO_IDLE_STATE,
        /// Property is in correct state or if operation on property was successful.
        Ok = indigo_property_state_INDIGO_OK_STATE,
        /// Property is transient state or if operation on property is pending.
        Busy = indigo_property_state_INDIGO_BUSY_STATE,
        /// Property is in incorrect state or if operation on property failed.
        Alert = indigo_property_state_INDIGO_ALERT_STATE,
    }
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[repr(u32)]
    /// Possible states of a `Property`.
    pub enum PropertyPermission  {
        ReadOnly = indigo_property_perm_INDIGO_RO_PERM,
        ReadWrite = indigo_property_perm_INDIGO_RW_PERM,
        WriteOnly = indigo_property_perm_INDIGO_WO_PERM,
    }
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[repr(u32)]
    /// Possible property types.
    pub enum PropertyType  {
        /// Strings of limited width.
        Text = indigo_property_type_INDIGO_TEXT_VECTOR,
        /// Float numbers with defined min, max values and increment.
        Number = indigo_property_type_INDIGO_NUMBER_VECTOR,
        /// Logical values representing “on” and “off” state.
        Switch = indigo_property_type_INDIGO_SWITCH_VECTOR,
        /// Status values with four possible values Idle, Ok, Busy, and Alert.
        Light = indigo_property_type_INDIGO_LIGHT_VECTOR,
        /// Binary data of any type and any length.
        Blob = indigo_property_type_INDIGO_BLOB_VECTOR,
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum PropertyValue {
    Text(String),
    Number {
        // TODO convert number format to pritnf format type with support for sexagesimal
        /// < item format (for number properties)
        format: String,
        /// < item min value (for number properties)
        min: f64,
        /// < item max value (for number properties)
        max: f64,
        /// < item increment value (for number properties)
        step: f64,
        /// < item value (for number properties)
        value: f64,
        /// < item target value (for number properties)
        target: f64,
    },
    Switch(bool),
    Light(u32),
    Blob {
        // TODO map blob file exension to mime-type or other more structured format
        /// < item format (for blob properties), known file type suffix like \".fits\" or \".jpeg\".
        format: String,
        /// < item URL on source server
        url: Option<Url>,
        /// < item size (for blob properties) in bytes
        size: c_long,
        /// < item value (for blob properties)
        value: Option<Vec<u8>>,
    },
}

impl Display for PropertyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyValue::Text(s) => write!(f, "{s}"),
            PropertyValue::Number {
                format,
                min,
                max,
                step,
                value,
                target,
            } => {
                write!(
                    f,
                    "format: '{format}'; min: {min}; max: {max}; \
                step: {step}; value: {value}; target: {target}"
                )
            }
            PropertyValue::Switch(v) => write!(f, "{v}"),
            PropertyValue::Light(n) => write!(f, "{n}"),
            PropertyValue::Blob {
                format,
                url,
                size,
                value,
            } => {
                write!(
                    f,
                    "format: '{format}'; size: {size}; value: {}; url: '{}'",
                    value.is_some(),
                    "todo"
                )
            }
        }
    }
}

impl Eq for Property { }

impl PartialEq for Property {
    fn eq(&self, other: &Self) -> bool {
        self.sys == other.sys
    }
}

impl Ord for Property {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(&other.name())
    }
}

impl PartialOrd for Property {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sys.partial_cmp(&other.sys)
    }
}

impl PropertyValue {
    fn item_to_text(item: &indigo_item) -> PropertyValue {
        let v = unsafe { item.__bindgen_anon_1.text.as_ref() };
        let text: String = if v.long_value.is_null() {
            buf_to_string(v.value)
        } else {
            let buf = unsafe {
                ptr::slice_from_raw_parts(v.long_value as *const u8, v.length as usize).as_ref()
            };
            let buf = buf.unwrap();
            CStr::from_bytes_until_nul(buf)
                .unwrap()
                .to_string_lossy()
                .to_string()
        };
        PropertyValue::Text(text)
    }

    fn item_to_number(item: &indigo_item) -> PropertyValue {
        let num = unsafe { item.__bindgen_anon_1.number.as_ref() };
        let format = buf_to_string(num.format);
        let min = num.min;
        let max = num.max;
        let step = num.step;
        let target = num.target;
        let value = num.value;

        PropertyValue::Number {
            format,
            min,
            max,
            step,
            value,
            target,
        }
    }

    fn item_to_switch(item: &indigo_item) -> PropertyValue {
        PropertyValue::Switch(unsafe { item.__bindgen_anon_1.sw.as_ref().value })
    }

    fn item_to_light(item: &indigo_item) -> PropertyValue {
        PropertyValue::Light(unsafe { item.__bindgen_anon_1.light.as_ref().value })
    }

    fn item_to_blob(item: &indigo_item) -> PropertyValue {
        let blob = unsafe { item.__bindgen_anon_1.blob.as_ref() };
        let format = buf_to_string(blob.format);
        let size = blob.size;
        let url = match Url::parse(buf_to_str(blob.url)) {
            Ok(url) => Some(url),
            Err(e) => {
                warn!("could not parse url: {}", e);
                None
            }
        };
        let value = if blob.value.is_null() {
            None
        } else {
            // TODO read blob byte vector from blob value
            Some(Vec::new())
        };
        PropertyValue::Blob {
            format,
            size,
            url,
            value,
        }
    }

    /// Create a new [PropertyValue] from an [indigo_item] struct.
    fn sys(property_type: &PropertyType, item: &indigo_item) -> PropertyValue {
        match property_type {
            PropertyType::Text => PropertyValue::item_to_text(item),
            PropertyType::Number => PropertyValue::item_to_number(item),
            PropertyType::Switch => PropertyValue::item_to_switch(item),
            PropertyType::Light => PropertyValue::item_to_light(item),
            PropertyType::Blob => PropertyValue::item_to_blob(item),
        }
    }
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
/// Possible property types.
pub enum SwitchRule  {
    /// Radio button group like behaviour with one switch in \"on\" state.
    OneOfMany = indigo_rule_INDIGO_ONE_OF_MANY_RULE,
    /// Radio button group like behaviour with none or one switch in \"on\" state.
    AtMostOne = indigo_rule_INDIGO_AT_MOST_ONE_RULE,
    /// Checkbox button group like behaviour.
    AnyOfMany = indigo_rule_INDIGO_ANY_OF_MANY_RULE,
}
}

#[derive(Clone)]
pub struct PropertyItem {
    pub name: String,
    pub label: String,
    pub hints: String,
    pub value: PropertyValue,
}

impl<'a> Display for PropertyItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}): '{}'", self.label, self.name, self.value)
    }
}

impl Eq for PropertyItem { }
impl PartialEq for PropertyItem {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}
impl PartialOrd for PropertyItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
impl Ord for PropertyItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl<'a> PropertyItem {
    /// Create a new [PropertyItem] from an [indigo_item].
    fn sys(prop: &'a PropertyType, item: &indigo_item) -> PropertyItem {
        let name = buf_to_string(item.name);
        let label = buf_to_string(item.label);
        let hints = buf_to_string(item.hints);
        let value = PropertyValue::sys(prop, item);

        PropertyItem {
            name,
            label,
            hints,
            value,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn hints(&self) -> &str {
        &self.hints
    }
    pub fn value(&self) -> &PropertyValue {
        &self.value
    }
}

/// Defines [items](PropertyItem) holding the [values](PropertyValue) of the property for
/// an INDIGO [device](crate::Device).
///
/// From the [INDIGO client documentation](https://github.com/indigo-astronomy/indigo/blob/master/indigo_docs/CLIENT_DEVELOPMENT_BASICS.md#properties):
/// > In case the client needs to check the values of some property item of a
/// > specified device it is always a good idea to check if the property is in OK state:
/// > ```rust
/// > if (!strcmp(device->name, "CCD Imager Simulator @ indigosky") &&
/// >     !strcmp(property->name, CCD_IMAGE_PROPERTY_NAME) &&
/// >     property->state == INDIGO_OK_STATE) {
/// > 			...
/// > }
/// > ```
/// > And if the client needs to change some item value this code may help:
/// > ```
/// > static const char * items[] = { CCD_IMAGE_FORMAT_FITS_ITEM_NAME };
/// > static bool values[] = { true };
/// > indigo_change_switch_property(
/// > 	client,
/// > 	CCD_SIMULATOR,
/// > 	CCD_IMAGE_FORMAT_PROPERTY_NAME,
/// > 	1,
/// > 	items,
/// > 	values
/// > );
/// > ```
pub struct Property {
    sys: *mut indigo_property,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct PropertyKey {
    pub dev: String,
    pub name: String,
}

impl Display for PropertyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.dev, self.name)
    }
}

enum_from_primitive! {
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]  // this really should be `c_uint` to safeguard agains platform specifics.
/// Bus operation return status.
pub enum BlobMode {
    Also = indigo_enable_blob_mode_INDIGO_ENABLE_BLOB_ALSO,
    Never = indigo_enable_blob_mode_INDIGO_ENABLE_BLOB_NEVER,
    URL = indigo_enable_blob_mode_INDIGO_ENABLE_BLOB_URL,
}
}

#[derive(Debug)]
pub struct Blob {
    prop: PropertyKey,
    mode: BlobMode,
}

impl Blob {
    pub fn new(prop: PropertyKey, mode: BlobMode) -> Blob {
        Blob { prop, mode }
    }
}

impl Display for Property {
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
        for item in self {
            write!(f, "    {item}")?;
        }
        Ok(())
    }
}

impl Property {
    pub(crate) fn new(property: *mut indigo_property) -> Self {
        Self { sys: property }
    }

    // -- getters

    pub fn key(&self) -> PropertyKey {
        PropertyKey {
            dev: self.device(),
            name: self.name(),
        }
    }

    pub fn name(&self) -> String {
        buf_to_string((unsafe { &*self.sys }).name)
    }

    pub fn device(&self) -> String {
        buf_to_string((unsafe { &*self.sys }).device)
    }

    pub fn group(&self) -> String {
        buf_to_string((unsafe { &*self.sys }).group)
    }

    pub fn label(&self) -> String {
        buf_to_string((unsafe { &*self.sys }).label)
    }

    pub fn hints(&self) -> String {
        buf_to_string((unsafe { &*self.sys }).hints)
    }

    pub fn state(&self) -> PropertyState {
        PropertyState::from_u32((unsafe { &*self.sys }).state).unwrap()
    }

    pub fn property_type(&self) -> PropertyType {
        PropertyType::from_u32((unsafe { &*self.sys }).type_).unwrap()
    }

    pub fn perm(&self) -> PropertyPermission {
        PropertyPermission::from_u32((unsafe { &*self.sys }).perm).unwrap()
    }

    /// Switch behaviour rule (for switch properties).
    pub fn rule(&self) -> SwitchRule {
        SwitchRule::from_u32((unsafe { &*self.sys }).rule).unwrap()
    }

    /// `true`if `Property` is hidden/unused by  driver (for optional properties).
    pub fn hidden(&self) -> bool {
        (unsafe { &*self.sys }).hidden
    }

    /// `true` if `Property` is defined.
    pub fn defined(&self) -> bool {
        (unsafe { &*self.sys }).defined
    }

    /// Number of allocated property items.
    pub fn items_allocated(&self) -> usize {
        (unsafe { &*self.sys }).allocated_count as usize
    }

    /// Number of used property items.
    pub fn items_used(&self) -> usize {
        (unsafe { &*self.sys }).count as usize
    }

    pub fn update(&mut self, p: &Property) {
        self.sys = p.sys;
    }

    pub fn items(&self) -> PropertyItemIterator {
        PropertyItemIterator {
            property_type: self.property_type(),
            items: unsafe { (&*self.sys).items.as_slice((&*self.sys).count as usize) },
            index: 0,
        }
    }

    pub fn get_item(&self, name: &str) -> Option<PropertyItem> {
        self.items()
            .filter(|i| i.name == name)
            .nth(0)
    }

    pub fn get_mut_item(&mut self, name: &str) -> Option<PropertyItem> {
        self.items()
            .filter(|i| i.name == name)
            .nth(0)
    }

    /*
    #[doc = "< allow change request on locked device"]
    pub access_token: indigo_token,
    #[doc = "< property version INDIGO_VERSION_NONE, INDIGO_VERSION_LEGACY or INDIGO_VERSION_2_0"]
    pub version: ::std::os::raw::c_short,

    #[doc = "< property items"]
    pub items: __IncompleteArrayField<indigo_item>,
    */
}

impl<'a> IntoIterator for &'a Property {
    type Item = PropertyItem;
    type IntoIter = PropertyItemIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let items = unsafe { (&*self.sys).items.as_slice((&*self.sys).count as usize) };
        let property_type = self.property_type();
        Self::IntoIter {
            property_type,
            items,
            index: 0,
        }
    }
}

/// Iterator for all `PropertyItem` items of this `Property`.
pub struct PropertyItemIterator<'a> {
    property_type: PropertyType,
    items: &'a [indigo_item],
    index: usize,
}

impl<'a> Iterator for PropertyItemIterator<'a> {
    type Item = PropertyItem;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.items.len() {
            let result = PropertyItem::sys(&self.property_type, &self.items[self.index]);
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}
