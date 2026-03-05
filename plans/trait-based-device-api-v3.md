# Trait-Based Device API (v0.3.0)

## Overview

Implement a high-level, trait-based API for common INDIGO device types that provides type-safe, ergonomic access to device-specific functionality. This builds on the low-level property-based API to offer device-specific abstractions that hide the complexity of property manipulation while maintaining full protocol compatibility.

**Version:** Planned for v0.3.0
**Status:** Not started
**Prerequisites:** Property extraction work (v0.2.0) must be completed first

## Prerequisites and Dependencies

This feature is planned for **v0.3.0** and depends on the completion of the property extraction work in v0.2.0:

1. **Phase 1 of Property Extraction** (v0.2.0): Automated generation of [`props.rs`](../props.rs:1) from INDIGO headers must be complete
2. **Phase 2 of Property Extraction** (v0.2.0): All hardcoded property name strings in [`src/strategies/rs/`](../src/strategies/rs/) and [`src/strategies/ffi.rs`](../src/strategies/ffi.rs:1) must be replaced with generated constants

The trait-based API will rely heavily on the generated property name constants to ensure type safety and consistency. Without the property extraction work, the trait implementations would be forced to use hardcoded strings, defeating the purpose of the extraction effort.

See [`plans/indigo-constants-extraction.md`](./indigo-constants-extraction.md:1) for details on the property extraction work.

## Goals

1. Provide type-safe traits for common device types (Camera, Mount, Focuser, FilterWheel, etc.)
2. Define required and optional properties for each device type using INDIGO standard names
3. Offer ergonomic methods that hide property manipulation details
4. Maintain compatibility with the low-level property API
5. Support async operations for device control
6. Enable compile-time verification of device capabilities where possible
7. Provide clear error messages for missing or incompatible properties

## Device Types

Based on INDIGO interfaces, implement traits for:

- **Camera**: Image capture, exposure control, cooling, binning, frame control
- **Mount**: Telescope positioning, tracking, slewing, parking, coordinate systems
- **Focuser**: Focus position control, temperature compensation, backlash
- **FilterWheel**: Filter selection and position management
- **Dome**: Dome rotation, shutter control, parking, slaving
- **GPS**: Location and time information
- **Guider**: Autoguiding control (RA/DEC pulse guiding)
- **AO (Adaptive Optics)**: Tip-tilt correction
- **Rotator**: Camera rotation control
- **Aux**: Auxiliary devices (power outlets, weather sensors, etc.)

## Architecture

### Core Trait Hierarchy

