//! What every pane is to the window (a live value and the work that keeps it current), and what
//! most panes are inside: a model's `step`, and the effects it asks for carried out over a link.

use crate::live::Live;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;

/// Work that runs on the window's executor and ends when its pane is no longer followed.
pub type Pending = Pin<Box<dyn Future<Output = ()>>>;

/// A pane the window can follow.
pub trait Pane: Clone + 'static {
    /// What the window draws.
    type State: Clone + Default + 'static;

    /// The live value; read its events for what changes.
    fn live(&self) -> &Live<Self::State>;

    /// Follows the service for as long as the returned work is polled: reads now, and again each
    /// time it may have changed. The window runs it while the pane is shown.
    fn watch(&self) -> Pending;
}

/// A pane whose state moves by the model's `step`, with the effects it asks for carried out here.
pub(crate) trait Machine: Pane {
    /// What can happen to the state.
    type Input;
    /// What a step asks to be done.
    type Effect;

    /// The model's step.
    fn step(state: Self::State, input: Self::Input) -> (Self::State, Vec<Self::Effect>);

    /// Carries out one effect; the inputs it leads to, in order.
    async fn run(&self, effect: Self::Effect) -> Vec<Self::Input>;

    /// Steps the state on `input` at once, and tells the window.
    fn feed(&self, input: Self::Input) -> Vec<Self::Effect> {
        self.live().step(Self::step, input)
    }

    /// Carries out `effects`, then every input they lead to, until nothing is left to do.
    async fn carry(&self, effects: Vec<Self::Effect>) {
        let mut queue = VecDeque::new();
        for effect in effects {
            queue.extend(self.run(effect).await);
        }
        while let Some(input) = queue.pop_front() {
            for effect in self.feed(input) {
                queue.extend(self.run(effect).await);
            }
        }
    }

    /// Steps the state on `input` now; the returned work carries out what that asks for.
    fn dispatch(&self, input: Self::Input) -> Pending
    where
        Self: Sized,
        Self::Input: 'static,
        Self::Effect: 'static,
    {
        let effects = self.feed(input);
        let me = self.clone();
        Box::pin(async move { me.carry(effects).await })
    }
}

/// Makes `$pane` a [`Pane`] over `$state` whose `watch` is its `$follow` method.
macro_rules! pane {
    ($pane:ty, $state:ty, $follow:ident) => {
        impl $crate::pane::Pane for $pane {
            type State = $state;

            fn live(&self) -> &$crate::live::Live<$state> {
                &self.live
            }

            fn watch(&self) -> $crate::pane::Pending {
                let me = self.clone();
                Box::pin(async move { me.$follow().await })
            }
        }
    };
}
pub(crate) use pane;
