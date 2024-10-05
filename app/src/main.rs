mod window;

use gtk::prelude::*;
use gtk::{gio, glib, Application};
use libindigo::{Bus, IndigoResult, LogLevel, ServerConnection};
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

    Bus::stop()?;

     */


fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("libindigo-rs-example-app.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}
fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    window.present();
}

fn init_indigo_server() -> IndigoResult<ServerConnection> {
     // prepare and start the bus
    Bus::set_log_level(LogLevel::Debug);
    Bus::start()?;
    // connect the bus to the remote server
    Bus::connect("Indigo", "localhost", 7624)
}
