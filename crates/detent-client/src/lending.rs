//! The switch "Let my other computers use this computer's models": turning it on asks first, then
//! sets sharing up through porter, and only when that worked is the setting written; turning it
//! off takes sharing away and clears the setting. The steps are `detent-model`'s `sharing_step`.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{Existing, Sharing, SharingEffect, SharingInput, sharing_step};
use detent_system::lending::Lender;

/// The switch's handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Lending {
    lender: Lender,
    live: Live<Sharing>,
}

pane!(Lending, Sharing, follow);

impl Lending {
    pub(crate) fn over(lender: Lender) -> Lending {
        Lending {
            lender,
            live: Live::default(),
        }
    }

    /// What is set up now.
    pub fn existing(&self) -> Existing {
        self.lender.existing()
    }

    /// Gives the switch `input` now; the returned work carries out what it asks for. A
    /// [`Notice::Recorded`] says when the setting may be written.
    pub fn send(&self, input: SharingInput) -> Pending {
        self.dispatch(input)
    }

    /// Nothing follows the switch: it moves when the person turns it.
    async fn follow(&self) {}
}

impl Machine for Lending {
    type Input = SharingInput;
    type Effect = SharingEffect;

    fn step(state: Sharing, input: SharingInput) -> (Sharing, Vec<SharingEffect>) {
        sharing_step(state, input)
    }

    async fn run(&self, effect: SharingEffect) -> Vec<SharingInput> {
        match effect {
            SharingEffect::Enable => {
                vec![SharingInput::Finished(self.lender.enable().await)]
            }
            SharingEffect::Disable => {
                vec![SharingInput::Finished(self.lender.disable().await)]
            }
            SharingEffect::Record(way) => {
                self.live.notice(Notice::Recorded(way));
                Vec::new()
            }
            SharingEffect::Tell(words) => {
                self.live.notice(Notice::Words(words));
                Vec::new()
            }
        }
    }
}
