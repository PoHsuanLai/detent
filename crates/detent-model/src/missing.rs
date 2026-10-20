//! Why one of the system pages (Wi-Fi, Bluetooth, Displays) has nothing to show.

/// Why a system page is empty. Each page words it for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing {
    /// The service that runs the hardware is not there, or this window has no way to reach it.
    NoService,
    /// The service is there and this computer has none of the hardware.
    NoDevice,
    /// The service did not answer, or answered something that made no sense.
    Failed,
}
