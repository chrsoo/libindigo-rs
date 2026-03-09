#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]

// Raw bindgen-generated FFI bindings to the INDIGO C library
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// INDIGO library version information.
///
/// This module contains version constants extracted from the INDIGO source code
/// at build time.
pub mod version {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}
