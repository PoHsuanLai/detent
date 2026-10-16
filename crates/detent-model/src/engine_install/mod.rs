//! Getting an engine that runs models on this computer (llama.cpp, vLLM, Strata) when the person
//! chooses it and it is not there: the engines as the picker lists them, the one sheet that asks
//! "Download llama.cpp?" and follows the download, and the words on it. Pure; the calls (what is
//! installed, an install, a folder chosen, the assistant restarted) are `detent-ui`'s.

mod rows;
mod sheet;
mod words;

pub use rows::{EngineRow, EngineStanding, EngineWay, InstallScope, engine_named};
pub use sheet::{
    EngineRef, EngineSheet, EngineSheetEffect, EngineSheetInput, EngineSheetState, Replacing,
    engine_sheet_step,
};
pub use words::{INSTALL_FOR, Press, SheetButton, SheetWords, Waiting, sheet_words, size_words};

#[cfg(test)]
mod tests;
