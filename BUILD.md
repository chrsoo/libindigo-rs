# Building and releasing libindigo

## Build

To build libindigo for the fist time:

1. Setup your environment to build INDIGO, cf [sys/NOTES.md](sys/NOTES.md)
1. Run...

   ```shell
   cargo build --workspace
   ```

   > [!NOTE]
   > The [build.rs](sys/build.rs) script should automatically checout the source code in the [sys/externals/indigo/](sys/externals/indigo/) git submodule. For more information, please refer to [sys/README.md](sys/README.md)

## Versioning

Versioning follows the general [SemVer 2.0.0](https://semver.org/) rules:

> - MAJOR version when you make incompatible API changes
> - MINOR version when you add functionality in a backward compatible manner
> - PATCH version when you make backward compatible bug fixes

The [sys](sys) adds build info for the specific upstream INDIGO version that was used to produce the build, e.g.

```text
0.1.1+INDIGO.2.0.301
```

Increasing the version can be done through [cargo bump](https://crates.io/crates/cargo-bump):

```text
USAGE:
    cargo bump [FLAGS] [<version> | major | minor | patch]

FLAGS:
    -h, --help       Prints help information
    -v, --version    Prints version information

ARGS:
    <version>    Version should be a semver (https://semver.org/) string or the
                 position of the current version to increment: major, minor or patch.
```

To install bump:

```shell
cargo install cargo-bump
```

When creating a new version, the `cargo bump` command has to be applied once for each module of the cargo workspace:

```shell
cargo bump patch
cd sys
cargo bump patch
cd ../relm
cargo bump patch
cd ..
```

## Release

Install [cargo-release](https://github.com/crate-ci/cargo-release), follow the instructions.
