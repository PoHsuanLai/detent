//! llama.cpp, the engine that runs models on this computer, as the Intelligence page meets it:
//! whether the assistant found its program, how to get it when it did not, and
//! the restart that makes the assistant look again. Pure; the calls and the words are
//! `detent-ui`'s.

mod advice;
mod programs;
mod step;

pub use advice::{InstallAdvice, install_advice};
pub use programs::{
    PROGRAMS_SECTION, ProgramRow, Programs, ProgramsEffect, ProgramsInput, ProgramsPane,
    program_rows, programs_step,
};
pub use step::{
    EnginesEffect, EnginesInput, EnginesState, LLAMA_CPP_ID, ListedEngine, LlamaCpp, Program,
    Restart, Restarted, SetupNote, StartStanding, engines_step, llama_cpp_of, setup_note,
};

#[cfg(test)]
mod programs_tests;
