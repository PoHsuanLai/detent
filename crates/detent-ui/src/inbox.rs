//! What the window is told from outside while it runs: a page to open, or files that changed
//! under it. The provider that answers the desktop's intents sends these; the window takes them
//! in order.

use detent_model::{AccountTarget, PageSlug};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc::{
    Receiver, Sender, channel,
    error::{SendError, TrySendError},
};

/// How many notices wait for the window at most. Whoever the activation object admits (the shell,
/// another Settings, the agent tier) can call it as often as it likes; past this the extra notices
/// are dropped (the window is already behind on the ones that wait), so the queue cannot grow
/// (DETENT-16). A person opens a page a second at most, so a few dozen is a long way behind.
pub(crate) const NOTICE_INBOX: usize = 32;

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
pub struct Outbox {
    sender: Sender<Notice>,
    /// How many notices were dropped since the last one that fit, so a flood is said once.
    dropped: Arc<AtomicUsize>,
}

impl Outbox {
    /// Tells the window `notice`; fails only when the window has gone. When [`NOTICE_INBOX`]
    /// notices are already waiting the new one is dropped and this still succeeds; the first one
    /// dropped after the window caught up is said in the log, the rest of that flood are not.
    pub fn send(&self, notice: Notice) -> Result<(), SendError<Notice>> {
        match self.sender.try_send(notice) {
            Ok(()) => {
                self.dropped.store(0, Ordering::Relaxed);
                Ok(())
            }
            Err(TrySendError::Full(_)) => {
                if self.dropped.fetch_add(1, Ordering::Relaxed) == 0 {
                    eprintln!(
                        "detent: {NOTICE_INBOX} notices are waiting for the window; more are dropped until it catches up"
                    );
                }
                Ok(())
            }
            Err(TrySendError::Closed(notice)) => Err(SendError(notice)),
        }
    }

    /// How many notices were dropped since the last one that fit.
    #[cfg(test)]
    pub(crate) fn dropped(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }
}

/// The receiving end: in the window's [`Env`](crate::Env), taken once by the root.
#[derive(Debug, Clone, Default)]
pub struct Inbox(Arc<Mutex<Option<Receiver<Notice>>>>);

impl Inbox {
    /// The receiver, the first time it is asked for; `None` after, or for an empty inbox.
    pub fn take(&self) -> Option<Receiver<Notice>> {
        let mut held: MutexGuard<'_, _> = self.0.lock().unwrap_or_else(|p| p.into_inner());
        held.take()
    }
}

/// A new pair: send on the [`Outbox`], the window reads the [`Inbox`].
pub fn inbox() -> (Outbox, Inbox) {
    let (sender, receiver) = channel(NOTICE_INBOX);
    let outbox = Outbox {
        sender,
        dropped: Arc::default(),
    };
    (outbox, Inbox(Arc::new(Mutex::new(Some(receiver)))))
}
