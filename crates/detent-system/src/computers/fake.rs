//! The computers over inferd from memory: the lists, the candidates, and each change asked.

use super::{Change, ChangeFailure, ComputersLink};
use crate::fake_kit::Kit;
use crate::lock::locked;
use crate::ticks::Ticks;
use detent_model::{Absence, ComputersFound, ComputersReason, ServerModel, Work};
use porter_tailnet::ComputerCandidate;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Offers {
    candidates: Vec<ComputerCandidate>,
    models: Vec<ServerModel>,
    failure: Option<ChangeFailure>,
}

/// A service that answers from what it was given.
#[derive(Debug, Clone)]
pub struct FakeComputers {
    kit: Arc<Kit<ComputersFound, Change>>,
    offers: Arc<Mutex<Offers>>,
}

impl FakeComputers {
    /// A service listing these computers and guests.
    pub fn listing(found: ComputersFound) -> FakeComputers {
        FakeComputers {
            kit: Kit::new(Ok(found)),
            offers: Arc::default(),
        }
    }

    /// No service, for this reason.
    pub fn missing(why: Absence) -> FakeComputers {
        FakeComputers {
            kit: Kit::new(Err(why)),
            offers: Arc::default(),
        }
    }

    /// Tailscale offers these computers to add.
    pub fn offers(&self, candidates: Vec<ComputerCandidate>) {
        locked(&self.offers).candidates = candidates;
    }

    /// A server may run these models.
    pub fn serves(&self, models: Vec<ServerModel>) {
        locked(&self.offers).models = models;
    }

    /// The next changes fail like this.
    pub fn fails(&self, failure: ChangeFailure) {
        locked(&self.offers).failure = Some(failure);
    }

    /// The changes that were asked, in order.
    pub fn asked(&self) -> Vec<Change> {
        self.kit.asked()
    }
}

impl ComputersLink for FakeComputers {
    fn read(&self) -> Work<'_, Result<ComputersFound, Absence>> {
        Box::pin(std::future::ready(self.kit.standing()))
    }

    fn candidates(&self) -> Work<'_, Result<Vec<ComputerCandidate>, ComputersReason>> {
        let found = locked(&self.offers).candidates.clone();
        Box::pin(std::future::ready(Ok(found)))
    }

    fn models(&self) -> Work<'_, Vec<ServerModel>> {
        let found = locked(&self.offers).models.clone();
        Box::pin(std::future::ready(found))
    }

    fn change(&self, change: Change) -> Work<'_, Result<(), ChangeFailure>> {
        self.kit.record(change);
        let failure = locked(&self.offers).failure.take();
        Box::pin(std::future::ready(failure.map_or(Ok(()), Err)))
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Absence>> {
        Box::pin(std::future::ready(Ok(self.kit.listen())))
    }
}
