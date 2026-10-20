//! BlueZ on the system bus, as the Bluetooth page's link. It is only ever looked for, never
//! started. One connection serves the page's calls and its agent: BlueZ asks the agent of the
//! program that started a pairing.

use super::actions;
use super::link::{BluetoothLink, BtSession};
use super::read::{managed, status_of};
use super::session::open;
use crate::live::BusSource;
use crate::system::{system_connection, watch_service};
use crate::ticks::Ticks;
use airwaves_bluez::SERVICE;
use detent_model::{BtAddress, BtPower, BtStatus, Missing, Work};
use std::collections::HashMap;
use tokio::sync::OnceCell;
use zbus::Connection;
use zbus::Message;
use zbus::zvariant::OwnedValue;

const NOT_AVAILABLE: &str = "Bluetooth isn\u{2019}t available right now. Try again in a moment.";

/// What BlueZ says about a device it hears all the time: its signal, and what it broadcasts.
const NOISE: [&str; 4] = ["RSSI", "TxPower", "ManufacturerData", "ServiceData"];

/// Whether a signal from BlueZ is worth reading it again for: not a device's constantly changing
/// signal strength, which would have the page reading all the time while devices are nearby.
fn matters(message: &Message) -> bool {
    let header = message.header();
    if header.member().map(|member| member.as_str()) != Some("PropertiesChanged") {
        return true;
    }
    let Ok((interface, changed, _)) =
        message
            .body()
            .deserialize::<(String, HashMap<String, OwnedValue>, Vec<String>)>()
    else {
        return true;
    };
    !(interface == "org.bluez.Device1" && changed.keys().all(|name| NOISE.contains(&name.as_str())))
}

/// BlueZ on `bus`.
#[derive(Debug)]
pub(crate) struct OnSystem {
    bus: BusSource,
    connection: OnceCell<Connection>,
}

impl OnSystem {
    pub(crate) fn new(bus: BusSource) -> OnSystem {
        OnSystem {
            bus,
            connection: OnceCell::new(),
        }
    }

    /// The one connection, made when first wanted.
    async fn connection(&self) -> Result<Connection, Missing> {
        self.connection
            .get_or_try_init(|| system_connection(&self.bus))
            .await
            .cloned()
    }

    /// The sentence for a request when there is no connection.
    async fn told<T, F, Fut>(&self, call: F) -> Result<T, String>
    where
        F: FnOnce(Connection) -> Fut + Send,
        Fut: Future<Output = Result<T, String>> + Send,
    {
        match self.connection().await {
            Ok(connection) => call(connection).await,
            Err(_) => Err(NOT_AVAILABLE.to_owned()),
        }
    }
}

impl BluetoothLink for OnSystem {
    fn read(&self) -> Work<'_, Result<BtStatus, Missing>> {
        Box::pin(async move { status_of(&managed(&self.connection().await?).await?) })
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Missing>> {
        Box::pin(watch_service(&self.bus, SERVICE, matters))
    }

    fn open(&self) -> Work<'_, Result<BtSession, Missing>> {
        Box::pin(async move { open(self.connection().await?).await })
    }

    fn set_power(&self, power: BtPower) -> Work<'_, Result<(), String>> {
        Box::pin(self.told(move |c| async move { actions::set_power(&c, power).await }))
    }

    fn connect(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        Box::pin(self.told(move |c| async move { actions::connect(&c, &address).await }))
    }

    fn disconnect(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        Box::pin(self.told(move |c| async move { actions::disconnect(&c, &address).await }))
    }

    fn pair(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        Box::pin(self.told(move |c| async move { actions::pair(&c, &address).await }))
    }

    fn forget(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        Box::pin(self.told(move |c| async move { actions::forget(&c, &address).await }))
    }
}
