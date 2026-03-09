//! Tests for the Device Driver API

use libindigo::device::{
    DeviceContext, DeviceDriver, DeviceInterface, DriverInfo, DriverRegistry, PropertyManager,
};
use libindigo::error::Result;
use libindigo::types::{
    Property, PropertyItem, PropertyPerm, PropertyState, PropertyType, PropertyValue, SwitchState,
};

/// Mock device driver for testing
struct MockCamera {
    name: String,
}

impl MockCamera {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DeviceDriver for MockCamera {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: self.name.clone(),
            description: "Mock Camera Driver".to_string(),
            version: "1.0.0".to_string(),
            interfaces: DeviceInterface::Ccd as u32,
        }
    }

    async fn attach(&mut self, ctx: &mut DeviceContext) -> Result<()> {
        // Register standard connection property
        ctx.property_manager().register_standard_connection()?;

        // Register device info
        ctx.property_manager().register_device_info(
            "Mock Camera Driver",
            "1.0.0",
            DeviceInterface::Ccd as u32,
        )?;

        Ok(())
    }

    async fn change_property(
        &mut self,
        _ctx: &mut DeviceContext,
        _property: &Property,
    ) -> Result<()> {
        // Handle property changes
        Ok(())
    }

    async fn detach(&mut self, _ctx: &mut DeviceContext) -> Result<()> {
        // Clean up
        Ok(())
    }
}

#[tokio::test]
async fn test_device_driver_info() {
    let driver = MockCamera::new("Test Camera");
    let info = driver.info();

    assert_eq!(info.name, "Test Camera");
    assert_eq!(info.description, "Mock Camera Driver");
    assert_eq!(info.version, "1.0.0");
    assert_eq!(info.interfaces, DeviceInterface::Ccd as u32);
}

#[tokio::test]
async fn test_device_interface_combine() {
    let interfaces = vec![
        DeviceInterface::Ccd,
        DeviceInterface::FilterWheel,
        DeviceInterface::Focuser,
    ];

    let combined = DeviceInterface::combine(&interfaces);

    // Check that all interfaces are present in the bitmask
    assert_ne!(combined & (DeviceInterface::Ccd as u32), 0);
    assert_ne!(combined & (DeviceInterface::FilterWheel as u32), 0);
    assert_ne!(combined & (DeviceInterface::Focuser as u32), 0);
    assert_eq!(combined & (DeviceInterface::Mount as u32), 0);
}

#[tokio::test]
async fn test_property_manager_new() {
    let pm = PropertyManager::new("Test Device");

    assert_eq!(pm.device_name(), "Test Device");
    assert_eq!(pm.property_count(), 0);
}

