//! How the Wi-Fi page reaches the network service: one handle the environment carries, so a test
//! hands in a service that only answers from memory.

use crate::ticks::Ticks;
use detent_model::{JoinFailure, Missing, Passphrase, Ssid, WifiRadio, WifiStatus, Work};
use std::fmt::Debug;

/// What the page needs of the network service.
pub trait WifiLink: Debug + Send + Sync {
    /// The switch and the networks in range now, or why there are none.
    fn read(&self) -> Work<'_, Result<WifiStatus, Missing>>;
    /// A tick each time the standing may have changed, also when the service appears or goes;
    /// only the bus being unreachable is an error.
    fn changes(&self) -> Work<'_, Result<Ticks, Missing>>;
    /// Asks the radio to look for networks again; the answer arrives as a change.
    fn scan(&self) -> Work<'_, ()>;
    /// Moves the switch; the service's words when it cannot.
    fn set_radio(&self, radio: WifiRadio) -> Work<'_, Result<(), String>>;
    /// Joins `ssid`, with `passphrase` when the person gave one, and answers when the join has
    /// ended on the network or has failed.
    fn join(&self, ssid: Ssid, passphrase: Option<Passphrase>)
    -> Work<'_, Result<(), JoinFailure>>;
}
