//! The network service's objects as this page calls them: the manager, a wireless device, an
//! access point and the saved connections.

use std::collections::HashMap;
use zbus::proxy;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

/// The name the network service owns.
pub(super) const SERVICE: &str = "org.freedesktop.NetworkManager";

/// What the manager is asked to add: setting groups, each a map of names to values.
pub(super) type Settings = HashMap<String, HashMap<String, Value<'static>>>;

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub(super) trait Manager {
    fn get_all_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    fn activate_connection(
        &self,
        connection: &ObjectPath<'_>,
        device: &ObjectPath<'_>,
        specific_object: &ObjectPath<'_>,
    ) -> zbus::Result<OwnedObjectPath>;

    fn add_and_activate_connection(
        &self,
        connection: Settings,
        device: &ObjectPath<'_>,
        specific_object: &ObjectPath<'_>,
    ) -> zbus::Result<(OwnedObjectPath, OwnedObjectPath)>;

    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_wireless_enabled(&self, value: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn wireless_hardware_enabled(&self) -> zbus::Result<bool>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
pub(super) trait Device {
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    /// Named apart from the `State` property, whose change stream would take the same name.
    #[zbus(signal, name = "StateChanged")]
    fn state_change(&self, new_state: u32, old_state: u32, reason: u32) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
pub(super) trait Wireless {
    fn request_scan(&self, options: HashMap<String, Value<'_>>) -> zbus::Result<()>;

    #[zbus(property)]
    fn access_points(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    #[zbus(property)]
    fn active_access_point(&self) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
pub(super) trait AccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;

    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;

    #[zbus(property)]
    fn flags(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn wpa_flags(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn rsn_flags(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
pub(super) trait SavedConnections {
    #[zbus(property)]
    fn connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    default_service = "org.freedesktop.NetworkManager"
)]
pub(super) trait SavedConnection {
    fn get_settings(&self) -> zbus::Result<HashMap<String, HashMap<String, OwnedValue>>>;

    fn delete(&self) -> zbus::Result<()>;
}
