//! Where the reason footer's events come from.

use detent_model::Reason;
use porter_infer::InferEvent;
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// The stream of `InferEvent`s the footer reads `Why` and `Declined` from. It is handed over once:
/// the first window that reads it owns it.
#[derive(Clone, Default)]
pub struct ReasonFeed(Arc<Mutex<Option<UnboundedReceiver<InferEvent>>>>);

impl fmt::Debug for ReasonFeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReasonFeed")
    }
}

impl ReasonFeed {
    /// A feed and the sender that fills it.
    pub fn channel() -> (UnboundedSender<InferEvent>, ReasonFeed) {
        let (sender, receiver) = unbounded_channel();
        (sender, ReasonFeed(Arc::new(Mutex::new(Some(receiver)))))
    }

    /// The reasons, if nobody has taken the feed yet.
    pub(crate) fn take(&self) -> Reasons {
        let taken = self.0.lock().unwrap_or_else(PoisonError::into_inner).take();
        Reasons(taken)
    }
}

/// The reasons the footer says, one after the other.
#[derive(Debug)]
pub struct Reasons(Option<UnboundedReceiver<InferEvent>>);

impl Reasons {
    /// The next reason; `None` when the feed was taken before or has ended.
    pub async fn next(&mut self) -> Option<Reason> {
        let events = self.0.as_mut()?;
        while let Some(event) = events.recv().await {
            if let Some(reason) = Reason::of(&event) {
                return Some(reason);
            }
        }
        None
    }
}
