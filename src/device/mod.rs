//! Device Driver SPI for implementing INDIGO devices in Rust.
//!
//! This module provides the foundational traits and types needed to write
//! INDIGO device drivers. Device drivers implement the [`DeviceDriver`] trait
//! and use the [`PropertyManager`] to register and manage device properties.
//!
//! ## High-Level Device Traits
//!
//! The [`traits`] module provides high-level, ergonomic traits for specific
//! device types (Camera, Mount, Focuser, etc.) that abstract away low-level
//! property manipulation.

mod context;
mod driver;
mod property_manager;
mod registry;

pub mod traits;

pub use context::DeviceContext;
pub use driver::{DeviceDriver, DeviceInterface, DriverInfo};
pub use property_manager::{PropertyManager, PropertyUpdate};
pub use registry::DriverRegistry;
