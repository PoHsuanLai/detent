//! Joining a network: a saved one is activated, a new one is added with the password the person
//! gave and activated, and the join ends when the wireless device is on the network or has
//! failed. A password the network refuses is a failure the page asks again about, and the
//! connection that held it is not kept.

use super::nm::{ManagerProxy, SavedConnectionProxy, Settings};
use super::read::{access_point_at, device_at, saved, wifi_device, wireless_at};
use crate::deadline::{Deadline, peer};
use crate::live::BusSource;
use crate::system::{next, system_connection};
use detent_model::{JoinFailure, Passphrase, Ssid};
use ds_peer::within;
use std::collections::HashMap;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, Value};

const NOT_AVAILABLE: &str = "Wi-Fi isn\u{2019}t available right now. Try again in a moment.";
const TOO_LONG: &str = "Joining took too long. Try again in a moment.";

/// `NM_DEVICE_STATE_*` as a join passes through them.
const PREPARE: u32 = 40;
const SECONDARIES: u32 = 90;
const NEED_AUTH: u32 = 60;
const DISCONNECTED: u32 = 30;
const ACTIVATED: u32 = 100;
const FAILED: u32 = 120;

/// `NM_DEVICE_STATE_REASON_NO_SECRETS` to `_SUPPLICANT_TIMEOUT`: the network did not accept the
/// password.
const REFUSED_REASONS: std::ops::RangeInclusive<u32> = 7..=11;

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

/// How a network is secured, from its access point's flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Security {
    Open,
    Psk,
    Sae,
    /// A sign-in or an old scheme this page does not do.
    Unsupported,
}

/// The security `flags`, `wpa` and `rsn` of an access point describe.
pub(super) fn security_of(flags: u32, wpa: u32, rsn: u32) -> Security {
    const PRIVACY: u32 = 0x1;
    const PSK: u32 = 0x100;
    const ENTERPRISE: u32 = 0x200;
    const SAE: u32 = 0x400;
    let keys = wpa | rsn;
    if keys & ENTERPRISE != 0 && keys & (PSK | SAE) == 0 {
        Security::Unsupported
    } else if keys & PSK != 0 {
        Security::Psk
    } else if keys & SAE != 0 {
        Security::Sae
    } else if flags & PRIVACY != 0 || keys != 0 {
        Security::Unsupported
    } else {
        Security::Open
    }
}

/// What a new connection to `ssid` is made of.
pub(super) fn settings_for(
    ssid: &Ssid,
    security: Security,
    passphrase: Option<&Passphrase>,
) -> Settings {
    let text = |value: &str| Value::from(value.to_owned());
    let mut settings = Settings::new();
    settings.insert(
        "connection".to_owned(),
        HashMap::from([
            ("type".to_owned(), text("802-11-wireless")),
            ("id".to_owned(), text(&ssid.0)),
        ]),
    );
    let mut wireless =
        HashMap::from([("ssid".to_owned(), Value::from(ssid.0.as_bytes().to_vec()))]);
    let key_management = match (security, passphrase) {
        (Security::Psk, Some(_)) => Some("wpa-psk"),
        (Security::Sae, Some(_)) => Some("sae"),
        _ => None,
    };
    if let (Some(management), Some(passphrase)) = (key_management, passphrase) {
        wireless.insert("security".to_owned(), text("802-11-wireless-security"));
        settings.insert(
            "802-11-wireless-security".to_owned(),
            HashMap::from([
                ("key-mgmt".to_owned(), text(management)),
                ("psk".to_owned(), text(&passphrase.clone().into_text())),
            ]),
        );
    }
    settings.insert("802-11-wireless".to_owned(), wireless);
    settings
}

/// Joins `ssid` and answers when the device is on it or the join has failed.
pub(super) async fn join(
    bus: &BusSource,
    ssid: Ssid,
    passphrase: Option<Passphrase>,
) -> Result<(), JoinFailure> {
    let connection = system_connection(bus).await.map_err(unavailable)?;
    match within(
        Deadline::ACTION,
        peer::NETWORK,
        joining(&connection, &ssid, passphrase),
    )
    .await
    {
        Ok(done) => done,
        Err(_) => Err(told(TOO_LONG)),
    }
}

