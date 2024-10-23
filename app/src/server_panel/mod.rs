mod imp;

use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

use crate::server_object::ServerObject;

glib::wrapper! {
    pub struct ServerPanel(ObjectSubclass<imp::ServerPanel>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for ServerPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerPanel {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn bind(&self, server_object: &ServerObject) {
        // Get state
        let message_label = self.imp().message_label.get();
        let hostname_entry = self.imp().hostname_entry.get();
        // let port_entry = self.imp().port_entry.get();
        // let server_button = self.imp().server_button.get();

        let mut bindings = self.imp().bindings.borrow_mut();

        // Bind `task_object.completed` to `task_row.completed_button.active`
        // let completed_button_binding = server_object
        //     .bind_property("completed", &completed_button, "active")
        //     .bidirectional()
        //     .sync_create()
        //     .build();
        // // Save binding
        // bindings.push(completed_button_binding);

        let message_label_binding = server_object
            .bind_property("message", &message_label, "label")
            .sync_create()
            .build();
        // Save binding
        bindings.push(message_label_binding);

        let hostname_label_binding = server_object
            .bind_property("hostname", &hostname_entry, "buffer.text")
            .sync_create()
            .build();
        // Save binding
        bindings.push(hostname_label_binding);

        // Bind `task_object.completed` to `task_row.content_label.attributes`
        // let content_label_binding = server_object
        //     .bind_property("completed", &content_label, "attributes")
        //     .sync_create()
        //     .transform_to(|_, active| {
        //         let attribute_list = AttrList::new();
        //         if active {
        //             // If "active" is true, content of the label will be strikethrough
        //             let attribute = AttrInt::new_strikethrough(true);
        //             attribute_list.insert(attribute);
        //         }
        //         Some(attribute_list.to_value())
        //     })
        //     .build();
        // // Save binding
        // bindings.push(content_label_binding);
    }

    pub fn unbind(&self) {
        // Unbind all stored bindings
        for binding in self.imp().bindings.borrow_mut().drain(..) {
               binding.unbind();
        }
    }
}