```rust
use crate::types::{Property, PropertyState, PropertyPerm};
use crate::error::Result;
use std::time::Duration;

/// Base trait for all INDIGO devices
pub trait Device {
    /// Get device name
    fn name(&self) -> &str;

    /// Get device interface bitmap
    fn interface(&self) -> u32;

    /// Get a property by name
    fn property(&self, name: &str) -> Option<&Property>;

    /// Get a mutable property by name
    fn property_mut(&mut self, name: &str) -> Option<&mut Property>;

    /// Check if device is connected
    fn is_connected(&self) -> bool;

    /// Connect to device
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from device
    async fn disconnect(&mut self) -> Result<()>;

    /// Wait for property to reach a specific state
    async fn wait_for_state(
        &self,
        property_name: &str,
        state: PropertyState,
        timeout: Duration,
    ) -> Result<()>;
}

/// Camera device trait
pub trait Camera: Device {
    // Required properties (must be present for a valid camera)
    fn ccd_exposure(&self) -> Result<&CcdExposureProperty>;
    fn ccd_exposure_mut(&mut self) -> Result<&mut CcdExposureProperty>;

    fn ccd_abort_exposure(&self) -> Result<&CcdAbortExposureProperty>;
    fn ccd_abort_exposure_mut(&mut self) -> Result<&mut CcdAbortExposureProperty>;

    fn ccd_image(&self) -> Result<&CcdImageProperty>;

    // Optional properties (may be present)
    fn ccd_temperature(&self) -> Option<&CcdTemperatureProperty>;
    fn ccd_temperature_mut(&mut self) -> Option<&mut CcdTemperatureProperty>;

    fn ccd_cooler(&self) -> Option<&CcdCoolerProperty>;
    fn ccd_cooler_mut(&mut self) -> Option<&mut CcdCoolerProperty>;

    fn ccd_binning(&self) -> Option<&CcdBinningProperty>;
    fn ccd_binning_mut(&mut self) -> Option<&mut CcdBinningProperty>;

    fn ccd_frame(&self) -> Option<&CcdFrameProperty>;
    fn ccd_frame_mut(&mut self) -> Option<&mut CcdFrameProperty>;

    fn ccd_info(&self) -> Option<&CcdInfoProperty>;

    fn ccd_gain(&self) -> Option<&CcdGainProperty>;
    fn ccd_gain_mut(&mut self) -> Option<&mut CcdGainProperty>;

    fn ccd_offset(&self) -> Option<&CcdOffsetProperty>;
    fn ccd_offset_mut(&mut self) -> Option<&mut CcdOffsetProperty>;

    // High-level methods

    /// Start an exposure with the given duration in seconds
    async fn start_exposure(&mut self, duration: f64) -> Result<()> {
        let prop = self.ccd_exposure_mut()?;
        prop.set_exposure(duration);
        self.send_property_update(prop).await
    }

    /// Abort the current exposure
    async fn abort_exposure(&mut self) -> Result<()> {
        let prop = self.ccd_abort_exposure_mut()?;
        prop.set_abort(true);
        self.send_property_update(prop).await
    }

    /// Wait for exposure to complete and download the image
    async fn capture_image(&mut self, duration: f64) -> Result<Image> {
        self.start_exposure(duration).await?;
        self.wait_for_state("CCD_EXPOSURE", PropertyState::Ok, Duration::from_secs(duration as u64 + 30)).await?;
        self.download_image().await
    }

    /// Download the most recent image
    async fn download_image(&self) -> Result<Image>;

    /// Set target CCD temperature in degrees Celsius
    async fn set_temperature(&mut self, temp: f64) -> Result<()> {
        let prop = self.ccd_temperature_mut()
            .ok_or(DeviceError::PropertyNotFound("CCD_TEMPERATURE".into()))?;
        prop.set_temperature(temp);
        self.send_property_update(prop).await
    }

    /// Enable or disable the CCD cooler
    async fn set_cooler(&mut self, enabled: bool) -> Result<()> {
        let prop = self.ccd_cooler_mut()
            .ok_or(DeviceError::PropertyNotFound("CCD_COOLER".into()))?;
        prop.set_enabled(enabled);
        self.send_property_update(prop).await
    }

    /// Set binning (horizontal, vertical)
    async fn set_binning(&mut self, h: u32, v: u32) -> Result<()> {
        let prop = self.ccd_binning_mut()
            .ok_or(DeviceError::PropertyNotFound("CCD_BIN".into()))?;
        prop.set_binning(h, v);
        self.send_property_update(prop).await
    }

    /// Set frame region (left, top, width, height)
    async fn set_frame(&mut self, left: u32, top: u32, width: u32, height: u32) -> Result<()> {
        let prop = self.ccd_frame_mut()
            .ok_or(DeviceError::PropertyNotFound("CCD_FRAME".into()))?;
        prop.set_frame(left, top, width, height);
        self.send_property_update(prop).await
    }

    /// Internal method to send property updates
    async fn send_property_update(&mut self, property: &Property) -> Result<()>;
}

/// Mount device trait
pub trait Mount: Device {
    // Required properties
    fn equatorial_coordinates(&self) -> Result<&EquatorialCoordinatesProperty>;
    fn equatorial_coordinates_mut(&mut self) -> Result<&mut EquatorialCoordinatesProperty>;

    fn horizontal_coordinates(&self) -> Result<&HorizontalCoordinatesProperty>;

    fn on_coordinates_set(&self) -> Result<&OnCoordinatesSetProperty>;
    fn on_coordinates_set_mut(&mut self) -> Result<&mut OnCoordinatesSetProperty>;

    // Optional properties
    fn mount_park(&self) -> Option<&MountParkProperty>;
    fn mount_park_mut(&mut self) -> Option<&mut MountParkProperty>;

    fn mount_tracking(&self) -> Option<&MountTrackingProperty>;
    fn mount_tracking_mut(&mut self) -> Option<&mut MountTrackingProperty>;

    fn mount_slew_rate(&self) -> Option<&MountSlewRateProperty>;
    fn mount_slew_rate_mut(&mut self) -> Option<&mut MountSlewRateProperty>;

    fn mount_abort_motion(&self) -> Option<&MountAbortMotionProperty>;
    fn mount_abort_motion_mut(&mut self) -> Option<&mut MountAbortMotionProperty>;

    // High-level methods

    /// Slew to equatorial coordinates (RA in hours, DEC in degrees)
    async fn slew_to(&mut self, ra: f64, dec: f64) -> Result<()> {
        let on_set = self.on_coordinates_set_mut()?;
        on_set.set_mode(CoordinatesSetMode::Slew);
        self.send_property_update(on_set).await?;

        let coords = self.equatorial_coordinates_mut()?;
        coords.set_coordinates(ra, dec);
        self.send_property_update(coords).await?;

        self.wait_for_state("MOUNT_EQUATORIAL_COORDINATES", PropertyState::Ok, Duration::from_secs(300)).await
    }

    /// Sync mount to equatorial coordinates (RA in hours, DEC in degrees)
    async fn sync_to(&mut self, ra: f64, dec: f64) -> Result<()> {
        let on_set = self.on_coordinates_set_mut()?;
        on_set.set_mode(CoordinatesSetMode::Sync);
        self.send_property_update(on_set).await?;

        let coords = self.equatorial_coordinates_mut()?;
        coords.set_coordinates(ra, dec);
        self.send_property_update(coords).await
    }

    /// Park the mount
    async fn park(&mut self) -> Result<()> {
        let prop = self.mount_park_mut()
            .ok_or(DeviceError::PropertyNotFound("MOUNT_PARK".into()))?;
        prop.set_parked(true);
        self.send_property_update(prop).await?;
        self.wait_for_state("MOUNT_PARK", PropertyState::Ok, Duration::from_secs(300)).await
    }

    /// Unpark the mount
    async fn unpark(&mut self) -> Result<()> {
        let prop = self.mount_park_mut()
            .ok_or(DeviceError::PropertyNotFound("MOUNT_PARK".into()))?;
        prop.set_parked(false);
        self.send_property_update(prop).await?;
        self.wait_for_state("MOUNT_PARK", PropertyState::Ok, Duration::from_secs(60)).await
    }

    /// Enable or disable tracking
    async fn set_tracking(&mut self, enabled: bool) -> Result<()> {
        let prop = self.mount_tracking_mut()
            .ok_or(DeviceError::PropertyNotFound("MOUNT_TRACKING".into()))?;
        prop.set_enabled(enabled);
        self.send_property_update(prop).await
    }

    /// Abort current motion
    async fn abort_motion(&mut self) -> Result<()> {
        let prop = self.mount_abort_motion_mut()
            .ok_or(DeviceError::PropertyNotFound("MOUNT_ABORT_MOTION".into()))?;
        prop.set_abort(true);
        self.send_property_update(prop).await
    }

    /// Internal method to send property updates
    async fn send_property_update(&mut self, property: &Property) -> Result<()>;
}

/// Focuser device trait
pub trait Focuser: Device {
    // Required properties
    fn focuser_position(&self) -> Result<&FocuserPositionProperty>;
    fn focuser_position_mut(&mut self) -> Result<&mut FocuserPositionProperty>;

    // Optional properties
    fn focuser_steps(&self) -> Option<&FocuserStepsProperty>;
    fn focuser_steps_mut(&mut self) -> Option<&mut FocuserStepsProperty>;

    fn focuser_temperature(&self) -> Option<&FocuserTemperatureProperty>;

    fn focuser_compensation(&self) -> Option<&FocuserCompensationProperty>;
    fn focuser_compensation_mut(&mut self) -> Option<&mut FocuserCompensationProperty>;

    fn focuser_abort_motion(&self) -> Option<&FocuserAbortMotionProperty>;
    fn focuser_abort_motion_mut(&mut self) -> Option<&mut FocuserAbortMotionProperty>;

    // High-level methods

    /// Move to absolute position
    async fn move_to(&mut self, position: u32) -> Result<()> {
        let prop = self.focuser_position_mut()?;
        prop.set_position(position);
        self.send_property_update(prop).await?;
        self.wait_for_state("FOCUSER_POSITION", PropertyState::Ok, Duration::from_secs(60)).await
    }

    /// Move by relative steps (positive = outward, negative = inward)
    async fn move_by(&mut self, steps: i32) -> Result<()> {
        let prop = self.focuser_steps_mut()
            .ok_or(DeviceError::PropertyNotFound("FOCUSER_STEPS".into()))?;
        prop.set_steps(steps.abs() as u32);
        prop.set_direction(if steps > 0 { Direction::Outward } else { Direction::Inward });
        self.send_property_update(prop).await?;
        self.wait_for_state("FOCUSER_STEPS", PropertyState::Ok, Duration::from_secs(60)).await
    }

    /// Abort current motion
    async fn abort_motion(&mut self) -> Result<()> {
        let prop = self.focuser_abort_motion_mut()
            .ok_or(DeviceError::PropertyNotFound("FOCUSER_ABORT_MOTION".into()))?;
        prop.set_abort(true);
        self.send_property_update(prop).await
    }

    /// Internal method to send property updates
    async fn send_property_update(&mut self, property: &Property) -> Result<()>;
}

/// FilterWheel device trait
pub trait FilterWheel: Device {
    // Required properties
    fn wheel_slot(&self) -> Result<&WheelSlotProperty>;
    fn wheel_slot_mut(&mut self) -> Result<&mut WheelSlotProperty>;

    // Optional properties
    fn wheel_slot_name(&self) -> Option<&WheelSlotNameProperty>;
    fn wheel_slot_offset(&self) -> Option<&WheelSlotOffsetProperty>;

    // High-level methods

    /// Move to filter slot (1-based index)
    async fn move_to_slot(&mut self, slot: u32) -> Result<()> {
        let prop = self.wheel_slot_mut()?;
        prop.set_slot(slot);
        self.send_property_update(prop).await?;
        self.wait_for_state("WHEEL_SLOT", PropertyState::Ok, Duration::from_secs(30)).await
    }

    /// Get current slot number
    fn current_slot(&self) -> Result<u32> {
        let prop = self.wheel_slot()?;
        Ok(prop.slot())
    }

    /// Get filter name for a slot
    fn slot_name(&self, slot: u32) -> Option<String> {
        self.wheel_slot_name()?.name_for_slot(slot)
    }

    /// Internal method to send property updates
    async fn send_property_update(&mut self, property: &Property) -> Result<()>;
}

// Additional traits for Dome, GPS, Guider, AO, Rotator, Aux...
```

