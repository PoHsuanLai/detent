//! The Privacy pane: the compositor's module, followed for its switches, and the choices the
//! person makes in it. The steps are `detent-model`'s `privacy_step`.

use crate::follow::{Followed, Slot, follow_with};
use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{PrivacyEffect, PrivacyInput, PrivacyState, privacy_step};
use detent_system::bus::Daemon;
use detent_system::modules::{ModulesLink, reread};
use detent_system::privacy::PrivacyLink;
use ds_settings::schema::KeyKind;
use std::sync::Arc;

/// What the person reads when the compositor is gone as a change is sent.
const NOT_AVAILABLE: &str = "Privacy isn\u{2019}t available right now. Try again in a moment.";

/// The rows whose value is read with the rows: the switches and the administrator's read-outs.
fn is_valued(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Toggle { .. } | KeyKind::Fixed { .. })
}

/// The Privacy pane's handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Privacy {
    modules: Arc<dyn ModulesLink>,
    link: Arc<dyn PrivacyLink>,
    module: Slot,
    live: Live<PrivacyState>,
}

pane!(Privacy, PrivacyState, follow);

impl Privacy {
    pub(crate) fn over(modules: Arc<dyn ModulesLink>, link: Arc<dyn PrivacyLink>) -> Privacy {
        Privacy {
            modules,
            link,
            module: Slot::default(),
            live: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: PrivacyInput) -> Pending {
        self.dispatch(input)
    }

    /// Follows the compositor's module until the bus goes away.
    async fn follow(&self) {
        follow_with(
            &*self.modules,
            &Daemon::compositor(),
            is_valued,
            async |followed| match followed {
                Followed::Opened(opened, described) => {
                    self.module.put(Some(opened));
                    self.dispatch(PrivacyInput::Found(Ok(described))).await;
                }
                Followed::Gone(absence) => {
                    self.module.put(None);
                    self.dispatch(PrivacyInput::Found(Err(absence))).await;
                }
                Followed::Changed(..) => {
                    let Some(opened) = self.module.get() else {
                        return;
                    };
                    let found = reread(&opened, is_valued).await;
                    self.dispatch(PrivacyInput::Found(found)).await;
                }
            },
        )
        .await;
    }
}

fn answer(result: Result<(), String>) -> PrivacyInput {
    match result {
        Ok(()) => PrivacyInput::Done,
        Err(why) => PrivacyInput::Failed(why),
    }
}

impl Machine for Privacy {
    type Input = PrivacyInput;
    type Effect = PrivacyEffect;

    fn step(state: PrivacyState, input: PrivacyInput) -> (PrivacyState, Vec<PrivacyEffect>) {
        privacy_step(state, input)
    }

    async fn run(&self, effect: PrivacyEffect) -> Vec<PrivacyInput> {
        let opened = self.module.get();
        match (effect, opened) {
            (PrivacyEffect::Tell(text), _) => {
                self.live.notice(Notice::Words(text));
                Vec::new()
            }
            (PrivacyEffect::Reload, Some(opened)) => {
                vec![PrivacyInput::Found(reread(&opened, is_valued).await)]
            }
            (PrivacyEffect::Set { path, to }, Some(opened)) => {
                vec![answer(self.link.set(&opened, &path, to).await)]
            }
            (PrivacyEffect::Forget(path), Some(opened)) => {
                vec![answer(self.link.forget(&opened, &path).await)]
            }
            (PrivacyEffect::Set { .. } | PrivacyEffect::Forget(_), None) => {
                vec![PrivacyInput::Failed(NOT_AVAILABLE.to_owned())]
            }
            (PrivacyEffect::Reload, None) => Vec::new(),
        }
    }
}
