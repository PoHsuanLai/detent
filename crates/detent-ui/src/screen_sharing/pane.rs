//! The list's state, read from the portal when the page is shown and driven by Remove.

use super::view::Listed;
use crate::copy::absence::{Subject, absence as absence_words};
use crate::live::{use_client, use_live};
use crate::privacy::use_program_book;
use crate::system::Absent;
use detent_client::Client;
use detent_model::{ScreenSharingInput, ScreenSharingState};
use dioxus::prelude::*;

/// The apps the portal lets share the screen without asking again, with a Remove on each.
#[component]
pub(crate) fn ScreenSharingPane() -> Element {
    let sharing = use_live(Client::screen_sharing);
    let state = sharing.state;
    let book = use_program_book();
    let now = use_client().now();
    let remove = EventHandler::new(move |input: ScreenSharingInput| {
        spawn(sharing.pane.send(input));
    });

    match state() {
        ScreenSharingState::Looking => rsx! {},
        ScreenSharingState::Unavailable(absence) => rsx! {
            div { class: "screen-sharing", "data-screen-sharing": "unavailable",
                Absent {
                    title: crate::copy::screen_sharing::SCREEN_SHARING,
                    words: absence_words(Subject::ScreenSharing, absence),
                }
            }
        },
        ScreenSharingState::Ready(ready) => rsx! {
            Listed { ready, book: book(), now, remove }
        },
    }
}
