//! What the in-memory fakes of the links share: a standing that is read, what was asked of it in
//! order, the words a refusal is made of, and the ticks that tell the ones that follow it.

use crate::lock::locked;
use crate::ticks::{Ticker, Ticks};
use detent_model::Absence;
use std::sync::{Arc, Mutex};

/// The shared part of a fake: `S` is what the service stands at, `A` what can be asked of it.
#[derive(Debug)]
pub(crate) struct Kit<S, A> {
    standing: Mutex<Result<S, Absence>>,
    asked: Mutex<Vec<A>>,
    refusal: Mutex<Option<String>>,
    listeners: Mutex<Vec<Ticker>>,
}

impl<S: Clone, A: Clone> Kit<S, A> {
    /// A kit standing as `standing`.
    pub(crate) fn new(standing: Result<S, Absence>) -> Arc<Kit<S, A>> {
        Arc::new(Kit {
            standing: Mutex::new(standing),
            asked: Mutex::default(),
            refusal: Mutex::default(),
            listeners: Mutex::default(),
        })
    }

    /// Where the service stands now.
    pub(crate) fn standing(&self) -> Result<S, Absence> {
        locked(&self.standing).clone()
    }

    /// Moves the service to `standing` and tells whoever follows it.
    pub(crate) fn set(&self, standing: Result<S, Absence>) {
        self.set_unseen(standing);
        self.tick();
    }

    /// Moves the service to `standing` without telling anyone.
    pub(crate) fn set_unseen(&self, standing: Result<S, Absence>) {
        *locked(&self.standing) = standing;
    }

    /// Changes the standing in place, when there is one, and tells whoever follows it.
    pub(crate) fn edit(&self, change: impl FnOnce(&mut S)) {
        if let Ok(standing) = &mut *locked(&self.standing) {
            change(standing);
        }
        self.tick();
    }

    /// Writes down what was asked.
    pub(crate) fn record(&self, asked: A) {
        locked(&self.asked).push(asked);
    }

    /// What was asked, in order.
    pub(crate) fn asked(&self) -> Vec<A> {
        locked(&self.asked).clone()
    }

    /// From now on asking is refused in these words; `None` takes it back.
    pub(crate) fn refuse_with(&self, words: Option<String>) {
        *locked(&self.refusal) = words;
    }

    /// The words asking is refused in, if it is.
    pub(crate) fn refusal(&self) -> Option<String> {
        locked(&self.refusal).clone()
    }

    /// `Ok` unless asking is refused.
    pub(crate) fn admit(&self) -> Result<(), String> {
        self.refusal().map_or(Ok(()), Err)
    }

    /// The ticks that say the standing changed.
    pub(crate) fn listen(&self) -> Ticks {
        let (listener, ticks) = Ticks::channel();
        locked(&self.listeners).push(listener);
        ticks
    }

    /// Tells every listener the standing may have changed.
    pub(crate) fn tick(&self) {
        locked(&self.listeners).retain(|listener| listener.tick().is_ok());
    }
}
