//! The Spaces list: one row per Space with Remove. The steps are `detent-model`'s `spaces_step`.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{SpacesEffect, SpacesInput, SpacesState, spaces_step};
use detent_system::spaces::SpacesLink;
use std::sync::Arc;

/// The Spaces' handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Spaces {
    link: Arc<dyn SpacesLink>,
    live: Live<SpacesState>,
}

pane!(Spaces, SpacesState, follow);

impl Spaces {
    pub(crate) fn over(link: Arc<dyn SpacesLink>) -> Spaces {
        Spaces {
            link,
            live: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: SpacesInput) -> Pending {
        self.dispatch(input)
    }

    /// Reads the list once, when the pane appears.
    async fn follow(&self) {
        self.dispatch(SpacesInput::Found(self.link.list().await))
            .await;
    }
}

impl Machine for Spaces {
    type Input = SpacesInput;
    type Effect = SpacesEffect;

    fn step(state: SpacesState, input: SpacesInput) -> (SpacesState, Vec<SpacesEffect>) {
        spaces_step(state, input)
    }

    async fn run(&self, effect: SpacesEffect) -> Vec<SpacesInput> {
        match effect {
            SpacesEffect::Tell(text) => {
                self.live.notice(Notice::Words(text));
                Vec::new()
            }
            SpacesEffect::Reload => vec![SpacesInput::Found(self.link.list().await)],
            SpacesEffect::Remove { space, removal } => {
                vec![SpacesInput::Removed(self.link.remove(space, removal).await)]
            }
        }
    }
}
