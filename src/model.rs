use std::collections::{hash_map::{Entry, Iter, ValuesMut}, HashMap};

use log::debug;
use parking_lot::{lock_api::RwLockWriteGuard, MappedRwLockWriteGuard, RawRwLock, RwLock};

use crate::{Client, DeviceImpl, GuardedStringMap, IndigoError, Property, PropertyKey};

/// Data model used by a [IndigoClient] with callback methods to handle [IndigoBus] events.
pub trait Model<'a> {
    type M: Model<'a>;

    /*
    fn device_map<'b>(
        &'b self,
    ) -> RwLockWriteGuard<RawRwLock, HashMap<String, DeviceImpl<'a>>>;

    fn devices<'b>(&'b self) -> GuardedStringMap<'b, DeviceImpl<'a>> {
        GuardedStringMap {
            lock: self.device_map(),
        }
    }
    */

    /// Called each time the property of a device is defined or its definition requested.
    fn on_define_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; DEFINED with message '{:?}'",
            d,
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time a property is updated for a device.
    fn on_update_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; UPDATED with message '{:?}'",
            d,
            p.name(),
            msg
        );
        Ok(())
    }

    /// Called each time a property is deleted.
    fn on_delete_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        debug!(
            "Device: '{}'; Property '{}'; DELETED with message '{:?}'",
            d,
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
        d: String,
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
pub struct DeviceModel<'a> {
    props: RwLock<HashMap<PropertyKey, Property>>,
    devices: RwLock<HashMap<String, DeviceImpl<'a>>>,

}

pub struct FlatPropertyModel<'a> {
    props: RwLock<HashMap<PropertyKey, Property>>,
     devices: RwLock<HashMap<String, DeviceImpl<'a>>>,
}

impl<'a> FlatPropertyModel<'a> {
    pub fn new() -> FlatPropertyModel<'a> {
        FlatPropertyModel {
            props: RwLock::new(HashMap::new()),
            devices: RwLock::new(HashMap::new()),
        }
    }

    // fn props(&self) -> MappedRwLockWriteGuard<ValuesMut<PropertyKey,Property>> {
    //     RwLockWriteGuard::map(self.props.write(), |props| &mut props.values_mut())
    // }

    pub fn props_map(&self) -> RwLockWriteGuard<RawRwLock, HashMap<PropertyKey, Property>> {
        self.props.write()
    }

    pub fn props_map1<'b>(&'b self) -> GuardedStringMap<'b, DeviceImpl<'a>> {
        GuardedStringMap {
            lock: self.devices.write(),
        }
    }

}

impl<'a> Model<'a> for FlatPropertyModel<'a> {
    type M = FlatPropertyModel<'a>;

    fn on_define_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        let mut props = self.props.write();
        match props.entry(p.key()) {
            Entry::Occupied(mut e) => e.get_mut().update(p),
            Entry::Vacant(e) => _ = e.insert(p),
        };

        Ok(())
    }

    fn on_update_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        let mut props = self.props.write();
        match props.get_mut(&p.key()) {
            Some(prop) => Ok(prop.update(p)),
            None => Err(IndigoError::Bus(crate::BusError::NotFound)),
        }
    }

    fn on_delete_property(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        p: Property,
        msg: Option<String>,
    ) -> Result<(), IndigoError> {
        let mut props = self.props.write();
        match props.remove(&p.key()) {
            Some(_) => Ok(()),
            None => Err(IndigoError::Bus(crate::BusError::NotFound)),
        }
    }

    fn on_send_message(
        &mut self,
        c: &mut Client<'a, Self::M>,
        d: String,
        msg: String,
    ) -> Result<(), IndigoError> {
        todo!()
    }
}