### Property Wrappers

Create type-safe property wrappers that encapsulate property access and provide domain-specific methods:

```rust
use crate::types::{Property, PropertyItem, PropertyValue};
use crate::constants::*;

/// Wrapper for CCD_EXPOSURE property
pub struct CcdExposureProperty {
    property: Property,
}

impl CcdExposureProperty {
    pub fn new(property: Property) -> Self {
        Self { property }
    }

    pub fn exposure(&self) -> f64 {
        self.property.items
            .get(CCD_EXPOSURE_ITEM)
            .and_then(|item| match &item.value {
                PropertyValue::Number(n) => Some(n.value),
                _ => None,
            })
            .unwrap_or(0.0)
    }

    pub fn set_exposure(&mut self, duration: f64) {
        if let Some(item) = self.property.items.get_mut(CCD_EXPOSURE_ITEM) {
            if let PropertyValue::Number(ref mut n) = item.value {
                n.target = duration;
            }
        }
    }

    pub fn state(&self) -> PropertyState {
        self.property.state
    }

    pub fn is_busy(&self) -> bool {
        self.property.state == PropertyState::Busy
    }
}

/// Wrapper for MOUNT_EQUATORIAL_COORDINATES property
pub struct EquatorialCoordinatesProperty {
    property: Property,
}

impl EquatorialCoordinatesProperty {
    pub fn new(property: Property) -> Self {
        Self { property }
    }

    pub fn ra(&self) -> f64 {
        self.get_number_value(MOUNT_EQUATORIAL_COORDINATES_RA_ITEM)
    }

    pub fn dec(&self) -> f64 {
        self.get_number_value(MOUNT_EQUATORIAL_COORDINATES_DEC_ITEM)
    }

    pub fn set_coordinates(&mut self, ra: f64, dec: f64) {
        self.set_number_target(MOUNT_EQUATORIAL_COORDINATES_RA_ITEM, ra);
        self.set_number_target(MOUNT_EQUATORIAL_COORDINATES_DEC_ITEM, dec);
    }

    fn get_number_value(&self, item_name: &str) -> f64 {
        self.property.items
            .get(item_name)
            .and_then(|item| match &item.value {
                PropertyValue::Number(n) => Some(n.value),
                _ => None,
            })
            .unwrap_or(0.0)
    }

    fn set_number_target(&mut self, item_name: &str, value: f64) {
        if let Some(item) = self.property.items.get_mut(item_name) {
            if let PropertyValue::Number(ref mut n) = item.value {
                n.target = value;
            }
        }
    }
}

// Similar wrappers for other properties...
```

