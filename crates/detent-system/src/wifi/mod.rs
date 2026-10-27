//! The network service behind the Wi-Fi page: one handle the environment carries, so a test hands
//! in a service that only answers from memory.

mod bus;
#[cfg(feature = "testing")]
mod fake;
mod join;
mod known;
mod profile;
mod read;

pub use bus::OnWifi;
#[cfg(feature = "testing")]
pub use fake::{FakeWifi, WifiAsked};

use crate::ticks::Ticks;
use detent_model::{
    Absence, HiddenNetwork, JoinFailure, Passphrase, Ssid, WifiLogin, WifiRadio, WifiStatus, Work,
};
use std::fmt::Debug;

/// What the page needs of the network service.
pub trait WifiLink: Debug + Send + Sync {
    /// The switch and the networks in range now, or why there are none.
    fn read(&self) -> Work<'_, Result<WifiStatus, Absence>>;
    /// A tick each time the standing may have changed, also when the service appears or goes;
    /// only the bus being unreachable is an error.
    fn changes(&self) -> Work<'_, Result<Ticks, Absence>>;
    /// Asks the radio to look for networks again; the answer arrives as a change.
    fn scan(&self) -> Work<'_, ()>;
    /// Moves the switch; the service's words when it cannot.
    fn set_radio(&self, radio: WifiRadio) -> Work<'_, Result<(), String>>;
    /// Joins `ssid`, with `passphrase` when the person gave one, and answers when the join has
    /// ended on the network or has failed.
    fn join(&self, ssid: Ssid, passphrase: Option<Passphrase>)
    -> Work<'_, Result<(), JoinFailure>>;
    /// Joins `ssid`, a network that signs in with an account, and answers when the join has ended.
    fn sign_in(&self, ssid: Ssid, account: WifiLogin) -> Work<'_, Result<(), JoinFailure>>;
    /// Joins a network that does not announce its name, and answers when the join has ended.
    fn join_hidden(&self, network: HiddenNetwork) -> Work<'_, Result<(), JoinFailure>>;
    /// Removes the kept password of `ssid`, so the computer no longer joins it by itself.
    fn forget(&self, ssid: Ssid) -> Work<'_, Result<(), String>>;
}
