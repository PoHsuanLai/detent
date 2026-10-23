//! The Extensions page: what is installed, grouped by what it provides, with a switch on each.
//! The folders and the list of the ones turned off are `detent-files::ExtensionHome`'s.

mod link;
mod pane;

pub(crate) use link::ExtensionLink;
pub(crate) use pane::ExtensionsPane;