### Device Implementation

Provide a generic device implementation that wraps the client and implements device traits:

```rust
use crate::client::Client;
use crate::types::{Device as DeviceData, Property};
use std::collections::HashMap;

/// Generic device handle that can implement multiple device traits
pub struct DeviceHandle {
    client: Arc<Client>,
    device_name: String,
    device_data: DeviceData,
}

impl DeviceHandle {
    pub fn new(client: Arc<Client>, device_name: String, device_data: DeviceData) -> Self {
        Self {
            client,
            device_name,
            device_data,
        }
    }

    /// Check if device supports a specific interface
    pub fn supports_interface(&self, interface: u32) -> bool {
        (self.device_data.info.interface & interface) != 0
    }
}

impl Device for DeviceHandle {
    fn name(&self) -> &str {
        &self.device_name
    }

    fn interface(&self) -> u32 {
        self.device_data.info.interface
    }

    fn property(&self, name: &str) -> Option<&Property> {
        // Get property from device_data or client
        None // TODO: implement
    }

    fn property_mut(&mut self, name: &str) -> Option<&mut Property> {
        // Get mutable property
        None // TODO: implement
    }

    fn is_connected(&self) -> bool {
        // Check CONNECTION property
        false // TODO: implement
    }

    async fn connect(&mut self) -> Result<()> {
        // Set CONNECTION.CONNECTED = true
        Ok(()) // TODO: implement
    }

    async fn disconnect(&mut self) -> Result<()> {
        // Set CONNECTION.DISCONNECTED = true
        Ok(()) // TODO: implement
    }

    async fn wait_for_state(
        &self,
        property_name: &str,
        state: PropertyState,
        timeout: Duration,
    ) -> Result<()> {
        // Wait for property to reach state
        Ok(()) // TODO: implement
    }
}

// Implement Camera trait for DeviceHandle when it has camera interface
impl Camera for DeviceHandle {
    fn ccd_exposure(&self) -> Result<&CcdExposureProperty> {
        self.property(CCD_EXPOSURE_PROPERTY)
            .map(|p| CcdExposureProperty::new(p.clone()))
            .ok_or(DeviceError::PropertyNotFound(CCD_EXPOSURE_PROPERTY.into()))
    }

    // ... implement other Camera trait methods

    async fn download_image(&self) -> Result<Image> {
        // Download CCD_IMAGE blob
        Ok(Image::default()) // TODO: implement
    }

    async fn send_property_update(&mut self, property: &Property) -> Result<()> {
        self.client.update_property(property).await
    }
}

// Similar implementations for Mount, Focuser, etc.
```