#[tokio::test]
async fn test_property_manager_define_property() {
    let mut pm = PropertyManager::new("Test Device");

    let mut items = std::collections::HashMap::new();
    items.insert(
        "EXPOSURE".to_string(),
        PropertyItem::new("EXPOSURE", "Exposure", PropertyValue::number(1.0)),
    );

    let property = Property {
        device: "Test Device".to_string(),
        name: "CCD_EXPOSURE".to_string(),
        group: "Main".to_string(),
        label: "Exposure".to_string(),
        state: PropertyState::Idle,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Number,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    pm.define_property(property).unwrap();

    assert_eq!(pm.property_count(), 1);
    assert!(pm.has_property("CCD_EXPOSURE"));
    assert!(pm.get_property("CCD_EXPOSURE").is_some());
}

#[tokio::test]
async fn test_property_manager_duplicate_property() {
    let mut pm = PropertyManager::new("Test Device");

    let mut items = std::collections::HashMap::new();
    items.insert(
        "EXPOSURE".to_string(),
        PropertyItem::new("EXPOSURE", "Exposure", PropertyValue::number(1.0)),
    );

    let property = Property {
        device: "Test Device".to_string(),
        name: "CCD_EXPOSURE".to_string(),
        group: "Main".to_string(),
        label: "Exposure".to_string(),
        state: PropertyState::Idle,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Number,
        items: items.clone(),
        timeout: None,
        timestamp: None,
        message: None,
    };

    pm.define_property(property.clone()).unwrap();

    // Try to define the same property again
    let result = pm.define_property(property);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_property_manager_standard_connection() {
    let mut pm = PropertyManager::new("Test Device");

    pm.register_standard_connection().unwrap();

    assert!(pm.has_property("CONNECTION"));

    let prop = pm.get_property("CONNECTION").unwrap();
    assert_eq!(prop.name, "CONNECTION");
    assert_eq!(prop.property_type, PropertyType::Switch);
    assert!(prop.items.contains_key("CONNECTED"));
    assert!(prop.items.contains_key("DISCONNECTED"));
}

#[tokio::test]
async fn test_property_manager_device_info() {
    let mut pm = PropertyManager::new("Test Device");

    pm.register_device_info("Test Description", "1.0.0", DeviceInterface::Ccd as u32)
        .unwrap();

    assert!(pm.has_property("INFO"));

    let prop = pm.get_property("INFO").unwrap();
    assert_eq!(prop.name, "INFO");
    assert_eq!(prop.property_type, PropertyType::Text);
    assert_eq!(prop.perm, PropertyPerm::ReadOnly);
    assert!(prop.items.contains_key("DEVICE_INTERFACE"));
    assert!(prop.items.contains_key("DEVICE_DESCRIPTION"));
    assert!(prop.items.contains_key("DRIVER_VERSION"));
}

#[tokio::test]
async fn test_property_manager_update_property() {
    let mut pm = PropertyManager::new("Test Device");

    let mut items = std::collections::HashMap::new();
    items.insert(
        "EXPOSURE".to_string(),
        PropertyItem::new("EXPOSURE", "Exposure", PropertyValue::number(1.0)),
    );

    let property = Property {
        device: "Test Device".to_string(),
        name: "CCD_EXPOSURE".to_string(),
        group: "Main".to_string(),
        label: "Exposure".to_string(),
        state: PropertyState::Idle,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Number,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    pm.define_property(property).unwrap();

    // Update the property
    pm.update_property(
        "CCD_EXPOSURE",
        PropertyState::Busy,
        vec![("EXPOSURE".to_string(), PropertyValue::number(2.0))],
    )
    .unwrap();

    let prop = pm.get_property("CCD_EXPOSURE").unwrap();
    assert_eq!(prop.state, PropertyState::Busy);

    // Check pending updates
    let updates = pm.drain_pending_updates();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].property_name, "CCD_EXPOSURE");
    assert_eq!(updates[0].state, PropertyState::Busy);
}

#[tokio::test]
async fn test_property_manager_delete_property() {
    let mut pm = PropertyManager::new("Test Device");

    pm.register_standard_connection().unwrap();
    assert!(pm.has_property("CONNECTION"));

    pm.delete_property("CONNECTION").unwrap();
    assert!(!pm.has_property("CONNECTION"));
}

#[tokio::test]
async fn test_property_manager_delete_all() {
    let mut pm = PropertyManager::new("Test Device");

    pm.register_standard_connection().unwrap();
    pm.register_device_info("Test", "1.0", 0).unwrap();

    assert_eq!(pm.property_count(), 2);

    pm.delete_all_properties();
    assert_eq!(pm.property_count(), 0);
}

#[tokio::test]
async fn test_device_context_new() {
    let ctx = DeviceContext::new("Test Device");

    assert_eq!(ctx.properties().device_name(), "Test Device");
    assert!(!ctx.is_connected());
}

#[tokio::test]
async fn test_device_context_connected_state() {
    let mut ctx = DeviceContext::new("Test Device");

    assert!(!ctx.is_connected());

    ctx.set_connected(true);
    assert!(ctx.is_connected());

    ctx.set_connected(false);
    assert!(!ctx.is_connected());
}

#[tokio::test]
async fn test_driver_registry_new() {
    let registry = DriverRegistry::new();

    assert_eq!(registry.count(), 0);
}

#[tokio::test]
async fn test_driver_registry_register() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();

    assert_eq!(registry.count(), 1);
    assert!(registry.is_registered("Test Camera"));
    assert!(!registry.is_attached("Test Camera"));
}

#[tokio::test]
async fn test_driver_registry_duplicate_register() {
    let mut registry = DriverRegistry::new();

    let driver1 = Box::new(MockCamera::new("Test Camera"));
    registry.register(driver1).unwrap();

    let driver2 = Box::new(MockCamera::new("Test Camera"));
    let result = registry.register(driver2);

    assert!(result.is_err());
}

