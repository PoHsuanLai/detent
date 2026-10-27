//! The bridge from `detent-client` to Dioxus, and nothing else: a pane's live value becomes a
//! signal, what the pane tells the person becomes a toast, and the work that keeps the pane
//! current runs while the component is mounted. A view reads the signal, draws it, and gives the
//! pane its inputs; it holds no follow loop, no deadline and no step.

use crate::copy;
use crate::env::Env;
use detent_client::{Client, Heard, Live, Pane};
use dioxus::prelude::*;
use ds::prelude::*;

/// The client the window was started with, from the context the launcher provided.
pub(crate) fn use_client() -> Client {
    use_context::<Env>().client
}

/// A pane followed by the component that asked for it.
pub(crate) struct Lived<P: Pane> {
    /// The pane: where the person's inputs go.
    pub pane: P,
    /// What the pane's state is now.
    pub state: Signal<P::State>,
}

impl<P: Pane> Clone for Lived<P> {
    fn clone(&self) -> Self {
        Lived {
            pane: self.pane.clone(),
            state: self.state,
        }
    }
}

/// The signal of `live`'s state, kept current while the component is mounted; what the pane tells
/// the person is shown as a toast.
pub(crate) fn use_events<S: Clone + 'static>(live: &Live<S>) -> Signal<S> {
    let toasts = use_toasts();
    let state = use_signal(|| live.state());
    use_future({
        let events = live.events();
        move || {
            let events = events.clone();
            let mut state = state;
            async move {
                while let Some(heard) = events.next().await {
                    match heard {
                        Heard::State(next) => state.set(next),
                        Heard::Notice(notice) => {
                            if let Some(words) = copy::notice(&notice) {
                                toasts.push(words, None);
                            }
                        }
                    }
                }
            }
        }
    });
    state
}

/// The pane `make` takes from the client, followed while the component is mounted: its work runs,
/// and its state is a signal.
pub(crate) fn use_live<P: Pane>(make: impl FnOnce(&Client) -> P) -> Lived<P> {
    let client = use_client();
    let pane = use_hook(|| make(&client));
    let state = use_events(pane.live());
    use_future({
        let pane = pane.clone();
        move || {
            let pane = pane.clone();
            async move { pane.watch().await }
        }
    });
    Lived { pane, state }
}
