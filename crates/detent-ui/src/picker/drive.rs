//! The picker a page draws: where inferd's rows stand, and what the person's asks do. Every page
//! that draws a part of the picker uses this hook, so they agree; the picker itself, with its
//! follow loop and its calls, is `detent-client`'s.

use crate::copy::picker::agent_word;
use crate::engine_install::use_board;
use crate::live::use_live;
use detent_model::{PickerInput, PickerState};
use dioxus::prelude::*;

/// The picker a page draws: where inferd's rows stand, and what the person's asks do.
#[derive(Clone, Copy)]
pub(crate) struct Picker {
    /// Where the picker stands.
    pub state: Signal<PickerState>,
    /// What the person asked of a row: a pick, an engine for Local, a model in or out of
    /// Automatic's list.
    pub ask: EventHandler<PickerInput>,
}

/// Looks for inferd's picker, follows it, and sends its picks. Call it once per page that draws
/// the picker; it never starts inferd.
pub(crate) fn use_picker() -> Picker {
    // The engines Local can run models with, and the sheet that gets one: every row of this page
    // finds them by context.
    let board = use_board();
    use_context_provider(|| board);
    let picker = use_live(|client| client.picker(agent_word));
    let state = picker.state;
    let ask = EventHandler::new(move |input: PickerInput| {
        spawn(picker.pane.send(input));
    });
    Picker { state, ask }
}
