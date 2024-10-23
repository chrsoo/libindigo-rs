mod imp;
use glib::Object;
use gtk::glib;
use libindigo::ServerConnection;

const DEFAULT_SERVER_NAME: &str = "INDIGO Sky";
const DEFAULT_SERVER_HOSTNAME: &str = "indigosky.local";
const DEFAULT_SERVER_PORT: i32 = 7624;

glib::wrapper! {
    pub struct ServerObject(ObjectSubclass<imp::ServerObject>);
}

impl ServerObject {
    pub fn new(name: &str, hostname: &str, port: i32) -> Self {
        Object::builder()
            .property("name", name)
            .property("hostname", hostname)
            .property("port", port)
            .property("message", None::<String>)
            .build()
    }
}

pub struct ServerData {
    pub name: String,
    pub hostname: String,
    pub port: i32,
    pub message: Option<String>,
    pub connection: Option<ServerConnection>,
}

impl Default for ServerData {
    fn default() -> Self {
        Self {
            name: DEFAULT_SERVER_NAME.to_string(),
            hostname: DEFAULT_SERVER_HOSTNAME.to_string(),
            port: DEFAULT_SERVER_PORT,
            message: None,
            connection: None,
        }
    }
}