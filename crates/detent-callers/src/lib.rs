//! Which program is on the other end of a bus name or a bus message.
//!
//! A well-known name on the session bus goes to whoever asks first, so Settings never trusts one:
//! before it reads a service, or hands it a typed secret, it asks whether the owner of the name is
//! the program that plays the service's role in the desktop's caller table ([`RoleCheck`], the
//! `OwnerCheck` quire's live client asks). The same peers judge who may ask the running Settings to
//! show itself (`detent-intents`). Everything here is peephole's: this crate names the roles and
//! the proof each question needs, and decides nothing about a process itself.

mod allow;
mod check;
#[cfg(feature = "dev-callers")]
mod dev;
mod peers;
mod services;
#[cfg(feature = "testing")]
mod testing;

pub use allow::allowing;
pub use check::RoleCheck;
#[cfg(feature = "dev-callers")]
pub use dev::{DevRowsFault, load_dev_rows, use_dev_rows};
pub use peephole_bus::peephole_core::{Proof, Refused, Requirement, RoleName, roles};
pub use peephole_bus::{BusPeers, access_denied};
pub use peers::desktop_peers;
pub use services::{
    ACCOUNTS_BUS, COMPOSITOR_BUS, DISPLAYS_BUS, INFERENCE_BUS, MEMORY_BUS, PORTAL_BUS,
    SERVICE_PROOF, SYNC_BUS, UPDATES_BUS, required_role,
};
#[cfg(feature = "testing")]
pub use testing::{introduce_service, private_peers};
