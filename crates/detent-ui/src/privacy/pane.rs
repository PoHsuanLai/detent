//! The page's state, kept current from the compositor and driven by the person's choices.

use super::book::use_program_book;
use super::program_view::Listed;
use crate::copy::absence::{Subject, absence as absence_words};
use crate::live::use_live;
use crate::system::Absent;
use detent_client::Client;
use detent_model::{PrivacyInput, PrivacyState};
use dioxus::prelude::*;

/// The programs that have been answered, with a choice for each power.
#[component]
pub(crate) fn PrivacyPane() -> Element {
    let privacy = use_live(Client::privacy);
    let state = privacy.state;
    let book = use_program_book();
    let pick = EventHandler::new(move |input: PrivacyInput| {
        spawn(privacy.pane.send(input));
    });

    match state() {
        PrivacyState::Looking => rsx! {},
        PrivacyState::Unavailable(absence) => rsx! {
            div { class: "privacy", "data-privacy": "unavailable",
                Absent {
                    words: absence_words(Subject::Privacy, absence),
                }
            }
        },
        PrivacyState::Ready(ready) => rsx! { Listed { ready, book: book(), pick } },
    }
}
