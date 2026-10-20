//! What the page asks of BlueZ: the switch, and connecting, pairing and forgetting a device. Each
//! call runs under a deadline, and a failure is a sentence for the person.

use super::bluez::{AdapterProxy, DeviceProxy};
use super::read::{find_device, managed};
use crate::deadline::{Deadline, peer};
use detent_model::{BtAddress, BtPower};
use ds_peer::within;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

const NOT_AVAILABLE: &str = "Bluetooth isn\u{2019}t available right now. Try again in a moment.";

fn quoted(name: &str) -> String {
    format!("\u{201c}{name}\u{201d}")
}

/// The adapter, the device at `address` and its name, or the sentence for not finding it.
async fn locate(
    connection: &Connection,
    address: &BtAddress,
) -> Result<(OwnedObjectPath, OwnedObjectPath, String), String> {
    let objects = managed(connection)
        .await
        .map_err(|_| NOT_AVAILABLE.to_owned())?;
    find_device(&objects, address)
        .ok_or_else(|| "That device isn\u{2019}t in range any more.".to_owned())
}

async fn device_at<'a>(
    connection: &'a Connection,
    path: &OwnedObjectPath,
) -> Result<DeviceProxy<'a>, String> {
    DeviceProxy::builder(connection)
        .path(path.clone())
        .map_err(|_| NOT_AVAILABLE.to_owned())?
        .build()
        .await
        .map_err(|_| NOT_AVAILABLE.to_owned())
}

/// Runs `call` under `deadline`; `told` is the sentence for BlueZ's refusal.
async fn bounded<T>(
    deadline: Deadline,
    call: impl Future<Output = zbus::Result<T>>,
    told: impl FnOnce(&zbus::Error) -> String,
) -> Result<T, String> {
    match within(deadline, peer::BLUETOOTH, call).await {
        Ok(Ok(done)) => Ok(done),
        Ok(Err(error)) => Err(told(&error)),
        Err(silent) => Err(format!("{silent}.")),
    }
}

/// Moves the switch. The adapter lets go of the radio when it is blocked outside the software.
pub(super) async fn set_power(connection: &Connection, power: BtPower) -> Result<(), String> {
    let objects = managed(connection)
        .await
        .map_err(|_| NOT_AVAILABLE.to_owned())?;
    let adapter = super::read::adapter_path(&objects).ok_or_else(|| NOT_AVAILABLE.to_owned())?;
    let proxy = AdapterProxy::builder(connection)
        .path(adapter)
        .map_err(|_| NOT_AVAILABLE.to_owned())?
        .build()
        .await
        .map_err(|_| NOT_AVAILABLE.to_owned())?;
    let on = power == BtPower::On;
    bounded(Deadline::ACTION, proxy.set_powered(on), |_| {
        "Bluetooth couldn\u{2019}t be switched. Check that it isn\u{2019}t switched off on this computer."
            .to_owned()
    })
    .await?;
    if on {
        // Looking for devices was refused while the radio was off.
        let _ = proxy.start_discovery().await;
    }
    Ok(())
}

pub(super) async fn connect(connection: &Connection, address: &BtAddress) -> Result<(), String> {
    let (_, path, name) = locate(connection, address).await?;
    let device = device_at(connection, &path).await?;
    bounded(Deadline::ACTION, device.connect(), |_| {
        format!(
            "Couldn\u{2019}t connect to {}. Make sure it is on and nearby.",
            quoted(&name)
        )
    })
    .await
}

pub(super) async fn disconnect(connection: &Connection, address: &BtAddress) -> Result<(), String> {
    let (_, path, name) = locate(connection, address).await?;
    let device = device_at(connection, &path).await?;
    bounded(Deadline::ACTION, device.disconnect(), |_| {
        format!("Couldn\u{2019}t disconnect from {}.", quoted(&name))
    })
    .await
}

/// Pairs with the device, trusts it so it may connect again, and connects. Pairing waits as long
/// as BlueZ does, since the person may be comparing numbers.
pub(super) async fn pair(connection: &Connection, address: &BtAddress) -> Result<(), String> {
    let (_, path, name) = locate(connection, address).await?;
    let device = device_at(connection, &path).await?;
    bounded(Deadline::PROMPT, device.pair(), |error| {
        pair_failure(error_code(error), &name)
    })
    .await?;
    let _ = device.set_trusted(true).await;
    // The pairing is done whether or not the first connection takes.
    let _ = within(Deadline::ACTION, peer::BLUETOOTH, device.connect()).await;
    Ok(())
}

/// The name of the error BlueZ refused with; empty for any other failure.
fn error_code(error: &zbus::Error) -> &str {
    match error {
        zbus::Error::MethodError(code, _, _) => code.as_str(),
        _ => "",
    }
}

/// What BlueZ's refusal of a pairing, by its error name, says to the person.
fn pair_failure(refused: &str, name: &str) -> String {
    let device = quoted(name);
    if refused.ends_with("AuthenticationCanceled") || refused.ends_with("AuthenticationRejected") {
        format!("Pairing with {device} was cancelled.")
    } else if refused.ends_with("AuthenticationTimeout") {
        format!("Pairing with {device} took too long. Try again.")
    } else {
        format!("Couldn\u{2019}t pair with {device}. Make sure it is in pairing mode and nearby.")
    }
}

pub(super) async fn forget(connection: &Connection, address: &BtAddress) -> Result<(), String> {
    let (adapter, path, name) = locate(connection, address).await?;
    let proxy = AdapterProxy::builder(connection)
        .path(adapter)
        .map_err(|_| NOT_AVAILABLE.to_owned())?
        .build()
        .await
        .map_err(|_| NOT_AVAILABLE.to_owned())?;
    bounded(
        Deadline::ACTION,
        proxy.remove_device(&ObjectPath::from(&path)),
        |_| format!("Couldn\u{2019}t forget {}.", quoted(&name)),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::pair_failure;

    #[test]
    fn a_pairing_that_was_cancelled_or_timed_out_says_so() {
        let cases = [
            (
                "org.bluez.Error.AuthenticationCanceled",
                "Pairing with \u{201c}Pad\u{201d} was cancelled.",
            ),
            (
                "org.bluez.Error.AuthenticationRejected",
                "Pairing with \u{201c}Pad\u{201d} was cancelled.",
            ),
            (
                "org.bluez.Error.AuthenticationTimeout",
                "Pairing with \u{201c}Pad\u{201d} took too long. Try again.",
            ),
            (
                "org.bluez.Error.Failed",
                "Couldn\u{2019}t pair with \u{201c}Pad\u{201d}. Make sure it is in pairing mode and nearby.",
            ),
        ];
        for (code, want) in cases {
            assert_eq!(pair_failure(code, "Pad"), want, "{code}");
        }
    }
}
