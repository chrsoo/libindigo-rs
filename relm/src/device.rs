use gtk::prelude::*;
use libindigo::property::PropertyData;
use libindigo::property::PropertyItem;
use libindigo::NamedObject;
use log::debug;
use log::trace;
use relm4::factory::FactoryHashMap;
use relm4::{gtk, prelude::FactoryComponent, FactorySender};

use crate::property::Property;
use crate::property::PropertyInput;

/// Relm model for a [ClientDevice].
#[derive(Debug)]
pub struct Device {
    name: String,
    props: FactoryHashMap<String, Property>,
    message: Option<String>,
}

/// Connect or disconnect the INDIGO Device.
#[derive(Debug)]
pub enum DeviceInput {
    /// Add a new property to the device.
    DefineProperty(PropertyData, Option<String>),
    /// Update an existing property to the device.
    UpdateProperty(PropertyData, Option<String>),
    /// Add a new property to the device.
    DeleteProperty(String, Option<String>),
    /// Send an INDIGO message to the device.
    Message(String),
}

#[derive(Debug, Clone, PartialEq)]
/// Events for an INDIGO [ClientDevice].
pub enum DeviceOutput {
    /// Connect [Device] to INDIGO bus.
    RequestConnection,
    /// Connect [Device] to INDIGO bus.
    RequestDisconnection,
    /// Request a new [Property] to be defined on the [Device].
    RequestDefinition(String, String),
    /// Request all properties for the [Device] to be defined.
    RequestEnumeration(String),
    /// Request update of a [Property] on the [Device].
    RequestUpdate(PropertyData),
    /// Request update of an [Item] of a [Property] on the [Device].
    RequestItemUpdate(PropertyItem, String, String),
    /// Request deletion of an [Property] on the [Device].
    RequestDeletion(String, String),
    /// Send a message to the [Device].
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
            // set_spacing: 10,
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
            // .forward(sender.input_sender(), |output| match output {
            //     DeviceInput::Connected(d) => AppCommand::UpdateStatus(format!("Connected device '{d}'")),
            //     DeviceInput::Disconnected(d) => AppCommand::UpdateStatus(format!("Disconnected device '{d}'")),
            //     DeviceInput::PropertyDefined(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
            //     DeviceInput::PropertyUpdated(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
            //     DeviceInput::PropertyDeleted(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
            //     DeviceInput::Busy(d) => AppCommand::UpdateStatus(format!("Device is busy '{d}'")),
            // })
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

    // fn update_view(&self, widgets: &mut Self::Widgets, sender: FactorySender<Self>) {
    //     sender.output(DeviceInput::PropertyDefined(widgets)).unwrap();
    // }
}

// impl CloneableFactoryComponent for Device {
//     fn get_init(&self) -> Self::Init {
//         self.device.clone()
//     }
// }

impl Device {
    fn define_property(
        &mut self,
        p: PropertyData,
        m: Option<String>,
        _sender: FactorySender<Self>,
    ) {
        let name = p.name();
        self.props.insert(name.to_owned(), p);
        debug!("defined property {name}")
    }

    fn update_property(
        &mut self,
        p: PropertyData,
        m: Option<String>,
        _sender: FactorySender<Self>,
    ) {
        let name = &p.name().to_string();
        self.props.send(&name, PropertyInput::UpdateProperty(p));
        trace!("updated property {name}");
    }

    fn delete_property(&mut self, p: String, m: Option<String>, _sender: FactorySender<Self>) {
        self.props.remove(&p);
        trace!("deleted property {p}");
    }

    fn set_message(&mut self, message: String) {
        self.message = Some(message);
    }
}
