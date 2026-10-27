//! Where the reason footer's events come from.

use porter_infer::InferEvent;
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// The stream of `InferEvent`s the footer reads `Why` and `Declined` from. It is handed over once:
/// the first page that reads it owns it.
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

    /// The receiving end, if nobody has taken it.
    pub(crate) fn take(&self) -> Option<UnboundedReceiver<InferEvent>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}
