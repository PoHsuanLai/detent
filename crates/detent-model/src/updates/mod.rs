//! The Software Update page, as Settings meets it: what the update service says, what each press
//! may ask of it, and what the page shows for every standing. Pure; the service, the clock and
//! the log-out are the window's (`detent-ui`).

mod model;
mod page;
mod shown;
mod step;
mod words;

pub use model::{
    ByteCount, Permille, UnixSeconds, UpdateOffer, UpdatePhase, UpdateStatus, UpdateWire,
};
pub use page::is_software_update;
pub(crate) use page::{SLUG, software_update_page};
pub use shown::{
    ButtonActivity, NotesSheet, UPDATES_NOT_AVAILABLE, UpdateButton, UpdateMeter, UpdatesPage,
    UpdatesShown, updates_shown,
};
pub use step::{NotesState, UpdatesEffect, UpdatesInput, UpdatesReady, UpdatesState, updates_step};

#[cfg(test)]
mod tests;
