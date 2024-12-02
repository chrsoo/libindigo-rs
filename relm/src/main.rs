mod server;
mod property;
mod device;

use std::env;

use device::{Device, DeviceCommand, DeviceEvent};
use gtk::{glib::ExitCode, prelude::*};
use log::{error, warn};
use gtk::glib;
use libindigo::{bus, Client, ClientCallbacks, ClientDevice, LogLevel, Property};
use relm4::{factory::FactoryHashMap, Component, ComponentController, ComponentParts, ComponentSender, Controller, MessageBroker, RelmApp, RelmWidgetExt};
use server::{IndigoServer, ServerCommand, ServerOutput};


static BROKER: MessageBroker<AppCommand> = MessageBroker::new();

fn main() -> glib::ExitCode {
    env::set_var("G_MESSAGES_DEBUG", "all");
    glib_logger::init(&glib_logger::SIMPLE);
    log::set_max_level(log::LevelFilter::Debug);

    // TODO make the INDIGO LogLevel configurable over GTK settings.
    // Set the log level and start the local INDIGO bus
    bus::set_log_level(LogLevel::Debug);
    if let Err(e) = bus::start() {
        error!("Could not start the INDIGO bus: {e}");
        return glib::ExitCode::FAILURE
    }

    let app = RelmApp::new("se.jabberwocky.libindigo-rs-example-app");
    app.with_broker(&BROKER).run::<IndigoApp>(());
    // app.run::<IndigoApp>(());

    if let Err(e) = bus::stop() {
        warn!("Error while stopping the INDIGO bus: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[derive(Debug, Clone)]
enum AppCommand {
    AttachClient(String),
    DetachClient,
    DefineProperty(Property),
    UpdateStatus(String),
}

#[derive(Debug, Clone)]
enum CommandOutput {
    ClientAttached(String),
    ClientDetached(String),
}

struct IndigoApp {
    server: Controller<IndigoServer>,
    client: Option<Client<'static, DeviceCallbackHandler>>,
    devices: FactoryHashMap<String,crate::device::Device>,
    status: String,
}

#[relm4::component]
impl Component for IndigoApp {
    type Init = ();
    type Input = AppCommand;
    type Output = ();
    type CommandOutput = CommandOutput;

    view! {
        window = gtk::ApplicationWindow {
            set_title: Some("libINDIGOrs Example App"),
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,
                gtk::Label {
                    #[watch]
                    set_label: &model.status,
                },
                #[local_ref] server ->
                gtk::Box { },
                gtk::Label {
                    set_label: "No devices.",
                    #[watch]
                    set_visible: model.devices.is_empty(),
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    // set_margin_all: 5,
                    // set_spacing: 5,
                    set_vexpand: true,
                    set_hexpand: true,
                    #[watch]
                    set_visible: !model.devices.is_empty(),
                    gtk::StackSidebar {
                        #[watch]
                        set_stack: model.devices.widget(),
                        set_visible: true,
                    },
                    #[local_ref] device_stack ->
                    gtk::Stack {
                    },
                }
            },
        }
    }

    /// Initialize the UI and model.
    fn init(
        _counter: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {

        let devices = FactoryHashMap::builder()
            .launch(gtk::Stack::default())
            .forward(sender.input_sender(), |output| match output {
                DeviceEvent::Connected(d) => AppCommand::UpdateStatus(format!("Connected device '{d}'")),
                DeviceEvent::Disconnected(d) => AppCommand::UpdateStatus(format!("Disconnected device '{d}'")),
                DeviceEvent::PropertyDefined(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
                DeviceEvent::PropertyUpdated(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
                DeviceEvent::PropertyDeleted(p) => AppCommand::UpdateStatus(format!("Defined property '{p}'")),
                DeviceEvent::Busy(d) => AppCommand::UpdateStatus(format!("Device is busy '{d}'")),
            });

        let server = IndigoServer::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg{
                ServerOutput::Detach => AppCommand::DetachClient,
                ServerOutput::Connected(m) => AppCommand::AttachClient(m),
                ServerOutput::Disconnected(m) => AppCommand::UpdateStatus(m),
                ServerOutput::StatusMessage(m) => AppCommand::UpdateStatus(m),
            });

        // let devices = Arc::new(devices);
        let model = IndigoApp { server, devices, client: None, status: String::new() };

        // local widget refs used in the view macro above.
        let device_stack = model.devices.widget();
        let server = model.server.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        input: <Self as relm4::Component>::Input,
        sender: ComponentSender<Self>,
        _root: &<Self as relm4::Component>::Root) {

        match input {
            AppCommand::AttachClient(m) => {
                self.status = m;
                self.attach_client(sender);
            },
            AppCommand::DetachClient => self.detach_client(sender),
            AppCommand::UpdateStatus(m) => self.status = m,
            AppCommand::DefineProperty(p) => self.define_property(p),
        }
    }

    fn update_cmd(
            &mut self,
            message: Self::CommandOutput,
            _sender: ComponentSender<Self>,
            _root: &Self::Root,
        ) {
        match message {
            CommandOutput::ClientAttached(m) => self.status = m,
            CommandOutput::ClientDetached(m) => {
                self.status = m;
                self.client = None;
                self.server.sender().send(ServerCommand::Disconnect).unwrap();
            },
        }
    }
}

impl IndigoApp {

    fn define_property(&mut self, property: Property) {
        let device = &property.device().to_string();
        if let None = self.devices.get(device) {
            self.devices.insert(device.clone(), ClientDevice::new(device));
        }
        // self.devices.widget().add_titled(child, Some(property.name()), property.name());
        self.devices.send(&device, DeviceCommand::DefineProperty(property));
    }

    fn attach_client(&mut self, sender: ComponentSender<Self>) {
        self.status = format!("Attaching the client to the INDIGO bus...");

        // let mut model = ClientDeviceModel::new();
        // let mut devices = self.devices.clone();
        // model.create_device_hook(move|device| {
        //     devices.guard().push_back(device);
        //     // noop
        // });

        let devices = self.devices.clone();
        let callback_handler = DeviceCallbackHandler::new(devices);
        self.client = Some(Client::new("INDIGO", callback_handler, true));

        let c = self.client.as_mut().unwrap();
        let s = sender.clone();
        if let Err(e) = c.attach(move |_c| {
            s.input(AppCommand::UpdateStatus(
                format!("Attached the client to the INDIGO bus.")
            ));
            Ok(())
        }) {
            sender.input(AppCommand::UpdateStatus(
                format!("Failed attching the client to the INDIGO bus: {e}.")
            ));
        }
    }

    fn detach_client(&mut self, sender: ComponentSender<Self>) {
        self.status = format!("Detaching the client from the INDIGO bus...");

        if let Some(client) = self.client.as_mut() {
            if let Err(e) = client.detach(move |_c| {
                sender.oneshot_command(async { CommandOutput::ClientDetached(
                    format!("Detached the client from the INDIGO bus.")
                )});
                Ok(())
            }) {
                self.status = format!("Failed detaching the client from the INDIGO bus: {e}.");
            }
        } else {
            self.status = format!("Error: trying to detach a non-exisisting client.");
        }
    }
}


pub(crate) struct DeviceCallbackHandler {
    devices: FactoryHashMap<String,Device>,
}

impl DeviceCallbackHandler {
    pub(crate) fn new(devices: FactoryHashMap<String,Device>) -> Self {
        Self {devices }
    }
}

impl<'a> ClientCallbacks<'a> for DeviceCallbackHandler {
    type M = DeviceCallbackHandler;

    fn on_define_property(
        &mut self,
        _c: &mut libindigo::Client<'a, Self::M>,
        d: String,
        p: libindigo::Property,
        msg: Option<String>,
    ) -> Result<(), libindigo::IndigoError> {

        let name = p.name().to_string();
        let device = p.device().to_string();
        BROKER.send(AppCommand::DefineProperty(p));
        log::debug!("Device: '{device}'; Property '{name}'; DEFINED with message '{:?}'", msg);
        Ok(())
    }

    fn on_update_property(
        &mut self,
        _c: &mut libindigo::Client<'a, Self::M>,
        d: String,
        p: libindigo::Property,
        msg: Option<String>,
    ) -> Result<(), libindigo::IndigoError> {

        let device = p.device().to_string();
        let name = p.name().to_string();
        if let None = self.devices.get(&device) {
            BROKER.send(AppCommand::DefineProperty(p));
            log::debug!("Device: '{device}'; Property '{name}'; DEFINED with message '{:?}'", msg);
        } else {
            self.devices.send(&device, DeviceCommand::UpdateProperty(p));
            log::debug!("Device: '{device}'; Property '{name}'; UPDATED with message '{:?}'", msg);
        }

        Ok(())
    }

    fn on_delete_property(
        &mut self,
        _c: &mut libindigo::Client<'a, Self::M>,
        d: String,
        p: libindigo::Property,
        msg: Option<String>,
    ) -> Result<(), libindigo::IndigoError> {
        log::debug!(
            "Device: '{}'; Property '{}'; DELETED with message '{:?}'",
            p.device(),
            p.name(),
            msg
        );

        let device = p.device().to_string();
        if let Some(_) = self.devices.get(&device) {
            self.devices.send(&p.device().to_string(), DeviceCommand::DeleteProperty(p));
        }
        Ok(())
    }

    fn on_send_message(
        &mut self,
        _c: &mut libindigo::Client<'a, Self::M>,
        d: String,
        msg: String,
    ) -> Result<(), libindigo::IndigoError> {
        log::info!("Device: '{d}';  SEND  message: '{msg}'");
        Ok(())
    }

}