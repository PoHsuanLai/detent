//! The network service on the system bus, as the Wi-Fi page's link. It is only ever looked for,
//! never started.

use super::join::{join, join_hidden, sign_in};
use super::known::forget;
use super::link::WifiLink;
use super::nm::{ManagerProxy, SERVICE};
use super::read::{read, wifi_device, wireless_at};
use crate::deadline::{Deadline, peer};
use crate::live::BusSource;
use crate::system::{every, system_connection, watch_service};
use crate::ticks::Ticks;
use detent_model::{
    Account, HiddenNetwork, JoinFailure, Missing, Passphrase, Ssid, WifiRadio, WifiStatus, Work,
};
use ds_peer::within;
use std::collections::HashMap;

const NOT_AVAILABLE: &str = "Wi-Fi isn\u{2019}t available right now. Try again in a moment.";
const COULD_NOT_SWITCH: &str = "Wi-Fi couldn\u{2019}t be switched. Try again in a moment.";

/// The network service on `bus`.
#[derive(Debug)]
pub(crate) struct OnSystem(pub(crate) BusSource);

impl WifiLink for OnSystem {
    fn read(&self) -> Work<'_, Result<WifiStatus, Missing>> {
        Box::pin(read(&self.0))
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Missing>> {
        Box::pin(watch_service(&self.0, SERVICE, every))
    }

    fn scan(&self) -> Work<'_, ()> {
        Box::pin(scan(&self.0))
    }

    fn set_radio(&self, radio: WifiRadio) -> Work<'_, Result<(), String>> {
        Box::pin(set_radio(&self.0, radio))
    }

    fn join(
        &self,
        ssid: Ssid,
        passphrase: Option<Passphrase>,
    ) -> Work<'_, Result<(), JoinFailure>> {
        Box::pin(join(&self.0, ssid, passphrase))
    }

    fn sign_in(&self, ssid: Ssid, account: Account) -> Work<'_, Result<(), JoinFailure>> {
        Box::pin(sign_in(&self.0, ssid, account))
    }

    fn join_hidden(&self, network: HiddenNetwork) -> Work<'_, Result<(), JoinFailure>> {
        Box::pin(join_hidden(&self.0, network))
    }

    fn forget(&self, ssid: Ssid) -> Work<'_, Result<(), String>> {
        Box::pin(forget(&self.0, ssid))
    }
}

async fn set_radio(bus: &BusSource, radio: WifiRadio) -> Result<(), String> {
    let unavailable = |_| NOT_AVAILABLE.to_owned();
    let connection = system_connection(bus).await.map_err(unavailable)?;
    let manager = ManagerProxy::new(&connection)
        .await
        .map_err(|_| NOT_AVAILABLE.to_owned())?;
    let on = radio == WifiRadio::On;
    within(
        Deadline::ACTION,
        peer::NETWORK,
        manager.set_wireless_enabled(on),
    )
    .await
    .map_err(|silent| format!("{silent}."))?
    .map_err(|_| COULD_NOT_SWITCH.to_owned())
}

/// Asks the wireless device to look again; a computer that cannot is left as it is.
async fn scan(bus: &BusSource) {
    let Ok(connection) = system_connection(bus).await else {
        return;
    };
    let asked = async {
        let manager = ManagerProxy::new(&connection).await.ok()?;
        let device = wifi_device(&connection, &manager).await.ok()?;
        let wireless = wireless_at(&connection, &device).await.ok()?;
        wireless.request_scan(HashMap::new()).await.ok()
    };
    let _ = within(Deadline::READ, peer::NETWORK, asked).await;
}
