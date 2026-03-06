# RELM Demonstrator Application

> [!CAUTION]
> **This application is currently DISABLED and non-functional.**
>
> The `libindigo-relm` demo has been temporarily disabled following the Phase 1-6 refactoring
> (GitHub issue #6). It was using deprecated FFI types that have been removed from the codebase.

## Status: Requires Refactoring

The relm crate needs to be refactored to use the new API structure:

- **Old API**: Used deprecated FFI types (`SysBus`, `SysClientController`, `SysRemoteResource`)
- **New API**: Should use `libindigo-rs` with `Client` + `RsClientStrategy`
- **Current State**: Code is commented out, dependency updated to `libindigo-rs`
- **Excluded**: Removed from workspace to avoid build failures (requires GTK4 system libraries)

## What Needs to Be Done

To restore this demo application:

1. **Refactor to new API pattern**:
   - Replace `SysBus::start()` with `RsClientStrategy::new()`
   - Replace `SysClientController` with `Client` from `ClientBuilder`
   - Update callback pattern to use async streams
   - Remove all FFI-specific code

2. **Update property handling**:
   - Use `libindigo::types::Property` instead of old `PropertyData`
   - Update to new `PropertyValue` enum structure
   - Adapt to new property update mechanisms

3. **Install GTK4 dependencies** (on systems where you want to run it):

   ```bash
   # macOS
   brew install gtk4

   # Ubuntu/Debian
   sudo apt-get install libgtk-4-dev
   ```

4. **Re-enable in workspace**:
   - Remove from `exclude` list in root `Cargo.toml`
   - Add back to `members` list

## Reference Implementation

See the `examples/` directory in the workspace root for working examples using the new API:

- `examples/discover_servers.rs` - Server discovery
- `examples/auto_connect.rs` - Client connection
- `examples/continuous_discovery.rs` - Property monitoring

## Original Description

The `libindigo-relm` module is an example INDIGO app for demonstrating the viability
of the `libindigo` API for building clients. It does not have a real purpose beyond this and should not be used for anything productive.

As a technology demonstration app, the `libindigo-relm` client has numerous known issues and limitations The following aspects of `libindigo-rs` client development has been successfully demonstrated:

- Connection of the INDIO client to the bus.
- List of client-side INDIGO devices and their properties.
- Rendering of TEXT properties.
- Rendering of NUMBER properties.

The following remains to be demonstrated:

- Deconnection and reconnection from the INDIGO server (cf. the connection issue)
- Server-side updates of INDIGO properties (cf. known issues below)
- Editing of INDIGO properties (not implemented)
- Rendering of SWITCH properties (partially implemented)
- Rendering of BLOB properties (not implemented)
- Rendering of LIGHT properties (not implemented)
- ...

# Known issues

- Property UPDATE events add new properties to the UI instead of updating the property.
- Scrolling for long lists of device properties is not yet supported.
- The `SwitchRule`is ignored when rendering SWITCH properties.
- Deconnection from the INDIGO server and detachement from the INDIGO bus (client is not detached).
- Reconnecting to the INDIGO server (DUPLICATE connection issue).
- It is not possible to abort an ongoing connection (e.g. when the DNS does not resolve).
- ...
