//! What the window is told from outside while it runs: a page to open, or files that changed
//! under it. The provider that answers the desktop's intents sends these; the window takes them
//! in order. The queue is plain std (a mutex and a waker), so this crate reaches no runtime.

use crate::{AccountTarget, PageSlug};
use ds_settings::target::SettingsTarget;
use std::collections::VecDeque;
use std::fmt;
use std::future::poll_fn;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Poll, Waker};

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
    /// Show where this target points: a page, a key's row, or an account.
    OpenTarget(SettingsTarget),
    /// A settings file changed behind the window's back: read every file again.
    Reload,
    /// Show the Accounts page at this account's detail, or its list.
    OpenAccount(AccountTarget),
    /// Show the window and raise it; the token is the one the compositor handed the launcher.
    Raise(Option<String>),
}

/// There is no window to tell: the app is closing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowGone;

impl fmt::Display for WindowGone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the Settings window has gone")
    }
}

impl std::error::Error for WindowGone {}

/// The notices that wait, and who is still on either end.
#[derive(Debug)]
struct Queue {
    waiting: VecDeque<Notice>,
    /// The window's task, to wake when a notice arrives or the last sender goes.
    waker: Option<Waker>,
    /// How many [`Outbox`]es are alive.
    senders: usize,
    /// Whether the window was dropped: nothing sent now will be heard.
    receiver_gone: bool,
}

type Shared = Arc<Mutex<Queue>>;

fn locked(shared: &Shared) -> MutexGuard<'_, Queue> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

fn wake(queue: &mut Queue) {
    if let Some(waker) = queue.waker.take() {
        waker.wake();
    }
}

/// The sending end: held by whoever tells the window.
#[derive(Debug)]
pub struct Outbox {
    queue: Shared,
    /// How many notices were dropped since the last one that fit, so a flood is said once.
    dropped: Arc<AtomicUsize>,
}

impl Clone for Outbox {
    fn clone(&self) -> Outbox {
        locked(&self.queue).senders += 1;
        Outbox {
            queue: self.queue.clone(),
            dropped: self.dropped.clone(),
        }
    }
}

impl Drop for Outbox {
    fn drop(&mut self) {
        let mut queue = locked(&self.queue);
        queue.senders -= 1;
        if queue.senders == 0 {
            wake(&mut queue);
        }
    }
}

impl Outbox {
    /// Tells the window `notice`; fails only when the window has gone. When [`NOTICE_INBOX`]
    /// notices are already waiting the new one is dropped and this still succeeds; the first one
    /// dropped after the window caught up is said in the log, the rest of that flood are not.
    pub fn send(&self, notice: Notice) -> Result<(), WindowGone> {
        let mut queue = locked(&self.queue);
        if queue.receiver_gone {
            return Err(WindowGone);
        }
        if queue.waiting.len() >= NOTICE_INBOX {
            drop(queue);
            if self.dropped.fetch_add(1, Ordering::Relaxed) == 0 {
                eprintln!(
                    "detent: {NOTICE_INBOX} notices are waiting for the window; more are dropped until it catches up"
                );
            }
            return Ok(());
        }
        queue.waiting.push_back(notice);
        wake(&mut queue);
        drop(queue);
        self.dropped.store(0, Ordering::Relaxed);
        Ok(())
    }

    /// How many notices were dropped since the last one that fit.
    #[cfg(test)]
    pub(crate) fn dropped(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }
}

/// The window's end of the queue.
#[derive(Debug)]
pub struct Receiver {
    queue: Shared,
}

impl Receiver {
    /// The next notice if one waits.
    pub fn try_recv(&mut self) -> Option<Notice> {
        locked(&self.queue).waiting.pop_front()
    }

    /// The next notice, waiting for it; `None` once every [`Outbox`] is gone and none waits.
    pub async fn recv(&mut self) -> Option<Notice> {
        poll_fn(|cx| {
            let mut queue = locked(&self.queue);
            match queue.waiting.pop_front() {
                Some(notice) => Poll::Ready(Some(notice)),
                None if queue.senders == 0 => Poll::Ready(None),
                None => {
                    queue.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        })
        .await
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        let mut queue = locked(&self.queue);
        queue.receiver_gone = true;
        queue.waiting.clear();
    }
}

/// The receiving end: in the window's environment, taken once by the root.
#[derive(Debug, Clone, Default)]
pub struct Inbox(Arc<Mutex<Option<Receiver>>>);

impl Inbox {
    /// The receiver, the first time it is asked for; `None` after, or for an empty inbox.
    pub fn take(&self) -> Option<Receiver> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

/// A queue with one sender and its receiver.
fn pair() -> (Outbox, Receiver) {
    let queue = Arc::new(Mutex::new(Queue {
        waiting: VecDeque::new(),
        waker: None,
        senders: 1,
        receiver_gone: false,
    }));
    let outbox = Outbox {
        queue: queue.clone(),
        dropped: Arc::default(),
    };
    (outbox, Receiver { queue })
}

/// A new pair: send on the [`Outbox`], the window reads the [`Inbox`].
pub fn inbox() -> (Outbox, Inbox) {
    let (outbox, receiver) = pair();
    (outbox, Inbox(Arc::new(Mutex::new(Some(receiver)))))
}

impl Inbox {
    /// A new queue for the next window to take from this inbox, and the end to send to it. A
    /// service that opens its windows one after another (`detent --service`) hands each the same
    /// [`Inbox`] through its environment; the receiver a window left behind is replaced.
    pub fn refill(&self) -> Outbox {
        let (outbox, receiver) = pair();
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(receiver);
        outbox
    }
}
