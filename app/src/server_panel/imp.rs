use std::cell::RefCell;

use glib::Binding;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate, Entry, Label, LayoutManager};
use libindigo::ServerConnection;

// Object holding the state
#[derive(Default, CompositeTemplate)]
#[template(resource = "/se/jabberwocky/libindigo-rs-example-app/server_panel.ui")]
pub struct ServerPanel {
    #[template_child]
    pub hostname_entry: TemplateChild<Entry>,
    #[template_child]
    pub port_entry: TemplateChild<Entry>,
    #[template_child]
    pub server_button: TemplateChild<Button>,
    #[template_child]
    pub message_label: TemplateChild<Label>,
    // Vector holding the bindings to properties of `ServerObject`
    pub bindings: RefCell<Vec<Binding>>,
}



// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ServerPanel {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "ServerPanel";
    type Type = super::ServerPanel;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        // klass.set_layout_manager_type();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for ServerPanel {}

// Trait shared by all widgets
impl WidgetImpl for ServerPanel {
    fn request_mode(&self) -> gtk::SizeRequestMode {
        gtk::SizeRequestMode::ConstantSize
    }
}

// Trait shared by all boxes
impl BoxImpl for ServerPanel {}