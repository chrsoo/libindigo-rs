mod device;
mod property;
mod server;

use std::env;

use device::{Device, DeviceInput, DeviceOutput};
use gtk::glib;
use gtk::{glib::ExitCode, prelude::*};
use libindigo::IndigoResult;
use libindigo::{
    property::{PropertyData, PropertyItem},
    sys::{LogLevel, SysBus, SysClientController, SysRemoteResource},
    Bus, ClientController, ClientDelegate, Controller as BusController, Delegate, NamedObject,
    Property, RemoteResource,
};
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

    // TODO make the INDIGO LogLevel configurable over GTK settings.
    // Set the log level and start the local INDIGO bus
    match SysBus::start("indigoApp") {
        Ok(bus) => {
            SysBus::enable_bus_log(LogLevel::Debug);

            let app = RelmApp::new("se.jabberwocky.libindigo-rs-example-app");
            app.with_broker(&BROKER).run::<IndigoApp>(bus);
            // app.run::<IndigoApp>(());

            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("Could not start the INDIGO bus: {e}");
            ExitCode::FAILURE
        }
    }
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
    RequestUpdate(PropertyData),
    /// Request update of a [PropertyItem] for a [Property] on a [Device].
    RequestItemUpdate(PropertyItem, String, String),
    /// Request deletion of an [Property].
    RequestDeletion(String, String),
    /// Send a message to the [Device].
    SendMessage(String, String),

    // -- events

    /// A [Property] of a [Device] was defined.
    PropertyDefined { data: PropertyData, msg: Option<String> },
    /// A [Property] of a Device was updated.
    PropertyUpdated { data: PropertyData, msg: Option<String> },
    /// A [Property] of a [Device] was deleted.
    PropertyDeleted { property: String, device: String, msg: Option<String> },
    /// Received a message from a Device
    MessageReceived { device: String, msg: String }
}

#[derive(Debug, Clone)]
enum AppOutput {
    /// Server remote server was Connected.
    ServerConnected,
    /// A remote server was Disconnected.
    ServerDisconnected,
    /// A [Property] of a [Device] was defined.
    PropertyDefined(PropertyData),
    /// A [Property] of a Device was updated.
    PropertyUpdated(PropertyData),
    /// A [Property] of a [Device] was deleted.
    PropertyDeleted { property: String, device: String },
    /// A message was sent by a [Device]
    MessageSent{ message: String, device: String },
}

#[derive(Debug)]
struct IndigoApp {
    // indigo
    bus: SysBus,
    client: SysClientController<DeviceCallbackHandler>,
    remote: SysRemoteResource,
    // realm
    status: String,
    server: Controller<Server>,
    // generic
    devices: FactoryHashMap<String, crate::device::Device>,
}

#[relm4::component]
impl Component for IndigoApp {
    type Init = SysBus;
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
        bus: Self::Init,
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
                DeviceOutput::RequestItemUpdate(pi, p, d) => AppInput::RequestItemUpdate(pi, p, d),
                DeviceOutput::RequestEnumeration(d) => AppInput::RequestEnumeration(d),
                DeviceOutput::SendMessage(d, m) => AppInput::SendMessage(d, m),
            });

        let server = Server::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                ServerOutput::ConnectServer(name, url) => AppInput::ConnectServer(name, url),
                ServerOutput::DisconnectServer => AppInput::DisconnectServer,
            });

        let delegate = DeviceCallbackHandler {};
        let client = SysClientController::new(delegate);

        let remote = SysRemoteResource::default();

        // let devices = Arc::new(devices);
        let model = IndigoApp {
            bus,
            client,
            remote,
            server,
            devices,
            status: String::new(),
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
            AppInput::RequestConnection=> todo!(),
            AppInput::RequestDisconnection=>todo!(),
            AppInput::RequestDefinition(d,p)=> self.client.request_definition(d.as_str(), p.as_str()),
            AppInput::RequestEnumeration(_)=>todo!(),
            AppInput::RequestUpdate(_)=>todo!(),
            AppInput::RequestItemUpdate(_,_,_)=>todo!(),
            AppInput::RequestDeletion(_,_)=>todo!(),
            AppInput::SendMessage(_,_)=>todo!(),
            // server commands
            AppInput::ConnectServer(name, url)=> self.connect(name, url),
            AppInput::DisconnectServer=> self.remote.disconnect(),
            // property events
            AppInput::PropertyDefined { data, msg } => self.define_property(data, msg),
            AppInput::PropertyUpdated { data, msg } => self.update_property(data, msg),
            AppInput::PropertyDeleted { property, device, msg } => self.delete_property(property, device, msg),
            // device events
            AppInput::MessageReceived { device, msg } => self.receive_message(device, msg),
        } {
            error!("{e}");
        }
    }

    // fn update_cmd(
    //     &mut self,
    //     message: Self::CommandOutput,
    //     _sender: ComponentSender<Self>,
    //     _root: &Self::Root,
    // ) {
    //     match message {

    //         AppOutput::ClientAttached(m) => self.status = m,
    //         AppOutput::ClientDetached(m) => {
    //             self.status = m;
    //             self.client = None;
    //             self.server.sender().send(ServerInput::Disconnect).unwrap();
    //         }
    //     }
    // }
}

