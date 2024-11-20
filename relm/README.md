The `libindigo-relm` module is an example INDIGO app for demonstrating the viability
of the `libindigo-rs` API for building clients. It does not have a real purpose beyond this and should not be used for anything productive.

> [!CAUTION]
> The libINDIGOrs Example App is highly unstable and mostly unusable.

As a technology demonstration app, the `libindigo-relm` client has numerous known issues and limitations The following aspects of `libindigo-rs` client development has been successfully demonstrated:

* Connection of the INDIO client to the bus.
* List of client-side INDIGO devices and their properties.
* Rendering of TEXT properties.
* Rendering of NUMBER properties.

The following remains to be demonstrated:

* Deconnection and reconnection from the INDIGO server (cf. the connection issue)
* Server-side updates of INDIGO properties (cf. known issues below)
* Editing of INDIGO properties (not implemented)
* Rendering of SWITCH properties (partially implemented)
* Rendering of BLOB properties (not implemented)
* Rendering of LIGHT properties (not implemented)
* ...

# Known issues

* Property UPDATE events add new properties to the UI instead of updating the property.
* Scrolling for long lists of device properties is not yet supported.
* The `SwitchRule`is ignored when rendering SWITCH properties.
* Deconnection from the INDIGO server and detachement from the INDIGO bus (client is not detached).
* Reconnecting to the INDIGO server (DUPLICATE connection issue).
* It is not possible to abort an ongoing connection (e.g. when the DNS does not resolve).
* ...