//! FilterWheel trait for filter wheel devices.

use super::base::Device;
use crate::error::Result;

/// Information about a filter in the wheel
#[derive(Debug, Clone)]
pub struct FilterInfo {
    /// Slot number (1-based)
    pub slot: u32,
    /// Filter name
    pub name: String,
    /// Filter offset (focus compensation)
    pub offset: i32,
}

/// High-level interface for filter wheel devices.
#[async_trait::async_trait]
pub trait FilterWheel: Device {
    /// Get the number of filter slots
    async fn slot_count(&self) -> Result<u32>;

    /// Get information about all filters
    async fn filters(&self) -> Result<Vec<FilterInfo>>;

    /// Move to a specific filter slot (1-based)
    async fn select_filter(&mut self, slot: u32) -> Result<()>;

    /// Get the current filter slot number
    async fn current_slot(&self) -> Result<u32>;

    /// Get the name of the current filter
    async fn current_filter_name(&self) -> Result<String>;

    /// Check if the wheel is currently moving
    async fn is_moving(&self) -> Result<bool>;

    /// Set the name for a filter slot
    async fn set_filter_name(&mut self, slot: u32, name: &str) -> Result<()>;

    /// Set the focus offset for a filter slot
    async fn set_filter_offset(&mut self, slot: u32, offset: i32) -> Result<()>;
}
