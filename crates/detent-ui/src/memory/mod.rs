//! The memory service's controls, as Settings uses them: the Spaces page settles a removed
//! Space's memories, and a page that holds memory settings asks which looser values wait for the
//! person's word and confirms them.

mod drive;
mod io;
mod proxy;
mod view;

pub(crate) use drive::{MemoryConsent, use_memory_consent};
pub(crate) use proxy::MemoryControlProxy;
pub(crate) use view::ConsentNote;
