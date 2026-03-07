mod device;
mod property;
mod server;

use std::env;

use device::{Device, DeviceInput, DeviceOutput};
use gtk::glib;
use gtk::{glib::ExitCode, prelude::*};

// New imports for libindigo-rs
use libindigo_rs::Property;
use libindigo_rs::Result as IndigoResult;
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};

use log::{error, warn};
use relm4::{
    factory::FactoryHashMap, Component, ComponentController, ComponentParts, ComponentSender,
    Controller, MessageBroker, RelmApp, RelmWidgetExt,
};
use server::{Server, ServerInput, ServerOutput};
use url_fork::Url;

static BROKER: MessageBroker<AppInput> = MessageBroker::new();

fn main() -> glib::ExitCode {
    env::set_var("G_MESSAGES_DEBUG", "all");
    glib_logger::init(&glib_logger::SIMPLE);
    log::set_max_level(log::LevelFilter::Debug);

    let app = RelmApp::new("se.jabberwocky.libindigo-rs-example-app");
    app.with_broker(&BROKER).run::<IndigoApp>(());

    ExitCode::SUCCESS
}

#[derive(Debug, Clone)]
enum AppInput {
    // -- commands
    /// Connect a remote [Server]
    ConnectServer(String, Url),
    /// Disconnect a remote [Server]
    DisconnectServer,
    /// Connect [Device] to INDIGO bus.
    RequestConnection,
    /// Connect [Device] to INDIGO bus.
    RequestDisconnection,
    /// Request a new [Property] to be defined for the [Device].
    RequestDefinition(String, String),
    /// Request all properties for the [Device] to be defined.
    RequestEnumeration(String),
    /// Request update of a [Property].
    RequestUpdate(Property),
    /// Request deletion of an [Property].
    RequestDeletion(String, String),
    /// Send a message to the [Device].
    SendMessage(String, String),

    // -- events
    /// A [Property] of a [Device] was defined.
    PropertyDefined { data: Property, msg: Option<String> },
    /// A [Property] of a Device was updated.
    PropertyUpdated { data: Property, msg: Option<String> },
    /// A [Property] of a [Device] was deleted.
    PropertyDeleted {
        property: String,
        device: String,
        msg: Option<String>,
    },
    /// Received a message from a Device
    MessageReceived { device: String, msg: String },
}

#[derive(Debug, Clone)]
enum AppOutput {
    /// Server remote server was Connected.
    ServerConnected,
    /// A remote server was Disconnected.
    ServerDisconnected,
    /// A [Property] of a [Device] was defined.
    PropertyDefined(Property),
    /// A [Property] of a Device was updated.
    PropertyUpdated(Property),
    /// A [Property] of a [Device] was deleted.
    PropertyDeleted { property: String, device: String },
    /// A message was sent by a [Device]
    MessageSent { message: String, device: String },
}

struct IndigoApp {
    // indigo client
    client: Option<Client>,
    // realm
    status: String,
    server: Controller<Server>,
    // generic
    devices: FactoryHashMap<String, crate::device::Device>,
}

#[relm4::component]
impl Component for IndigoApp {
    type Init = ();
    type Input = AppInput;
    type Output = AppOutput;
    type CommandOutput = ();

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
        _init: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let devices = FactoryHashMap::builder()
            .launch(gtk::Stack::default())
            .forward(sender.input_sender(), |output| match output {
                DeviceOutput::RequestConnection => AppInput::RequestConnection,
                DeviceOutput::RequestDisconnection => AppInput::RequestDisconnection,
                DeviceOutput::RequestDefinition(p, d) => AppInput::RequestDefinition(p, d),
                DeviceOutput::RequestUpdate(p) => AppInput::RequestUpdate(p),
                DeviceOutput::RequestDeletion(p, d) => AppInput::RequestDeletion(p, d),
                DeviceOutput::RequestEnumeration(d) => AppInput::RequestEnumeration(d),
                DeviceOutput::SendMessage(d, m) => AppInput::SendMessage(d, m),
            });

