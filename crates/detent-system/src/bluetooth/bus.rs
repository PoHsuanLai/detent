//! BlueZ on the system bus, as the Bluetooth page's link. It is only ever looked for, never
//! started. One connection serves the page's calls and its agent: BlueZ asks the agent of the
//! program that started a pairing.

use super::actions;
use super::read::{managed, status_of};
use super::session::open;
use super::{BluetoothLink, BtSession};
use crate::bus::{BusSource, system_connection, watch_service};
use crate::ticks::Ticks;
use airwaves_bluez::SERVICE;
use detent_model::{Absence, BtAddress, BtStatus, Work, bt_change_matters};
use std::collections::HashMap;
use tokio::sync::OnceCell;
use zbus::Connection;
use zbus::Message;
use zbus::zvariant::OwnedValue;

const NOT_AVAILABLE: &str = "Bluetooth isn\u{2019}t available right now. Try again in a moment.";

/// Whether a signal from BlueZ is worth reading it again for.
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
    let names: Vec<&str> = changed.keys().map(String::as_str).collect();
    bt_change_matters(&interface, &names)
}

/// BlueZ on the bus it is built over.
#[derive(Debug)]
pub struct OnBluetooth {
    bus: BusSource,
    connection: OnceCell<Connection>,
}

impl OnBluetooth {
    /// BlueZ looked for on `bus`.
    pub fn new(bus: BusSource) -> OnBluetooth {
        OnBluetooth {
            bus,
            connection: OnceCell::new(),
        }
    }

    /// The one connection, made when first wanted.
    async fn connection(&self) -> Result<Connection, Absence> {
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

impl BluetoothLink for OnBluetooth {
    fn read(&self) -> Work<'_, Result<BtStatus, Absence>> {
        Box::pin(async move { status_of(&managed(&self.connection().await?).await?) })
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Absence>> {
        Box::pin(watch_service(&self.bus, SERVICE, matters))
    }

    fn open(&self) -> Work<'_, Result<BtSession, Absence>> {
        Box::pin(async move { open(self.connection().await?).await })
    }

    fn set_power(&self, on: bool) -> Work<'_, Result<(), String>> {
        Box::pin(self.told(move |c| async move { actions::set_power(&c, on).await }))
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
