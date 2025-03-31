#![allow(dead_code, unused_variables)]

use std::{
    sync::{Arc, Condvar, Mutex},
    thread::sleep,
    time::Duration,
};
use test_log::test;

// TODO run indigio sky in Docker as test harness
use libindigo::*;
use libindigo::server;
use libindigo::bus;
use log::debug;

#[ignore = "pending refactoring"]
#[test]
fn start_stop_bus() -> Result<(), IndigoError> {
    bus::set_log_level(LogLevel::Debug);
    bus::start()?;
    bus::start()?; // second call to make sure the function is reentrant
    sleep(Duration::from_secs(1));
    bus::stop()?;
    bus::stop() // second call to make sure the function is reentrant
}

#[ignore = "pending refactoring"]
#[test]
fn server_connect() -> Result<(), IndigoError> {
    bus::set_log_level(LogLevel::Debug);
    let mut con = server::connect("INDIGO", "localhost", 7624)?;
    sleep(Duration::from_secs(1));
    con.shutdown()?;
    bus::stop()
}

struct TestMonitor {
    pub visited: Arc<(Mutex<bool>, Condvar)>,
}

impl TestMonitor {
    fn new() -> TestMonitor {
        TestMonitor {
            visited: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    fn visit(&self) -> Result<(), IndigoError> {
        let (lock, cvar) = &*self.visited;
        let mut visited = lock.lock().unwrap();
        *visited = true; // set
        cvar.notify_one();
        Ok(())
    }
    fn wait(&self) -> Result<(), IndigoError> {
        let (lock, cvar) = &*self.visited;
        let mut visited = lock.lock().unwrap();
        while !*visited {
            visited = cvar.wait(visited).unwrap();
        }
        *visited = false; // reset
        Ok(())
    }
}

#[ignore = "pending refactoring"]
#[test]
fn client() -> Result<(), IndigoError> {
    // prepare the monitor to be used for async testing
    let monitor = Arc::new(TestMonitor::new());

    // prepare and start the bus
    bus::set_log_level(LogLevel::Debug);
    bus::start()?;
    // connect the bus to the remote server
    let mut connection = server::connect("INDIGO", "localhost", 7624)?;

    // create a client for the remove server
    let mut client = ClientController::new("TestClient", ClientDeviceModel::new(), false);

    // attach the client to the INDIGO bus with a callback reference to the monitor
    let m = monitor.clone();
    client.attach(move |c| {
        debug!("Attach callback closure called.");
        m.visit()
    })?;
    // wait for the async callback to happen
    monitor.wait()?;

    // initialise all properties
    client.define_properties()?;
    // give some time for the property definition callbacks to happen
    // NOTE should there not be a callback signalling that all the props have been defined?
    sleep(Duration::from_secs(3));

    client.model(|m| {
        Ok(m.devices().for_each(|d| {
            println!("{d}");
            d.props().for_each(|p| println!("    {}", p.name()));
        }))
    })?;

    client.blobs().iter().for_each(|b| debug!("{:?}", b));

    connection.dicsonnect()?;

    // client.disconnect_device("Indigo", |r| debug!("Disconnect callback: '{:?}'", r))?;

    let m = monitor.clone();
    client.detach(move |c| {
        debug!("Detach callback closure called.");
        m.visit()
    })?;
    monitor.wait()?;

    // server.shutdown()?;

    sleep(Duration::from_secs(5));
    bus::stop()
}


#[ignore = "until discovery is implemented"]
#[test]
fn server_discovery()  -> Result<(),IndigoError> {
    bus::set_log_level(LogLevel::Debug);
    bus::start()?;
    // if let Err(_e) = bus::discover(server_callback) {
    //     todo!("log error {}", _e);
    // };
    sleep(Duration::from_secs(1));
    bus::stop()?;
    bus::stop() // second call to make sure the function is reentrant
}
