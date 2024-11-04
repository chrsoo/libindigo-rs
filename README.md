> [!CAUTION]
> The current version of LibINDIGO-rs should be considered **alpha** quality, i.e.
> a first draft for how INDIDGO development could look like with Rust. Please refer
> to the [known issues]](ISSUES.md) for details.

# File Structure
```bash
➜  libindigo git:(master) ✗ tree --gitignore
# files elided for brevity
...
├── Cargo.toml                  # `libindigo` crate definition
...
├── src
│   ├── bus.rs                  # internal bus module
│   ├── client.rs               # internal client module
│   ├── device.rs               # internal device module
│   ├── lib.rs                  # libindigo public interface
│   └── server.rs               # internal server module
├── sys
...
│   ├── Cargo.toml              # `libindigo-sys` crate definition
...
│   ├── build.rs                # builds the indigo static library
...
│   ├── externals               # git submodules
│   │   └── indigo              # -> https://github.com/indigo-astronomy/indigo.git
│   └── src
│       └── lib.rs              # libindigo public interface
└── tests                       # integration tests
    └── sanity.rs               # sanity tests, requires indigo running at localhost
```