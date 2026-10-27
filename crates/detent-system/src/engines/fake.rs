//! The assistant's engine list from memory: where llama.cpp stands, and how a restart ends.

use super::EnginesLink;
use crate::lock::locked;
use detent_model::{LlamaCpp, Restarted, Work};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Book {
    llama: LlamaCpp,
    restart: Restarted,
    restarts: usize,
}

/// An assistant that answers from what it was given.
#[derive(Debug, Clone)]
pub struct FakeEngines(Arc<Mutex<Book>>);

impl FakeEngines {
    /// An assistant that lists llama.cpp as `llama`; a restart succeeds.
    pub fn listing(llama: LlamaCpp) -> FakeEngines {
        FakeEngines(Arc::new(Mutex::new(Book {
            llama,
            restart: Restarted::Done,
            restarts: 0,
        })))
    }

    /// llama.cpp stands like this from now on.
    pub fn lists(&self, llama: LlamaCpp) {
        locked(&self.0).llama = llama;
    }

    /// Restarts end like this from now on.
    pub fn restarts_end(&self, restart: Restarted) {
        locked(&self.0).restart = restart;
    }

    /// How many restarts were asked.
    pub fn restarts(&self) -> usize {
        locked(&self.0).restarts
    }
}

impl EnginesLink for FakeEngines {
    fn look(&self) -> Work<'_, LlamaCpp> {
        let llama = locked(&self.0).llama;
        Box::pin(std::future::ready(llama))
    }

    fn restart(&self) -> Work<'_, Restarted> {
        let mut book = locked(&self.0);
        book.restarts += 1;
        Box::pin(std::future::ready(book.restart))
    }
}
