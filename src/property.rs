use std::{ffi::{c_int, c_long}, fmt::Display};

use enum_primitive::*;
use libindigo_sys::{self, *};
use log::warn;
use url::Url;

use crate::{buf_to_str, buf_to_str2};

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
    }
}

impl PropertyValue {

    fn item_to_text(item: &indigo_item) -> PropertyValue {
        let value = unsafe { item.__bindgen_anon_1.text.as_ref()};
        let text: String = if value.long_value.is_null() {
            buf_to_str(value.value)
        } else {
            todo!("read long text value")
            // let buf = [0u8;value.long_value];
            // buf_to_str(value.long_value)
        };
        PropertyValue::Text(text)
    }

    fn item_to_number(item: &indigo_item) -> PropertyValue {
        let num = unsafe { item.__bindgen_anon_1.number.as_ref() };
        let format = buf_to_str(num.format);
        let min = num.min;
        let max = num.max;
        let step = num.step;
        let target = num.target;
        let value = num.value;

        PropertyValue::Number { format, min, max, step, value, target }
    }

    fn item_to_switch(item: &indigo_item) -> PropertyValue {
        PropertyValue::Switch(unsafe { item.__bindgen_anon_1.sw.as_ref().value })
    }

    fn item_to_light(item: &indigo_item) -> PropertyValue {
        PropertyValue::Light(unsafe { item.__bindgen_anon_1.light.as_ref().value })
    }

    fn item_to_blob(item: &indigo_item) -> PropertyValue {
        let blob = unsafe { item.__bindgen_anon_1.blob.as_ref() };
        let format = buf_to_str(blob.format);
        let size = blob.size;
        let url = match Url::parse(buf_to_str2(blob.url)) {
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
        PropertyValue::Blob { format, size, url, value }
    }

    fn new(property_type: PropertyType, item: &indigo_item) -> PropertyValue {
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

pub struct PropertyItem {
    name: String,
    label: String,
    hints: String,
    value: PropertyValue,
}

impl PropertyItem {
    fn new(property_type: PropertyType, item: &indigo_item) -> PropertyItem {
        let name = buf_to_str(item.name);
        let label = buf_to_str(item.label);
        let hints = buf_to_str(item.hints);
        let value = PropertyValue::new(property_type, item);

        PropertyItem {
            name, label, hints, value
        }
    }
}

pub struct Property<'a> {
    sys: &'a indigo_property,
}

impl Display for Property<'static> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Property[Name: {}; Device: {}; Group: {}; Label: {}; Hints: {}]",
            self.name(),
            self.device(),
            self.group(),
            self.label(),
            self.hints(),
        )
    }
}

impl<'a> Property<'a> {

    pub(crate) fn new(property: *mut indigo_property) -> Self {
        Self { sys: unsafe { &*property } }
    }

    // -- getters

    pub fn name(&self) -> String {
        buf_to_str(self.sys.name)
    }

    pub fn device(&self) -> String {
        buf_to_str(self.sys.device)
    }

    pub fn group(&self) -> String {
        buf_to_str(self.sys.group)
    }

    pub fn label(&self) -> String {
        buf_to_str(self.sys.label)
    }

    pub fn hints(&self) -> String {
        buf_to_str(self.sys.hints)
    }

    pub fn state(&self) -> PropertyState {
        PropertyState::from_u32(self.sys.state).unwrap()
    }

    pub fn property_type(&self) -> PropertyType {
        PropertyType::from_u32(self.sys.type_).unwrap()
    }

    pub fn perm(&self) -> PropertyPermission {
        PropertyPermission::from_u32(self.sys.perm).unwrap()
    }

    /// Switch behaviour rule (for switch properties).
    pub fn rule(&self) -> SwitchRule {
        SwitchRule::from_u32(self.sys.rule).unwrap()
    }

    /// `true`if `Property` is hidden/unused by  driver (for optional properties).
    pub fn hidden(&self) -> bool {
        self.sys.hidden
    }

    /// `true` if `Property` is defined.
    pub fn defined(&self) -> bool {
        self.sys.defined
    }

    /// Number of allocated property items.
    pub fn items_allocated(&self) -> c_int {
        self.sys.allocated_count
    }

    /// Number of used property items.
    pub fn items_used(&self) -> c_int {
        self.sys.count
    }

    pub fn items(&self) -> Vec<PropertyItem> {
        let items = unsafe { self.sys.items.as_slice(self.sys.count as usize) };
        items.iter()
            .map(|i| PropertyItem::new(self.property_type(), i))
            .collect()
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
