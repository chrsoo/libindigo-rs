use std::time::Duration;
use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};

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

impl<B> Controller<B> for AutoDiscovery
where
    B: Bus,
{
    fn attach(&mut self, _bus: &mut B) -> crate::IndigoResult<()> {
        todo!()
    }

    fn detach(&mut self) -> crate::IndigoResult<()> {
        todo!()
    }
}

impl AutoDiscovery {
    #[allow(dead_code)]
    fn listen(_name: &str, _sub_type: Option<&str>) {
        // This function is incomplete and not currently used
        // TODO: Implement proper mDNS service discovery
        unimplemented!("mDNS service discovery not yet implemented");

        /*
        let sub_types: Vec<&str> = match sub_type {
            Some(sub_type) => vec![sub_type],
            None => vec![],
        };

        let protocol = "_tcp"; // Default protocol
        let service_type = ServiceType::with_sub_types(
            name, protocol, sub_types).expect("invalid service type");

        let mut browser = MdnsBrowser::new(service_type);

        let on_service_discovered = |_result: zeroconf::Result<ServiceDiscovery>, _context: Option<std::sync::Arc<dyn std::any::Any>>| {
            // Handle service discovery
        };

        browser.set_service_discovered_callback(Box::new(on_service_discovered));

        let event_loop = browser.browse_services().unwrap();
        println!("poll for zeroconf services");
        loop {
            // calling `poll()` will keep this browser alive
            event_loop.poll(Duration::from_secs(0)).unwrap();
        }
        */

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