        let server = Server::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                ServerOutput::ConnectServer(name, url) => AppInput::ConnectServer(name, url),
                ServerOutput::DisconnectServer => AppInput::DisconnectServer,
            });

        let model = IndigoApp {
            client: None,
            server,
            devices,
            status: "Ready. Connect to a server to begin.".to_string(),
        };

        // local widget refs used in the view macro above.
        let device_stack = model.devices.widget();
        let server = model.server.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        input: <Self as relm4::Component>::Input,
        _sender: ComponentSender<Self>,
        _root: &<Self as relm4::Component>::Root,
    ) {
        if let Err(e) = match input {
            // device commands
            AppInput::RequestConnection => {
                warn!("RequestConnection not yet implemented");
                Ok(())
            }
            AppInput::RequestDisconnection => {
                warn!("RequestDisconnection not yet implemented");
                Ok(())
            }
            AppInput::RequestDefinition(_d, _p) => {
                warn!("RequestDefinition not yet implemented");
                Ok(())
            }
            AppInput::RequestEnumeration(_) => {
                warn!("RequestEnumeration not yet implemented");
                Ok(())
            }
            AppInput::RequestUpdate(_) => {
                warn!("RequestUpdate not yet implemented");
                Ok(())
            }
            AppInput::RequestDeletion(_, _) => {
                warn!("RequestDeletion not yet implemented");
                Ok(())
            }
            AppInput::SendMessage(_, _) => {
                warn!("SendMessage not yet implemented");
                Ok(())
            }
            // server commands
            AppInput::ConnectServer(name, url) => self.connect(name, url),
            AppInput::DisconnectServer => self.disconnect(),
            // property events
            AppInput::PropertyDefined { data, msg } => self.define_property(data, msg),
            AppInput::PropertyUpdated { data, msg } => self.update_property(data, msg),
            AppInput::PropertyDeleted {
                property,
                device,
                msg,
            } => self.delete_property(property, device, msg),
            // device events
            AppInput::MessageReceived { device, msg } => self.receive_message(device, msg),
        } {
            error!("{e}");
        }
    }
}

impl IndigoApp {
    fn connect(&mut self, name: String, url: Url) -> IndigoResult<()> {
        // Create a new client with RS strategy
        let strategy = RsClientStrategy::new();
        let client = ClientBuilder::new()
            .with_strategy(Box::new(strategy))
            .build()?;

        self.client = Some(client);
        self.status = format!("Connected to {}", name);

        // TODO: Actually connect to the server asynchronously
        // This will require spawning a tokio task and handling the async connection
        warn!(
            "Async connection to {} at {} not yet fully implemented",
            name, url
        );

        Ok(())
    }

    fn disconnect(&mut self) -> IndigoResult<()> {
        if self.client.is_some() {
            // TODO: Actually disconnect from the server asynchronously
            self.client = None;
            self.status = "Disconnected".to_string();
            warn!("Async disconnection not yet fully implemented");
        }
        Ok(())
    }

    // -- device events

    fn define_property(&self, data: Property, msg: Option<String>) -> IndigoResult<()> {
        let device = &data.device.clone();
        self.devices
            .send(device, DeviceInput::DefineProperty(data, msg));
        Ok(())
    }

    fn update_property(&self, data: Property, msg: Option<String>) -> IndigoResult<()> {
        let device = &data.device.clone();
        self.devices
            .send(device, DeviceInput::UpdateProperty(data, msg));
        Ok(())
    }

    fn delete_property(
        &self,
        property: String,
        device: String,
        msg: Option<String>,
    ) -> IndigoResult<()> {
        self.devices
            .send(&device, DeviceInput::DeleteProperty(property, msg));
        Ok(())
    }

    fn receive_message(&self, device: String, msg: String) -> IndigoResult<()> {
        self.devices.send(&device, DeviceInput::Message(msg));
        Ok(())
    }
}
