//! Tests for the trait-based device API.

use libindigo::device::traits::*;
use libindigo::device::DeviceInterface;
use libindigo::error::Result;
use libindigo::types::{Property, PropertyState};

// ============================================================================
// Test Data Structures
// ============================================================================

#[test]
fn test_ccd_info_creation() {
    let info = CcdInfo {
        width: 1920,
        height: 1080,
        pixel_size: 5.4,
        max_bin_x: 4,
        max_bin_y: 4,
        bits_per_pixel: 16,
    };

    assert_eq!(info.width, 1920);
    assert_eq!(info.height, 1080);
    assert_eq!(info.pixel_size, 5.4);
    assert_eq!(info.max_bin_x, 4);
    assert_eq!(info.max_bin_y, 4);
    assert_eq!(info.bits_per_pixel, 16);
}

#[test]
fn test_frame_type_display() {
    assert_eq!(FrameType::Light.to_string(), "FRAME_LIGHT");
    assert_eq!(FrameType::Dark.to_string(), "FRAME_DARK");
    assert_eq!(FrameType::Bias.to_string(), "FRAME_BIAS");
    assert_eq!(FrameType::Flat.to_string(), "FRAME_FLAT");
}

#[test]
fn test_binning_mode_constructors() {
    let bin1 = BinningMode::new(2, 2);
    assert_eq!(bin1.x, 2);
    assert_eq!(bin1.y, 2);

    let bin2 = BinningMode::symmetric(3);
    assert_eq!(bin2.x, 3);
    assert_eq!(bin2.y, 3);

    let bin_default = BinningMode::default();
    assert_eq!(bin_default.x, 1);
    assert_eq!(bin_default.y, 1);
}

#[test]
fn test_exposure_state_variants() {
    let idle = ExposureState::Idle;
    let exposing = ExposureState::Exposing(5.0);
    let complete = ExposureState::Complete;
    let aborted = ExposureState::Aborted;
    let error = ExposureState::Error;

    assert!(matches!(idle, ExposureState::Idle));
    assert!(matches!(exposing, ExposureState::Exposing(5.0)));
    assert!(matches!(complete, ExposureState::Complete));
    assert!(matches!(aborted, ExposureState::Aborted));
    assert!(matches!(error, ExposureState::Error));
}

#[test]
fn test_coordinates_creation() {
    let coords = Coordinates::new(12.5, 45.0);
    assert_eq!(coords.ra, 12.5);
    assert_eq!(coords.dec, 45.0);
}

#[test]
fn test_mount_type_variants() {
    let eq = MountType::Equatorial;
    let altaz = MountType::AltAz;
    let fork = MountType::Fork;
    let unknown = MountType::Unknown;

    assert_eq!(eq, MountType::Equatorial);
    assert_eq!(altaz, MountType::AltAz);
    assert_eq!(fork, MountType::Fork);
    assert_eq!(unknown, MountType::Unknown);
}

#[test]
fn test_tracking_mode_variants() {
    let sidereal = TrackingMode::Sidereal;
    let solar = TrackingMode::Solar;
    let lunar = TrackingMode::Lunar;
    let custom = TrackingMode::Custom(5);
    let off = TrackingMode::Off;

    assert_eq!(sidereal, TrackingMode::Sidereal);
    assert_eq!(solar, TrackingMode::Solar);
    assert_eq!(lunar, TrackingMode::Lunar);
    assert!(matches!(custom, TrackingMode::Custom(5)));
    assert_eq!(off, TrackingMode::Off);
}

#[test]
fn test_slew_rate_variants() {
    let guide = SlewRate::Guide;
    let centering = SlewRate::Centering;
    let find = SlewRate::Find;
    let max = SlewRate::Max;

    assert_eq!(guide, SlewRate::Guide);
    assert_eq!(centering, SlewRate::Centering);
    assert_eq!(find, SlewRate::Find);
    assert_eq!(max, SlewRate::Max);
}

