mod server;

use std::env;

use gtk::{glib::ExitCode, prelude::*};
use log::{error, warn};

use gtk::glib;
use libindigo::{bus, LogLevel};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp, RelmWidgetExt, SimpleComponent};
use server::IndigoServer;

fn main() -> glib::ExitCode {
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

    let app = RelmApp::new("se.jabberwocky.libindigo-rs-example-app");
    app.run::<IndigoAppModel>(());

    if let Err(e) = bus::stop() {
        warn!("Error while stopping the INDIGO bus: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[derive(Debug)]
enum AppMode {
    Connected,
    Disconnected,
}

struct IndigoAppModel {
    // TODO add IndigoServer
    server: Controller<IndigoServer>,
}

#[relm4::component]
impl SimpleComponent for IndigoAppModel {
    type Init = ();
    type Input = server::ServerStatus;
    type Output = ();

    view! {
        window = gtk::ApplicationWindow {
            set_title: Some("libINDIGOrs Example App"),
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,
                append = model.server.widget(),
            },
        }
    }

    /// Initialize the UI and model.
    fn init(
        _counter: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {

        // relm4_icons::initialize_icons();
        let server = IndigoServer::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| msg);
        server.widget().set_visible(true);

        let model = IndigoAppModel { server };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    /*
    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::Connect => self.connect(),
            AppInput::Disconnect => self.disconnect(),
        }
        if self.is_connected() {
            self.icon = DISCONNECT_ICON_NAME;
        } else {
            self.icon = CONNECT_ICON_NAME;
        }
    }
    */

    // /// Update the view to represent the updated model.
    // fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
    //     if self.is_connected() {
    //         widgets.button.set_icon_name(DISCONNECT_ICON_NAME);
    //         widgets.button.set_tooltip_text(Some(SERVER_DISCONNECT_MSG))
    //         // TODO enable entry values
    //     } else {
    //         widgets.button.set_icon_name(CONNECT_ICON_NAME);
    //         widgets.button.set_tooltip_text(Some(SERVER_CONNECT_MSG))
    //         // TODO disable entry of values
    //     }
    // }
}