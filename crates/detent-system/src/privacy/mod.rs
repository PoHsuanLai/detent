//! The calls the Privacy page makes on the compositor's module: allowing or denying a power for a
//! program, and asking again.

mod bus;
mod set;

pub use bus::OnClient;

use detent_model::{PrivacySwitch, Work};
use ds_settings::live::LiveClient;
use ds_settings::schema::KeyPath;
use std::fmt::Debug;

/// What the Privacy page asks of the compositor's module, through the client the pane follows it
/// with.
pub trait PrivacyLink: Debug + Send + Sync {
    /// Allows or denies at `path`; the module's own words when it refuses.
    fn set<'a>(
        &'a self,
        client: &'a LiveClient,
        path: &'a KeyPath,
        to: PrivacySwitch,
    ) -> Work<'a, Result<(), String>>;
    /// Asks again at `path`; the module's own words when it refuses.
    fn forget<'a>(
        &'a self,
        client: &'a LiveClient,
        path: &'a KeyPath,
    ) -> Work<'a, Result<(), String>>;
}
