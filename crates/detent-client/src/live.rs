//! A pane's live value: the state the window draws, and the stream of what changed in it. The
//! client steps the state with the model's `step`; the window only reads what arrives.

use crate::notice::Notice;
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use tokio::sync::Mutex as Waiting;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// What a pane tells its window, in the order it happens.
#[derive(Debug, Clone)]
pub enum Heard<S> {
    /// The pane's state is now this.
    State(S),
    /// The person is told this.
    Notice(Notice),
}

/// The receiving end of a [`Live`]: one window reads it.
pub struct Events<S>(Arc<Waiting<UnboundedReceiver<Heard<S>>>>);

impl<S> Clone for Events<S> {
    fn clone(&self) -> Self {
        Events(Arc::clone(&self.0))
    }
}

impl<S> fmt::Debug for Events<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Events")
    }
}

impl<S> Events<S> {
    /// The next thing the pane tells; `None` once every holder of the pane has gone.
    pub async fn next(&self) -> Option<Heard<S>> {
        self.0.lock().await.recv().await
    }
}

/// A pane's state, and the stream that carries each change of it to the window.
pub struct Live<S> {
    state: Arc<Mutex<S>>,
    out: UnboundedSender<Heard<S>>,
    events: Events<S>,
}

impl<S> Clone for Live<S> {
    fn clone(&self) -> Self {
        Live {
            state: Arc::clone(&self.state),
            out: self.out.clone(),
            events: self.events.clone(),
        }
    }
}

impl<S> fmt::Debug for Live<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Live").finish_non_exhaustive()
    }
}

impl<S: Clone + Default> Default for Live<S> {
    fn default() -> Self {
        Live::starting(S::default())
    }
}

impl<S: Clone> Live<S> {
    /// A live value that starts as `state`.
    pub fn starting(state: S) -> Live<S> {
        let (out, receiver) = unbounded_channel();
        Live {
            state: Arc::new(Mutex::new(state)),
            out,
            events: Events(Arc::new(Waiting::new(receiver))),
        }
    }

    /// The state now.
    pub fn state(&self) -> S {
        self.lock().clone()
    }

    /// What the pane tells, for the window to read.
    pub fn events(&self) -> Events<S> {
        self.events.clone()
    }

    /// Steps the state with `step` on `input` and tells the window; the effects it asks for.
    pub(crate) fn step<I, E>(&self, step: fn(S, I) -> (S, Vec<E>), input: I) -> Vec<E> {
        let mut state = self.lock();
        let (next, effects) = step(state.clone(), input);
        *state = next.clone();
        let _gone = self.out.send(Heard::State(next));
        effects
    }

    /// Replaces the state by what `change` makes of it, and tells the window.
    pub(crate) fn update(&self, change: impl FnOnce(S) -> S) {
        let mut state = self.lock();
        let next = change(state.clone());
        *state = next.clone();
        let _gone = self.out.send(Heard::State(next));
    }

    /// Tells the window to tell the person.
    pub(crate) fn notice(&self, notice: Notice) {
        let _gone = self.out.send(Heard::Notice(notice));
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, S> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
