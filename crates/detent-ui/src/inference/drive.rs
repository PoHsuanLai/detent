//! Following inferd's model picker and carrying out what its steps ask: the one place the picker
//! calls the bus. Every page that draws a part of the picker uses this hook, so they agree.

use super::{is_choice, set, set_pool};
use crate::accounts::add_account;
use crate::engine_install::use_board;
use crate::env::Env;
use crate::live::{BusSource, Daemon, Followed, follow, reread, words_of};
use detent_model::{PickerEffect, PickerInput, PickerState, step};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use ds_settings::live::LiveClient;
use std::collections::VecDeque;

/// The picker a page draws: where inferd's rows stand, and what the person's asks do.
#[derive(Clone, Copy)]
pub(crate) struct Picker {
    /// Where the picker stands.
    pub state: Signal<PickerState>,
    /// What the person asked of a row: a pick, an engine for Local, a model in or out of
    /// Automatic's list.
    pub ask: EventHandler<PickerInput>,
}

/// Applies `input` to the picker and returns the effects it asks for.
fn send(state: &mut Signal<PickerState>, input: PickerInput) -> Vec<PickerEffect> {
    let (next, effects) = step(state.peek().clone(), input);
    state.set(next);
    effects
}

/// Looks for inferd's picker, follows it, and sends its picks. Call it once per page that draws
/// the picker; it never starts inferd.
pub(crate) fn use_picker() -> Picker {
    let env = use_context::<Env>();
    // The engines Local can run models with, and the sheet that gets one: every row of this page
    // finds them by context.
    let board = use_board();
    use_context_provider(|| board);
    let toasts = use_toasts();
    let mut state = use_signal(PickerState::default);
    let mut client = use_signal(|| None::<LiveClient>);

    let look = env.clone();
    use_future(move || {
        let bus = look.bus.clone();
        async move {
            follow(
                &bus,
                &Daemon::inferd(),
                is_choice,
                |followed| match followed {
                    Followed::Opened(opened, described) => {
                        send(&mut state, PickerInput::Found(Ok(described)));
                        client.set(Some(opened));
                    }
                    Followed::Gone(absence) => {
                        send(&mut state, PickerInput::Found(Err(absence)));
                        client.set(None);
                    }
                    Followed::Changed(path, value) => match value {
                        toml::Value::String(word) => {
                            send(&mut state, PickerInput::Changed { path, word });
                        }
                        toml::Value::Array(items) => {
                            let words = words_of(&items);
                            send(&mut state, PickerInput::PoolChanged { path, words });
                        }
                        _ => {}
                    },
                },
            )
            .await;
        }
    });

    let bus = env.bus.clone();
    let ask = EventHandler::new(move |input: PickerInput| {
        spawn(drive(state, client, bus.clone(), toasts, input));
    });
    Picker { state, ask }
}

/// Applies `first` to the picker, then every input its effects lead to, until nothing is left to
/// do. The calls wait here, so a caller spawns this and returns.
async fn drive(
    mut state: Signal<PickerState>,
    client: Signal<Option<LiveClient>>,
    bus: BusSource,
    toasts: ToastHub,
    first: PickerInput,
) {
    let mut queue = VecDeque::from([first]);
    while let Some(input) = queue.pop_front() {
        for effect in send(&mut state, input) {
            match effect {
                PickerEffect::Tell(text) => toasts.push(text, None),
                PickerEffect::Set { path, word } => {
                    let Some(opened) = client.peek().clone() else {
                        continue;
                    };
                    queue.push_back(match set(&opened, &path, &word).await {
                        Ok(()) => PickerInput::Applied(path),
                        Err(why) => PickerInput::Refused { path, why },
                    });
                }
                PickerEffect::SetPool { path, words } => {
                    let Some(opened) = client.peek().clone() else {
                        continue;
                    };
                    queue.push_back(match set_pool(&opened, &path, &words).await {
                        Ok(()) => PickerInput::PoolApplied(path),
                        Err(_) => PickerInput::PoolRefused(path),
                    });
                }
                PickerEffect::AddAccount => {
                    queue.push_back(PickerInput::AccountSheetEnded(add_account(&bus).await));
                }
                PickerEffect::ReadAgain => {
                    let Some(opened) = client.peek().clone() else {
                        continue;
                    };
                    queue.push_back(match reread(&opened, is_choice).await {
                        Ok(described) => PickerInput::Reread(described),
                        Err(absence) => PickerInput::Found(Err(absence)),
                    });
                }
            }
        }
    }
}
