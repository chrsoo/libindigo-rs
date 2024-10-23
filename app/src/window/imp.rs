use std::cell::RefCell;
use std::net::ToSocketAddrs;
use std::rc::Rc;

use glib::subclass::InitializingObject;

use gtk::{gio, glib::*, EntryBuffer, InfoBar, Label};
use gtk::{prelude::*, Entry};
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate};

use libindigo::{server, IndigoError, IndigoResult, ServerConnection};
use log::warn;

use crate::server_panel::ServerPanel;

const CONNECT_ICON_NAME: &str = "media-playback-start";
const DISCONNECT_ICON_NAME: &str = "media-playback-stop";

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/se/jabberwocky/libindigo-rs-example-app/window.ui")]
pub struct Window {
    #[template_child]
    pub server_panel: TemplateChild<ServerPanel>,
    #[template_child]
    pub info: TemplateChild<Label>,
    #[template_child]
    pub button: TemplateChild<Button>,
    #[template_child]
    pub host: TemplateChild<Entry>,
    #[template_child]
    pub port: TemplateChild<Entry>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "LibINDIGOExampleApp";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        ServerPanel::ensure_type();
        klass.bind_template();
        // klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        // Setup
        let obj = self.obj();
        obj.setup_callbacks();
        // obj.setup_factory();


        let connection: Rc<RefCell<Option<ServerConnection>>> = Rc::new(RefCell::new(None));

        // TODO Refactor to use a custom component that owns the server connection state
        // TODO Spawn connect/disonnect in a blocking call -  Connection does not implement send!
        self.port.connect_icon_release(move |entry, _pos| {
            glib::spawn_future_local(clone!(
                #[strong] connection,
                #[weak] entry,
                async move {
                    entry.set_sensitive(false);
                    let mut c = connection.borrow_mut();
                    if let Some(sc) = &mut *c {
                        // a connection is established, disconnect
                        if let Err(e) = disconnect(sc) {
                            warn!("Could not disconnect from {sc}: {e}");
                        }
                        *c = None;
                        entry.set_editable(true);
                        entry.set_secondary_icon_name(Some(CONNECT_ICON_NAME));
                    } else {
                        // there is no server connection established, connect
                        let buffer = entry.buffer();
                        if let Ok(sc) = connect(buffer).inspect_err(|e| {
                            // TODO set status message to error
                            warn!("Could not connect to server: {e}");
                        }) {
                            *c = Some(sc);
                            entry.set_editable(false);
                            entry.set_secondary_icon_name(Some(DISCONNECT_ICON_NAME));
                        }
                    }
                    entry.set_sensitive(true);
                }
            ));
        });

        // Connect to "clicked" signal of `button`
        self.button.connect_clicked(move |button| {
            // Set the label to "Hello World!" after the button has been clicked on
            button.set_label("Hello World!");
        });

        // Setup
        let obj = self.obj();
        obj.setup_callbacks();
    }

}

pub fn connect(buffer: EntryBuffer) -> IndigoResult<ServerConnection> {
    // TODO parse hostname and port
    let name = &buffer.text().to_string();
    let host = name;
    // let port = addr.port() as i32;
    let port = 7624;

    let sc = server::connect(name, host, port).inspect_err(|e| {
        warn!("Could not connect to {name}@{host}:{port}: {e}")
    })?;
    Ok(sc)
}

pub fn connect_old(buffer: EntryBuffer) -> IndigoResult<ServerConnection> {
    let name = buffer.text().to_string();
    let mut addrs = name.as_str().to_socket_addrs()?;
    if let Some(addr) = addrs.next() {
        let ip = addr.ip().to_string();

        let host = ip.as_str();
        let port = addr.port() as i32;

        let sc = server::connect(name.as_str(), host, port).inspect_err(|e| {
            warn!("Could not connect to {name}@{host}:{port}: {e}")
        })?;
        Ok(sc)
    } else {
        Err(IndigoError::Other(format!(
            "Could not parse connection string '{name}'"
        )))
    }
}

pub fn disconnect(server: &mut ServerConnection) -> IndigoResult<()> {
    server.dicsonnect()
}

pub fn toggle_connection(entry: &Entry, server: &mut ServerConnection) -> IndigoResult<()> {
    if server.is_connected()? {
        server.dicsonnect()?;
        entry.set_editable(true);
        entry.set_secondary_icon_name(Some(CONNECT_ICON_NAME));
    } else {
        server.reconnect()?;
        entry.set_editable(false);
        entry.set_secondary_icon_name(Some(DISCONNECT_ICON_NAME));
    }
    Ok(())
}

impl Window {

}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}