impl IndigoApp {

    fn connect(&mut self, name: String, url: Url) -> IndigoResult<()> {
        let mut remote = SysRemoteResource::new(name.as_str(), url)?;
        remote.attach(&mut self.bus)?;
        remote.reconnect()?;
        self.remote = remote;
        Ok(())
    }

    // -- device events

    fn define_property(&self, data: PropertyData, msg: Option<String>)
    -> IndigoResult<()> {
        let device = &data.device().to_owned();
        self.devices.send(device, DeviceInput::DefineProperty(data, msg));
        Ok(())
    }

    fn update_property(&self, data: PropertyData, msg: Option<String>)
    -> IndigoResult<()> {
        let device = &data.device().to_owned();
        self.devices.send(device, DeviceInput::UpdateProperty(data, msg));
        Ok(())
    }

    fn delete_property(&self, property: String, device: String, msg: Option<String>)
    -> IndigoResult<()> {
        self.devices.send(&device, DeviceInput::DeleteProperty(property, msg));
        Ok(())
    }

    fn receive_message(&self, device: String, msg: String) -> IndigoResult<()> {
        self.devices.send(&device, DeviceInput::Message(msg));
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct DeviceCallbackHandler { }

impl NamedObject for DeviceCallbackHandler {
    fn name(&self) -> &str {
        "RealmApp"
    }
}

impl Delegate for DeviceCallbackHandler {
    type Bus = SysBus;
    type BusController = SysClientController<Self>;
}

impl ClientDelegate for DeviceCallbackHandler {
    type Property = PropertyData;
    type ClientController = SysClientController<Self>;

    fn on_define_property<'a>(
        &'a mut self,
        _c: &mut Self::ClientController,
        _d: &'a str,
        p: Self::Property,
        msg: Option<&'a str>,
    ) -> libindigo::IndigoResult<()> {

        let msg = msg.map(|m| m.to_owned());
        let input = AppInput::PropertyDefined { data: p.into(), msg };
        BROKER.send(input);
        Ok(())
    }

    fn on_update_property<'a>(
        &mut self,
        _c: &mut Self::ClientController,
        _d: &'a str,
        p: Self::Property,
        msg: Option<&'a str>,
    ) -> libindigo::IndigoResult<()> {

        let msg = msg.map(|m| m.to_owned());
        let input = AppInput::PropertyUpdated { data: p.into(), msg };
        BROKER.send(input);
        Ok(())
    }

    fn on_delete_property<'a>(
        &mut self,
        _c: &mut Self::ClientController,
        _d: &'a str,
        p: Self::Property,
        msg: Option<&'a str>,
    ) -> libindigo::IndigoResult<()> {

        let device = p.device().to_owned();
        let property = p.name().to_owned();
        let msg = msg.map(|m| m.to_owned());
        let input = AppInput::PropertyDeleted{ property, device, msg };

        BROKER.send(input);
        Ok(())
    }

    fn on_message_broadcast<'a>(
        &mut self,
        _c: &mut Self::ClientController,
        d: &'a str,
        msg: &'a str
    ) -> libindigo::IndigoResult<()> {

        let device = d.to_owned();
        let input = AppInput::MessageReceived { device, msg: msg.to_owned() };
        BROKER.send(input);
        Ok(())
    }
}
