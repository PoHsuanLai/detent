//! Reading the network service: the switch, the access points of the wireless device and the
//! networks with a kept password. Every read runs under one deadline, so a service that hangs
//! mid-list costs one failed read.

use super::nm::{
    AccessPointProxy, DeviceProxy, ManagerProxy, SERVICE, SavedConnectionProxy,
    SavedConnectionsProxy, WirelessProxy,
};
use crate::deadline::{Deadline, peer};
use crate::live::BusSource;
use crate::system::{is_running, system_connection};
use detent_model::{
    AccessPoint, Missing, NetworkLink, NetworkLock, SignalStrength, Ssid, WifiRadio, WifiStatus,
    WifiWire,
};
use ds_peer::within;
use zbus::Connection;
use zbus::zvariant::OwnedObjectPath;

/// The most access points listed: a crowded building shows a few hundred, and each costs bus calls.
const MOST_ACCESS_POINTS: usize = 128;

/// `NM_DEVICE_TYPE_WIFI`.
const WIFI_DEVICE: u32 = 2;

fn failed<E>(_: E) -> Missing {
    Missing::Failed
}

/// The switch, the networks in range and the ones with a kept password.
pub(super) async fn read(bus: &BusSource) -> Result<WifiStatus, Missing> {
    let connection = system_connection(bus).await?;
    if !is_running(&connection, SERVICE).await {
        return Err(Missing::NoService);
    }
    match within(Deadline::READ, peer::NETWORK, wire_of(&connection)).await {
        Ok(wire) => wire.map(WifiStatus::of_wire),
        Err(_) => Err(Missing::Failed),
    }
}

async fn wire_of(connection: &Connection) -> Result<WifiWire, Missing> {
    let manager = ManagerProxy::new(connection).await.map_err(failed)?;
    let radio = radio_of(&manager).await;
    let device = wifi_device(connection, &manager).await?;
    Ok(WifiWire {
        radio,
        access_points: access_points(connection, &device).await?,
        saved: saved(connection)
            .await
            .into_iter()
            .map(|(_, ssid)| ssid)
            .collect(),
    })
}

async fn radio_of(manager: &ManagerProxy<'_>) -> WifiRadio {
    let enabled = manager.wireless_enabled().await.unwrap_or(false);
    let hardware = manager.wireless_hardware_enabled().await.unwrap_or(true);
    match (hardware, enabled) {
        (false, _) => WifiRadio::Blocked,
        (true, true) => WifiRadio::On,
        (true, false) => WifiRadio::Off,
    }
}

/// The first wireless device, or `NoDevice` for a computer without one.
pub(super) async fn wifi_device(
    connection: &Connection,
    manager: &ManagerProxy<'_>,
) -> Result<OwnedObjectPath, Missing> {
    for path in manager.get_all_devices().await.map_err(failed)? {
        let device = device_at(connection, &path).await?;
        if device.device_type().await.map_err(failed)? == WIFI_DEVICE {
            return Ok(path);
        }
    }
    Err(Missing::NoDevice)
}

pub(super) async fn device_at<'a>(
    connection: &'a Connection,
    path: &OwnedObjectPath,
) -> Result<DeviceProxy<'a>, Missing> {
    DeviceProxy::builder(connection)
        .path(path.clone())
        .map_err(failed)?
        .build()
        .await
        .map_err(failed)
}

pub(super) async fn wireless_at<'a>(
    connection: &'a Connection,
    path: &OwnedObjectPath,
) -> Result<WirelessProxy<'a>, Missing> {
    WirelessProxy::builder(connection)
        .path(path.clone())
        .map_err(failed)?
        .build()
        .await
        .map_err(failed)
}

async fn access_points(
    connection: &Connection,
    device_path: &OwnedObjectPath,
) -> Result<Vec<AccessPoint>, Missing> {
    let wireless = wireless_at(connection, device_path).await?;
    let state = device_at(connection, device_path)
        .await?
        .state()
        .await
        .unwrap_or(0);
    let active = wireless.active_access_point().await.ok();
    let paths = wireless.access_points().await.map_err(failed)?;
    let mut points = Vec::new();
    for path in paths.into_iter().take(MOST_ACCESS_POINTS) {
        let link = if active.as_ref() == Some(&path) {
            link_of(state)
        } else {
            NetworkLink::Idle
        };
        if let Some(point) = access_point(connection, &path, link).await {
            points.push(point);
        }
    }
    Ok(points)
}