#[test]
fn test_mount_axis_variants() {
    let primary = MountAxis::Primary;
    let secondary = MountAxis::Secondary;

    assert_eq!(primary, MountAxis::Primary);
    assert_eq!(secondary, MountAxis::Secondary);
}

#[test]
fn test_axis_direction_variants() {
    let forward = AxisDirection::Forward;
    let reverse = AxisDirection::Reverse;

    assert_eq!(forward, AxisDirection::Forward);
    assert_eq!(reverse, AxisDirection::Reverse);
}

#[test]
fn test_focuser_info_creation() {
    let info = FocuserInfo {
        max_position: 10000,
        has_absolute: true,
        has_temperature_compensation: true,
    };

    assert_eq!(info.max_position, 10000);
    assert!(info.has_absolute);
    assert!(info.has_temperature_compensation);
}

#[test]
fn test_filter_info_creation() {
    let info = FilterInfo {
        slot: 1,
        name: "Red".to_string(),
        offset: 50,
    };

    assert_eq!(info.slot, 1);
    assert_eq!(info.name, "Red");
    assert_eq!(info.offset, 50);
}

#[test]
fn test_guide_direction_variants() {
    let north = GuideDirection::North;
    let south = GuideDirection::South;
    let east = GuideDirection::East;
    let west = GuideDirection::West;

    assert_eq!(north, GuideDirection::North);
    assert_eq!(south, GuideDirection::South);
    assert_eq!(east, GuideDirection::East);
    assert_eq!(west, GuideDirection::West);
}

#[test]
fn test_guide_pulse_creation() {
    let pulse = GuidePulse::new(GuideDirection::North, 500);
    assert_eq!(pulse.direction, GuideDirection::North);
    assert_eq!(pulse.duration_ms, 500);
}

// ============================================================================
// DeviceProxy Tests
// ============================================================================

#[test]
fn test_device_proxy_creation() {
    let proxy = DeviceProxy::new("Test Device");
    assert_eq!(proxy.device_name(), "Test Device");
    assert!(!proxy.is_connected());
    assert_eq!(proxy.description(), None);
    assert_eq!(proxy.driver_version(), None);
}

#[test]
fn test_device_proxy_connection_state() {
    let mut proxy = DeviceProxy::new("Test Device");
    assert!(!proxy.is_connected());

    proxy.set_connected(true);
    assert!(proxy.is_connected());

    proxy.set_connected(false);
    assert!(!proxy.is_connected());
}

#[test]
fn test_device_proxy_description() {
    let mut proxy = DeviceProxy::new("Test Device");
    assert_eq!(proxy.description(), None);

    proxy.set_description("Test Description");
    assert_eq!(proxy.description(), Some("Test Description"));
}

#[test]
fn test_device_proxy_driver_version() {
    let mut proxy = DeviceProxy::new("Test Device");
    assert_eq!(proxy.driver_version(), None);

    proxy.set_driver_version("1.0.0");
    assert_eq!(proxy.driver_version(), Some("1.0.0"));
}

#[test]
fn test_device_proxy_property_caching() {
    use libindigo::types::{Property, PropertyPerm, PropertyState, PropertyType};
    use std::collections::HashMap;

    let mut proxy = DeviceProxy::new("Test Device");

    let property = Property {
        device: "Test Device".to_string(),
        name: "TEST_PROPERTY".to_string(),
        group: "Main".to_string(),
        label: "Test".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Text,
        items: HashMap::new(),
        timeout: None,
        timestamp: None,
        message: None,
    };

    proxy.cache_property(property.clone());

    let cached = proxy.get_cached_property("TEST_PROPERTY");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().name, "TEST_PROPERTY");
}

