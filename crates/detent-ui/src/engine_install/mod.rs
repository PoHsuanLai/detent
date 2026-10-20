//! Getting llama.cpp, vLLM or Strata when the person chooses one in "Runs with" and it is not
//! there: the engines as the Local row lists them, the sheet that asks to download one, and the
//! download in this window. The desk is what the sheet does to the computer; the board follows the
//! engines and carries out what the sheet asks.

mod board;
mod desk;
mod system;
mod view;

pub(crate) use board::{Board, GET, use_board};
pub use desk::{Desk, Quiet};
pub(crate) use system::System;
pub(crate) use view::EngineSheetView;
