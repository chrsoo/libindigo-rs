use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut},
};

use strum_macros::Display;
use url_fork::Url;

use crate::{
    BlobItem, LightItem, NamedObject, NumberFormat, NumberItem, Property, PropertyPermission,
    PropertyState as Light, PropertyState as State, PropertyType, SwitchItem, SwitchRule, TextItem,
};

#[derive(PartialEq, Debug, Clone)]
pub struct Text {
    value: String,
}

impl TextItem for Text {
    fn value(&self) -> &str {
        &self.value
    }
}

impl Text {
    pub fn new(text: &str) -> Self {
        Text {
            value: text.to_owned(),
        }
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Number {
    /// < item value (for number properties)
    value: f64,
    /// < item target value (for number properties)
    target: f64,
    // TODO convert format value to pritnf format type with support for sexagesimal
    /// < item format (for number properties)
    format: NumberFormat,
    /// < item min value (for number properties)
    min: f64,
    /// < item max value (for number properties)
    max: f64,
    /// < item increment value (for number properties)
    step: f64,
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl NumberItem for Number {
    fn value(&self) -> f64 {
        self.value
    }

    fn format(&self) -> &NumberFormat {
        &self.format
    }

    fn min(&self) -> f64 {
        self.min
    }

    fn max(&self) -> f64 {
        self.max
    }

    fn step(&self) -> f64 {
        self.step
    }

    fn target(&self) -> f64 {
        self.target
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Switch {
    value: bool,
}

impl SwitchItem for Switch {
    fn on(&self) -> bool {
        self.value
    }
}

impl Display for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Blob {
    // TODO map blob file exension to mime-type or other more structured format
    /// < item format (for blob properties), known file type suffix like \".fits\" or \".jpeg\".
    ext: String,
    /// < item value (for blob properties)
    value: Option<Vec<u8>>,
    /// < item size (for blob properties) in bytes
    size: usize,
    /// < item URL on source server
    url: Option<Url>,
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<blob>")
    }
}

impl BlobItem for Blob {
    fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    fn data(&self) -> Option<&[u8]> {
        if let Some(data) = self.value.as_ref() {
            Some(data.as_slice())
        } else {
            None
        }
    }

    fn extension(&self) -> &str {
        &self.ext
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl LightItem for Light {
    fn state(&self) -> Light {
        self.to_owned()
    }
}

#[derive(PartialEq, Debug, Clone, Display)]
pub enum PropertyValue {
    Text(Text),
    Number(Number),
    Switch(Switch),
    Light(Light),
    Blob(Blob),
}

impl PropertyValue {
    pub fn text(value: &str) -> PropertyValue {
        PropertyValue::Text(Text {
            value: value.to_owned(),
        })
    }
    pub fn number(
        value: f64,
        target: f64,
        format: NumberFormat,
        step: f64,
        max: f64,
        min: f64,
    ) -> PropertyValue {
        PropertyValue::Number(Number {
            value,
            target,
            min,
            max,
            step,
            format,
        })
    }

    pub fn light(value: Light) -> PropertyValue {
        PropertyValue::Light(value)
    }

    pub fn switch(value: bool) -> PropertyValue {
        PropertyValue::Switch(Switch { value })
    }

    pub fn blob(size: usize, ext: &str, value: Option<Vec<u8>>, url: Option<Url>) -> PropertyValue {
        PropertyValue::Blob(Blob {
            value,
            size,
            ext: ext.to_owned(),
            url,
        })
    }
}

impl From<PropertyValue> for String {
    fn from(value: PropertyValue) -> Self {
        match value {
            PropertyValue::Text(text) => text.value,
            PropertyValue::Number(number) => number.value.to_string(),
            PropertyValue::Switch(switch) => switch.value.to_string(),
            PropertyValue::Light(light) => light.to_string(),
            PropertyValue::Blob(blob) => blob
                .url
                .map_or_else(|| "blob".to_string(), |url| url.to_string()),
        }
    }
}

impl From<&PropertyValue> for PropertyType {
    fn from(value: &PropertyValue) -> Self {
        match value {
            PropertyValue::Text(_) => PropertyType::Text,
            PropertyValue::Number { .. } => PropertyType::Number,
            PropertyValue::Switch(_) => PropertyType::Switch,
            PropertyValue::Light(_) => PropertyType::Light,
            PropertyValue::Blob { .. } => PropertyType::Blob,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyItem {
    name: String,
    value: PropertyValue,
    dirty: bool,
}

impl Eq for PropertyItem {}

impl NamedObject for PropertyItem {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Display for PropertyItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            PropertyValue::Text(text) => Display::fmt(text, f),
            PropertyValue::Number(number) => Display::fmt(number, f),
            PropertyValue::Switch(switch) => Display::fmt(switch, f),
            PropertyValue::Light(state) => Display::fmt(state, f),
            PropertyValue::Blob(blob) => Display::fmt(blob, f),
        }
    }
}

impl PropertyItem {
    /// Create a new [PropertyItem].
    pub fn new(name: &str, value: PropertyValue) -> PropertyItem {
        PropertyItem {
            name: name.to_owned(),
            value,
            dirty: false,
        }
    }

    pub fn value(&self) -> &PropertyValue {
        &self.value
    }

    pub fn value_mut(&mut self) -> &PropertyValue {
        &self.value
    }

    /// Request that the device changes the item's value.
    pub fn request(&mut self, value: PropertyValue) {
        self.value = value;
        self.dirty = true;
    }

    /// Indicate if the item has pending changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Update the item with a new value and reset the [is_dirty] flag.
    pub fn update(&mut self, value: PropertyValue) {
        self.value = value;
        self.dirty = false;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyData {
    name: String,
    device: String,
    group: String,
    hints: String,
    state: State,
    type_: PropertyType,
    perm: PropertyPermission,
    rule: SwitchRule,
    hidden: bool,
    defined: bool,
    items: Vec<PropertyItem>,
}

impl NamedObject for PropertyData {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Property for PropertyData {
    fn device(&self) -> &str {
        &self.device
    }

    fn group(&self) -> &str {
        &self.group
    }

    fn hints(&self) -> &str {
        &self.hints
    }

    fn state(&self) -> &crate::PropertyState {
        &self.state
    }

    fn property_type(&self) -> &crate::PropertyType {
        &self.type_
    }

    fn perm(&self) -> &crate::PropertyPermission {
        &self.perm
    }

    fn rule(&self) -> &crate::SwitchRule {
        &self.rule
    }

    fn hidden(&self) -> bool {
        self.hidden
    }

    fn defined(&self) -> bool {
        self.defined
    }

    fn items(&self) -> impl Iterator<Item = &PropertyItem> {
        self.items.iter()
    }

    fn update(&mut self, p: &impl Property) {
        // Strings
        self.name = p.name().to_owned();
        self.device = p.device().to_owned();
        self.group = p.group().to_owned();
        self.hints = p.hints().to_owned();
        // Other
        self.hidden = p.hidden();
        self.defined = p.defined();
        self.perm = p.perm().to_owned();
        self.rule = p.rule().to_owned();
        self.state = p.state().to_owned();
        self.type_ = p.property_type().to_owned();
        // Items
        let mut i = 0;
        for item in p.items() {
            self.items[i] = item.to_owned();
            i += 1;
        }
        self.items.truncate(i);
    }
}

impl Index<usize> for PropertyData {
    type Output = PropertyItem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl IndexMut<usize> for PropertyData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl PropertyData {
    pub fn new(
        name: &str,
        device: &str,
        group: &str,
        hints: &str,
        state: State,
        type_: PropertyType,
        perm: PropertyPermission,
        rule: SwitchRule,
        hidden: bool,
        defined: bool,
        items: Vec<PropertyItem>,
    ) -> PropertyData {
        PropertyData {
            name: name.to_owned(),
            device: device.to_owned(),
            group: group.to_owned(),
            hints: hints.to_owned(),
            state,
            type_,
            perm,
            rule,
            hidden,
            defined,
            items,
        }
    }
    /// Return `true` if at least one [PropertyItem] has [requested change](PropertyItem#request).
    pub fn is_dirty(&self) -> bool {
        self.items.iter().any(|i| i.is_dirty())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}