/// What the device's state says of the access point it is on: `NM_DEVICE_STATE_ACTIVATED` is
/// joined, the states from preparing to the last check are joining.
fn link_of(state: u32) -> NetworkLink {
    match state {
        100 => NetworkLink::Joined,
        40..=90 => NetworkLink::Joining,
        _ => NetworkLink::Idle,
    }
}

/// `Flags`' privacy bit.
const PRIVACY: u32 = 0x1;

/// An access point with no privacy flag and no protection flags needs no secret to join.
fn lock_of(flags: u32, wpa: u32, rsn: u32) -> NetworkLock {
    if flags & PRIVACY != 0 || wpa != 0 || rsn != 0 {
        NetworkLock::Secured
    } else {
        NetworkLock::Open
    }
}

pub(super) async fn access_point_at<'a>(
    connection: &'a Connection,
    path: &OwnedObjectPath,
) -> Option<AccessPointProxy<'a>> {
    AccessPointProxy::builder(connection)
        .path(path.clone())
        .ok()?
        .build()
        .await
        .ok()
}

async fn access_point(
    connection: &Connection,
    path: &OwnedObjectPath,
    link: NetworkLink,
) -> Option<AccessPoint> {
    let point = access_point_at(connection, path).await?;
    let ssid = point.ssid().await.unwrap_or_default();
    Some(AccessPoint {
        ssid: Ssid(String::from_utf8_lossy(&ssid).into_owned()),
        strength: SignalStrength(point.strength().await.unwrap_or(0).min(100)),
        lock: lock_of(
            point.flags().await.unwrap_or(0),
            point.wpa_flags().await.unwrap_or(0),
            point.rsn_flags().await.unwrap_or(0),
        ),
        link,
    })
}

/// The saved connections that name a wireless network, with that network.
pub(super) async fn saved(connection: &Connection) -> Vec<(OwnedObjectPath, Ssid)> {
    let Ok(connections) = SavedConnectionsProxy::new(connection).await else {
        return Vec::new();
    };
    let Ok(paths) = connections.connections().await else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for path in paths {
        if let Some(ssid) = saved_ssid(connection, &path).await {
            found.push((path, ssid));
        }
    }
    found
}

/// A saved connection's wireless network name; a wired or VPN connection has none.
async fn saved_ssid(connection: &Connection, path: &OwnedObjectPath) -> Option<Ssid> {
    let saved = SavedConnectionProxy::builder(connection)
        .path(path.clone())
        .ok()?
        .build()
        .await
        .ok()?;
    let mut all = saved.get_settings().await.ok()?;
    let ssid = all.remove("802-11-wireless")?.remove("ssid")?;
    let bytes = Vec::<u8>::try_from(ssid).ok()?;
    Some(Ssid(String::from_utf8_lossy(&bytes).into_owned()))
}

#[cfg(test)]
mod tests {
    use super::{link_of, lock_of};
    use detent_model::{NetworkLink, NetworkLock};

    #[test]
    fn a_network_is_locked_by_a_privacy_or_protection_flag() {
        const CASES: &[(&str, u32, u32, u32, NetworkLock)] = &[
            ("open", 0, 0, 0, NetworkLock::Open),
            ("privacy", 1, 0, 0, NetworkLock::Secured),
            ("wpa", 0, 0x100, 0, NetworkLock::Secured),
            ("rsn", 0, 0, 0x200, NetworkLock::Secured),
        ];
        for (case, flags, wpa, rsn, want) in CASES {
            assert_eq!(lock_of(*flags, *wpa, *rsn), *want, "{case}");
        }
    }

    #[test]
    fn the_device_state_says_how_joined_the_active_network_is() {
        const CASES: &[(u32, NetworkLink)] = &[
            (30, NetworkLink::Idle),
            (40, NetworkLink::Joining),
            (60, NetworkLink::Joining),
            (90, NetworkLink::Joining),
            (100, NetworkLink::Joined),
            (120, NetworkLink::Idle),
        ];
        for (state, want) in CASES {
            assert_eq!(link_of(*state), *want, "{state}");
        }
    }
}
