> [!CAUTION]
> The current version of LibINDIGO-rs should be considered alpha quality, i.e.
> a first draft for how INDIDGO development could look like with Rust.

# INDIGO issues

## INDIGO server connections
The INDIGO call for connecting to an INDIGO server is asynchronous (like everything else in INDIGO), but there is no callback method indicating connection success or failure.

* The `indigo_connection_status` INDIGO API call seems to indicate that the connection is OK even when no server connection is established.
* DNS failures are not reported and the server client keeps trying a long time (forever?) even whent the name does not resolve.
* The INDIGO API does not seem to have a way of interrupting ongoing connection attempts.

TODO: Better document the issues and raise bug issues with upstream INDIGO, if applicable.

# Structural issues
Apart from any bugs in the existing code base, the following structural issues are known and being worked on.

## Mapping of INDIGO constants
INDIGO contains a large number of string constants for names of interfaces, well-known properties, etc. These constants are automatically mapped to Rust code by [bindgen](https://github.com/rust-lang/rust-bindgen), for example:

```rust
pub const CONNECTION_PROPERTY_NAME: &[u8; 11] = b"CONNECTION\0";
```

Ideally these constants would be wrapped to a `&str` reference and/or mapped to an enum variant to be used in safe rust code. Exactly how to be figured out. The solution very likely includes adding custom `bindgen` code to [sys/build.rs](sys/build.rs).

## Using `&str` references for C strings
The INDIGO C-API uses C byte buffers for storing strings, and ideally we would point directly to these structures without copying the buffer to a Rust string. This requires a very good understanding and control of ownership.

Currently the code base copies string buffers right and left.

## Using the bindgen structures in client and device code.
Due to the issues of sending memory pointers between threads, the `bindgen` generated INDIGO structures in the [sys](sys) crate are transformed to safe libINDIGO-rs structures. This is wasteful, and ideally the `bindgen` code is used as the data carrier for the safe Rust code.

However, a case could be made for creating an INDIGO API in Rust that could be both be used not only for abstracting the INDIGO C-API, but also for a protocol level rewrite that parses messages in XML or JSON and is independent of the default C-implementation.

This might prove to be the best option for writing embedded Rust code that is unencumbered by legacy C-code. To be explored...
