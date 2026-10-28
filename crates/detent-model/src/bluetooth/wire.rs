//! BlueZ's objects as plain values, and what they say: which devices the page lists, and which of
//! BlueZ's signals are worth reading it again for. The system reads the wire into these values;
//! nothing here knows the bus.

use super::model::{BatteryLevel, BtAddress, BtDevice, BtKind, BtLink};

/// What BlueZ's `Device1` (and `Battery1`) properties say of one device, as the system read them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtDeviceWire {
    /// `Address`, when the object has one.
    pub address: Option<String>,
    /// `Alias`, when it has one.
    pub alias: Option<String>,
    /// `Icon`, when it has one.
    pub icon: Option<String>,
    /// `Paired`.
    pub paired: bool,
    /// `Connected`.
    pub link: BtLink,
    /// Whether BlueZ hears the device now (`RSSI` is there).
    pub heard: bool,
    /// The battery `Percentage`, when `Battery1` reports one.
    pub battery: Option<u8>,
}

/// The device `wire` describes. A device this computer is not paired with is listed only while
/// BlueZ hears it: BlueZ remembers the ones it saw for a while. A device with no address is none.
pub fn bt_device_of(wire: BtDeviceWire) -> Option<BtDevice> {
    let address = wire.address?;
    if !wire.paired && !wire.heard {
        return None;
    }
    let name = wire
        .alias
        .filter(|alias| !alias.is_empty())
        .unwrap_or_else(|| address.clone());
    Some(BtDevice {
        address: BtAddress(address),
        name,
        kind: BtKind::of_icon(&wire.icon.unwrap_or_default()),
        link: wire.link,
        paired: wire.paired,
        battery: wire.battery.map(BatteryLevel),
    })
}

/// Whether the object at `path` sits under the adapter at `adapter` (`/org/bluez/hci0/dev_..`
/// under `/org/bluez/hci0`).
pub fn is_under_adapter(path: &str, adapter: &str) -> bool {
    path.strip_prefix(adapter)
        .is_some_and(|rest| rest.starts_with('/'))
}

/// What BlueZ says about a device it hears all the time: its signal, and what it broadcasts.
const NOISE: [&str; 4] = ["RSSI", "TxPower", "ManufacturerData", "ServiceData"];

/// Whether a `PropertiesChanged` of `interface` that changed `names` is worth reading BlueZ again
/// for: not a device's constantly changing signal strength, which would have the page reading all
/// the time while devices are nearby.
pub fn bt_change_matters(interface: &str, names: &[&str]) -> bool {
    !(interface == "org.bluez.Device1" && names.iter().all(|name| NOISE.contains(name)))
}

#[cfg(test)]
mod tests {
    use super::{BtDeviceWire, bt_change_matters, bt_device_of, is_under_adapter};
    use crate::bluetooth::{BtKind, BtLink};

    fn wire(paired: bool, heard: bool) -> BtDeviceWire {
        BtDeviceWire {
            address: Some("AA:00".to_owned()),
            alias: Some("Mouse".to_owned()),
            icon: Some("input-mouse".to_owned()),
            paired,
            link: BtLink::Connected,
            heard,
            battery: Some(80),
        }
    }

    #[test]
    fn a_device_is_listed_when_paired_or_heard() {
        const CASES: &[(&str, bool, bool, bool)] = &[
            ("paired and silent", true, false, true),
            ("paired and heard", true, true, true),
            ("seen and heard", false, true, true),
            ("remembered only", false, false, false),
        ];
        for (case, paired, heard, listed) in CASES {
            assert_eq!(
                bt_device_of(wire(*paired, *heard)).is_some(),
                *listed,
                "{case}"
            );
        }
    }

    #[test]
    fn a_device_is_named_by_its_alias_then_its_address() {
        let named = bt_device_of(wire(true, false));
        assert_eq!(named.as_ref().map(|d| d.name.as_str()), Some("Mouse"));
        assert_eq!(named.as_ref().map(|d| d.kind), Some(BtKind::Mouse));
        let unnamed = BtDeviceWire {
            alias: Some(String::new()),
            ..wire(true, false)
        };
        assert_eq!(
            bt_device_of(unnamed).map(|d| d.name),
            Some("AA:00".to_owned())
        );
        let nameless = BtDeviceWire {
            address: None,
            ..wire(true, false)
        };
        assert_eq!(bt_device_of(nameless), None);
    }

    #[test]
    fn only_objects_below_the_adapter_belong_to_it() {
        const CASES: &[(&str, &str, bool)] = &[
            ("/org/bluez/hci0/dev_A", "/org/bluez/hci0", true),
            ("/org/bluez/hci1/dev_D", "/org/bluez/hci0", false),
            ("/org/bluez/hci0", "/org/bluez/hci0", false),
            ("/org/bluez/hci01/dev_A", "/org/bluez/hci0", false),
        ];
        for (path, adapter, under) in CASES {
            assert_eq!(is_under_adapter(path, adapter), *under, "{path}");
        }
    }

    #[test]
    fn a_devices_signal_strength_alone_is_not_worth_reading_again() {
        const CASES: &[(&str, &[&str], bool)] = &[
            ("org.bluez.Device1", &["RSSI"], false),
            ("org.bluez.Device1", &["RSSI", "TxPower"], false),
            ("org.bluez.Device1", &["RSSI", "Connected"], true),
            ("org.bluez.Adapter1", &["RSSI"], true),
        ];
        for (interface, names, matters) in CASES {
            assert_eq!(bt_change_matters(interface, names), *matters, "{interface}");
        }
    }
}
