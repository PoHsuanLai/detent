//! The list's state, read from the portal when the page is shown and driven by Remove.

use super::view::Listed;
use crate::copy::{Subject, absence as absence_words};
use crate::live::{use_client, use_live};
use crate::privacy::use_program_book;
use detent_client::Client;
use detent_model::{ScreenSharingInput, ScreenSharingState};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;

/// The title of the group, and of its row when the portal is not there.
pub(super) const TITLE: &str = "Screen sharing";

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
                FieldGroup { title: Some(TITLE.to_owned()),
                    FieldRow {
                        label: TextLine::from(TITLE),
                        help: Some(TextLine::from(absence_words(Subject::ScreenSharing, absence))),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        ScreenSharingState::Ready(ready) => rsx! {
            Listed { ready, book: book(), now, remove }
        },
    }
}
