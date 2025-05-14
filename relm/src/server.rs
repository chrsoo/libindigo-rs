use gtk::{
    glib::{self},
    prelude::*,
    EntryBuffer, Label,
};
use libindigo::{INDIGO_DEFAULT_HOST, INDIGO_DEFAULT_PORT};
use log::{error, warn};
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt};
use url_fork::Url;

const DEFAULT_SERVER_NAME: &str = "INDIGO";
// const DEFAULT_SERVER_HOSTNAME: &str = "indigosky.local";
// const DEFAULT_SERVER_HOST: &str = "localhost";
// const DEFAULT_SERVER_PORT: u16 = 7624;

const SERVER_CONNECT_MSG: &str = "Press play to connect";
const SERVER_CONNECTING_MSG: &str = "Connecting to server...";
const SERVER_DISCONNECT_MSG: &str = "Press stop to disconnect";
const SERVER_DISCONNECTING_MSG: &str = "Disconnecting from server...";
const SERVER_BUSY_MSG: &str = "Busy...";

const CONNECT_ICON_NAME: &str = "media-playback-start";
const DISCONNECT_ICON_NAME: &str = "media-playback-stop";

#[derive(Debug)]
pub struct Server {
    name: gtk::EntryBuffer,
    host: gtk::EntryBuffer,
    port: gtk::EntryBuffer,
    status: ServerStatus,
    message: gtk::Label,
}

/// Commands for connecting and disconnecting to an IndigoServer.
#[derive(Debug)]
pub enum ServerInput {
    /// Connect to the server.
    Connect,
    /// Disconnect from the server.
    Disconnect,
    /// Update the server component status.
    UpdateStatus(ServerStatus),
}

#[derive(Debug, Clone)]
pub enum ServerOutput {
    ConnectServer(String, Url),
    DisconnectServer,
}