async fn joining(
    connection: &Connection,
    ssid: &Ssid,
    passphrase: Option<Passphrase>,
) -> Result<(), JoinFailure> {
    let manager = ManagerProxy::new(connection).await.map_err(unavailable)?;
    let device_path = wifi_device(connection, &manager)
        .await
        .map_err(unavailable)?;
    let device = device_at(connection, &device_path)
        .await
        .map_err(unavailable)?;
    // Subscribed before the join starts, so no state falls between starting and listening.
    let mut changes = device.receive_state_change().await.map_err(unavailable)?;
    let access = access_for(connection, &device_path, ssid).await;
    let security = access
        .as_ref()
        .map_or(Security::Open, |(_, security)| *security);
    let added = start(
        connection,
        &manager,
        &device_path,
        ssid,
        access.map(|(path, _)| path),
        security,
        passphrase,
    )
    .await?;
    let mut under_way = false;
    let ended = loop {
        let Some(change) = next(&mut changes).await else {
            break Err(could_not(ssid));
        };
        let Ok(args) = change.args() else { continue };
        match verdict(*args.new_state(), *args.reason(), under_way, security, ssid) {
            Verdict::Done => break Ok(()),
            Verdict::Failed(why) => break Err(why),
            Verdict::Going => under_way = true,
            Verdict::Waiting => {}
        }
    };
    if let (Err(_), Some(path)) = (&ended, added) {
        forget(connection, &path).await;
    }
    ended
}

/// The access point of `ssid` with the best signal, and how it is secured.
async fn access_for(
    connection: &Connection,
    device_path: &OwnedObjectPath,
    ssid: &Ssid,
) -> Option<(OwnedObjectPath, Security)> {
    let wireless = wireless_at(connection, device_path).await.ok()?;
    let mut best: Option<(u8, OwnedObjectPath, Security)> = None;
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
        let security = security_of(
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
    manager: &ManagerProxy<'_>,
    device_path: &OwnedObjectPath,
    ssid: &Ssid,
    access: Option<OwnedObjectPath>,
    security: Security,
    passphrase: Option<Passphrase>,
) -> Result<Option<OwnedObjectPath>, JoinFailure> {
    let remembered: Vec<OwnedObjectPath> = saved(connection)
        .await
        .into_iter()
        .filter(|(_, saved_ssid)| saved_ssid == ssid)
        .map(|(path, _)| path)
        .collect();
    let device = ObjectPath::from(device_path);
    let none = ObjectPath::default();
    if passphrase.is_none()
        && let Some(path) = remembered.first()
    {
        manager
            .activate_connection(&ObjectPath::from(path), &device, &none)
            .await
            .map_err(|_| could_not(ssid))?;
        return Ok(None);
    }
    if security == Security::Unsupported {
        return Err(told(format!(
            "\u{201c}{}\u{201d} needs a sign-in this computer can\u{2019}t do yet.",
            ssid.0
        )));
    }
    if passphrase.is_some() {
        for old in &remembered {
            forget(connection, old).await;
        }
    }
    let specific = access.as_ref().map(ObjectPath::from).unwrap_or_default();
    let settings = settings_for(ssid, security, passphrase.as_ref());
    let (added, _active) = manager
        .add_and_activate_connection(settings, &device, &specific)
        .await
        .map_err(|_| could_not(ssid))?;
    Ok(Some(added))
}

/// What a change of the device's state means for a join.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Verdict {
    /// The device is on the network.
    Done,
    /// The join failed.
    Failed(JoinFailure),
    /// The device has started on the join.
    Going,
    /// Nothing to conclude yet.
    Waiting,
}

