use gtk::{glib::{self}, prelude::*, EntryBuffer};
use libindigo::server::ServerConnection;
use relm4::{
    gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt
};

const DEFAULT_SERVER_NAME: &str = "INDIGO";
// const DEFAULT_SERVER_HOSTNAME: &str = "indigosky.local";
const DEFAULT_SERVER_HOST: &str = "localhost";
const DEFAULT_SERVER_PORT: i32 = 7624;

const SERVER_CONNECT_MSG: &str = "Press play to connect";
const SERVER_DISCONNECT_MSG: &str = "Press stop to disconnect";
const SERVER_BUSY_MSG: &str = "Busy...";

const CONNECT_ICON_NAME: &str = "media-playback-start";
const DISCONNECT_ICON_NAME: &str = "media-playback-stop";

#[derive(Debug)]
pub struct IndigoServer {
    name: gtk::EntryBuffer,
    host: gtk::EntryBuffer,
    port: gtk::EntryBuffer,
    status: ServerStatus,
}

/// Commands for connecting and disconnecting to an IndigoServer.
#[derive(Debug)]
pub enum ServerCommand {
    /// Connect to the server.
    Connect,
    /// Disconnect from the server.
    Disconnect,
}

#[derive(Debug, Clone)]
pub enum ServerStatus {
    /// Connected to the server.
    Connected(ServerConnection),
    /// Disconnected from the server.
    Disconnected,
    /// Error connecting or disconnecting from the server.
    Error(String),
}

#[derive(Debug, Clone)]
pub enum ServerOutput {
    Detach,
    Connected(String),
    Disconnected(String),
    StatusMessage(String),
}

impl ServerStatus {
    pub fn is_connected(&self) -> bool {
        match self {
            ServerStatus::Connected(_) => true,
            _ => false,
        }
    }
}

#[relm4::component(pub)]
impl Component for IndigoServer {
    type Init = ();
    type Input = ServerCommand;
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
                    set_placeholder_text: Some(DEFAULT_SERVER_HOST),
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
                    set_placeholder_text: Some(&DEFAULT_SERVER_PORT.to_string()),
                    set_tooltip_text: Some("Server Port"),
                    connect_insert_text => move |entry, text, position| {
                        if text.contains(is_non_ascii_digit) {
                            glib::signal::signal_stop_emission_by_name(entry, "insert-text");
                            entry.insert_text(&text.replace(is_non_ascii_digit, ""), position);
                        }
                    }
                },
                gtk::Button {
                    #[track(model.status.is_connected())]
                    set_icon_name: DISCONNECT_ICON_NAME,
                    #[track(!model.status.is_connected())]
                    set_icon_name: CONNECT_ICON_NAME,
                    set_tooltip_text: Some(SERVER_CONNECT_MSG),
                    connect_clicked => move |b| {
                        if let Some(icon) = b.icon_name() {
                            match icon.as_str() {
                                CONNECT_ICON_NAME => sender.input(ServerCommand::Connect),
                                DISCONNECT_ICON_NAME => sender.output(ServerOutput::Detach).unwrap(),
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

        let model = IndigoServer {
            name: EntryBuffer::default(),
            host: EntryBuffer::default(),
            port: EntryBuffer::default(),
            status: ServerStatus::Disconnected,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        // tokio::time::sleep(Duration::from_secs(1)).await;
        // TODO spawn async connect and disconnect commmands
        match msg {
            ServerCommand::Connect => self.connect(sender),
            ServerCommand::Disconnect => self.disconnect(sender),
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

impl IndigoServer {

    fn connect(&mut self, sender: ComponentSender<Self>) {
        match &self.status {
            ServerStatus::Connected(c) => {
                if let Ok(connected) = c.is_connected() {
                    if connected {
                        sender.output(ServerOutput::Connected(format!("Connected to {c}")));
                        return;
                    }
                }
                self.reconnect(c.clone(), sender);
            },
            _ => {
                let name = text_entry(&self.name, DEFAULT_SERVER_NAME.to_string());
                let host = text_entry(&self.host, DEFAULT_SERVER_HOST.to_string());
                let port = port_entry(&self.port, DEFAULT_SERVER_PORT);
                match libindigo::server::connect(&name, &host, port)  {
                    Ok(c) => {
                        sender.output(ServerOutput::Connected(format!("Connected to {c}")));
                        self.status = ServerStatus::Connected(c);
                    },
                    Err(e) => {
                        let e = format!("Could not connect to server: {}", e);
                        self.status = ServerStatus::Error(e.clone());
                        sender.output(ServerOutput::StatusMessage(e));
                    },
                }
            }
        }
    }

    fn reconnect(&mut self, mut c: ServerConnection, sender: ComponentSender<Self>) {
        if let Err(e)= c.reconnect() {
            let e = format!("Error reconnecting to '{}': {}", c, e);
            self.status = ServerStatus::Disconnected;
            sender.output(ServerOutput::StatusMessage(e));
        } else {
            sender.output(ServerOutput::Connected(c.to_string()));
            self.status = ServerStatus::Connected(c);
        }
    }

    fn disconnect(&mut self, sender: ComponentSender<Self>) {
        if let ServerStatus::Connected(mut c) = self.status.clone() {
            if let Err(e) = c.dicsonnect() {
                let e = format!("Could not disconnect from '{}': {}", c, e);
                self.status = ServerStatus::Error(e.clone());
                sender.output(ServerOutput::StatusMessage(e));
            } else {
                self.status = ServerStatus::Disconnected;
                sender.output(ServerOutput::Disconnected(format!("Disconnected from {c}")));
            }
        } else {
            let e = "Disconnect failed: no connection established.".to_string();
            self.status = ServerStatus::Error(e.clone());
            sender.output(ServerOutput::StatusMessage(e));
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

fn port_entry<'a>(buffer: &EntryBuffer, default: i32) -> i32 {
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
