
use std::time::Duration;

use gtk::{glib::{self}, prelude::*, EntryBuffer};
use libindigo::ServerConnection;
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

pub struct IndigoServer {
    name: gtk::EntryBuffer,
    host: gtk::EntryBuffer,
    port: gtk::EntryBuffer,
    message: String,
    status: ServerStatus,
    connection: Option<ServerConnection>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServerStatus {
    /// Connected to the server.
    Connected,
    /// Disconnected from the server.
    Disconnected,
    /// Busy connecting or disconnecting the server.
    Busy(String),
    /// Error connecting or disconnecting from the server.
    Error(String),
}

/// Commands for connecting and disconnecting to an IndigoServer.
#[derive(Debug)]
pub enum ServerCommand {
    /// Connect to the server.
    Connect,
    /// Disconnect from the server.
    Disconnect,
}

#[relm4::component(pub)]
impl Component for IndigoServer {
    type Init = ();
    type Input = ServerCommand;
    type Output = ServerStatus;
    type CommandOutput = ServerStatus;

    view! {
        server = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            gtk::Label {
                #[watch]
                set_label: &model.message
            },
            gtk::Box {  // server connection panel
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 5,
                set_spacing: 5,
                gtk::Entry {
                    #[watch]
                    set_sensitive: model.connection.is_none(),
                    set_buffer: &model.name,
                    set_tooltip_text: Some("Server Name"),
                    set_placeholder_text: Some(DEFAULT_SERVER_NAME),
                },
                gtk::Entry {
                    #[watch]
                    set_sensitive: model.connection.is_none(),
                    set_buffer: &model.host,
                    set_tooltip_text: Some("Server Host"),
                    set_placeholder_text: Some(DEFAULT_SERVER_HOST),
                },
                gtk::Label {
                    set_label: ":",
                },
                gtk::Entry {
                    #[watch]
                    set_sensitive: model.connection.is_none(),
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
                    #[track(model.status == ServerStatus::Connected)]
                    set_icon_name: DISCONNECT_ICON_NAME,
                    #[track(model.status == ServerStatus::Disconnected)]
                    set_icon_name: CONNECT_ICON_NAME,
                    set_tooltip_text: Some(SERVER_CONNECT_MSG),
                    connect_clicked => move |b| {
                        if let Some(icon) = b.icon_name() {
                            match icon.as_str() {
                                CONNECT_ICON_NAME => sender.input(ServerCommand::Connect),
                                DISCONNECT_ICON_NAME => sender.input(ServerCommand::Disconnect),
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
            connection: None,
            status: ServerStatus::Disconnected,
            message: SERVER_CONNECT_MSG.to_string(),
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

    fn update_cmd(
        &mut self,
        status: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _: &Self::Root
    ){
        self.status = status;
        // TODO set the server status
        match &self.status {
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

    }

}

impl IndigoServer {

    fn connect(&mut self, sender: ComponentSender<Self>) {
        // check if there is a server connection object
        if let Some(c) = self.connection.as_mut() {
            // check if the connection is alive
            if let Ok(connected) = c.is_connected() {
                if connected {
                    sender.oneshot_command(async { ServerStatus::Connected });
                    return;
                } else {
                    IndigoServer::reconnect(c, sender);
                }
            } else {
                IndigoServer::reconnect(c, sender);
            }
        } else {
            let name = text_entry(&self.name, DEFAULT_SERVER_NAME.to_string());
            let host = text_entry(&self.host, DEFAULT_SERVER_HOST.to_string());
            let port = port_entry(&self.port, DEFAULT_SERVER_PORT);
            match libindigo::server::connect(&name, &host, port)  {
                Ok(c) => {
                    self.connection = Some(c);
                    // while let ServerStatus::Busy(_) = self.status {
                    //     tokio::time::sleep(Duration::from_secs(1)).await;
                    // }
                    sender.oneshot_command(async { ServerStatus::Connected });
                },
                Err(e) => {
                    let e = format!("Could not connect to server: {}", e);
                    sender.oneshot_command(async { ServerStatus::Error(e) });
                },
            }
        }
    }

    fn reconnect(c: &mut ServerConnection, sender: ComponentSender<Self>) {
        if let Err(e)= c.reconnect() {
            let e = format!("Error reconnecting to '{}': {}", c, e);
            sender.oneshot_command(async { ServerStatus::Error(e) });
        } else {
            sender.oneshot_command(async { ServerStatus::Connected });
        }
    }

    fn disconnect(&mut self, sender: ComponentSender<Self>) {
        if let Some(c) = self.connection.as_mut() {
            if let Err(e) = c.dicsonnect() {
                let e = format!("Could not disconnect from '{}': {}", c, e);
                sender.oneshot_command(async { ServerStatus::Error(e) });
            } else {
                sender.oneshot_command(async { ServerStatus::Disconnected });
            }
        } else {
            let e = "No server connection".to_string();
            sender.oneshot_command(async { ServerStatus::Error(e) });
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
