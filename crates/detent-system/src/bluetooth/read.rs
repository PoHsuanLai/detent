//! Reading BlueZ: one `GetManagedObjects` answer holds the adapter, every device and each one's
//! properties, so a read is one moment.

use crate::bus::is_running;
use crate::deadline::{Deadline, peer};
use airwaves_bluez::{Objects, SERVICE};
use detent_model::{
    Absence, BtAddress, BtDeviceWire, BtLink, BtStatus, bt_device_of, is_under_adapter,
};
use ds_peer::within;
use std::collections::HashMap;
use zbus::Connection;
use zbus::fdo::ObjectManagerProxy;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

const ADAPTER: &str = "org.bluez.Adapter1";
const DEVICE: &str = "org.bluez.Device1";
const BATTERY: &str = "org.bluez.Battery1";

type Properties = HashMap<String, OwnedValue>;

/// Every object BlueZ has, or why it cannot be read.
pub async fn managed(connection: &Connection) -> Result<Objects, Absence> {
    if !is_running(connection, SERVICE).await {
        return Err(Absence::NotRunning);
    }
    let asked = async {
        ObjectManagerProxy::builder(connection)
            .destination(SERVICE)?
            .path("/")?
            .build()
            .await?
            .get_managed_objects()
            .await
    };
    match within(Deadline::READ, peer::BLUETOOTH, asked).await {
        Ok(Ok(objects)) => Ok(objects),
        Ok(Err(_)) | Err(_) => Err(Absence::Failed),
    }
}

fn properties_of<'a>(
    interfaces: &'a HashMap<zbus::names::OwnedInterfaceName, Properties>,
    interface: &str,
) -> Option<&'a Properties> {
    interfaces
        .iter()
        .find(|(name, _)| name.as_str() == interface)
        .map(|(_, properties)| properties)
}

fn text(properties: &Properties, name: &str) -> Option<String> {
    <&str>::try_from(properties.get(name)?)
        .ok()
        .map(str::to_owned)
}

fn flag(properties: &Properties, name: &str) -> bool {
    properties
        .get(name)
        .and_then(|value| bool::try_from(value).ok())
        .unwrap_or(false)
}

/// The adapter's path and its properties.
fn adapter(objects: &Objects) -> Option<(&OwnedObjectPath, &Properties)> {
    objects.iter().find_map(|(path, interfaces)| {
        properties_of(interfaces, ADAPTER).map(|properties| (path, properties))
    })
}

/// The adapter's path.
pub fn adapter_path(objects: &Objects) -> Option<OwnedObjectPath> {
    adapter(objects).map(|(path, _)| path.clone())
}

fn under(path: &OwnedObjectPath, adapter: &OwnedObjectPath) -> bool {
    is_under_adapter(path.as_str(), adapter.as_str())
}

/// The status `objects` describe. A device this computer is not paired with is listed only while
/// BlueZ hears it (it has a signal strength): BlueZ remembers the ones it saw for a while.
pub fn status_of(objects: &Objects) -> Result<BtStatus, Absence> {
    let (adapter_path, adapter) = adapter(objects).ok_or(Absence::NoDevice)?;
    let powered = flag(adapter, "Powered");
    let devices = objects
        .iter()
        .filter(|(path, _)| under(path, adapter_path))
        .filter_map(|(_, interfaces)| {
            let device = properties_of(interfaces, DEVICE)?;
            bt_device_of(wire_of(device, properties_of(interfaces, BATTERY)))
        })
        .collect();
    Ok(BtStatus::new(powered, devices))
}

/// The battery `Percentage` the device's `Battery1` interface reports, when it has one.
fn battery_of(battery: Option<&Properties>) -> Option<u8> {
    u8::try_from(battery?.get("Percentage")?).ok()
}

/// What BlueZ's properties say of one device, as plain values.
fn wire_of(properties: &Properties, battery: Option<&Properties>) -> BtDeviceWire {
    BtDeviceWire {
        address: text(properties, "Address"),
        alias: text(properties, "Alias"),
        icon: text(properties, "Icon"),
        paired: flag(properties, "Paired"),
        link: if flag(properties, "Connected") {
            BtLink::Connected
        } else {
            BtLink::Disconnected
        },
        heard: properties.contains_key("RSSI"),
        battery: battery_of(battery),
    }
}

