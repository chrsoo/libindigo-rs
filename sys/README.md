The `libindigo-sys` crate is a [bindgen](https://rust-lang.github.io/rust-bindgen) generated FFI API from the upstream [INDIGO](https://www.indigo-astronomy.org/) source code found on [GitHub](https://github.com/indigo-astronomy/indigo).

Using `libindigo-sys` means writing a lot of unsafe Rust code using C-pointers.

For more information on using the INDIGO system API, please refer to [INDIGO For Developers](https://www.indigo-astronomy.org/for-developers.html).

The [NOTES](NOTES.md) document contains some raw notes on how to compile and set the development environment for  `libindigo-sys`. At some point these raw notes should be upgraded to proper documentation.