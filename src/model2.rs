use std::sync::RwLock;

use crate::{BusError, IndigoError};

pub struct IndigoModel {

}

impl IndigoModel {
    pub fn new() -> IndigoModel {
        IndigoModel {}
    }
}

pub struct IndigoElement<T> {
    dirty: RwLock<bool>,
    current: T,
    pending: T,
}

impl<T: Clone + PartialEq> IndigoElement<T> {

    /// Create a new element and assign a value.
    pub fn new(val: T) -> IndigoElement<T> {
        let p = val.clone();
        IndigoElement {
            dirty: RwLock::new(false),
            current: val,
            pending: p,
        }
    }

    /// Get the element value.
    pub fn get(&self) -> T {
        let d = self.dirty.read();
        self.current.clone()
    }

    /// Return `true` if a new value is requested but has not yet been confirmed by the server.
    pub fn dirty(&self) -> bool {
        return *self.dirty.read().unwrap();
    }

    /// Request an update of the element value, returns an error if a write lock cannot be obtained or if the element is dirty.
    pub fn request(&mut self, val: T) -> Result<(),IndigoError> {

        if val == self.current {
            return Err(IndigoError::Other("No change from the current value.".to_string()))
        }

        let Ok(mut d) = self.dirty.write() else {
            return Err(IndigoError::Other("Could not obtain element write lock.".to_string()))
        };

        if *d {
            return Err(IndigoError::Other("Element is dirty.".to_string()));
        }

        self.pending = val;
        *d = true;

        Ok(())
    }

    /// Update the element with a new value. If the element is dirty, it is assumed that the update matches the pending value. Should this not be the case, an `IndigoError:Bus(BusError::Failed` is returned
    fn update(&mut self, val: T) -> Result<(),IndigoError> {
        let Ok(mut d) = self.dirty.write() else {
            return Err(IndigoError::Other("Could not obtain element write lock.".to_string()))
        };

        // if dirty, i.e. an update has been requested but not yet finished and the update differs from the
        // pending value, the return an error.
        if *d && val != self.pending {
            return Err(IndigoError::Bus(BusError::Failed));
        }

        self.current = val;
        *d = false;

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model() -> Result<(),IndigoError>{
        let mut e = IndigoElement::new(42);
        e.request(43)?;
        assert!(e.dirty(), "element should be dirty after successful request");
        e.update(43)?;
        assert!(!e.dirty(), "element should be clean after successful update");
        assert_eq!(e.get(), 43, "element should contain the updated value");
        Ok(())
    }
}
