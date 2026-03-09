//! Camera trait for CCD/CMOS camera devices.

use super::base::Device;
use crate::error::Result;

/// CCD/CMOS camera information
#[derive(Debug, Clone)]
pub struct CcdInfo {
    /// Sensor width in pixels
    pub width: u32,
    /// Sensor height in pixels
    pub height: u32,
    /// Pixel size in microns
    pub pixel_size: f64,
    /// Maximum binning supported
    pub max_bin_x: u32,
    pub max_bin_y: u32,
    /// Bits per pixel
    pub bits_per_pixel: u32,
}

/// Frame type for camera exposures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Light,
    Bias,
    Dark,
    Flat,
}

impl std::fmt::Display for FrameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameType::Light => write!(f, "FRAME_LIGHT"),
            FrameType::Bias => write!(f, "FRAME_BIAS"),
            FrameType::Dark => write!(f, "FRAME_DARK"),
            FrameType::Flat => write!(f, "FRAME_FLAT"),
        }
    }
}

/// Binning mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinningMode {
    pub x: u32,
    pub y: u32,
}

impl BinningMode {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
    pub fn symmetric(n: u32) -> Self {
        Self { x: n, y: n }
    }
}

impl Default for BinningMode {
    fn default() -> Self {
        Self { x: 1, y: 1 }
    }
}

/// Exposure state tracking
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExposureState {
    /// Idle, ready for exposure
    Idle,
    /// Exposure in progress, time remaining in seconds
    Exposing(f64),
    /// Exposure complete, image ready for download
    Complete,
    /// Exposure was aborted
    Aborted,
    /// Error during exposure
    Error,
}

/// High-level interface for CCD/CMOS camera devices.
///
/// # Example
/// ```rust,no_run
/// use libindigo::device::traits::{Camera, FrameType, BinningMode};
///
/// async fn capture(camera: &mut dyn Camera) -> libindigo::error::Result<Vec<u8>> {
///     camera.connect().await?;
///     camera.set_frame_type(FrameType::Light).await?;
///     camera.set_binning(BinningMode::symmetric(2)).await?;
///     camera.start_exposure(5.0).await?;
///     let image = camera.download_image().await?;
///     Ok(image)
/// }
/// ```
#[async_trait::async_trait]
pub trait Camera: Device {
    /// Get CCD sensor information
    async fn ccd_info(&self) -> Result<CcdInfo>;

    /// Start an exposure with the given duration in seconds
    async fn start_exposure(&mut self, duration: f64) -> Result<()>;

    /// Abort a running exposure
    async fn abort_exposure(&mut self) -> Result<()>;

    /// Get the current exposure state
    async fn exposure_state(&self) -> Result<ExposureState>;

    /// Download the latest captured image as raw bytes
    async fn download_image(&self) -> Result<Vec<u8>>;

    /// Set the frame type (Light, Dark, Bias, Flat)
    async fn set_frame_type(&mut self, frame_type: FrameType) -> Result<()>;

    /// Get the current frame type
    async fn frame_type(&self) -> Result<FrameType>;

    /// Set binning mode
    async fn set_binning(&mut self, binning: BinningMode) -> Result<()>;

    /// Get current binning mode
    async fn binning(&self) -> Result<BinningMode>;

    /// Set the subframe/ROI (x, y, width, height in pixels)
    async fn set_frame(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<()>;

    /// Get current frame/ROI settings
    async fn frame(&self) -> Result<(u32, u32, u32, u32)>;

    /// Set CCD temperature target (in Celsius)
    async fn set_temperature(&mut self, target: f64) -> Result<()>;

    /// Get current CCD temperature (in Celsius)
    async fn temperature(&self) -> Result<f64>;

    /// Enable/disable cooler
    async fn set_cooler(&mut self, enabled: bool) -> Result<()>;

    /// Check if cooler is enabled
    async fn cooler_enabled(&self) -> Result<bool>;

    /// Get cooler power percentage (0-100)
    async fn cooler_power(&self) -> Result<f64>;

    /// Set gain (camera-specific units)
    async fn set_gain(&mut self, gain: f64) -> Result<()>;

    /// Get current gain
    async fn gain(&self) -> Result<f64>;

    /// Set offset (camera-specific units)
    async fn set_offset(&mut self, offset: f64) -> Result<()>;

    /// Get current offset
    async fn offset(&self) -> Result<f64>;
}
