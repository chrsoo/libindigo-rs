//! Mount trait for telescope mount devices.

use super::base::Device;
use crate::error::Result;

/// Equatorial coordinates (J2000)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    /// Right Ascension in hours (0-24)
    pub ra: f64,
    /// Declination in degrees (-90 to +90)
    pub dec: f64,
}

impl Coordinates {
    pub fn new(ra: f64, dec: f64) -> Self {
        Self { ra, dec }
    }
}

/// Mount type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountType {
    Equatorial,
    AltAz,
    Fork,
    Unknown,
}

/// Tracking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackingMode {
    Sidereal,
    Solar,
    Lunar,
    Custom(u32), // Custom rate index
    Off,
}

/// Slew rate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlewRate {
    Guide,
    Centering,
    Find,
    Max,
}

/// Mount axis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountAxis {
    /// RA or Azimuth axis
    Primary,
    /// Dec or Altitude axis
    Secondary,
}

/// Axis movement direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisDirection {
    Forward,
    Reverse,
}

/// High-level interface for telescope mount devices.
///
/// Provides methods for slewing, tracking, parking, and coordinate management.
#[async_trait::async_trait]
pub trait Mount: Device {
    /// Get the mount type
    async fn mount_type(&self) -> Result<MountType>;

    /// Slew to equatorial coordinates (J2000)
    async fn slew_to(&mut self, coords: Coordinates) -> Result<()>;

    /// Slew to horizontal coordinates (Alt/Az)
    async fn slew_to_altaz(&mut self, alt: f64, az: f64) -> Result<()>;

    /// Sync the mount to the given coordinates
    async fn sync_to(&mut self, coords: Coordinates) -> Result<()>;

    /// Abort any slew in progress
    async fn abort_slew(&mut self) -> Result<()>;

    /// Get current equatorial coordinates
    async fn coordinates(&self) -> Result<Coordinates>;

    /// Get current horizontal coordinates (alt, az)
    async fn altaz(&self) -> Result<(f64, f64)>;

    /// Check if the mount is currently slewing
    async fn is_slewing(&self) -> Result<bool>;

    /// Set tracking mode
    async fn set_tracking(&mut self, mode: TrackingMode) -> Result<()>;

    /// Get current tracking mode
    async fn tracking(&self) -> Result<TrackingMode>;

    /// Check if tracking is active
    async fn is_tracking(&self) -> Result<bool>;

    /// Park the mount
    async fn park(&mut self) -> Result<()>;

    /// Unpark the mount
    async fn unpark(&mut self) -> Result<()>;

    /// Check if mount is parked
    async fn is_parked(&self) -> Result<bool>;

    /// Set park position to current position
    async fn set_park_position(&mut self) -> Result<()>;

    /// Move the mount in a direction at a given rate
    async fn move_axis(
        &mut self,
        axis: MountAxis,
        direction: AxisDirection,
        rate: SlewRate,
    ) -> Result<()>;

    /// Stop motion on an axis
    async fn stop_axis(&mut self, axis: MountAxis) -> Result<()>;

    /// Get the local sidereal time
    async fn sidereal_time(&self) -> Result<f64>;

    /// Set the geographic location (latitude, longitude in degrees)
    async fn set_location(&mut self, latitude: f64, longitude: f64) -> Result<()>;

    /// Get the geographic location
    async fn location(&self) -> Result<(f64, f64)>;
}
