//! inferd's running models from memory: the list, and each start and stop that was asked.

use super::RunningLink;
use crate::fake_kit::Kit;
use crate::ticks::Ticks;
use detent_model::{Absence, ModelRun, RunAction, RunStanding, Work};
use std::sync::Arc;

/// A service that answers from what it was given.
#[derive(Debug, Clone)]
pub struct FakeRunning(Arc<Kit<Vec<ModelRun>, (String, RunAction)>>);

impl FakeRunning {
    /// A service running these models.
    pub fn running(rows: Vec<ModelRun>) -> FakeRunning {
        FakeRunning(Kit::new(Ok(rows)))
    }

    /// No service, for this reason.
    pub fn missing(why: Absence) -> FakeRunning {
        FakeRunning(Kit::new(Err(why)))
    }

    /// The next starts and stops are refused, in these words.
    pub fn refuses(&self, words: &str) {
        self.0.refuse_with(Some(words.to_owned()));
    }

    /// What was asked, in order.
    pub fn asked(&self) -> Vec<(String, RunAction)> {
        self.0.asked()
    }
}

impl RunningLink for FakeRunning {
    fn read(&self) -> Work<'_, Result<Vec<ModelRun>, Absence>> {
        Box::pin(std::future::ready(self.0.standing()))
    }

    fn act(&self, model: String, action: RunAction) -> Work<'_, Result<(), String>> {
        self.0.record((model.clone(), action));
        let done = self.0.admit();
        if done.is_ok() {
            let (standing, memory_mib) = match action {
                RunAction::Start => (RunStanding::Ready, 1),
                RunAction::Stop => (RunStanding::Stopped, 0),
            };
            self.0.edit(|rows| {
                for row in rows.iter_mut().filter(|row| row.model == model) {
                    row.standing = standing.clone();
                    row.memory_mib = memory_mib;
                }
            });
        }
        Box::pin(std::future::ready(done))
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Absence>> {
        Box::pin(std::future::ready(Ok(self.0.listen())))
    }
}
