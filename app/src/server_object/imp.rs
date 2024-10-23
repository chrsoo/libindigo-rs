use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::ServerData;

// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::ServerObject)]
pub struct ServerObject {
    #[property(name = "name", get, set, type = String, member = name)]
    #[property(name = "hostname", get, set, type = String, member = hostname)]
    #[property(name = "port", get, set, type = i32, member = port)]
    #[property(name = "message", get, set, type = i32, member = message)]
    pub data: RefCell<ServerData>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ServerObject {
    const NAME: &'static str = "ServerObject";
    type Type = super::ServerObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for ServerObject {}