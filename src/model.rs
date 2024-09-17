use std::collections::{hash_map::Entry, HashMap};

use log::debug;
use parking_lot::{lock_api::RwLockWriteGuard, RawRwLock, RwLock};

use crate::{Client, Device, GuardedStringMap, IndigoError, Property, PropertyKey};

/// Data model used by a [IndigoClient] with callback methods to handle [IndigoBus] events.
pub trait Model<'a> {
    type M: Model<'a>;

    fn device_map<'b>(
        &'b self,
    ) -> RwLockWriteGuard<RawRwLock, HashMap<String, Device<'a>>>;

    fn devices<'b>(&'b self) -> GuardedStringMap<'b, Device<'a>> {
        GuardedStringMap {
            lock: self.device_map(),
        }
    }

    /// Called each time the property of a device is defined or its definition requested.
    fn on_define_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: Device<'a>,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; DEFINED with message '{:?}' defined for ",
            d.name(),
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time a property is updated for a device.
    fn on_update_property(
        &mut self,
        c: &'a mut Client<'a, Self::M>,
        d: Device<'a>,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; UPDATED with message '{:?}' defined for ",
            d.name(),
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time a property is deleted.
    fn on_delete_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: Device,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; DELETED with message '{:?}' defined for ",
            d.name(),
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time message has been sent.
    // TODO Move to client and use closure instead.
    fn on_send_message(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: Device,
        msg: String,
    ) -> Result<(), IndigoError> {
        debug!("Message '{:?}' SENT", msg);
        Ok(())
    }
}

// struct DefaultModelIterator<'a> {
//     // lock: RwLockReadGuard<'a,HashMap<String,Device<'a>>>,
//     devices: Values<'a,String, Device<'a>>,
// }

// impl<'a> Iterator for DefaultModelIterator<'a> {
//     type Item = &'a mut Device;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next()
//     }
// }

// impl<'a> IntoIterator for DefaultModel<'a> {
//     type Item = &'a mut Device;
//     type IntoIter = DefaultModelIterator<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         let devices = lock.values();

//         DefaultModelIterator {
//             devices: self.devices.read().unwrap().values(),
//         }
//     }
// }

/// A default implementation of [Model] that manages the set of all enumerated devices
/// and their properties that are defined on the [Bus] .
pub struct DefaultModel<'a> {
    props: RwLock<HashMap<PropertyKey, Property>>,
    devices: RwLock<HashMap<String, Device<'a>>>,
}

impl<'a> DefaultModel<'a> {
    pub fn new() -> DefaultModel<'a> {
        DefaultModel {
            props: RwLock::new(HashMap::new()),
            devices: RwLock::new(HashMap::new()),
        }
    }
}

impl<'a> Model<'a> for DefaultModel<'a> {
    type M = DefaultModel<'a>;

    fn device_map<'b>(
        &'b self,
    ) -> RwLockWriteGuard<RawRwLock, HashMap<String, Device<'a>>> {
        self.devices.write()
        //RwLockWriteGuard::map(self.devices.write(), |d| d)
    }

    fn devices<'b>(&'b self) -> GuardedStringMap<'b, Device<'a>> {
        GuardedStringMap {
            lock: self.devices.write(),
        }
    }

    fn on_define_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: Device<'a>,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        let mut props = self.props.write();
        // props.entry(p.key()).and_modify(|pi| { pi.update(&p); }).or_insert(p);

        match props.entry(p.key()) {
            Entry::Occupied(mut e) => e.get_mut().update(p),
            Entry::Vacant(e) => _ = e.insert(p),
        };

        let mut devs = self.devices.write();
        match devs.entry(d.name().to_string()) {
            Entry::Occupied(e) => e.get().assert_same(d)?,
            Entry::Vacant(e) => _ = e.insert(d),
        };

        Ok(())
    }
    /*
        fn on_update_property(
            &mut self,
            c: &'a mut Client<'a, impl Model<'a,T>>,
            d: &'a mut Device<'a, impl Model<'a,T>>,
            p: Property,
            msg: Option<String>,
        ) -> Result<(), IndigoError> {

    */
    fn on_update_property(
        &mut self,
        c: &'a mut Client<'a, Self::M>,
        d: Device,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        Ok(())
    }

    fn on_delete_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: Device,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        Ok(())
    }

    fn on_send_message(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: Device,
        msg: String,
    ) -> Result<(), IndigoError> {
        Ok(())
    }
}