/// The adapter and the device at `address`, and the device's name.
pub fn find_device(
    objects: &Objects,
    address: &BtAddress,
) -> Option<(OwnedObjectPath, OwnedObjectPath, String)> {
    let adapter = adapter_path(objects)?;
    objects.iter().find_map(|(path, interfaces)| {
        let properties = properties_of(interfaces, DEVICE)?;
        if !under(path, &adapter) || text(properties, "Address")? != address.0 {
            return None;
        }
        let name = text(properties, "Alias").unwrap_or_else(|| address.0.clone());
        Some((adapter.clone(), path.clone(), name))
    })
}

#[cfg(test)]
mod tests {
    use super::{Objects, find_device, status_of};
    use detent_model::{Absence, BtAddress, BtLink};
    use std::collections::HashMap;
    use zbus::names::OwnedInterfaceName;
    use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

    fn value(value: impl Into<Value<'static>>) -> OwnedValue {
        OwnedValue::try_from(value.into()).expect("a plain value")
    }

    fn path(text: &str) -> OwnedObjectPath {
        OwnedObjectPath::try_from(text).expect("a path")
    }

    fn object(
        interface: &str,
        properties: Vec<(&str, OwnedValue)>,
    ) -> HashMap<OwnedInterfaceName, HashMap<String, OwnedValue>> {
        HashMap::from([(
            OwnedInterfaceName::try_from(interface).expect("an interface"),
            properties
                .into_iter()
                .map(|(name, v)| (name.to_owned(), v))
                .collect(),
        )])
    }

    /// A device as BlueZ reports it.
    struct Seen<'a> {
        address: &'a str,
        alias: &'a str,
        paired: bool,
        link: BtLink,
        heard: bool,
    }

    impl Seen<'_> {
        fn object(&self) -> HashMap<OwnedInterfaceName, HashMap<String, OwnedValue>> {
            let mut properties = vec![
                ("Address", value(self.address.to_owned())),
                ("Alias", value(self.alias.to_owned())),
                ("Icon", value("input-mouse".to_owned())),
                ("Paired", value(self.paired)),
                ("Connected", value(self.link == BtLink::Connected)),
            ];
            if self.heard {
                properties.push(("RSSI", value(-60_i16)));
            }
            object("org.bluez.Device1", properties)
        }
    }

    fn world(powered: bool) -> Objects {
        let seen = |address, alias, paired, link, heard| {
            Seen {
                address,
                alias,
                paired,
                link,
                heard,
            }
            .object()
        };
        HashMap::from([
            (
                path("/org/bluez/hci0"),
                object("org.bluez.Adapter1", vec![("Powered", value(powered))]),
            ),
            (
                path("/org/bluez/hci0/dev_A"),
                seen("AA:00", "Mouse", true, BtLink::Connected, false),
            ),
            (
                path("/org/bluez/hci0/dev_B"),
                seen("BB:00", "Seen Pad", false, BtLink::Disconnected, true),
            ),
            (
                path("/org/bluez/hci0/dev_C"),
                seen("CC:00", "Gone Pad", false, BtLink::Disconnected, false),
            ),
            (
                path("/org/bluez/hci1/dev_D"),
                seen("DD:00", "Other Adapter", true, BtLink::Disconnected, false),
            ),
        ])
    }

    #[test]
    fn the_adapter_and_the_devices_under_it_make_the_status() {
        let status = status_of(&world(true)).expect("a status");
        assert!(status.powered);
        let listed: Vec<(&str, bool, BtLink)> = status
            .devices
            .iter()
            .map(|d| (d.name.as_str(), d.paired, d.link))
            .collect();
        assert_eq!(
            listed,
            [
                ("Mouse", true, BtLink::Connected),
                ("Seen Pad", false, BtLink::Disconnected),
            ],
            "a device nobody hears and one under another adapter are not listed"
        );
        assert!(!status_of(&world(false)).expect("a status").powered);
    }

    #[test]
    fn no_adapter_is_no_bluetooth_on_this_computer() {
        assert_eq!(status_of(&HashMap::new()), Err(Absence::NoDevice));
    }

    #[test]
    fn a_device_is_found_by_its_address_with_its_adapter_and_name() {
        let objects = world(true);
        let found = find_device(&objects, &BtAddress("BB:00".to_owned()));
        assert_eq!(
            found,
            Some((
                path("/org/bluez/hci0"),
                path("/org/bluez/hci0/dev_B"),
                "Seen Pad".to_owned()
            ))
        );
        assert_eq!(find_device(&objects, &BtAddress("DD:00".to_owned())), None);
    }
}
