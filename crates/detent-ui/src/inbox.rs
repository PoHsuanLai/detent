//! What the window is told from outside while it runs: a page to open, or files that changed
//! under it. The provider that answers the desktop's intents sends these; the window takes them
//! in order.

use detent_model::{AccountTarget, PageSlug};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, error::SendError, unbounded_channel};

/// One thing the window is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    /// Show this page, with an empty search.
    OpenPage(PageSlug),
    /// A settings file changed behind the window's back: read every file again.
    Reload,
    /// Show the Accounts page at this account's detail, or its list.
    OpenAccount(AccountTarget),
    /// Show the window and raise it; the token is the one the compositor handed the launcher.
    Raise(Option<String>),
}

/// The sending end: held by whoever tells the window.
#[derive(Debug, Clone)]
pub struct Outbox(UnboundedSender<Notice>);

impl Outbox {
    /// Tells the window `notice`; fails only when the window has gone.
    pub fn send(&self, notice: Notice) -> Result<(), SendError<Notice>> {
        self.0.send(notice)
    }
}

/// The receiving end: in the window's [`Env`](crate::Env), taken once by the root.
#[derive(Debug, Clone, Default)]
pub struct Inbox(Arc<Mutex<Option<UnboundedReceiver<Notice>>>>);

impl Inbox {
    /// The receiver, the first time it is asked for; `None` after, or for an empty inbox.
    pub fn take(&self) -> Option<UnboundedReceiver<Notice>> {
        let mut held: MutexGuard<'_, _> = self.0.lock().unwrap_or_else(|p| p.into_inner());
        held.take()
    }
}

/// A new pair: send on the [`Outbox`], the window reads the [`Inbox`].
pub fn inbox() -> (Outbox, Inbox) {
    let (sender, receiver) = unbounded_channel();
    (Outbox(sender), Inbox(Arc::new(Mutex::new(Some(receiver)))))
}
