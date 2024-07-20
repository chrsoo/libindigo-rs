use std::{thread::sleep, time::Duration};

// TODO run indigio sky in Docker as test harness
use libindigo::*;

#[test]
fn start_stop_bus() -> Result<(),IndigoError> {
    log(LogLevel::Debug);
    start()?;
    start()?; // make sure the function is reentrant
    sleep(Duration::from_secs(1));
    stop()?;
    stop() // second call to make sure the function is reentrant
}

#[test]
fn server_connection() -> Result<(),IndigoError> {
    log(LogLevel::Debug);
    let mut server = ServerConnection::new("Indigo", "localhost", 7624)?;
    server.connect()?;
    sleep(Duration::from_secs(10));
    server.dicsonnect()?;
    sleep(Duration::from_secs(10));
    stop()
}

fn server_callback(mut s: ServerConnection) -> Result<(),IndigoError> {
    s.connect();
    Ok(())
}

#[ignore = "until discovery is implemented"]
#[test]
fn server_discovery()  -> Result<(),IndigoError> {
    log(LogLevel::Debug);
    start()?;
    if let Err(_e) = discover(server_callback) {
        todo!("log error {}", _e);
    };
    sleep(Duration::from_secs(1));
    stop()?;
    stop() // second call to make sure the function is reentrant
}