#[test]
fn test_device_proxy_get_number() {
    use libindigo::types::{
        Property, PropertyItem, PropertyPerm, PropertyState, PropertyType, PropertyValue,
    };
    use std::collections::HashMap;

    let mut proxy = DeviceProxy::new("Test Device");

    let mut items = HashMap::new();
    items.insert(
        "VALUE".to_string(),
        PropertyItem::new(
            "VALUE",
            "Value",
            PropertyValue::Number {
                value: 42.5,
                min: 0.0,
                max: 100.0,
                step: 0.1,
                format: "%.1f".to_string(),
            },
        ),
    );

    let property = Property {
        device: "Test Device".to_string(),
        name: "TEST_NUMBER".to_string(),
        group: "Main".to_string(),
        label: "Test".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Number,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    proxy.cache_property(property);

    let value = proxy.get_number("TEST_NUMBER", "VALUE");
    assert!(value.is_ok());
    assert_eq!(value.unwrap(), 42.5);
}

#[test]
fn test_device_proxy_get_text() {
    use libindigo::types::{
        Property, PropertyItem, PropertyPerm, PropertyState, PropertyType, PropertyValue,
    };
    use std::collections::HashMap;

    let mut proxy = DeviceProxy::new("Test Device");

    let mut items = HashMap::new();
    items.insert(
        "VALUE".to_string(),
        PropertyItem::new("VALUE", "Value", PropertyValue::text("Hello, World!")),
    );

    let property = Property {
        device: "Test Device".to_string(),
        name: "TEST_TEXT".to_string(),
        group: "Main".to_string(),
        label: "Test".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Text,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    proxy.cache_property(property);

    let value = proxy.get_text("TEST_TEXT", "VALUE");
    assert!(value.is_ok());
    assert_eq!(value.unwrap(), "Hello, World!");
}

#[test]
fn test_device_proxy_get_switch() {
    use libindigo::types::{
        Property, PropertyItem, PropertyPerm, PropertyState, PropertyType, PropertyValue,
        SwitchState,
    };
    use std::collections::HashMap;

    let mut proxy = DeviceProxy::new("Test Device");

    let mut items = HashMap::new();
    items.insert(
        "ON".to_string(),
        PropertyItem::new("ON", "On", PropertyValue::switch(SwitchState::On)),
    );
    items.insert(
        "OFF".to_string(),
        PropertyItem::new("OFF", "Off", PropertyValue::switch(SwitchState::Off)),
    );

    let property = Property {
        device: "Test Device".to_string(),
        name: "TEST_SWITCH".to_string(),
        group: "Main".to_string(),
        label: "Test".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Switch,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    proxy.cache_property(property);

    let on_value = proxy.get_switch("TEST_SWITCH", "ON");
    assert!(on_value.is_ok());
    assert!(on_value.unwrap());

    let off_value = proxy.get_switch("TEST_SWITCH", "OFF");
    assert!(off_value.is_ok());
    assert!(!off_value.unwrap());
}

#[test]
fn test_device_proxy_get_blob() {
    use libindigo::types::{
        Property, PropertyItem, PropertyPerm, PropertyState, PropertyType, PropertyValue,
    };
    use std::collections::HashMap;

    let mut proxy = DeviceProxy::new("Test Device");

    let blob_data = vec![1, 2, 3, 4, 5];
    let mut items = HashMap::new();
    items.insert(
        "IMAGE".to_string(),
        PropertyItem::new(
            "IMAGE",
            "Image",
            PropertyValue::blob(blob_data.clone(), ".fits"),
        ),
    );

    let property = Property {
        device: "Test Device".to_string(),
        name: "TEST_BLOB".to_string(),
        group: "Main".to_string(),
        label: "Test".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Blob,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    proxy.cache_property(property);

    let result = proxy.get_blob("TEST_BLOB", "IMAGE");
    assert!(result.is_ok());
    let (data, format) = result.unwrap();
    assert_eq!(data, blob_data);
    assert_eq!(format, ".fits");
}

#[test]
fn test_device_proxy_property_not_found() {
    let proxy = DeviceProxy::new("Test Device");

    let result = proxy.get_number("NONEXISTENT", "VALUE");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        libindigo::error::IndigoError::PropertyNotFound(_)
    ));
}

#[test]
fn test_device_proxy_type_mismatch() {
    use libindigo::types::{
        Property, PropertyItem, PropertyPerm, PropertyState, PropertyType, PropertyValue,
    };
    use std::collections::HashMap;

    let mut proxy = DeviceProxy::new("Test Device");

    let mut items = HashMap::new();
    items.insert(
        "VALUE".to_string(),
        PropertyItem::new("VALUE", "Value", PropertyValue::text("Not a number")),
    );

    let property = Property {
        device: "Test Device".to_string(),
        name: "TEST_TEXT".to_string(),
        group: "Main".to_string(),
        label: "Test".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Text,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    proxy.cache_property(property);

    // Try to get text property as number
    let result = proxy.get_number("TEST_TEXT", "VALUE");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        libindigo::error::IndigoError::InvalidParameter(_)
    ));
}

#[test]
fn test_device_proxy_clear_cache() {
    use libindigo::types::{Property, PropertyPerm, PropertyState, PropertyType};
    use std::collections::HashMap;

    let mut proxy = DeviceProxy::new("Test Device");

    let property = Property {
        device: "Test Device".to_string(),
        name: "TEST_PROPERTY".to_string(),
        group: "Main".to_string(),
        label: "Test".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Text,
        items: HashMap::new(),
        timeout: None,
        timestamp: None,
        message: None,
    };

    proxy.cache_property(property);
    assert!(proxy.get_cached_property("TEST_PROPERTY").is_some());

    proxy.clear_cache();
    assert!(proxy.get_cached_property("TEST_PROPERTY").is_none());
}

// ============================================================================
// Mock Device Implementations
// ============================================================================

/// Mock camera device for testing
struct MockCamera {
    name: String,
    connected: bool,
}

impl MockCamera {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            connected: false,
        }
    }
}

