//! The list's state and what carries its effects out: the files are read and written here, the
//! decisions are `detent-model`'s `shortcuts_step`.

use crate::copy::shortcuts::{chord_reserved, file_left_alone};
use crate::env::Env;
use detent_client::FilesError;
use detent_model::{ShortcutsEffect, ShortcutsInput, ShortcutsState, shortcuts_step};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;

/// The list of one window.
#[derive(Clone)]
pub(super) struct Pane {
    env: Env,
    toasts: ToastHub,
    /// Where the list stands.
    pub(super) state: Signal<ShortcutsState>,
}

/// The pane over the files as they are now.
pub(super) fn use_pane() -> Pane {
    let env = use_context::<Env>();
    let state = use_signal({
        let env = env.clone();
        move || {
            let read = ShortcutsInput::Read(env.keymap.read());
            shortcuts_step(ShortcutsState::over(env.keymap.base()), read).0
        }
    });
    Pane {
        env,
        toasts: use_toasts(),
        state,
    }
}

impl Pane {
    /// Gives the list `input` and carries out what follows.
    pub(super) fn feed(&self, input: ShortcutsInput) {
        let mut state = self.state;
        let (next, effects) = shortcuts_step(state.peek().clone(), input);
        state.set(next);
        for effect in effects {
            self.run(effect);
        }
    }

    fn run(&self, effect: ShortcutsEffect) {
        match effect {
            ShortcutsEffect::Save { base, text } => {
                let ended =
                    self.env
                        .keymap
                        .save_person(&base, &text)
                        .map_err(|error| match error {
                            FilesError::Changed { .. } => {
                                crate::copy::shortcuts::SHORTCUTS_CHANGED_ELSEWHERE.to_owned()
                            }
                            _ => crate::copy::shortcuts::SHORTCUTS_NOT_SAVED.to_owned(),
                        });
                self.feed(ShortcutsInput::Saved(ended));
            }
            ShortcutsEffect::Reread => self.feed(ShortcutsInput::Read(self.env.keymap.read())),
            ShortcutsEffect::Tell(words) => self.toasts.push(words, None),
            ShortcutsEffect::FileLeftAlone { trouble } => {
                self.toasts.push(file_left_alone(&trouble), None);
            }
            ShortcutsEffect::Reserved { chord, reason } => {
                self.toasts.push(chord_reserved(&chord, &reason), None);
            }
        }
    }

    /// The list follows the keymap for as long as the window lives: a change made elsewhere
    /// (by keycap's other clients, by hand) shows within a moment.
    pub(super) async fn follow(self) {
        let (tell, mut changed) = tokio::sync::mpsc::unbounded_channel();
        let _watch = self.env.keymap.watch(move || {
            let _ = tell.send(());
        });
        while changed.recv().await.is_some() {
            self.feed(ShortcutsInput::Read(self.env.keymap.read()));
        }
    }
}
