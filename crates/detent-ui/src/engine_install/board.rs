//! Following the engines and carrying out what the offer asks. Every page that draws the picker
//! calls [`use_board`] through `use_picker`, so its Local row and the sheet agree; the follow loop
//! and the desk are `detent-client`'s.

use crate::live::use_live;
use detent_client::Client;
use detent_model::{EngineOfferInput, EngineOfferState};
use dioxus::prelude::*;

/// What an engine's menu word starts with when picking it asks to get the engine, so it is never
/// mistaken for the name of one that is there.
pub(crate) const GET: &str = "get:";

/// The engines and the offer, as a page draws them.
#[derive(Clone, Copy)]
pub(crate) struct Board {
    /// Where the offer stands, and the engines it knows.
    pub state: Signal<EngineOfferState>,
    /// What the person did: chose an engine, took an action.
    pub ask: EventHandler<EngineOfferInput>,
}

/// Looks at the engines, follows them, and sends the person's choices. Called once per page that
/// draws the picker.
pub(crate) fn use_board() -> Board {
    let install = use_live(Client::engine_install);
    let state = install.state;
    let ask = EventHandler::new(move |input: EngineOfferInput| {
        spawn(install.pane.send(input));
    });
    Board { state, ask }
}
