//! The Software Update page, as Settings meets it: what the update service says, what each action
//! may ask of it, and the facts the page shows for every standing. Pure; the service, the clock and
//! the log-out are the window's (`detent-ui`).

mod limits;
mod model;
mod page;
mod shown;
mod step;

pub use model::{
    ByteCount, Permille, ProblemKind, UnixSeconds, UpdateOffer, UpdatePhase, UpdateStatus,
    UpdateVersion, UpdateWire,
};
pub(crate) use page::{SOFTWARE_UPDATE_NAME, SOFTWARE_UPDATE_SLUG, software_update_page};
pub use shown::{
    Checked, Downloaded, ReleaseNotes, UpdateAction, UpdateMeter, UpdateStanding, UpdatesPage,
    UpdatesShown, updates_shown,
};
pub use step::{
    UpdatesEffect, UpdatesInput, UpdatesNotice, UpdatesReady, UpdatesState, updates_step,
};

#[cfg(test)]
mod tests;
