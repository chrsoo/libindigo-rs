mod window;
mod server_object;
mod server_panel;

use std::env;

use gtk::glib::ExitCode;
use gtk::prelude::*;
use gtk::{gio, glib, Application};
use libindigo::{bus, LogLevel};
use log::{error, warn};
use window::Window;

const APP_ID: &str = "se.jabberwocky.libindigo-rs-example-app";

    /*
    let remote_server = init_indigo_server()?;

    // create a client for the remove server
    let mut client = Client::new("TestClient", ClientDeviceModel::new(), false);

    // attach the client to the INDIGO bus with a callback reference to the monitor
    client.attach(move |c| Ok(info!("attached client")))?;
    // initialise all properties
    client.define_properties()?;

    client.model(|m| {
        Ok(m.devices().for_each(|d| {
            println!("{d}");
            d.props().for_each(|p| println!("    {}", p.name()));
        }))
    })?;

    client.blobs().iter().for_each(|b| debug!("{:?}", b));

    remote_server.dicsonnect()?;

    // client.disconnect_device("Indigo", |r| debug!("Disconnect callback: '{:?}'", r))?;

    client.detach(move |c| {
        debug!("Detach callback closure called.");
        Ok(())
    })?;

    // server.shutdown()?;

    bus::stop()?;

     */


fn main() -> glib::ExitCode {
    // TODO make the glib log level configurable over GTK settings.
    env::set_var("G_MESSAGES_DEBUG", "all");
    glib_logger::init(&glib_logger::SIMPLE);
    log::set_max_level(log::LevelFilter::Debug);

    // TODO make the INDIGO LogLevel configurable over GTK settings.
    // Set the log level and start the local INDIGO bus
    bus::set_log_level(LogLevel::Debug);
    if let Err(e) = bus::start() {
        error!("Could not start the INDIGO bus: {e}");
        return glib::ExitCode::FAILURE
    }

    // Register and include resources
    gio::resources_register_include!("libindigo-rs-example-app.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();

    if let Err(e) = bus::stop() {
        warn!("Error while stopping the INDIGO bus: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    window.present();
}