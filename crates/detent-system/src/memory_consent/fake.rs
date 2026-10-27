//! The memory service's controls from memory: what waits for the person, and what was confirmed.

use super::{MemoryFailure, MemoryLink};
use crate::lock::locked;
use almanac_core::WaitingSetting;
use detent_model::{Confinement, SettingPair, Work};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Book {
    waiting: Vec<WaitingSetting>,
    confinement: Confinement,
    confirmed: Vec<Vec<SettingPair>>,
    failure: Option<MemoryFailure>,
}

/// A memory that answers from what it was given.
#[derive(Debug, Clone)]
pub struct FakeMemory(Arc<Mutex<Book>>);

impl FakeMemory {
    /// A memory where these values wait, confining itself like this.
    pub fn holding(waiting: Vec<WaitingSetting>, confinement: Confinement) -> FakeMemory {
        FakeMemory(Arc::new(Mutex::new(Book {
            waiting,
            confinement,
            confirmed: Vec::new(),
            failure: None,
        })))
    }

    /// Every call fails like this from now on.
    pub fn fails(&self, failure: MemoryFailure) {
        locked(&self.0).failure = Some(failure);
    }

    /// The pairs that were confirmed, one entry per call.
    pub fn confirmed(&self) -> Vec<Vec<SettingPair>> {
        locked(&self.0).confirmed.clone()
    }
}

impl MemoryLink for FakeMemory {
    fn waiting(&self) -> Work<'_, Result<Vec<WaitingSetting>, MemoryFailure>> {
        let book = locked(&self.0);
        let found = book.failure.map_or_else(|| Ok(book.waiting.clone()), Err);
        Box::pin(std::future::ready(found))
    }

    fn confirm(&self, pairs: Vec<SettingPair>) -> Work<'_, Result<(), MemoryFailure>> {
        let mut book = locked(&self.0);
        let done = match book.failure {
            Some(failure) => Err(failure),
            None => {
                book.waiting
                    .retain(|waiting| pairs.iter().all(|pair| pair.key != waiting.key));
                Ok(())
            }
        };
        book.confirmed.push(pairs);
        Box::pin(std::future::ready(done))
    }

    fn confinement(&self) -> Work<'_, Result<Confinement, MemoryFailure>> {
        let book = locked(&self.0);
        let found = book.failure.map_or(Ok(book.confinement), Err);
        Box::pin(std::future::ready(found))
    }
}
