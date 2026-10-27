//! Joining a network: a saved one is activated, a new one is added with the password the person
//! gave and activated, and the join ends when the wireless device is on the network or has
//! failed. A password the network refuses is a failure the page asks again about, and the
//! connection that held it is not kept.

use super::known::delete_saved;
use super::profile::connection_of;
use super::read::{access_point_at, device_at, saved, wifi_device, wireless_at};
use crate::bus::{BusSource, next, system_connection};
use crate::deadline::{Deadline, peer};
use airwaves_nm::NetworkManagerProxy;
use detent_model::{
    DeviceProgress, HiddenLogin, HiddenNetwork, JoinFailure, JoinSecret, JoinVerdict, Passphrase,
    Ssid, WifiLogin, WifiSecurity, WifiVisibility, join_verdict, wifi_security_of,
};
use ds_peer::within;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

const NOT_AVAILABLE: &str = "Wi-Fi isn\u{2019}t available right now. Try again in a moment.";
const TOO_LONG: &str = "Joining took too long. Try again in a moment.";

fn told(words: impl Into<String>) -> JoinFailure {
    JoinFailure::Told(words.into())
}

/// The failure for a service that cannot be reached, whatever the reason.
fn unavailable<E>(_: E) -> JoinFailure {
    told(NOT_AVAILABLE)
}

fn could_not(ssid: &Ssid) -> JoinFailure {
    told(format!(
        "Couldn\u{2019}t join \u{201c}{}\u{201d}. Try again in a moment.",
        ssid.0
    ))
}

/// What a join is for: the network, what the person gave, and where to find the network.
struct Attempt {
    ssid: Ssid,
    secret: JoinSecret,
    reach: Reach,
}

/// Whether the network is in the list of access points, or only named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reach {
    /// Its access points are in range; they say how it is secured.
    InRange,
    /// It does not announce itself; the person said how it is secured.
    Hidden(WifiSecurity),
}

impl Reach {
    fn visibility(self) -> WifiVisibility {
        match self {
            Reach::InRange => WifiVisibility::Announced,
            Reach::Hidden(_) => WifiVisibility::Hidden,
        }
    }
}

/// Joins `ssid` and answers when the device is on it or the join has failed.
pub async fn join(
    bus: &BusSource,
    ssid: Ssid,
    passphrase: Option<Passphrase>,
) -> Result<(), JoinFailure> {
    let secret = JoinSecret::of(passphrase);
    run(
        bus,
        Attempt {
            ssid,
            secret,
            reach: Reach::InRange,
        },
    )
    .await
}

/// Joins `ssid`, a network in range that signs in with an account.
pub async fn sign_in(bus: &BusSource, ssid: Ssid, account: WifiLogin) -> Result<(), JoinFailure> {
    run(
        bus,
        Attempt {
            ssid,
            secret: JoinSecret::Account(account),
            reach: Reach::InRange,
        },
    )
    .await
}

/// Joins a network that does not announce its name, secured the way the person said.
pub async fn join_hidden(bus: &BusSource, network: HiddenNetwork) -> Result<(), JoinFailure> {
    let (secret, security) = match network.login {
        HiddenLogin::Open => (JoinSecret::Kept, WifiSecurity::Open),
        HiddenLogin::Personal(passphrase) => (JoinSecret::Password(passphrase), WifiSecurity::Psk),
        HiddenLogin::PersonalWpa3(passphrase) => {
            (JoinSecret::Password(passphrase), WifiSecurity::Sae)
        }
        HiddenLogin::Enterprise(account) => (JoinSecret::Account(account), WifiSecurity::Eap),
    };
    run(
        bus,
        Attempt {
            ssid: network.ssid,
            secret,
            reach: Reach::Hidden(security),
        },
    )
    .await
}

/// Joins as `attempt` says, within the deadline for an action.
async fn run(bus: &BusSource, attempt: Attempt) -> Result<(), JoinFailure> {
    let connection = system_connection(bus).await.map_err(unavailable)?;
    match within(
        Deadline::ACTION,
        peer::NETWORK,
        joining(&connection, &attempt),
    )
    .await
    {
        Ok(done) => done,
        Err(_) => Err(told(TOO_LONG)),
    }
}

