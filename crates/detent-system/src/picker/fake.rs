//! inferd's module from memory: what the model picker set, and the words a refusal is made of.

use super::PickerLink;
use crate::fake_kit::Kit;
use crate::modules::Module;
use detent_model::Work;
use ds_settings::schema::KeyPath;
use std::sync::Arc;

/// What the picker set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerAsked {
    /// A row was set to a word.
    Set(KeyPath, String),
    /// A pool was set to these models.
    Pool(KeyPath, Vec<String>),
}

/// A module that takes what it is given.
#[derive(Debug, Clone)]
pub struct FakePicker(Arc<Kit<(), PickerAsked>>);

impl FakePicker {
    /// A module that takes every change.
    pub fn taking() -> FakePicker {
        FakePicker(Kit::new(Ok(())))
    }

    /// The next changes are refused, in these words.
    pub fn refuses(&self, words: &str) {
        self.0.refuse_with(Some(words.to_owned()));
    }

    /// What was set, in order.
    pub fn asked(&self) -> Vec<PickerAsked> {
        self.0.asked()
    }
}

impl PickerLink for FakePicker {
    fn set<'a>(
        &'a self,
        _client: &'a Module,
        path: &'a KeyPath,
        word: &'a str,
    ) -> Work<'a, Result<(), String>> {
        self.0
            .record(PickerAsked::Set(path.clone(), word.to_owned()));
        Box::pin(std::future::ready(self.0.admit()))
    }

    fn set_pool<'a>(
        &'a self,
        _client: &'a Module,
        path: &'a KeyPath,
        words: &'a [String],
    ) -> Work<'a, Result<(), String>> {
        self.0
            .record(PickerAsked::Pool(path.clone(), words.to_vec()));
        Box::pin(std::future::ready(self.0.admit()))
    }
}
