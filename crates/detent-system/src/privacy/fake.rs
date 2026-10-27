//! The compositor's privacy module from memory: what the page allowed, denied or asked again.

use super::PrivacyLink;
use crate::fake_kit::Kit;
use detent_model::{PrivacySwitch, Work};
use ds_settings::live::LiveClient;
use ds_settings::schema::KeyPath;
use std::sync::Arc;

/// What the page asked of the module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivacyAsked {
    /// A power was allowed or denied.
    Set(KeyPath, PrivacySwitch),
    /// A program is asked again.
    Forget(KeyPath),
}

/// A module that takes what it is given.
#[derive(Debug, Clone)]
pub struct FakePrivacy(Arc<Kit<(), PrivacyAsked>>);

impl FakePrivacy {
    /// A module that takes every change.
    pub fn taking() -> FakePrivacy {
        FakePrivacy(Kit::new(Ok(())))
    }

    /// The next changes are refused, in these words.
    pub fn refuses(&self, words: &str) {
        self.0.refuse_with(Some(words.to_owned()));
    }

    /// What was asked, in order.
    pub fn asked(&self) -> Vec<PrivacyAsked> {
        self.0.asked()
    }
}

impl PrivacyLink for FakePrivacy {
    fn set<'a>(
        &'a self,
        _client: &'a LiveClient,
        path: &'a KeyPath,
        to: PrivacySwitch,
    ) -> Work<'a, Result<(), String>> {
        self.0.record(PrivacyAsked::Set(path.clone(), to));
        Box::pin(std::future::ready(self.0.admit()))
    }

    fn forget<'a>(
        &'a self,
        _client: &'a LiveClient,
        path: &'a KeyPath,
    ) -> Work<'a, Result<(), String>> {
        self.0.record(PrivacyAsked::Forget(path.clone()));
        Box::pin(std::future::ready(self.0.admit()))
    }
}