/// The meaning of the device entering `state` for `reason`, `under_way` once it had started.
fn verdict(state: u32, reason: u32, under_way: bool, security: Security, ssid: &Ssid) -> Verdict {
    let refused = security != Security::Open && REFUSED_REASONS.contains(&reason);
    match state {
        ACTIVATED => Verdict::Done,
        NEED_AUTH if security != Security::Open => Verdict::Failed(JoinFailure::WrongPassword),
        FAILED if refused => Verdict::Failed(JoinFailure::WrongPassword),
        DISCONNECTED if refused && under_way => Verdict::Failed(JoinFailure::WrongPassword),
        FAILED => Verdict::Failed(could_not(ssid)),
        DISCONNECTED if under_way => Verdict::Failed(could_not(ssid)),
        PREPARE..=SECONDARIES => Verdict::Going,
        _ => Verdict::Waiting,
    }
}

/// Deletes the saved connection at `path`; one that is already gone is as good.
async fn forget(connection: &Connection, path: &OwnedObjectPath) {
    let Ok(builder) = SavedConnectionProxy::builder(connection).path(path.clone()) else {
        return;
    };
    if let Ok(saved) = builder.build().await {
        let _ = saved.delete().await;
    }
}

#[cfg(test)]
mod tests {
    use super::{Security, Verdict, security_of, settings_for, verdict};
    use detent_model::{JoinFailure, Passphrase, Ssid};

    #[test]
    fn the_device_state_ends_a_join_or_does_not() {
        let home = Ssid("Home".to_owned());
        let wrong = Verdict::Failed(JoinFailure::WrongPassword);
        // (case, state, reason, under way, security, verdict)
        let cases = [
            (
                "on the network",
                100,
                0,
                false,
                Security::Psk,
                Verdict::Done,
            ),
            (
                "asked for a secret",
                60,
                0,
                true,
                Security::Psk,
                wrong.clone(),
            ),
            ("no secrets", 120, 7, true, Security::Psk, wrong.clone()),
            (
                "timed out in the handshake",
                120,
                11,
                true,
                Security::Sae,
                wrong.clone(),
            ),
            (
                "an open network cannot be wrong",
                120,
                7,
                true,
                Security::Open,
                Verdict::Failed(super::could_not(&home)),
            ),
            (
                "leaving the old network",
                30,
                0,
                false,
                Security::Psk,
                Verdict::Waiting,
            ),
            (
                "dropped after starting",
                30,
                0,
                true,
                Security::Psk,
                Verdict::Failed(super::could_not(&home)),
            ),
            ("preparing", 40, 0, false, Security::Psk, Verdict::Going),
        ];
        for (case, state, reason, under_way, security, want) in cases {
            assert_eq!(
                verdict(state, reason, under_way, security, &home),
                want,
                "{case}"
            );
        }
    }

    #[test]
    fn flags_say_how_a_network_is_secured() {
        const CASES: &[(&str, u32, u32, u32, Security)] = &[
            ("open", 0, 0, 0, Security::Open),
            ("wpa2 personal", 1, 0, 0x108, Security::Psk),
            ("wpa3", 1, 0, 0x400, Security::Sae),
            ("wpa2 and wpa3", 1, 0, 0x500, Security::Psk),
            ("enterprise", 1, 0, 0x200, Security::Unsupported),
            ("wep", 1, 0, 0, Security::Unsupported),
        ];
        for (case, flags, wpa, rsn, want) in CASES {
            assert_eq!(security_of(*flags, *wpa, *rsn), *want, "{case}");
        }
    }

    #[test]
    fn a_secured_connection_carries_its_password_and_an_open_one_has_no_security_group() {
        let ssid = Ssid("Home".to_owned());
        let secured = settings_for(
            &ssid,
            Security::Psk,
            Some(&Passphrase::new("correct horse")),
        );
        assert!(secured.contains_key("802-11-wireless-security"));
        let open = settings_for(&ssid, Security::Open, None);
        assert!(!open.contains_key("802-11-wireless-security"));
        assert!(open.contains_key("802-11-wireless"));
    }
}
