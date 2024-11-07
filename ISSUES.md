# libINDIGO issues

## Bugs

### INDIGO server connections
The INDIGO call for connecting to an INDIGO server is asynchronous (like everything else in INDIGO), but there is no callback method indicating connection success or failure.

* The `indigo_connection_status` INDIGO API call seems to indicate that the connection is OK even when no server connection is established.
* DNS failures are not reported and the server client keeps trying a long time (forever?) even whent the name does not resolve.
* The INDIGO API does not seem to have a way of interrupting ongoing connection attempts.

TODO: Better document the issues and raise bug issues with upstream INDIGO, if applicable.

## libINDGO structural issues
Apart from any bugs in the existing code base, the following structural issues are known and being worked on.

### Mapping of INDIGO constants
INDIGO contains a large number of string constants for names of interfaces, well-known properties, etc. These constants are automatically mapped to Rust code by [bindgen](https://github.com/rust-lang/rust-bindgen), for example:

```rust
pub const CONNECTION_PROPERTY_NAME: &[u8; 11] = b"CONNECTION\0";
```

Ideally these constants would be wrapped to a `&str` reference and/or mapped to an enum variant to be used in safe rust code. Exactly how to be figured out. The solution very likely includes adding custom `bindgen` code to [sys/build.rs](sys/build.rs).

### Using `&str` references for C strings
Beyond the INDIGO string constants, the INDIGO C-API uses C byte buffers for storing strings on all data carriers. Ideally we would point directly to these byte buffers without copying the buffer to a Rust string.

Currently the code base copies string buffers right and left, which seems like less than ideal.

Assuming this is feasible it, this requires a very good understanding and control of ownership.

### Using the bindgen structures in client and device code.
Due to the limitations of sending unsafe pointers to raw memory between threads, the `bindgen` generated INDIGO structures in the [sys](sys) crate are transformed to safe libINDIGO structures. This is wasteful, and ideally the `bindgen` code is used as the data carrier for the safe Rust code.

However, a case could be made for creating an INDIGO API in Rust that could be both be used not only for abstracting the INDIGO C-API, but also for a protocol level rewrite that parses messages in XML or JSON and is independent of the default C-implementation. The benefit of this would be better control of memory management but at the cost of a second protocol implementation that would require stringent testing to ensure compatibility and interoperability, not only with INDIGO but also with the legacy INDI protocol.

This might prove to be the best option for writing embedded Rust code that is unencumbered by legacy C-code. To be explored further...
