//! Reading BlueZ: one `GetManagedObjects` answer holds the adapter, every device and each one's
//! properties, so a read is one moment.

use super::bluez::{Objects, SERVICE};
use crate::deadline::{Deadline, peer};
use crate::system::is_running;
use detent_model::{
    BatteryLevel, BtAddress, BtDevice, BtKind, BtLink, BtPairing, BtPower, BtStatus, Missing,
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
pub(super) async fn managed(connection: &Connection) -> Result<Objects, Missing> {
    if !is_running(connection, SERVICE).await {
        return Err(Missing::NoService);
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
        Ok(Err(_)) | Err(_) => Err(Missing::Failed),
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
pub(super) fn adapter_path(objects: &Objects) -> Option<OwnedObjectPath> {
    adapter(objects).map(|(path, _)| path.clone())
}

/// Whether `path` sits under `adapter` (`/org/bluez/hci0/dev_..` under `/org/bluez/hci0`).
fn under(path: &OwnedObjectPath, adapter: &OwnedObjectPath) -> bool {
    path.as_str()
        .strip_prefix(adapter.as_str())
        .is_some_and(|rest| rest.starts_with('/'))
}

/// The status `objects` describe. A device this computer is not paired with is listed only while
/// BlueZ hears it (it has a signal strength): BlueZ remembers the ones it saw for a while.
pub(super) fn status_of(objects: &Objects) -> Result<BtStatus, Missing> {
    let (adapter_path, adapter) = adapter(objects).ok_or(Missing::NoDevice)?;
    let power = if flag(adapter, "Powered") {
        BtPower::On
    } else {
        BtPower::Off
    };
    let devices = objects
        .iter()
        .filter(|(path, _)| under(path, adapter_path))
        .filter_map(|(_, interfaces)| {
            let device = properties_of(interfaces, DEVICE)?;
            device_of(device, properties_of(interfaces, BATTERY))
        })
        .collect();
    Ok(BtStatus::new(power, devices))
}

/// The battery `Percentage` the device's `Battery1` interface reports, when it has one.
fn battery_of(battery: Option<&Properties>) -> Option<BatteryLevel> {
    let percentage = battery?.get("Percentage")?;
    u8::try_from(percentage).ok().map(BatteryLevel)
}

fn device_of(properties: &Properties, battery: Option<&Properties>) -> Option<BtDevice> {
    let address = text(properties, "Address")?;
    let pairing = if flag(properties, "Paired") {
        BtPairing::Paired
    } else {
        BtPairing::Unpaired
    };
    if pairing == BtPairing::Unpaired && !properties.contains_key("RSSI") {
        return None;
    }
    let name = text(properties, "Alias")
        .filter(|alias| !alias.is_empty())
        .unwrap_or_else(|| address.clone());
    Some(BtDevice {
        address: BtAddress(address),
        name,
        kind: BtKind::of_icon(&text(properties, "Icon").unwrap_or_default()),
        link: if flag(properties, "Connected") {
            BtLink::Connected
        } else {
            BtLink::Disconnected
        },
        pairing,
        battery: battery_of(battery),
    })
}

/// The adapter and the device at `address`, and the device's name.
pub(super) fn find_device(
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
    use detent_model::{BtAddress, BtLink, BtPairing, BtPower, Missing};
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

    /// Whether BlueZ hears a device now (it then reports a signal strength).
    #[derive(Clone, Copy)]
    enum Hearing {
        Heard,
        Silent,
    }

    /// A device as BlueZ reports it.
    struct Seen<'a> {
        address: &'a str,
        alias: &'a str,
        pairing: BtPairing,
        link: BtLink,
        hearing: Hearing,
    }

    impl Seen<'_> {
        fn object(&self) -> HashMap<OwnedInterfaceName, HashMap<String, OwnedValue>> {
            let mut properties = vec![
                ("Address", value(self.address.to_owned())),
                ("Alias", value(self.alias.to_owned())),
                ("Icon", value("input-mouse".to_owned())),
                ("Paired", value(self.pairing == BtPairing::Paired)),
                ("Connected", value(self.link == BtLink::Connected)),
            ];
            if matches!(self.hearing, Hearing::Heard) {
                properties.push(("RSSI", value(-60_i16)));
            }
            object("org.bluez.Device1", properties)
        }
    }

    fn world(power: BtPower) -> Objects {
        let seen = |address, alias, pairing, link, hearing| {
            Seen {
                address,
                alias,
                pairing,
                link,
                hearing,
            }
            .object()
        };
        HashMap::from([
            (
                path("/org/bluez/hci0"),
                object(
                    "org.bluez.Adapter1",
                    vec![("Powered", value(power == BtPower::On))],
                ),
            ),
            (
                path("/org/bluez/hci0/dev_A"),
                seen(
                    "AA:00",
                    "Mouse",
                    BtPairing::Paired,
                    BtLink::Connected,
                    Hearing::Silent,
                ),
            ),
            (
                path("/org/bluez/hci0/dev_B"),
                seen(
                    "BB:00",
                    "Seen Pad",
                    BtPairing::Unpaired,
                    BtLink::Disconnected,
                    Hearing::Heard,
                ),
            ),
            (
                path("/org/bluez/hci0/dev_C"),
                seen(
                    "CC:00",
                    "Gone Pad",
                    BtPairing::Unpaired,
                    BtLink::Disconnected,
                    Hearing::Silent,
                ),
            ),
            (
                path("/org/bluez/hci1/dev_D"),
                seen(
                    "DD:00",
                    "Other Adapter",
                    BtPairing::Paired,
                    BtLink::Disconnected,
                    Hearing::Silent,
                ),
            ),
        ])
    }

    #[test]
    fn the_adapter_and_the_devices_under_it_make_the_status() {
        let status = status_of(&world(BtPower::On)).expect("a status");
        assert_eq!(status.power, BtPower::On);
        let listed: Vec<(&str, BtPairing, BtLink)> = status
            .devices
            .iter()
            .map(|d| (d.name.as_str(), d.pairing, d.link))
            .collect();
        assert_eq!(
            listed,
            [
                ("Mouse", BtPairing::Paired, BtLink::Connected),
                ("Seen Pad", BtPairing::Unpaired, BtLink::Disconnected),
            ],
            "a device nobody hears and one under another adapter are not listed"
        );
        assert_eq!(
            status_of(&world(BtPower::Off)).expect("a status").power,
            BtPower::Off
        );
    }

    #[test]
    fn no_adapter_is_no_bluetooth_on_this_computer() {
        assert_eq!(status_of(&HashMap::new()), Err(Missing::NoDevice));
    }

    #[test]
    fn a_device_is_found_by_its_address_with_its_adapter_and_name() {
        let objects = world(BtPower::On);
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
