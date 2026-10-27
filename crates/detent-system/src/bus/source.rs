//! Which bus the daemons are looked for on, and who vouches for the programs that own their names.

use super::vouch::Vouch;

/// Where the window looks for inferd and accountd. A test never names the session bus: it hands the address of
/// a private one, or none.
#[derive(Debug, Clone)]
pub enum BusSource {
    /// The user's session bus. The owner of each service's name must hold that service's role in
    /// the desktop's caller table (`detent-callers`), or the service is unavailable.
    Session,
    /// A bus at this address, whose service owners `Vouch` judges (a test's private bus: it names
    /// its own check, since no caller table describes its stand-ins).
    Address(String, Vouch),
    /// No bus: the model picker and the Accounts pane are shown as unavailable.
    Absent,
}
