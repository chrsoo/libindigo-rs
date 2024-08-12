#![allow(dead_code, unused_variables)]

use std::{
    iter, net::Shutdown, path::Iter, sync::{Arc, Condvar, Mutex, RwLock}, thread::sleep, time::Duration
};
use test_log::test;

// TODO run indigio sky in Docker as test harness
use libindigo::*;
use log::{debug, error, info, warn};

#[test]
fn start_stop_bus() -> Result<(), IndigoError> {
    Bus::set_log_level(LogLevel::Debug);
    Bus::start()?;
    Bus::start()?; // second call to make sure the function is reentrant
    sleep(Duration::from_secs(1));
    Bus::stop()?;
    Bus::stop() // second call to make sure the function is reentrant
}

#[test]
fn server_connection() -> Result<(), IndigoError> {
    Bus::set_log_level(LogLevel::Debug);
    let mut server = ServerConnection::new("Indigo", "localhost", 7624)?;
    server.connect()?;
    sleep(Duration::from_secs(1));
    server.shutdown()?;
    Bus::stop()
}

#[test]
fn client_callbacks() -> Result<(), IndigoError> {
    Bus::set_log_level(LogLevel::Debug);
    Bus::start()?;

    let mut model = IndigoModel::new();
    let mut client = Client::new("TestClient", &mut model)?;

    let mut server = ServerConnection::new("INDIGO", "localhost", 7624)?;
    server.connect()?;

    client.attach()?;
    client.handler.wait_until_visited();

    client.get_all_properties()?;

    sleep(Duration::from_secs(3));

    {
        let props = client.handler.props.read().unwrap();
        props.iter()
            //.filter(|p| matches!(p.property_type(), PropertyType::Blob))
            // create an iterator of type tuple (PropertyKey,PropertyItem)
            .map(|(k,p)| iter::repeat(k).take(p.items_used()).zip(p.items()))
            .flatten()
            .for_each(|(k,i)| debug!("{k}, {i}"));

        debug!("----------------");
        client.handler.devices.read().unwrap().iter()
            .for_each(|(_,d)| debug!("{}", d));
        client.blobs().iter().for_each(|b| debug!("{:?}", b));
        client.handler.devices.read().unwrap().iter().for_each(|(k,d)| debug!("Interfaces: {:?}", d.interfaces()));

    }

    client.detach()?;
    client.handler.wait_until_visited();

    server.dicsonnect()?;
    // server.shutdown()?;

    sleep(Duration::from_secs(5));
    Bus::stop()
}

/*
#[ignore = "until discovery is implemented"]
#[test]
fn server_discovery()  -> Result<(),IndigoError> {
    set_log_leve(LogLevel::Debug);
    start()?;
    if let Err(_e) = discover(server_callback) {
        todo!("log error {}", _e);
    };
    sleep(Duration::from_secs(1));
    stop()?;
    stop() // second call to make sure the function is reentrant
}
  */
