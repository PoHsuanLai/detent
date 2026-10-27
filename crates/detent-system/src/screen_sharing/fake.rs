//! The screen-sharing portal from memory: the apps it lets share, and what was taken back.

use super::ScreenSharingLink;
use crate::fake_kit::Kit;
use detent_model::{Absence, ProgramSubject, ScreenGrant, Work};
use std::sync::Arc;

/// A portal that answers from what it was given.
#[derive(Debug, Clone)]
pub struct FakeScreenSharing(Arc<Kit<Vec<ScreenGrant>, ProgramSubject>>);

impl FakeScreenSharing {
    /// A portal that lets these apps share.
    pub fn granting(grants: Vec<ScreenGrant>) -> FakeScreenSharing {
        FakeScreenSharing(Kit::new(Ok(grants)))
    }

    /// No portal, for this reason.
    pub fn missing(why: Absence) -> FakeScreenSharing {
        FakeScreenSharing(Kit::new(Err(why)))
    }

    /// The next revokes are refused, in these words.
    pub fn refuses(&self, words: &str) {
        self.0.refuse_with(Some(words.to_owned()));
    }

    /// The apps that were taken back, in order.
    pub fn revoked(&self) -> Vec<ProgramSubject> {
        self.0.asked()
    }
}

impl ScreenSharingLink for FakeScreenSharing {
    fn list(&self) -> Work<'_, Result<Vec<ScreenGrant>, Absence>> {
        Box::pin(std::future::ready(self.0.standing()))
    }

    fn revoke(&self, app: ProgramSubject) -> Work<'_, Result<(), String>> {
        self.0.record(app.clone());
        let done = self.0.admit();
        if done.is_ok() {
            self.0
                .edit(|grants| grants.retain(|grant| grant.app != app));
        }
        Box::pin(std::future::ready(done))
    }
}
