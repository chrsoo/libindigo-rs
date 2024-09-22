#![allow(dead_code, unused_variables)]

use std::{
    sync::{Arc, Condvar, Mutex},
    thread::sleep,
    time::Duration,
};
use test_log::test;

// TODO run indigio sky in Docker as test harness
use libindigo::*;
use log::debug;

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
fn bus_connect() -> Result<(), IndigoError> {
    Bus::set_log_level(LogLevel::Debug);
    let mut con = Bus::connect("INDIGO", "localhost", 7624)?;
    sleep(Duration::from_secs(1));
    con.shutdown()?;
    Bus::stop()
}

struct TestMonitor {
    pub visited: Arc<(Mutex<bool>, Condvar)>,
}

impl TestMonitor {
    fn visit(&self) -> Result<(), IndigoError> {
        let (lock, cvar) = &*self.visited;
        let mut visited = lock.lock().unwrap();
        *visited = true; // set
        cvar.notify_one();
        Ok(())
    }
    pub fn wait(&self) -> Result<(), IndigoError> {
        let (lock, cvar) = &*self.visited;
        let mut visited = lock.lock().unwrap();
        while !*visited {
            visited = cvar.wait(visited).unwrap();
        }
        *visited = false; // reset
        Ok(())
    }
}

#[test]
fn client_callbacks() -> Result<(), IndigoError> {
    Bus::set_log_level(LogLevel::Debug);
    Bus::start()?;

    let mut server = Bus::connect("INDIGO", "localhost", 7624)?;
    let model = FlatPropertyModel::new();
    let mut client = Client::new("TestClient", model, false);
    let monitor = Arc::new(TestMonitor {
        visited: Arc::new((Mutex::new(false), Condvar::new())),
    });

    let m = monitor.clone();
    client.attach(move |c| {
        debug!("Attach callback closure called.");
        m.visit()
    })?;
    client.define_properties()?;

    sleep(Duration::from_secs(3));
    monitor.wait()?;
    {
        /*
        let props = client
            .model()
            .devices()
            .into_iter()
            .flatten()
            .filter(|p| matches!(p.property_type(), PropertyType::Blob))
            // create an iterator of type tuple (PropertyKey,PropertyItem)
            //.map(|(k, p)| iter::repeat(k).take(p.items_used()).zip(p.items()))
            //.flatten()
            //.for_each(|(k, i)| debug!("{k}, {i}"));
            .for_each(|p| debug!("{p}"));

        debug!("----------------");
        */
        client.model(|m| Ok(
            m.props_map()
            .iter()
            .for_each(|(_, p)| {
                println!("{p}");
                for i in p.items() {
                    println!("    {i}");
                }
            })
        ))?;

        client.blobs().iter().for_each(|b| debug!("{:?}", b));

        /*
        client.model(|m| Ok(
            m.device_map()
            .iter()
            .for_each(|(k, d)| debug!("Interfaces: {:?}", d.interfaces()))
        ))?;

         */
    }

    let m = monitor.clone();
    client.detach(move |c| {
        debug!("Detach callback closure called.");
        m.visit()
    })?;
    monitor.wait()?;

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
