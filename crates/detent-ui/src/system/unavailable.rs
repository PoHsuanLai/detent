//! What a page says when there is nothing to show: its tile, its name and one plain sentence, and
//! the action when the page has one.

use crate::copy;
use crate::copy::absence::Subject;
use detent_model::Absence;
use dioxus::prelude::*;
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;

/// A page's place when it has nothing to show: `face` over `title`, then `words`, then `action`
/// when the page offers one. Nothing is greyed: the whole place says what is missing and why. A
/// page that opens with its own hero leaves out the `face` and the `title`, which the hero
/// already says.
#[component]
pub(crate) fn Absent(
    #[props(default)] face: Option<TileFace>,
    #[props(into, default)] title: String,
    #[props(into)] words: String,
    #[props(default)] action: Option<Element>,
) -> Element {
    rsx! {
        div { class: "absent",
            EmptyState { title, description: Some(TextLine::from(words)), face, action }
        }
    }
}

/// A system page whose service is not there: the page's tile and name, and why, in the words the
/// copy has for `subject`.
#[component]
pub(crate) fn Unavailable(
    face: TileFace,
    title: String,
    subject: Subject,
    absence: Absence,
) -> Element {
    let words = copy::absence::absence(subject, absence);
    rsx! {
        div { class: "system-page", "data-system": "unavailable",
            Absent { face: Some(face), title, words }
        }
    }
}
