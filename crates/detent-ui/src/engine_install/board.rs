//! Following the engines and carrying out what the offer asks. Every page that draws the picker
//! calls [`use_board`] through `use_picker`, so its Local row and the sheet agree.

use super::desk::Desk;
use crate::env::Env;
use detent_model::{
    EngineOfferEffect, EngineOfferInput, EngineOfferState, EngineStanding, engine_offer_step,
};
use dioxus::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

/// What an engine's menu word starts with when picking it asks to get the engine, so it is never
/// mistaken for the name of one that is there.
pub(crate) const GET: &str = "get:";

/// How often the engines are looked at while nothing is going on.
const IDLE: Duration = Duration::from_secs(2);

/// How often while a download is under way.
const BUSY: Duration = Duration::from_millis(400);

/// The engines and the offer, as a page draws them.
#[derive(Clone, Copy)]
pub(crate) struct Board {
    /// Where the offer stands, and the engines it knows.
    pub state: Signal<EngineOfferState>,
    /// What the person did: chose an engine, took an action.
    pub ask: EventHandler<EngineOfferInput>,
}

/// Looks at the engines, follows them, and carries out the offer's effects. Called once per page
/// that draws the picker.
pub(crate) fn use_board() -> Board {
    let env = use_context::<Env>();
    let mut state = use_signal(EngineOfferState::default);
    let looking = env.engines.clone();
    use_future(move || {
        let desk = looking.clone();
        async move {
            loop {
                let rows = desk.rows().await;
                let busy = rows
                    .iter()
                    .any(|row| matches!(row.standing, EngineStanding::Installing(_)));
                send(&mut state, EngineOfferInput::Rows(rows));
                tokio::time::sleep(if busy { BUSY } else { IDLE }).await;
            }
        }
    });
    let desk = env.engines.clone();
    let ask = EventHandler::new(move |input: EngineOfferInput| {
        spawn(drive(state, desk.clone(), input));
    });
    Board { state, ask }
}

/// Applies `input` to the offer and returns what it asks for.
fn send(state: &mut Signal<EngineOfferState>, input: EngineOfferInput) -> Vec<EngineOfferEffect> {
    let (next, effects) = engine_offer_step(state.peek().clone(), input);
    state.set(next);
    effects
}

/// Applies `first`, then every input its effects lead to, until nothing is left to do.
async fn drive(mut state: Signal<EngineOfferState>, desk: Arc<dyn Desk>, first: EngineOfferInput) {
    let mut queue = VecDeque::from([first]);
    while let Some(input) = queue.pop_front() {
        for effect in send(&mut state, input) {
            match effect {
                EngineOfferEffect::Install { engine, scope } => {
                    if let Err(why) = desk.install(&engine, scope) {
                        queue.push_back(EngineOfferInput::NotStarted(why));
                    }
                }
                EngineOfferEffect::Cancel(engine) => desk.cancel(&engine),
                EngineOfferEffect::ChooseFolder => {
                    queue.push_back(EngineOfferInput::Folder(desk.choose_folder().await));
                }
                EngineOfferEffect::Point { engine, folder } => {
                    queue.push_back(EngineOfferInput::Pointed(
                        desk.point(&engine, &folder).await,
                    ));
                }
                EngineOfferEffect::Restart => desk.restart().await,
            }
        }
    }
}
