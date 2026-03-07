use gtk::prelude::*;
use libindigo_rs::Property;

use log::debug;
use log::trace;

use relm4::factory::FactoryHashMap;
use relm4::{gtk, prelude::FactoryComponent, FactorySender};

use crate::property::RelmProperty;
use crate::property::RelmPropertyInput;

/// Relm model for a Device.
#[derive(Debug)]
pub struct Device {
    name: String,
    props: FactoryHashMap<String, RelmProperty>,
    message: Option<String>,
}

/// Connect or disconnect the INDIGO Device.
#[derive(Debug)]
pub enum DeviceInput {
    /// Add a new property to the device.
    DefineProperty(Property, Option<String>),
    /// Update an existing property to the device.
    UpdateProperty(Property, Option<String>),
    /// Add a new property to the device.
    DeleteProperty(String, Option<String>),
    /// Send an INDIGO message to the device.
    Message(String),
}

#[derive(Debug, Clone, PartialEq)]
/// Events for an INDIGO Device.
pub enum DeviceOutput {
    /// Connect Device to INDIGO bus.
    RequestConnection,
    /// Connect Device to INDIGO bus.
    RequestDisconnection,
    /// Request a new Property to be defined on the Device.
    RequestDefinition(String, String),
    /// Request all properties for the Device to be defined.
    RequestEnumeration(String),
    /// Request update of a Property on the Device.
    RequestUpdate(Property),
    /// Request deletion of a Property on the Device.
    RequestDeletion(String, String),
    /// Send a message to the Device.
    SendMessage(String, String),
}

#[relm4::factory(pub)]
impl FactoryComponent for Device {
    type Init = String;
    type Input = DeviceInput;
    type Output = DeviceOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Stack;
    type Index = String;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            self.props.widget() ->
            &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,
            },
        },
        #[local_ref]
        returned_widget -> gtk::StackPage {
            set_name: self.name.as_str(),
            set_title: self.name.as_str(),
        }
    }

    fn init_model(name: Self::Init, _index: &String, sender: FactorySender<Self>) -> Self {
        let props = FactoryHashMap::builder()
            .launch(gtk::Box::default())
            .detach();

        Self {
            name,
            props,
            message: None,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            DeviceInput::DefineProperty(p, m) => self.define_property(p, m, sender),
            DeviceInput::UpdateProperty(p, m) => self.update_property(p, m, sender),
            DeviceInput::DeleteProperty(p, m) => self.delete_property(p, m, sender),
            DeviceInput::Message(m) => self.set_message(m),
        }
    }
}

impl Device {
    fn message(&self) -> &str {
        self.message
            .as_ref()
            .map(|m| m.as_str())
            .unwrap_or("<none>")
    }

    fn define_property(&mut self, p: Property, m: Option<String>, _sender: FactorySender<Self>) {
        let name = p.name.clone();
        self.props.insert(name.clone(), p);
        trace!("defined property {name}: {}", self.message());
        self.message = m;
    }

    fn update_property(&mut self, p: Property, m: Option<String>, _sender: FactorySender<Self>) {
        let name = p.name.clone();
        self.props.send(&name, RelmPropertyInput::UpdateProperty(p));
        self.message = m;
        trace!("updated property {name}: {}", self.message());
    }

    fn delete_property(&mut self, p: String, m: Option<String>, _sender: FactorySender<Self>) {
        self.props.remove(&p);
        trace!("deleted property {p}: {}", self.message());
        self.message = m;
    }

    fn set_message(&mut self, message: String) {
        self.message = Some(message);
    }
}
