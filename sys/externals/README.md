# Building from a cloned git repository
When cloning the [libindigo-rs](https://github.com/chrsoo/libindigo-rs) repository,this directory contains the git `indigo` submodule pointing to the upstream [INDIGO](https://github.com/indigo-astronomy/indigo) GitHub repository.

Unless the submodule is properly initialised, it will not be pulled when cloning the repository. To initialise the submodule, use the following command:

```bash
git submodule update --init --recursive
```

The submodule will also be pulled by the [build.rs](../build.rs) script in the parent directory providing an external INDIGO source repository has not been defined by specifying the `INDIGO_SOURCE` environment variable or the INDIGO headers and libraries can be found in on the local system.

# Building using system libraries

> [!CAUTION]
> Using system libraries only works on Linux.

On a Linux system, the [build.rs](../build.rs) script only checks for the existence of the `/usr/include/indigo/indigo_version.h` header file and then assumes that all other headers are present and that the INDIGO library files can be found in `/usr/lib/libindigo.a`.

# Building from a crate

When the [libindigo-sys] dependency is added either directly or indirectly through another dependency to the `Cargo.toml` manifest of a crate, `cargo` will pull the crate from [crates.io](https://crates.io). The crate package file only contains a subset of the full source code found in the upstream [INDIGO](https://github.com/indigo-astronomy/indigo) GitHub repository.