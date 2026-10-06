//! Which bus the daemons are looked for on.

/// Where the window looks for inferd and accountd. A test never names the session bus: it hands the address of
/// a private one, or none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusSource {
    /// The user's session bus.
    Session,
    /// A bus at this address.
    Address(String),
    /// No bus: the model picker and the Accounts pane are shown as unavailable.
    Absent,
}
