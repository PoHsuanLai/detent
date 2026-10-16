//! llama.cpp on this computer: the note under "Models on this computer" while the assistant has
//! not found its program, with the line to install it and the button that makes the assistant
//! look again.

mod io;
mod view;

pub(crate) use io::restart;
pub(crate) use view::LocalEnginesNote;
