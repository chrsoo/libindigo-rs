use std::{error::Error, marker::PhantomData};

use strum_macros::Display;

pub trait Property{ /* methods elided for brevity */ }
pub trait ClientController<P: Property, C: ClientDelegate<P>> { /* methods elided for brevity */ }
pub trait ClientDelegate<P: Property> { /* methods elided for brevity */ }

pub trait Bus<P: Property, D: ClientDelegate<P>> {
    /// Create a [ClientController], attached to the [Bus] that forwards all
    /// property events to a [ClientDelegate]
    fn attach_client(
        &mut self,
        client: D,
    ) -> Result<impl ClientController<P,D>,impl Error>;

    /* methods elided for brevity */
}
// -- SYS ----------------------------------------------------------------------

#[derive(Debug)]
pub struct SysError {}
impl std::fmt::Display for SysError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl Error for SysError {}
pub struct SysProperty<'a> {
    name: &'a str,
}
impl<'a> Property for SysProperty<'a> { }

pub struct SysClientController<'a,D: ClientDelegate<SysProperty<'a>>> {
    delegate: D,
    phantom: &'a PhantomData<D>
}

impl<'a, D: ClientDelegate<SysProperty<'a>>> ClientController<SysProperty<'a>,D>
for SysClientController<'a,D> { }

pub struct SysBus<'a, C: ClientDelegate<SysProperty<'a>>> {
    clients: Vec<SysClientController<'a,C>>,
}

impl<'s,C> Bus<SysProperty<'s>,C> for SysBus<'s,C>
where C: ClientDelegate<SysProperty<'s>> {
    fn attach_client(
        &mut self,
        delegate: C,
    ) -> Result<impl ClientController<SysProperty<'s>,C>,impl Error> {
        let controller = SysClientController { delegate, phantom: &PhantomData };
        Ok::<SysClientController<'s,C>, SysError>(controller)
    }
}