#[async_trait::async_trait]
impl Device for MockCamera {
    fn name(&self) -> &str {
        &self.name
    }

    fn device_type(&self) -> DeviceInterface {
        DeviceInterface::Ccd
    }

    async fn connect(&mut self) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn description(&self) -> Option<&str> {
        Some("Mock Camera Device")
    }

    fn driver_version(&self) -> Option<&str> {
        Some("1.0.0")
    }

    async fn get_property(&self, _name: &str) -> Result<Option<Property>> {
        Ok(None)
    }

    async fn get_properties(&self) -> Result<Vec<Property>> {
        Ok(Vec::new())
    }

    async fn wait_for_property_state(
        &self,
        _name: &str,
        _state: PropertyState,
        _timeout: std::time::Duration,
    ) -> Result<Property> {
        use libindigo::types::{Property, PropertyPerm, PropertyType};
        use std::collections::HashMap;

        Ok(Property {
            device: self.name.clone(),
            name: "TEST".to_string(),
            group: "Main".to_string(),
            label: "Test".to_string(),
            state: PropertyState::Ok,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Text,
            items: HashMap::new(),
            timeout: None,
            timestamp: None,
            message: None,
        })
    }
}

#[async_trait::async_trait]
impl Camera for MockCamera {
    async fn ccd_info(&self) -> Result<CcdInfo> {
        Ok(CcdInfo {
            width: 1920,
            height: 1080,
            pixel_size: 5.4,
            max_bin_x: 4,
            max_bin_y: 4,
            bits_per_pixel: 16,
        })
    }

    async fn start_exposure(&mut self, _duration: f64) -> Result<()> {
        Ok(())
    }

    async fn abort_exposure(&mut self) -> Result<()> {
        Ok(())
    }

    async fn exposure_state(&self) -> Result<ExposureState> {
        Ok(ExposureState::Idle)
    }

    async fn download_image(&self) -> Result<Vec<u8>> {
        Ok(vec![0; 1024])
    }

