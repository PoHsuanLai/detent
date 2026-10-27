//! inferd's settings module on the bus, as the picker's link.

use super::PickerLink;
use super::set::{set, set_pool};
use crate::modules::Module;
use detent_model::Work;
use ds_settings::schema::KeyPath;

/// inferd's module, reached through the client the pane follows it with.
#[derive(Debug, Clone, Copy)]
pub struct OnClient;

impl PickerLink for OnClient {
    fn set<'a>(
        &'a self,
        client: &'a Module,
        path: &'a KeyPath,
        word: &'a str,
    ) -> Work<'a, Result<(), String>> {
        Box::pin(set(client, path, word))
    }

    fn set_pool<'a>(
        &'a self,
        client: &'a Module,
        path: &'a KeyPath,
        words: &'a [String],
    ) -> Work<'a, Result<(), String>> {
        Box::pin(set_pool(client, path, words))
    }
}
