//! The Displays pane: following the compositor and asking it, and the brightness of the displays
//! the shell lists. The steps are `detent-model`'s `displays_step`.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{
    Brightness, BrightnessList, DisplaysEffect, DisplaysInput, DisplaysState, OutputId,
    displays_step,
};
use detent_system::displays::DisplaysLink;
use std::sync::Arc;

/// The Displays pane's handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Displays {
    link: Arc<dyn DisplaysLink>,
    live: Live<DisplaysState>,
    /// The displays the shell lists a brightness for, and where each stands.
    brightness: Live<BrightnessList>,
}

pane!(Displays, DisplaysState, follow_both);

impl Displays {
    pub(crate) fn over(link: Arc<dyn DisplaysLink>) -> Displays {
        Displays {
            link,
            live: Live::default(),
            brightness: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: DisplaysInput) -> Pending {
        self.dispatch(input)
    }

    /// The brightness of each display that has one, as a live value of its own.
    pub fn brightness(&self) -> &Live<BrightnessList> {
        &self.brightness
    }

    /// Moves `output`'s slider to `level` now and asks the shell to set it; the shell's word on
    /// the next change decides where the slider rests.
    pub fn set_brightness(&self, output: OutputId, level: Brightness) -> Pending {
        self.brightness.update(|list| list.with(&output, level));
        let me = self.clone();
        Box::pin(async move {
            if let Err(words) = me.link.set_brightness(output, level).await {
                me.live.notice(Notice::Words(words));
                me.reread_brightness().await;
            }
        })
    }

    async fn follow_both(&self) {
        tokio::join!(self.follow(), self.follow_brightness());
    }

    /// Reads the compositor now and each time the displays may have changed.
    async fn follow(&self) {
        match self.link.changes().await {
            Err(missing) => self.dispatch(DisplaysInput::Heard(Err(missing))).await,
            Ok(mut ticks) => {
                self.reread().await;
                while ticks.recv().await.is_some() {
                    self.reread().await;
                }
            }
        }
    }

    /// Reads the shell's brightness list now and each time it says one changed. Without a shell
    /// the list stays empty and the pane shows no sliders.
    async fn follow_brightness(&self) {
        let Ok(mut ticks) = self.link.brightness_changes().await else {
            return;
        };
        self.reread_brightness().await;
        while ticks.recv().await.is_some() {
            self.reread_brightness().await;
        }
    }

    async fn reread_brightness(&self) {
        let found = self.link.brightness().await.unwrap_or_default();
        self.brightness.update(|_| found);
    }

    async fn reread(&self) {
        self.dispatch(DisplaysInput::Heard(self.link.read().await))
            .await;
    }
}

impl Machine for Displays {
    type Input = DisplaysInput;
    type Effect = DisplaysEffect;

    fn step(state: DisplaysState, input: DisplaysInput) -> (DisplaysState, Vec<DisplaysEffect>) {
        displays_step(state, input)
    }

    async fn run(&self, effect: DisplaysEffect) -> Vec<DisplaysInput> {
        match effect {
            DisplaysEffect::Apply(change) => {
                let output = change.output.clone();
                let done = self.link.apply(change).await;
                vec![
                    DisplaysInput::Applied(output, done),
                    DisplaysInput::Heard(self.link.read().await),
                ]
            }
            DisplaysEffect::Tell(words) => {
                self.live.notice(Notice::Words(words));
                Vec::new()
            }
        }
    }
}
