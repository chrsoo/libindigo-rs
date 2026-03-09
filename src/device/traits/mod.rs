//! High-level device traits for type-safe INDIGO device interaction.
//!
//! These traits provide ergonomic, domain-specific APIs for interacting with
//! INDIGO devices. Instead of manipulating raw properties, users can call
//! typed methods like `camera.start_exposure(5.0)` or `mount.slew_to(ra, dec)`.

mod base;
mod camera;
mod filter_wheel;
mod focuser;
mod guider;
mod mount;
mod proxy;

pub use base::Device;
pub use camera::{BinningMode, Camera, CcdInfo, ExposureState, FrameType};
pub use filter_wheel::{FilterInfo, FilterWheel};
pub use focuser::{Focuser, FocuserInfo};
pub use guider::{GuideDirection, GuidePulse, Guider};
pub use mount::{AxisDirection, Coordinates, Mount, MountAxis, MountType, SlewRate, TrackingMode};
pub use proxy::DeviceProxy;