## Implementation Phases

### Phase 1: Foundation (Week 1)

- [x] Review existing property and device types
- [ ] Define core `Device` trait with base functionality
- [ ] Create property wrapper infrastructure
- [ ] Implement `DeviceHandle` base structure
- [ ] Add property caching and update mechanisms
- [ ] Write unit tests for base functionality

### Phase 2: Camera Implementation (Week 2)

- [ ] Define `Camera` trait with required/optional properties
- [ ] Implement property wrappers for camera properties:
  - [ ] `CcdExposureProperty`
  - [ ] `CcdImageProperty`
  - [ ] `CcdTemperatureProperty`
  - [ ] `CcdCoolerProperty`
  - [ ] `CcdBinningProperty`
  - [ ] `CcdFrameProperty`
  - [ ] `CcdInfoProperty`
- [ ] Implement `Camera` trait for `DeviceHandle`
- [ ] Add image download functionality
- [ ] Write comprehensive integration tests
- [ ] Create camera usage examples

### Phase 3: Mount Implementation (Week 3)

- [ ] Define `Mount` trait
- [ ] Implement property wrappers for mount properties:
  - [ ] `EquatorialCoordinatesProperty`
  - [ ] `HorizontalCoordinatesProperty`
  - [ ] `MountParkProperty`
  - [ ] `MountTrackingProperty`
  - [ ] `MountSlewRateProperty`
  - [ ] `OnCoordinatesSetProperty`
