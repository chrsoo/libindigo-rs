mod imp;

use glib::Object;
use gtk::{gio, glib::{self, clone}, subclass::prelude::*, Application};
use gtk::prelude::*;
use log::info;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_server(&self) {
        
    }

    fn setup_callbacks(&self) {
        // // Setup callback for activation of the entry
        // self.imp().entry.connect_activate(clone!(
        //     #[weak(rename_to = window)]
        //     self,
        //     move |_| {
        //         window.server_connection();
        //     }
        // ));

        // // Setup callback for clicking (and the releasing) the icon of the entry
        // self.imp().entry.connect_icon_release(clone!(
        //     #[weak(rename_to = window)]
        //     self,
        //     move |_, _| {
        //         window.server_connection();
        //     }
        // ));
    }
}