    async fn set_frame_type(&mut self, _frame_type: FrameType) -> Result<()> {
        Ok(())
    }

    async fn frame_type(&self) -> Result<FrameType> {
        Ok(FrameType::Light)
    }

    async fn set_binning(&mut self, _binning: BinningMode) -> Result<()> {
        Ok(())
    }

    async fn binning(&self) -> Result<BinningMode> {
        Ok(BinningMode::default())
    }

    async fn set_frame(&mut self, _x: u32, _y: u32, _width: u32, _height: u32) -> Result<()> {
        Ok(())
    }

    async fn frame(&self) -> Result<(u32, u32, u32, u32)> {
        Ok((0, 0, 1920, 1080))
    }

    async fn set_temperature(&mut self, _target: f64) -> Result<()> {
        Ok(())
    }

    async fn temperature(&self) -> Result<f64> {
        Ok(-10.0)
    }

    async fn set_cooler(&mut self, _enabled: bool) -> Result<()> {
        Ok(())
    }

    async fn cooler_enabled(&self) -> Result<bool> {
        Ok(true)
    }

    async fn cooler_power(&self) -> Result<f64> {
        Ok(75.0)
    }

    async fn set_gain(&mut self, _gain: f64) -> Result<()> {
        Ok(())
    }

    async fn gain(&self) -> Result<f64> {
        Ok(100.0)
    }

    async fn set_offset(&mut self, _offset: f64) -> Result<()> {
        Ok(())
    }

    async fn offset(&self) -> Result<f64> {
        Ok(10.0)
    }
}

#[tokio::test]
async fn test_mock_camera_device_trait() {
    let mut camera = MockCamera::new("Test Camera");

    assert_eq!(camera.name(), "Test Camera");
    assert_eq!(camera.device_type(), DeviceInterface::Ccd);
    assert!(!camera.is_connected());
    assert_eq!(camera.description(), Some("Mock Camera Device"));
    assert_eq!(camera.driver_version(), Some("1.0.0"));

    camera.connect().await.unwrap();
    assert!(camera.is_connected());

    camera.disconnect().await.unwrap();
    assert!(!camera.is_connected());
}

#[tokio::test]
async fn test_mock_camera_trait() {
    let mut camera = MockCamera::new("Test Camera");

    let info = camera.ccd_info().await.unwrap();
    assert_eq!(info.width, 1920);
    assert_eq!(info.height, 1080);

    camera.start_exposure(5.0).await.unwrap();
    camera.abort_exposure().await.unwrap();

    let state = camera.exposure_state().await.unwrap();
    assert!(matches!(state, ExposureState::Idle));

    let image = camera.download_image().await.unwrap();
    assert_eq!(image.len(), 1024);

    camera.set_frame_type(FrameType::Dark).await.unwrap();
    let frame_type = camera.frame_type().await.unwrap();
    assert_eq!(frame_type, FrameType::Light);

    camera.set_binning(BinningMode::symmetric(2)).await.unwrap();
    let binning = camera.binning().await.unwrap();
    assert_eq!(binning.x, 1);
    assert_eq!(binning.y, 1);

    camera.set_frame(0, 0, 1920, 1080).await.unwrap();
    let frame = camera.frame().await.unwrap();
    assert_eq!(frame, (0, 0, 1920, 1080));

    camera.set_temperature(-10.0).await.unwrap();
    let temp = camera.temperature().await.unwrap();
    assert_eq!(temp, -10.0);

    camera.set_cooler(true).await.unwrap();
    let cooler = camera.cooler_enabled().await.unwrap();
    assert!(cooler);

    let power = camera.cooler_power().await.unwrap();
    assert_eq!(power, 75.0);

    camera.set_gain(100.0).await.unwrap();
    let gain = camera.gain().await.unwrap();
    assert_eq!(gain, 100.0);

    camera.set_offset(10.0).await.unwrap();
    let offset = camera.offset().await.unwrap();
    assert_eq!(offset, 10.0);
}