- [ ] Implement coordinate transformations if needed
- [ ] Add slewing, parking, and tracking methods
- [ ] Write comprehensive integration tests
- [ ] Create mount usage examples

### Phase 4: Focuser & FilterWheel (Week 4)

- [ ] Define `Focuser` trait
- [ ] Implement focuser property wrappers
- [ ] Define `FilterWheel` trait
- [ ] Implement filter wheel property wrappers
- [ ] Write integration tests for both
- [ ] Create usage examples

### Phase 5: Additional Devices (Week 5-6)

- [ ] Implement `Dome` trait and property wrappers
- [ ] Implement `GPS` trait and property wrappers
- [ ] Implement `Guider` trait and property wrappers
- [ ] Implement `AO` trait and property wrappers
- [ ] Implement `Rotator` trait and property wrappers
- [ ] Implement `Aux` trait and property wrappers
- [ ] Write tests for each device type

### Phase 6: Integration & Polish (Week 7)

- [ ] Add device discovery with automatic type detection
- [ ] Create builder patterns for device construction
- [ ] Implement property state waiting with timeouts
- [ ] Add comprehensive error handling
- [ ] Create examples for each device type
- [ ] Update API documentation
- [ ] Performance testing and optimization

## Standard Property Names

All property names are defined in [`src/constants.rs`](../src/constants.rs:1). Key properties by device type:

**Camera Properties:**

- `CONNECTION` - Device connection control
- `INFO` - Device information
- `CCD_EXPOSURE` - Exposure control (required)
- `CCD_ABORT_EXPOSURE` - Abort exposure (required)
- `CCD_IMAGE` - Image data (required)
- `CCD_TEMPERATURE` - Temperature control (optional)
- `CCD_COOLER` - Cooler on/off (optional)
- `CCD_COOLER_POWER` - Cooler power percentage (optional)
- `CCD_BINNING` - Binning control (optional)
- `CCD_FRAME` - Frame/ROI control (optional)
- `CCD_FRAME_TYPE` - Frame type (light/dark/bias/flat) (optional)
- `CCD_INFO` - CCD chip information (optional)
- `CCD_GAIN` - Gain control (optional)
- `CCD_OFFSET` - Offset control (optional)

**Mount Properties:**

- `CONNECTION` - Device connection control
- `INFO` - Device information
- `MOUNT_EQUATORIAL_COORDINATES` - RA/DEC coordinates (required)
- `MOUNT_HORIZONTAL_COORDINATES` - Alt/Az coordinates (required)
- `MOUNT_ON_COORDINATES_SET` - Slew/Sync/Track mode (required)
- `MOUNT_PARK` - Park/Unpark (optional)
- `MOUNT_TRACKING` - Tracking on/off (optional)
- `MOUNT_SLEW_RATE` - Slew rate selection (optional)
- `MOUNT_ABORT_MOTION` - Abort motion (optional)
- `MOUNT_MOTION_DEC` - DEC motion control (optional)
- `MOUNT_MOTION_RA` - RA motion control (optional)

**Focuser Properties:**

