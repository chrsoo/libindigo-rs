//! Focuser trait for focuser devices.

use super::base::Device;
use crate::error::Result;

/// Focuser device information
#[derive(Debug, Clone)]
pub struct FocuserInfo {
    /// Maximum position value
    pub max_position: u32,
    /// Whether the focuser supports absolute positioning
    pub has_absolute: bool,
    /// Whether temperature compensation is available
    pub has_temperature_compensation: bool,
}

/// High-level interface for focuser devices.
#[async_trait::async_trait]
pub trait Focuser: Device {
    /// Get focuser information
    async fn focuser_info(&self) -> Result<FocuserInfo>;

    /// Move to an absolute position
    async fn move_to(&mut self, position: u32) -> Result<()>;

    /// Move relative steps (positive = outward, negative = inward)
    async fn move_relative(&mut self, steps: i32) -> Result<()>;

    /// Abort any movement in progress
    async fn abort_move(&mut self) -> Result<()>;

    /// Get current position
    async fn position(&self) -> Result<u32>;

    /// Check if focuser is currently moving
    async fn is_moving(&self) -> Result<bool>;

    /// Get temperature reading (if sensor available)
    async fn temperature(&self) -> Result<f64>;

    /// Enable/disable temperature compensation
    async fn set_temperature_compensation(&mut self, enabled: bool) -> Result<()>;

    /// Check if temperature compensation is enabled
    async fn temperature_compensation(&self) -> Result<bool>;

    /// Set backlash compensation (in steps)
    async fn set_backlash(&mut self, steps: u32) -> Result<()>;

    /// Get backlash setting
    async fn backlash(&self) -> Result<u32>;

    /// Set movement speed (0.0 to 1.0)
    async fn set_speed(&mut self, speed: f64) -> Result<()>;

    /// Get movement speed
    async fn speed(&self) -> Result<f64>;
}
