use std::collections::HashMap;

use gtk::prelude::*;
use gtk::StackPage;
use libindigo::ClientDevice;
use libindigo::DeviceController as IndigoDevice;
use libindigo::Property as IndigoProperty;
use relm4::factory::CloneableFactoryComponent;
use relm4::factory::FactoryHashMap;
use relm4::{
    gtk, prelude::FactoryComponent, FactorySender
};

use crate::property::Property;

/// Relm model for a [ClientDevice].
pub struct Device {
    name: String,
    props: FactoryHashMap<String,Property>,
    message: Option<String>,
}

/// Connect or disconnect the INDIGO Device.
#[derive(Debug)]
pub enum DeviceCommand {
    /// Connect the device to the INDIGO bus.
    Connect,
    /// Disconnect the device from the INDIGO bus.
    Disconnect,
    /// Add a new property to the device.
    DefineProperty(IndigoProperty),
    /// Add a new property to the device.
    UpdateProperty(IndigoProperty),
    /// Add a new property to the device.
    DeleteProperty(IndigoProperty),
    /// Send an INDIGO message to the device.
    Message(String)
}


#[derive(Debug, Clone, PartialEq)]
/// Events for an INDIGO [ClientDevice].
pub enum DeviceEvent  {
    /// Device is connected to the INDIGO bus.
    Connected(String),
    /// Device is disconnected from the INDIGO bus.
    Disconnected(String),
    /// A new property was defined for the device.
    PropertyDefined(String),
    /// A property was updated.
    PropertyUpdated(String),
    /// A property was deleted.
    PropertyDeleted(String),
    /// Device is busy changing its connection status.
    Busy(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for Device {
    type Init = String;
    type Input = DeviceCommand;
    type Output = DeviceEvent;
    type CommandOutput = DeviceEvent;
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
            set_name: self.device.name(),
            set_title: self.device.name(),
        }
    }

    fn init_model(name: Self::Init, _index: &String, _sender: FactorySender<Self>) -> Self {
        let props = FactoryHashMap::builder()
            .launch(gtk::Box::default())
            .detach();
            // .forward(sender.input_sender(), |output| match output {
            //     DeviceEvent::Connected(d) => AppCommand::UpdateStatus(format!("Connected device '{d}'")),
            //     DeviceEvent::Disconnected(d) => AppCommand::UpdateStatus(format!("Disconnected device '{d}'")),
            //     DeviceEvent::PropertyDefined(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
            //     DeviceEvent::PropertyUpdated(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
            //     DeviceEvent::PropertyDeleted(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
            //     DeviceEvent::Busy(d) => AppCommand::UpdateStatus(format!("Device is busy '{d}'")),
            // });

        Self { name, props, message: None }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            DeviceCommand::Connect => todo!(),
            DeviceCommand::Disconnect => todo!(),
            DeviceCommand::DefineProperty(p) => self.define_property(p, sender),
            DeviceCommand::UpdateProperty(p) => self.update_property(p, sender),
            DeviceCommand::DeleteProperty(p) => self.delete_property(p, sender),
            DeviceCommand::Message(m) => self.message = Some(m),
        }
    }

    // fn update_view(&self, widgets: &mut Self::Widgets, sender: FactorySender<Self>) {
    //     sender.output(DeviceEvent::PropertyDefined(widgets)).unwrap();
    // }
}

impl CloneableFactoryComponent for Device {
    fn get_init(&self) -> Self::Init {
        self.device.clone()
    }
}

impl Device {

    fn define_property(&mut self, p: IndigoProperty, sender: FactorySender<Self>) {
        let name = p.name().to_owned();
        // self.device.upsert_property(p).unwrap();
        self.props.insert(name.to_owned(), p);
        sender.output(DeviceEvent::PropertyDefined(name)).unwrap();
    }

    fn update_property(&mut self, p: IndigoProperty, sender: FactorySender<Self>) {
        let name = p.name().to_string();
        self.device.upsert_property(p).unwrap();
        sender.output(DeviceEvent::PropertyUpdated(name)).unwrap();
    }

    fn delete_property(&mut self, p: IndigoProperty, sender: FactorySender<Self>) {
        let name = p.name().to_string();
        self.device.delete_property(p).unwrap();
        sender.output(DeviceEvent::PropertyDeleted(name)).unwrap();
    }

}


/// A device as seen from a client implementation.
#[derive(Debug,Clone)]
pub struct ClientDevice {
    // name: &'a str,
    // props: &'a mut HashMap<String, Property>,
    name: String,
    props: HashMap<String,Property>,
}

// TODO move ClientDevice methods to Device trait
impl ClientDevice {


    pub fn upsert_property(&mut self, p: Property) -> IndigoResult<()> {
        self.props
        .entry(p.name().to_string())
        .and_modify(|prop| prop.update(&p))
        .or_insert(p);
        // if let Some(hook) = self.create_property_hook.as_deref_mut() {
        //     hook(self.props.get(&name).unwrap())
        // }
        Ok(())
    }

    pub fn delete_property(&mut self, p: Property) -> IndigoResult<Property> {
        if let Some(prop) = self.props.remove(p.name()) {
            Ok(prop)
        } else {
            Err(IndigoError::Message(
                "Trying to delete an undefined property.",
            ))
        }
    }
}