- `CONNECTION` - Device connection control
- `INFO` - Device information
- `FOCUSER_POSITION` - Absolute position (required)
- `FOCUSER_STEPS` - Relative movement (optional)
- `FOCUSER_TEMPERATURE` - Temperature reading (optional)
- `FOCUSER_COMPENSATION` - Temperature compensation (optional)
- `FOCUSER_ABORT_MOTION` - Abort motion (optional)
- `FOCUSER_BACKLASH` - Backlash compensation (optional)

**FilterWheel Properties:**

- `CONNECTION` - Device connection control
- `INFO` - Device information
- `WHEEL_SLOT` - Current slot position (required)
- `WHEEL_SLOT_NAME` - Filter names (optional)
- `WHEEL_SLOT_OFFSET` - Focus offsets per filter (optional)

See [`src/constants.rs`](../src/constants.rs:1) for the complete list of all INDIGO standard properties.

## Error Handling

Define device-specific error types:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    #[error("Property is read-only: {0}")]
    ReadOnlyProperty(String),

    #[error("Property is write-only: {0}")]
    WriteOnlyProperty(String),

    #[error("Device not connected")]
    NotConnected,

    #[error("Device does not support interface: {0}")]
    InterfaceNotSupported(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Timeout waiting for property state")]
    Timeout,

    #[error("Invalid property value: {0}")]
    InvalidValue(String),

    #[error("Property state is Alert: {0}")]
    PropertyAlert(String),

    #[error("Client error: {0}")]
    ClientError(#[from] crate::error::Error),
}

pub type Result<T> = std::result::Result<T, DeviceError>;
```

## Testing Strategy

### Unit Tests

- Property wrapper value getters/setters
- Property state checking
- Error condition handling
- Type conversions

### Integration Tests

- Connect/disconnect operations
- Property updates and state changes
- Each device type's high-level methods
- Timeout handling
- Error propagation

### Mock Device Tests

- Create mock INDIGO devices for testing
- Test all device traits without hardware
- Test edge cases and error conditions

### Examples

- Simple camera capture
- Mount slewing and tracking
- Focuser movement
- Filter wheel operation
- Multi-device coordination

## Documentation

### API Documentation

- Comprehensive rustdoc for all traits and methods
- Code examples in documentation
- Property requirements clearly documented
- Error conditions documented

### User Guide

- Getting started with device APIs
- Examples for each device type
- Best practices for async operations
- Error handling patterns
- Migration guide from low-level API

### Developer Guide

- Adding new device types
- Creating property wrappers
- Extending existing traits
- Testing device implementations

## Success Criteria

- ✅ Type-safe traits for all major device types
- ✅ Ergonomic API that hides property manipulation details
- ✅ Required vs optional properties clearly distinguished
- ✅ Comprehensive test coverage (>80%)
- ✅ Clear documentation with examples
- ✅ Backward compatible with low-level property API
- ✅ Performance comparable to low-level API (<5% overhead)
- ✅ Compile-time interface checking where possible
- ✅ Clear, actionable error messages

## Future Enhancements

### Device Capability Detection

- Runtime detection of optional features
- Capability queries before operations
- Graceful degradation for missing features

### Automatic Property Polling

- Background property updates
- Change notifications
- Efficient polling strategies

### Event-Driven Updates

- Property change callbacks
- Async streams of property updates
- Reactive programming patterns

### Device Simulation

- Mock devices for testing
- Simulator framework
- Behavior scripting

### Code Generation

- Generate traits from INDIGO device definitions
- Automatic property wrapper generation
- Keep in sync with INDIGO updates

### Advanced Features

- Property batching for efficiency
- Transaction-like property updates
- Undo/redo for property changes
- Property history tracking

## References

- INDIGO Protocol: `sys/externals/indigo/indigo_docs/PROTOCOLS.md` (access restricted)
- INDIGO Headers: `sys/externals/indigo/indigo_libs/` (access restricted)
- Current property implementation: [`src/property.rs`](../src/property.rs:1)
- Current device types: [`src/types/device.rs`](../src/types/device.rs:1)
- Property types: [`src/types/property.rs`](../src/types/property.rs:1)
- Constants: [`src/constants.rs`](../src/constants.rs
