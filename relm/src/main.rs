mod server;
mod property;
mod device;

use std::env;

use device::DeviceStatus;
use gtk::{glib::ExitCode, prelude::*};
use log::{error, warn};

use gtk::glib;
use libindigo::{bus, Client, ClientDeviceModel, LogLevel};
use relm4::{factory::FactoryHashMap, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp, RelmWidgetExt};
use server::{IndigoServer, ServerCommand, ServerOutput};

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
    app.run::<IndigoAppModel>(());

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
    UpdateStatus(String),
}

#[derive(Debug, Clone)]
enum CommandOutput {
    ClientAttached(String),
    ClientDetached(String),
}

struct IndigoAppModel {
    // TODO add IndigoServer
    server: Controller<IndigoServer>,
    client: Option<Client<'static, ClientDeviceModel>>,
    devices: FactoryHashMap<String,crate::device::Device<'static>>,
    status: String,
}

#[relm4::component]
impl Component for IndigoAppModel {
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
                    set_margin_all: 5,
                    set_spacing: 5,
                    #[watch]
                    set_visible: !model.devices.is_empty(),
                    gtk::StackSidebar {
                        set_stack: device_stack,
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
                DeviceStatus::Connected => todo!(),
                DeviceStatus::Disconnected => todo!(),
                DeviceStatus::Busy => todo!(),
            });

            // relm4_icons::initialize_icons();
        let server = IndigoServer::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg{
                ServerOutput::Detach => AppCommand::DetachClient,
                ServerOutput::Connected(m) => AppCommand::AttachClient(m),
                ServerOutput::Disconnected(m) => AppCommand::UpdateStatus(m),
                ServerOutput::StatusMessage(m) => AppCommand::UpdateStatus(m),
            });

        let model = IndigoAppModel { server, devices, client: None, status: String::new()};

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

impl IndigoAppModel {
    fn attach_client(&mut self, sender: ComponentSender<Self>) {
        self.status = format!("Attaching the client to the INDIGO bus...");

        let model = ClientDeviceModel::new();
        self.client = Some(Client::new("INDIGO", model, true));

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