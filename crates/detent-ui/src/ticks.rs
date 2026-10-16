//! "Something changed, look again": a wake-up that cannot pile up and ends its producers when
//! the one waiting for it goes (audit DETENT-09).
//!
//! A page that follows a service by its signals keeps relay tasks that turn each signal into a
//! tick. Before, the ticks queued without bound (a service that floods signals filled memory
//! faster than the page re-read) and the relays outlived the page until the service's next
//! signal, so every visit leaked a connection's worth of tasks. Now at most one tick waits (the
//! re-read that answers it sees everything that changed before it), and dropping [`Ticks`] aborts
//! its relays.

use std::fmt;
use tokio::sync::mpsc::{Receiver, Sender, channel, error::TrySendError};
use tokio::task::JoinHandle;

/// Nobody is waiting for ticks any more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gone;

/// The producing end of [`Ticks`].
#[derive(Debug, Clone)]
pub struct Ticker(Sender<()>);

impl Ticker {
    /// Wakes the listener. A tick that is already waiting absorbs this one; `Err` when the
    /// listener has gone, which is the producer's cue to stop.
    pub fn tick(&self) -> Result<(), Gone> {
        match self.0.try_send(()) {
            Ok(()) | Err(TrySendError::Full(())) => Ok(()),
            Err(TrySendError::Closed(())) => Err(Gone),
        }
    }
}

/// The waiting end: ticks that arrive while one waits are folded into it.
pub struct Ticks {
    receiver: Receiver<()>,
    relays: Vec<JoinHandle<()>>,
}

impl fmt::Debug for Ticks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ticks").finish_non_exhaustive()
    }
}

impl Ticks {
    /// A ticker and the ticks it wakes.
    pub fn channel() -> (Ticker, Ticks) {
        let (sender, receiver) = channel(1);
        (
            Ticker(sender),
            Ticks {
                receiver,
                relays: Vec::new(),
            },
        )
    }

    /// These ticks, with `relay` ended when they are dropped.
    #[must_use]
    pub fn with_relay(mut self, relay: JoinHandle<()>) -> Ticks {
        self.relays.push(relay);
        self
    }

    /// The next tick; `None` when every producer has gone.
    pub async fn recv(&mut self) -> Option<()> {
        self.receiver.recv().await
    }
}

impl Drop for Ticks {
    fn drop(&mut self) {
        for relay in &self.relays {
            relay.abort();
        }
    }
}
