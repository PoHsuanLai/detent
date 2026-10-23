//! llama.cpp on this computer: the note under "Models on this computer" while the assistant has
//! not found its program, with the line to install it and the button that makes the assistant
//! look again; and the rows that name an engine's program when it is not found by itself.

mod io;
mod programs;
mod view;

pub(crate) use io::restart;
pub(crate) use programs::EnginePrograms;
pub(crate) use view::LocalEnginesNote;
