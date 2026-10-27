//! Getting llama.cpp, vLLM or Strata when the person chooses one in "Runs with" and it is not
//! there: the engines as the Local row lists them, followed, and what the offer asks of the
//! desk. The steps are `detent-model`'s `engine_offer_step`.

use crate::live::Live;
use crate::pane::{Machine, Pending, pane};
use detent_model::{
    EngineOfferEffect, EngineOfferInput, EngineOfferState, EngineStanding, engine_offer_step,
};
use detent_system::engine_install::Desk;
use std::sync::Arc;
use std::time::Duration;

/// How often the engines are looked at while nothing is going on.
const IDLE: Duration = Duration::from_secs(2);

/// How often while a download is under way.
const BUSY: Duration = Duration::from_millis(400);

/// The engines' handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct EngineInstall {
    desk: Arc<dyn Desk>,
    live: Live<EngineOfferState>,
}

pane!(EngineInstall, EngineOfferState, follow);

impl EngineInstall {
    pub(crate) fn over(desk: Arc<dyn Desk>) -> EngineInstall {
        EngineInstall {
            desk,
            live: Live::default(),
        }
    }

    /// Gives the offer `input` now: the person chose an engine or took an action. The returned
    /// work carries out what it asks for.
    pub fn send(&self, input: EngineOfferInput) -> Pending {
        self.dispatch(input)
    }

    /// Looks at the engines, and again after a short wait, for as long as it is followed.
    async fn follow(&self) {
        loop {
            let rows = self.desk.rows().await;
            let busy = rows
                .iter()
                .any(|row| matches!(row.standing, EngineStanding::Installing(_)));
            self.dispatch(EngineOfferInput::Rows(rows)).await;
            tokio::time::sleep(if busy { BUSY } else { IDLE }).await;
        }
    }
}

impl Machine for EngineInstall {
    type Input = EngineOfferInput;
    type Effect = EngineOfferEffect;

    fn step(
        state: EngineOfferState,
        input: EngineOfferInput,
    ) -> (EngineOfferState, Vec<EngineOfferEffect>) {
        engine_offer_step(state, input)
    }

    async fn run(&self, effect: EngineOfferEffect) -> Vec<EngineOfferInput> {
        match effect {
            EngineOfferEffect::Install { engine, scope } => {
                match self.desk.install(&engine, scope) {
                    Ok(()) => Vec::new(),
                    Err(why) => vec![EngineOfferInput::NotStarted(why)],
                }
            }
            EngineOfferEffect::Cancel(engine) => {
                self.desk.cancel(&engine);
                Vec::new()
            }
            EngineOfferEffect::ChooseFolder => {
                vec![EngineOfferInput::Folder(self.desk.choose_folder().await)]
            }
            EngineOfferEffect::Point { engine, folder } => {
                vec![EngineOfferInput::Pointed(
                    self.desk.point(&engine, &folder).await,
                )]
            }
            EngineOfferEffect::Restart => {
                self.desk.restart().await;
                Vec::new()
            }
        }
    }
}
