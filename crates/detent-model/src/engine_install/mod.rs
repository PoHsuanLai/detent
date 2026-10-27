//! Getting an engine that runs models on this computer (llama.cpp, vLLM, Strata) when the person
//! chooses it and it is not there: the engines as the picker lists them, the one offer that asks
//! whether to download it and follows the download, and the facts it tells. Pure; the calls (what
//! is installed, an install, a folder chosen, the assistant restarted) and the words are
//! `detent-ui`'s.

mod brief;
mod offer;
mod rows;

pub use brief::{EngineAction, EngineBrief, EngineMatter, Waiting, engine_brief};
pub use offer::{
    EngineOffer, EngineOfferEffect, EngineOfferInput, EngineOfferState, EngineRef, Replacing,
    engine_offer_step,
};
pub use rows::{EngineRow, EngineStanding, EngineWay, InstallScope, engine_named};

#[cfg(test)]
mod tests;