#[derive(Debug, Clone)]
pub enum ServerStatus {
    /// Connecting to the server.
    Connecting(&'static str),
    /// Connected to the server.
    Connected(&'static str),
    /// Disconnecting from the server.
    Disconnecting(&'static str),
    /// Disconnected from the server.
    Disconnected(&'static str),
}

impl ServerStatus {
    /// Returns `true` if the server status is `connected` and `false`
    /// otherwise.
    pub fn is_connected(&self) -> bool {
        match self {
            ServerStatus::Connected(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the component is `connecting` or `disconnecting`.
    pub fn is_transiting(&self) -> bool {
        match self {
            ServerStatus::Connecting(_) => true,
            ServerStatus::Disconnecting(_) => true,
            _ => false,
        }
    }
    pub fn message<'a>(&self) -> &'a str {
        match self {
            ServerStatus::Connecting(m) => m,
            ServerStatus::Connected(m) => m,
            ServerStatus::Disconnecting(m) => m,
            ServerStatus::Disconnected(m) => m,
        }
    }
}

#[relm4::component(pub)]
impl Component for Server {
    type Init = ();
    type Input = ServerInput;
    type Output = ServerOutput;
    type CommandOutput = ();

    view! {
        server = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            gtk::Box {  // server connection panel
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 5,
                set_spacing: 5,
                gtk::Entry {
                    #[watch]
                    set_sensitive: !model.status.is_connected(),
                    set_buffer: &model.name,
                    set_tooltip_text: Some("Server Name"),
                    set_placeholder_text: Some(DEFAULT_SERVER_NAME),
                },
                gtk::Label {
                    set_label: "@",
                },
                gtk::Entry {
                    #[watch]
                    set_sensitive: !model.status.is_connected(),
                    set_buffer: &model.host,
                    set_tooltip_text: Some("Server Host"),
                    set_placeholder_text: Some(INDIGO_DEFAULT_HOST),
                },
                gtk::Label {
                    set_label: ":",
                },
                gtk::Entry {
                    #[watch]
                    set_sensitive: !model.status.is_connected(),
                    set_buffer: &model.port,
                    set_max_width_chars: 5,
                    set_max_length: 5,
                    set_placeholder_text: Some(&INDIGO_DEFAULT_PORT.to_string()),
                    set_tooltip_text: Some("Server Port"),
                    connect_insert_text => move |entry, text, position| {
                        if text.contains(is_non_ascii_digit) {
                            glib::signal::signal_stop_emission_by_name(entry, "insert-text");
                            entry.insert_text(&text.replace(is_non_ascii_digit, ""), position);
                        }
                    }
                },
                gtk::Button {
                    #[watch]
                    set_sensitive: !model.status.is_transiting(),
                    #[track(model.status.is_connected())]
                    set_icon_name: DISCONNECT_ICON_NAME,
                    #[track(!model.status.is_connected())]
                    set_icon_name: CONNECT_ICON_NAME,
                    set_tooltip_text: Some(SERVER_CONNECT_MSG),
                    connect_clicked => move |b| {
                        if let Some(icon) = b.icon_name() {
                            match icon.as_str() {
                                CONNECT_ICON_NAME => sender.input(ServerInput::Connect),
                                DISCONNECT_ICON_NAME => sender.input(ServerInput::Disconnect),
                                name => unreachable!("unrecogniced icon name: '{name}'"),
                            }
                        } else {
                            unreachable!("no icon name")
                        }
                    },
                }
            },
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Server {
            name: EntryBuffer::default(),
            host: EntryBuffer::default(),
            port: EntryBuffer::default(),
            message: Label::default(),
            status: ServerStatus::Disconnected(SERVER_CONNECT_MSG),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            ServerInput::Connect => self.connect(sender),
            ServerInput::Disconnect => self.disconnect(sender),
            ServerInput::UpdateStatus(s) => self.update_status(s),
        }
    }

    /*
    fn update_cmd(
        &mut self,
        status: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _: &Self::Root
    ){
        self.status = status;
        // TODO set the server status
        match self.status {
            ServerStatus::Connected => {
                self.message = format!("Connected to '{}'.", self.connection.as_ref().unwrap());
            },
            ServerStatus::Disconnected => {
                self.message = format!("Disconnected from '{}'", self.connection.as_ref().unwrap());
                self.connection = None;
            },
            ServerStatus::Busy(m) => {
                self.message = m.clone();
            },
            ServerStatus::Error(e) => {
                self.connection = None;
                self.message = format!("{e}");
            },
        }
        sender.output(self.status.clone()).unwrap();
    }
     */
}

impl Server {
    fn error(&self, msg: &str) {
        error!("{msg}");
        self.message.set_text(msg);
    }

    fn update_status(&mut self, status: ServerStatus) {
        match status {
            ServerStatus::Connecting(m) => self.update_and_toggle(m, false),
            ServerStatus::Connected(m) => self.update_and_toggle(m, false),
            ServerStatus::Disconnecting(m) => self.update_and_toggle(m, false),
            ServerStatus::Disconnected(m) => self.update_and_toggle(m, true),
        }
        self.status = status;
    }

    fn update_and_toggle(&mut self, message: &str, editable: bool) {
        self.message.set_text(message);
    }

    fn connect(&mut self, sender: ComponentSender<Self>) {
        match &self.status {
            ServerStatus::Connected(_) => self.error("cannot connect when already connected"),
            ServerStatus::Connecting(_) => self.error("cannot connect while connecting"),
            ServerStatus::Disconnecting(_) => self.error("cannot connect while disconnnecting"),
            ServerStatus::Disconnected(_) => {
                let name = text_entry(&self.name, DEFAULT_SERVER_NAME.to_string());
                let host = text_entry(&self.host, INDIGO_DEFAULT_HOST.to_string());
                let port = port_entry(&self.port, INDIGO_DEFAULT_PORT);
                let url = &format!("tcp://{host}:{port}");
                match Url::parse(url) {
                    Ok(url) => {
                        sender
                            .output(ServerOutput::ConnectServer(name, url))
                            .inspect_err(|_| warn!("could not request connect"));
                    }
                    Err(e) => {
                        warn!("could not parse url: {e}");
                        self.message.set_text(e.to_string().as_str());
                    }
                };
            }
        }
    }

    fn disconnect(&mut self, sender: ComponentSender<Self>) {
        match &self.status {
            ServerStatus::Disconnected(_) => {
                self.error("cannot disconnect when already disconnected")
            }
            ServerStatus::Disconnecting(_) => self.error("cannot disconnect while disconnnecting"),
            ServerStatus::Connecting(_) => self.error("cannot disconnect while connecting"),
            ServerStatus::Connected(_) => {
                sender
                    .output(ServerOutput::DisconnectServer)
                    .inspect_err(|_| warn!("could not request disconnect"));
            }
        }
    }
}

fn text_entry<'a>(buffer: &EntryBuffer, default: String) -> String {
    let text = buffer.text().trim().to_string();
    if text.is_empty() {
        default
    } else {
        buffer.text().to_string()
    }
}

fn port_entry<'a>(buffer: &EntryBuffer, default: u16) -> u16 {
    let number = buffer.text().trim().to_string();
    if number.is_empty() {
        default
    } else {
        number.parse().unwrap()
    }
}

// https://stackoverflow.com/questions/10279579/force-numeric-input-in-a-gtkentry-widget
fn is_non_ascii_digit(c: char) -> bool {
    !c.is_ascii_digit()
}