async fn joining(connection: &Connection, attempt: &Attempt) -> Result<(), JoinFailure> {
    let ssid = &attempt.ssid;
    let manager = NetworkManagerProxy::new(connection)
        .await
        .map_err(unavailable)?;
    let device_path = wifi_device(connection, &manager)
        .await
        .map_err(unavailable)?;
    let device = device_at(connection, &device_path)
        .await
        .map_err(unavailable)?;
    // Subscribed before the join starts, so no state falls between starting and listening.
    let mut changes = device.receive_state_change().await.map_err(unavailable)?;
    let access = match attempt.reach {
        Reach::InRange => access_for(connection, &device_path, ssid).await,
        Reach::Hidden(_) => None,
    };
    let security = match attempt.reach {
        Reach::Hidden(security) => security,
        Reach::InRange => access
            .as_ref()
            .map_or(WifiSecurity::Open, |(_, security)| *security),
    };
    let added = start(
        connection,
        &manager,
        &device_path,
        attempt,
        access.map(|(path, _)| path),
        security,
    )
    .await?;
    let mut progress = DeviceProgress::NotStarted;
    let ended = loop {
        let Some(change) = next(&mut changes).await else {
            break Err(could_not(ssid));
        };
        let Ok(args) = change.args() else { continue };
        match join_verdict(*args.new_state(), *args.reason(), progress, security) {
            JoinVerdict::Done => break Ok(()),
            JoinVerdict::WrongPassword => break Err(JoinFailure::WrongPassword),
            JoinVerdict::CouldNot => break Err(could_not(ssid)),
            JoinVerdict::Going => progress = DeviceProgress::Started,
            JoinVerdict::Waiting => {}
        }
    };
    if let (Err(_), Some(path)) = (&ended, added) {
        delete_saved(connection, &path).await;
    }
    ended
}

/// The access point of `ssid` with the best signal, and how it is secured.
async fn access_for(
    connection: &Connection,
    device_path: &OwnedObjectPath,
    ssid: &Ssid,
) -> Option<(OwnedObjectPath, WifiSecurity)> {
    let wireless = wireless_at(connection, device_path).await.ok()?;
    let mut best: Option<(u8, OwnedObjectPath, WifiSecurity)> = None;
    for path in wireless.access_points().await.ok()? {
        let Some(point) = access_point_at(connection, &path).await else {
            continue;
        };
        if point.ssid().await.ok()? != ssid.0.as_bytes() {
            continue;
        }
        let strength = point.strength().await.unwrap_or(0);
        if best.as_ref().is_some_and(|(seen, _, _)| *seen >= strength) {
            continue;
        }
        let security = wifi_security_of(
            point.flags().await.unwrap_or(0),
            point.wpa_flags().await.unwrap_or(0),
            point.rsn_flags().await.unwrap_or(0),
        );
        best = Some((strength, path, security));
    }
    best.map(|(_, path, security)| (path, security))
}

/// Starts the join; the path of the connection it made, when it made one.
async fn start(
    connection: &Connection,
    manager: &NetworkManagerProxy<'_>,
    device_path: &OwnedObjectPath,
    attempt: &Attempt,
    access: Option<OwnedObjectPath>,
    security: WifiSecurity,
) -> Result<Option<OwnedObjectPath>, JoinFailure> {
    let ssid = &attempt.ssid;
    let remembered: Vec<OwnedObjectPath> = saved(connection)
        .await
        .into_iter()
        .filter(|(_, saved_ssid)| saved_ssid == ssid)
        .map(|(path, _)| path)
        .collect();
    let device = ObjectPath::from(device_path);
    let none = ObjectPath::default();
    if !attempt.secret.is_given()
        && let Some(path) = remembered.first()
    {
        manager
            .activate_connection(&ObjectPath::from(path), &device, &none)
            .await
            .map_err(|_| could_not(ssid))?;
        return Ok(None);
    }
    if security == WifiSecurity::Unsupported {
        return Err(told(format!(
            "\u{201c}{}\u{201d} needs a sign-in this computer can\u{2019}t do yet.",
            ssid.0
        )));
    }
    if attempt.secret.is_given() {
        for old in &remembered {
            delete_saved(connection, old).await;
        }
    }
    let specific = access.as_ref().map(ObjectPath::from).unwrap_or_default();
    let settings = connection_of(ssid, security, &attempt.secret, attempt.reach.visibility());
    let (added, _active) = manager
        .add_and_activate_connection(settings, &device, &specific)
        .await
        .map_err(|_| could_not(ssid))?;
    Ok(Some(added))
}
