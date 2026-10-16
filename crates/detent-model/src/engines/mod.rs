//! llama.cpp, the engine that runs models on this computer, as the Intelligence page meets it:
//! whether the assistant found its program, the line that says how to get it when it did not, and
//! the restart that makes the assistant look again. Pure; the calls are `detent-ui`'s.

mod advice;
mod step;

pub use advice::{InstallAdvice, RELEASES_LINE, install_advice};
pub use step::{
    EnginesEffect, EnginesInput, EnginesState, LLAMA_CPP_ID, ListedEngine, LlamaCpp, Program,
    RESTART, Restart, Restarted, SETUP_LINE, START_USING, SetupNote, StartButton, engines_step,
    llama_cpp_of, restart_question, setup_note,
};
