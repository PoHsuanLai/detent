//! The page's state, kept current from the compositor and driven by the person's choices.

use super::book::use_program_book;
use super::program_view::Listed;
use crate::copy::{Subject, absence as absence_words};
use crate::live::use_live;
use detent_client::Client;
use detent_model::{PrivacyInput, PrivacyState};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;

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
                FieldGroup { title: Some("Privacy & Security".to_owned()),
                    FieldRow {
                        label: TextLine::from("Privacy & Security"),
                        help: Some(TextLine::from(absence_words(Subject::Privacy, absence))),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        PrivacyState::Ready(ready) => rsx! { Listed { ready, book: book(), pick } },
    }
}
