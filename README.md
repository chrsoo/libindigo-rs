> [!CAUTION]
> The current version of LibINDIGO-rs should be considered **alpha** quality, i.e.
> a first draft for how INDIDGO development could look like with Rust. Please refer
> to the [known issues](ISSUES.md) for details.

The `libindigo-rs` API is consists of five public modules:

* `libindigo` - Shared API for client and device driver development.
* `libindigo/bus` - API related to controlling the INDIGO bus.
* `libindigo/client` - API specific for developing INDIGO clients.
* `libindigo/device` - API specific for developing INDIGO device drivers.
* `libindigo/server` - API specific for managing INDIGO servers.

The `client` and `device` modules contains implementations of traits defined in the `libindigo` API, notable `Device`

# File Structure
```bash
tree --gitignore
# files elided for brevity
├── relm/...                    # Example Relm4 Client APP crate for testing libindigo.
├── src                         # libindigo source code
│   ├── bus.rs                  # internal bus module
│   ├── client.rs               # public API specific to client development
│   ├── device.rs               # public API specific for device and driver development
│   ├── lib.rs                  # generic INDIGO code applicable for both devices and clients
│   ├── property.rs             # internal property module
│   └── server.rs               # internal server module
├── sys                         # INDIGO system level API crate
│   ├── externals               # git submodules
│   │   └── indigo              # -> https://github.com/indigo-astronomy/indigo.git
│   └── build.rs                # generates Rust API using `bindgen`
├── tests                       # integration tests
│   └── sanity.rs               # sanity tests, requires indigo running at localhost
├── Cargo.toml                  # `libindigo` crate definition
├── ISSUES.md                   # known issues
├── NOTES.md                    # development notes
└── READM.md                    # this file
```