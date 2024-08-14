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
    pub props: RwLock<HashMap<PropertyKey, Property<'a>>>,
    pub devices: RwLock<HashMap<String, Device<'a>>>,
}

impl<'a> IndigoModel<'a> {
    pub fn new() -> IndigoModel<'a> {
        IndigoModel {
            props: RwLock::new(HashMap::new()),
            devices: RwLock::new(HashMap::new()),
        }
    }
}

impl<'a> CallbackHandler<'a> for IndigoModel<'a> {

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
        d: Arc<Device>,
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
