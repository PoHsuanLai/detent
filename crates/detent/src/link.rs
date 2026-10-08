//! The window, as the intents provider reaches it: a notice into the window's inbox.

use detent_intents::{Window, WindowGone};
use detent_model::{AccountTarget, PageSlug};
use detent_ui::{Notice, Outbox};

/// Tells the running window what the provider did.
#[derive(Debug, Clone)]
pub struct WindowLink(pub Outbox);

impl WindowLink {
    fn tell(&self, notice: Notice) -> Result<(), WindowGone> {
        self.0.send(notice).map_err(|_| WindowGone)
    }
}

impl Window for WindowLink {
    fn open_page(&self, page: PageSlug) -> Result<(), WindowGone> {
        self.tell(Notice::OpenPage(page))
    }

    fn files_changed(&self) -> Result<(), WindowGone> {
        self.tell(Notice::Reload)
    }

    fn open_account(&self, target: AccountTarget) -> Result<(), WindowGone> {
        self.tell(Notice::OpenAccount(target))
    }

    fn raise(&self, token: Option<String>) -> Result<(), WindowGone> {
        self.tell(Notice::Raise(token))
    }
}

#[cfg(test)]
mod tests {
    use super::WindowLink;
    use detent_intents::{Window, WindowGone};
    use detent_model::{AccountTarget, PageSlug};
    use detent_ui::{Notice, inbox};

    #[test]
    fn the_window_is_told_in_order() {
        let (outbox, inbox) = inbox();
        let link = WindowLink(outbox);
        link.open_page(PageSlug("dock".to_owned())).expect("sent");
        link.files_changed().expect("sent");
        let mut notices = inbox.take().expect("the receiver");
        assert_eq!(
            notices.try_recv(),
            Ok(Notice::OpenPage(PageSlug("dock".to_owned())))
        );
        assert_eq!(notices.try_recv(), Ok(Notice::Reload));
    }

    #[test]
    fn a_window_that_has_gone_cannot_be_told() {
        let (outbox, inbox) = inbox();
        drop(inbox);
        let link = WindowLink(outbox);
        assert_eq!(link.files_changed(), Err(WindowGone));
    }
}
