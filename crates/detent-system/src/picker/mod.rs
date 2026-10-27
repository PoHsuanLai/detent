//! Setting a row of inferd's settings module for the model picker: the module answers when it has
//! put the row in force.

mod bus;
mod set;

pub use bus::OnClient;

use detent_model::Work;
use ds_settings::live::LiveClient;
use ds_settings::schema::KeyPath;
use std::fmt::Debug;

/// What the picker asks of inferd's module, through the client the pane follows it with.
pub trait PickerLink: Debug + Send + Sync {
    /// Sets the row at `path` to `word`; the module's own words when it refuses.
    fn set<'a>(
        &'a self,
        client: &'a LiveClient,
        path: &'a KeyPath,
        word: &'a str,
    ) -> Work<'a, Result<(), String>>;
    /// Sets the pool at `path` to `words`; the module refuses a model it does not know.
    fn set_pool<'a>(
        &'a self,
        client: &'a LiveClient,
        path: &'a KeyPath,
        words: &'a [String],
    ) -> Work<'a, Result<(), String>>;
}
