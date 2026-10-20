//! BlueZ's objects as this page calls them: the adapter, a device and the agent manager.

use std::collections::HashMap;
use zbus::proxy;
use zbus::zvariant::{ObjectPath, OwnedInterfaceName, OwnedObjectPath, OwnedValue};

/// The name BlueZ owns.
pub(super) const SERVICE: &str = "org.bluez";

/// What `GetManagedObjects` answers: every object, its interfaces and their properties.
pub(super) type Objects =
    HashMap<OwnedObjectPath, HashMap<OwnedInterfaceName, HashMap<String, OwnedValue>>>;

#[proxy(interface = "org.bluez.Adapter1", default_service = "org.bluez")]
pub(super) trait Adapter {
    fn start_discovery(&self) -> zbus::Result<()>;

    fn stop_discovery(&self) -> zbus::Result<()>;

    fn remove_device(&self, device: &ObjectPath<'_>) -> zbus::Result<()>;

    #[zbus(property)]
    fn set_powered(&self, value: bool) -> zbus::Result<()>;
}

#[proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
pub(super) trait Device {
    fn connect(&self) -> zbus::Result<()>;

    fn disconnect(&self) -> zbus::Result<()>;

    fn pair(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn alias(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn address(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn set_trusted(&self, value: bool) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.bluez.AgentManager1",
    default_service = "org.bluez",
    default_path = "/org/bluez"
)]
pub(super) trait AgentManager {
    fn register_agent(&self, agent: &ObjectPath<'_>, capability: &str) -> zbus::Result<()>;

    fn unregister_agent(&self, agent: &ObjectPath<'_>) -> zbus::Result<()>;
}
