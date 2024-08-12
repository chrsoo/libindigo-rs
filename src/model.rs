use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Condvar, Mutex, RwLock},
};

use log::debug;

use crate::{CallbackHandler, Client, Device, IndigoError, Property, PropertyKey};

/// A default implementation of `CallbackHandler` for use with INDIGO `Client` that manages
/// the set of all enumerated devices and their properties that are defined on the INDIGO
/// bus.
pub struct IndigoModel<'a> {
    pub visited: Arc<(Mutex<bool>, Condvar)>,
    pub props: RwLock<HashMap<PropertyKey, Property<'a>>>,
    pub devices: RwLock<HashMap<String, Device<'a>>>,
}

impl<'a> IndigoModel<'a> {
    pub fn new() -> IndigoModel<'a> {
        let visited = Arc::new((Mutex::new(false), Condvar::new()));
        let props = RwLock::new(HashMap::new());
        let devices = RwLock::new(HashMap::new());
        IndigoModel {
            visited,
            props,
            devices,
        }
    }

    fn visit(&self) {
        let (lock, cvar) = &*self.visited;
        let mut visited = lock.lock().unwrap();
        *visited = true; // set
        cvar.notify_one();
    }

    pub fn wait_until_visited(&mut self) {
        let (lock, cvar) = &*self.visited;
        let mut visited = lock.lock().unwrap();
        while !*visited {
            visited = cvar.wait(visited).unwrap();
        }
        *visited = false; // reset
    }
}

impl<'a> CallbackHandler<'a> for IndigoModel<'a> {
    fn on_client_attach(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
    ) -> Result<(), IndigoError> {
        debug!("... client attached");
        self.visit();
        Ok(())
    }

    fn on_client_detach(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
    ) -> Result<(), IndigoError> {
        self.visit();
        Ok(())
    }

    fn on_define_property(
        &'a mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
        d: Device<'a>,
        p: Property<'a>,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        let mut props = self.props.write().unwrap();
        match props.entry(p.key()) {
            Entry::Occupied(mut e) => e.get_mut().update(p)?,
            Entry::Vacant(e) => _ = e.insert(p),
        };

        let mut devs = self.devices.write().unwrap();
        match devs.entry(d.name()) {
            Entry::Occupied(e) => e.get().check_ref_eq(d)?,
            Entry::Vacant(e) => _ = e.insert(d),
        };

        Ok(())
    }

    fn on_update_property(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
        d: Device,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        Ok(())
    }

    fn on_delete_property(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
        d: Device,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        Ok(())
    }

    fn on_send_message(
        &mut self,
        c: &mut Client<'a, impl CallbackHandler<'a>>,
        d: Device,
        msg: String,
    ) -> Result<(), IndigoError> {
        Ok(())
    }
}
