The `libindigo-sys` crate is a [bindgen](https://rust-lang.github.io/rust-bindgen) generated FFI for the upstream [INDIGO](https://www.indigo-astronomy.org/) source code in C found on [GitHub](https://github.com/indigo-astronomy/indigo).

> [!CAUTION]
> Using `libindigo-sys` directly instead of the [libindigo](..) API means writing a lot of
> unsafe Rust code based on C-pointers.

For more information on using the INDIGO system API, please refer to [INDIGO For Developers](https://www.indigo-astronomy.org/for-developers.html).

The [NOTES](NOTES.md) document contains some raw notes on how to compile and set the development environment for  `libindigo-sys`.

The `libindigo-sys` crate will do one of the following in order of priority when building the crate

1. Attempt to build INDIGO from the path of the envar INDIGO_SOURCE, if specified.
1. Attempt to build INDIGO from the path of the git submodule `externals/indigo`, if it has been initialised by git. To initialise the git submodule, execute the following command:

    ```bash
    git submodule update --init --recursive
    ```

1. Use INDIGO system headers in `/usr/include` and libs from `/usr/lib`, if `/usr/include/indigo/indigo_version.h` exists.
1. Initialise the git `externals/indigo` git submodule with the command above and build it.