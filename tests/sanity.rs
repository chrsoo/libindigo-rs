#![allow(dead_code, unused_variables)]

use std::{
    net::Shutdown, sync::{Arc, Condvar, Mutex}, thread::sleep, time::Duration
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

struct TestHandler<'a> {
    pub visited: Arc<(Mutex<bool>, Condvar)>,
    props: Vec<Property<'a>>,
}

impl<'a> TestHandler<'a> {
    fn new() -> TestHandler<'a> {
        let visited = Arc::new((Mutex::new(false), Condvar::new()));
        let props = Vec::new();
        TestHandler { visited, props }
    }

    fn visit(&self) {
        let (lock, cvar) = &*self.visited;
        let mut visited = lock.lock().unwrap();
        *visited = true; // set
        cvar.notify_one();
    }

    fn wait_until_visited(&mut self) {
        let (lock, cvar) = &*self.visited;
        let mut visited = lock.lock().unwrap();
        while !*visited {
            visited = cvar.wait(visited).unwrap();
        }
        *visited = false; // reset
    }
}

impl<'a> CallbackHandler for TestHandler<'a> {
    fn on_client_attach(
        &mut self,
        c: &mut Client<impl CallbackHandler>,
    ) -> Result<(), IndigoError> {
        debug!("... client attached");
        self.visit();
        Ok(())
    }

    fn on_client_detach(
        &mut self,
        c: &mut Client<impl CallbackHandler>,
    ) -> Result<(), IndigoError> {
        self.visit();
        Ok(())
    }

    fn on_define_property(
        &mut self,
        c: &mut Client<impl CallbackHandler>,
        d: Device,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {

        // self.props.push(p);
        Ok(())
    }

    fn on_update_property(
        &mut self,
        c: &mut Client<impl CallbackHandler>,
        d: &Device,
        p: &Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        Ok(())
    }

    fn on_delete_property(
        &mut self,
        c: &mut Client<impl CallbackHandler>,
        d: &Device,
        p: &Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        Ok(())
    }

    fn on_send_message(
        &mut self,
        c: &mut Client<impl CallbackHandler>,
        d: &Device,
        msg: String,
    ) -> Result<(), IndigoError> {
        Ok(())
    }
}

#[test]
fn client_callbacks() -> Result<(), IndigoError> {
    Bus::set_log_level(LogLevel::Debug);
    Bus::start()?;

    let mut client = Client::new("TestClient", TestHandler::new())?;

    let mut server = ServerConnection::new("INDIGO", "localhost", 7624)?;
    server.connect()?;
    // client.get_all_properties()?;

    client.attach()?;
    client.handler.wait_until_visited();

    client.get_all_properties()?;

    sleep(Duration::from_secs(1));

    client.detach()?;
    client.handler.wait_until_visited();

    // server.dicsonnect(true)?;
    server.shutdown()?;

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
