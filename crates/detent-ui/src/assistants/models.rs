//! The models inferd lists, read where a place offers them to choose from.

use crate::live::use_live;
use detent_client::Client;
use detent_model::Ready;
use dioxus::prelude::*;

/// inferd's picker rows, none while inferd is not there.
pub(super) fn use_picker() -> Signal<Option<Ready>> {
    use_live(Client::models).state
}
