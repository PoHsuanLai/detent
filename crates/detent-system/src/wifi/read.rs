//! Reading the network service: the switch, the access points of the wireless device and the
//! networks with a kept password. Every read runs under one deadline, so a service that hangs
//! mid-list costs one failed read.

use crate::bus::{BusSource, is_running, system_connection};
use crate::deadline::{Deadline, peer};
use airwaves_nm::{
    AccessPointProxy, ConnectionSettingsProxy, DeviceProxy, NetworkManagerProxy, SERVICE,
    SettingsProxy, WirelessDeviceProxy,
};
use detent_model::{
    Absence, AccessPoint, NetworkLink, SignalStrength, Ssid, WifiRadio, WifiStatus, WifiWire,
    wifi_link_of, wifi_security_of,
};
use ds_peer::within;
use zbus::Connection;
use zbus::zvariant::OwnedObjectPath;

/// The most access points listed: a crowded building shows a few hundred, and each costs bus calls.
const MOST_ACCESS_POINTS: usize = 128;

/// `NM_DEVICE_TYPE_WIFI`.
const WIFI_DEVICE: u32 = 2;

fn failed<E>(_: E) -> Absence {
    Absence::Failed
}

/// The switch, the networks in range and the ones with a kept password.
pub async fn read(bus: &BusSource) -> Result<WifiStatus, Absence> {
    let connection = system_connection(bus).await?;
    if !is_running(&connection, SERVICE).await {
        return Err(Absence::NotRunning);
    }
    match within(Deadline::READ, peer::NETWORK, wire_of(&connection)).await {
        Ok(wire) => wire.map(WifiStatus::of_wire),
        Err(_) => Err(Absence::Failed),
    }
}

async fn wire_of(connection: &Connection) -> Result<WifiWire, Absence> {
    let manager = NetworkManagerProxy::new(connection).await.map_err(failed)?;
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

async fn radio_of(manager: &NetworkManagerProxy<'_>) -> WifiRadio {
    let enabled = manager.wireless_enabled().await.unwrap_or(false);
    let hardware = manager.wireless_hardware_enabled().await.unwrap_or(true);
    match (hardware, enabled) {
        (false, _) => WifiRadio::Blocked,
        (true, true) => WifiRadio::On,
        (true, false) => WifiRadio::Off,
    }
}

/// The first wireless device, or `NoDevice` for a computer without one.
pub async fn wifi_device(
    connection: &Connection,
    manager: &NetworkManagerProxy<'_>,
) -> Result<OwnedObjectPath, Absence> {
    for path in manager.get_all_devices().await.map_err(failed)? {
        let device = device_at(connection, &path).await?;
        if device.device_type().await.map_err(failed)? == WIFI_DEVICE {
            return Ok(path);
        }
    }
    Err(Absence::NoDevice)
}

pub async fn device_at<'a>(
    connection: &'a Connection,
    path: &OwnedObjectPath,
) -> Result<DeviceProxy<'a>, Absence> {
    DeviceProxy::builder(connection)
        .path(path.clone())
        .map_err(failed)?
        .build()
        .await
        .map_err(failed)
}

pub async fn wireless_at<'a>(
    connection: &'a Connection,
    path: &OwnedObjectPath,
) -> Result<WirelessDeviceProxy<'a>, Absence> {
    WirelessDeviceProxy::builder(connection)
        .path(path.clone())
        .map_err(failed)?
        .build()
        .await
        .map_err(failed)
}

async fn access_points(
    connection: &Connection,
    device_path: &OwnedObjectPath,
) -> Result<Vec<AccessPoint>, Absence> {
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
            wifi_link_of(state)
        } else {
            NetworkLink::Idle
        };
        if let Some(point) = access_point(connection, &path, link).await {
            points.push(point);
        }
    }
    Ok(points)
}

pub async fn access_point_at<'a>(
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
        security: wifi_security_of(
            point.flags().await.unwrap_or(0),
            point.wpa_flags().await.unwrap_or(0),
            point.rsn_flags().await.unwrap_or(0),
        ),
        link,
    })
}

/// The saved connections that name a wireless network, with that network.
pub async fn saved(connection: &Connection) -> Vec<(OwnedObjectPath, Ssid)> {
    let Ok(connections) = SettingsProxy::new(connection).await else {
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
    let saved = ConnectionSettingsProxy::builder(connection)
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
