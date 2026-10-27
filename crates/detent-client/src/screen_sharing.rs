//! Screen sharing: the apps the portal lets share the screen without asking again, with a Remove
//! on each. The steps are `detent-model`'s `screen_sharing_step`.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{
    ScreenSharingEffect, ScreenSharingInput, ScreenSharingState, screen_sharing_step,
};
use detent_system::screen_sharing::ScreenSharingLink;
use std::sync::Arc;

/// The pane's handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct ScreenSharing {
    link: Arc<dyn ScreenSharingLink>,
    live: Live<ScreenSharingState>,
}

pane!(ScreenSharing, ScreenSharingState, follow);

impl ScreenSharing {
    pub(crate) fn over(link: Arc<dyn ScreenSharingLink>) -> ScreenSharing {
        ScreenSharing {
            link,
            live: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: ScreenSharingInput) -> Pending {
        self.dispatch(input)
    }

    /// Reads the list once, when the pane appears.
    async fn follow(&self) {
        self.dispatch(ScreenSharingInput::Found(self.link.list().await))
            .await;
    }
}

impl Machine for ScreenSharing {
    type Input = ScreenSharingInput;
    type Effect = ScreenSharingEffect;

    fn step(
        state: ScreenSharingState,
        input: ScreenSharingInput,
    ) -> (ScreenSharingState, Vec<ScreenSharingEffect>) {
        screen_sharing_step(state, input)
    }

    async fn run(&self, effect: ScreenSharingEffect) -> Vec<ScreenSharingInput> {
        match effect {
            ScreenSharingEffect::Tell(text) => {
                self.live.notice(Notice::Words(text));
                Vec::new()
            }
            ScreenSharingEffect::Reload => {
                vec![ScreenSharingInput::Found(self.link.list().await)]
            }
            ScreenSharingEffect::Revoke(app) => {
                vec![ScreenSharingInput::Removed(self.link.revoke(app).await)]
            }
        }
    }
}
