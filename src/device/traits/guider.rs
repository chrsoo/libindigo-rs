//! Guider trait for autoguider devices.

use super::base::Device;
use crate::error::Result;

/// Guide direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideDirection {
    North,
    South,
    East,
    West,
}

/// A guide pulse command
#[derive(Debug, Clone, Copy)]
pub struct GuidePulse {
    /// Direction to guide
    pub direction: GuideDirection,
    /// Duration in milliseconds
    pub duration_ms: u32,
}

impl GuidePulse {
    pub fn new(direction: GuideDirection, duration_ms: u32) -> Self {
        Self {
            direction,
            duration_ms,
        }
    }
}

/// High-level interface for autoguider devices.
///
/// Typically used via the guider port on a mount or dedicated guide camera.
#[async_trait::async_trait]
pub trait Guider: Device {
    /// Send a guide pulse in a single direction
    async fn guide(&mut self, pulse: GuidePulse) -> Result<()>;

    /// Send simultaneous guide pulses (RA and Dec)
    async fn guide_dual(
        &mut self,
        ra_pulse: Option<GuidePulse>,
        dec_pulse: Option<GuidePulse>,
    ) -> Result<()>;

    /// Check if a guide pulse is currently active
    async fn is_guiding(&self) -> Result<bool>;

    /// Set the guide rate (fraction of sidereal, typically 0.5)
    async fn set_guide_rate(&mut self, ra_rate: f64, dec_rate: f64) -> Result<()>;

    /// Get the current guide rate
    async fn guide_rate(&self) -> Result<(f64, f64)>;
}
