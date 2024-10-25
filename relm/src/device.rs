use gtk::prelude::*;
use libindigo::ClientDevice;
use relm4::{
    gtk, prelude::{DynamicIndex, FactoryComponent}, FactorySender
};

/// Relm model for a [ClientDevice].
pub struct Device<'a> {
    device: ClientDevice<'a>,
}

/// Connect or disconnect the INDIGO Device.
#[derive(Debug)]
pub enum DeviceCommand {
    /// Connect the device to the INDIGO bus.
    Connect,
    /// Disconnect the device from the INDIGO bus.
    Disconnect,
}


#[derive(Debug, Copy, Clone, PartialEq)]
/// Status of an INDIGO [ClientDevice].
pub enum DeviceStatus  {
    /// Device is connected to the INDIGO bus.
    Connected,
    /// Device is disconnected from the INDIGO bus.
    Disconnected,
    /// Device is busy changing its connection status.
    Busy,
}

#[relm4::factory(pub)]
impl FactoryComponent for Device<'static> {
    type Init = ClientDevice<'static>;
    type Input = DeviceCommand;
    type Output = DeviceStatus;
    type CommandOutput = DeviceStatus;
    type ParentWidget = gtk::Stack;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,
            gtk::Label {
                #[watch]
                set_label: &self.device.name(),
            }
        }
    }

    fn init_model(device: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { device }
    }

}

impl<'a> Device<'a> {

}