#[tokio::test]
async fn test_driver_registry_attach() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();
    assert!(!registry.is_attached("Test Camera"));

    registry.attach("Test Camera").await.unwrap();
    assert!(registry.is_attached("Test Camera"));
}

#[tokio::test]
async fn test_driver_registry_attach_unregistered() {
    let mut registry = DriverRegistry::new();

    let result = registry.attach("Nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_driver_registry_double_attach() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();
    registry.attach("Test Camera").await.unwrap();

    let result = registry.attach("Test Camera").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_driver_registry_detach() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();
    registry.attach("Test Camera").await.unwrap();
    assert!(registry.is_attached("Test Camera"));

    registry.detach("Test Camera").await.unwrap();
    assert!(!registry.is_attached("Test Camera"));
}

#[tokio::test]
async fn test_driver_registry_detach_not_attached() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();

    let result = registry.detach("Test Camera").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_driver_registry_attach_all() {
    let mut registry = DriverRegistry::new();

    registry
        .register(Box::new(MockCamera::new("Camera 1")))
        .unwrap();
    registry
        .register(Box::new(MockCamera::new("Camera 2")))
        .unwrap();

    registry.attach_all().await.unwrap();

    assert!(registry.is_attached("Camera 1"));
    assert!(registry.is_attached("Camera 2"));
}

#[tokio::test]
async fn test_driver_registry_detach_all() {
    let mut registry = DriverRegistry::new();

    registry
        .register(Box::new(MockCamera::new("Camera 1")))
        .unwrap();
    registry
        .register(Box::new(MockCamera::new("Camera 2")))
        .unwrap();

    registry.attach_all().await.unwrap();
    registry.detach_all().await.unwrap();

    assert!(!registry.is_attached("Camera 1"));
    assert!(!registry.is_attached("Camera 2"));
}

#[tokio::test]
async fn test_driver_registry_list_drivers() {
    let mut registry = DriverRegistry::new();

    registry
        .register(Box::new(MockCamera::new("Camera 1")))
        .unwrap();
    registry
        .register(Box::new(MockCamera::new("Camera 2")))
        .unwrap();

    registry.attach("Camera 1").await.unwrap();

    let drivers = registry.list_drivers();
    assert_eq!(drivers.len(), 2);

    // Check that one is attached and one is not
    let attached_count = drivers.iter().filter(|(_, attached)| *attached).count();
    assert_eq!(attached_count, 1);
}

#[tokio::test]
async fn test_driver_registry_unregister() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();
    assert!(registry.is_registered("Test Camera"));

    registry.unregister("Test Camera").unwrap();
    assert!(!registry.is_registered("Test Camera"));
}

#[tokio::test]
async fn test_driver_registry_unregister_attached() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();
    registry.attach("Test Camera").await.unwrap();

    // Should not be able to unregister while attached
    let result = registry.unregister("Test Camera");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_driver_registry_handle_property_change() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();
    registry.attach("Test Camera").await.unwrap();

    let mut items = std::collections::HashMap::new();
    items.insert(
        "CONNECTED".to_string(),
        PropertyItem::new(
            "CONNECTED",
            "Connected",
            PropertyValue::switch(SwitchState::On),
        ),
    );

    let property = Property {
        device: "Test Camera".to_string(),
        name: "CONNECTION".to_string(),
        group: "Main".to_string(),
        label: "Connection".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Switch,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    // Should succeed
    registry
        .handle_property_change("Test Camera", &property)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_driver_registry_handle_property_change_not_attached() {
    let mut registry = DriverRegistry::new();
    let driver = Box::new(MockCamera::new("Test Camera"));

    registry.register(driver).unwrap();
    // Don't attach

    let mut items = std::collections::HashMap::new();
    items.insert(
        "CONNECTED".to_string(),
        PropertyItem::new(
            "CONNECTED",
            "Connected",
            PropertyValue::switch(SwitchState::On),
        ),
    );

    let property = Property {
        device: "Test Camera".to_string(),
        name: "CONNECTION".to_string(),
        group: "Main".to_string(),
        label: "Connection".to_string(),
        state: PropertyState::Ok,
        perm: PropertyPerm::ReadWrite,
        property_type: PropertyType::Switch,
        items,
        timeout: None,
        timestamp: None,
        message: None,
    };

    // Should fail because driver is not attached
    let result = registry
        .handle_property_change("Test Camera", &property)
        .await;
    assert!(result.is_err());
}
