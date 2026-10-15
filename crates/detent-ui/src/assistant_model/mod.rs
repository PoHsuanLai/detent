//! The assistant's model: the one row of inferd's picker the assistant is routed by, drawn as a
//! page of the sidebar and as the short first-run step that asks for it. Nothing is chosen for
//! the person: both show an unset model as nothing chosen.

mod pane;
mod picker;
mod setup;

pub(crate) use pane::AssistantModelPane;
pub(crate) use setup::AssistantSetup;
