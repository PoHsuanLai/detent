//! The assistant's model, as Settings meets it: the one row of inferd's picker the assistant is
//! routed by, the page that shows only that row, and the first-run step that asks for it. The
//! person chooses; nothing is chosen for them.

mod page;
mod row;
mod setup;

pub(crate) use page::ASSISTANT_MODEL_NAME;
pub use page::{ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, assistant_model_page};
pub use row::assistant_of;
pub use setup::{SetupAction, SetupAvailability, setup_availability};

#[cfg(test)]
mod tests;
