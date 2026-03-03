use zeroconf::{MdnsBrowser, ServiceType};

use crate::{Bus, Controller, NamedObject};

#[derive(Debug)]
pub struct AutoDiscovery {
    name: String,
    port: u64,
}

impl NamedObject for AutoDiscovery {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<B> Controller<B> for AutoDiscovery where B: Bus {

    fn attach(&mut self, bus: &mut B) -> crate::IndigoResult<()> {
        todo!()
    }

    fn detach(&mut self) -> crate::IndigoResult<()> {
        todo!()
    }
}

impl AutoDiscovery {
    fn listen(name: &str, sub_type: &str) {
        let sub_types: Vec<&str> = match sub_type.as_ref() {
            Some(sub_type) => vec![sub_type],
            None => vec![],
        };

        let service_type = ServiceType::with_sub_types(
            &name, &protocol, sub_types).expect("invalid service type");

        let mut browser = MdnsBrowser::new(service_type);

        browser.set_service_discovered_callback(Box::new(on_service_discovered));

        let event_loop = browser.browse_services().unwrap();
        println!("poll for zeroconf services");
        loop {
            // calling `poll()` will keep this browser alive
            event_loop.poll(Duration::from_secs(0)).unwrap();
        }

        /*
        // Connect to local INDI server.
        let connection = TcpStream::connect("indigosky.local:7624").expect("Connecting to INDI server");

        // Write command to server instructing it to track all properties.
        connection.write(&indi::serialization::GetProperties {
            version: indi::INDI_PROTOCOL_VERSION.to_string(),
            device: None,
            name: None,
        })
        .expect("Sending GetProperties command");

        // Loop through commands recieved from the INDI server
        for command in connection.iter().expect("Creating iterator over commands") {
            println!("Received from server: {:?}", command);
        }
        */

    }
}
