//! The list: a row for each app with its name, when it was allowed, and Remove; or the line that
//! says no app is.

use super::pane::TITLE;
use crate::key_row::row_id;
use detent_model::{
    Absence, ProgramBook, ProgramSubject, ScreenGrant, ScreenSharingInput, ScreenSharingReady,
    UnixSeconds, grant_line,
};
use dioxus::prelude::*;
use ds::components::controls::button_model::ButtonRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// What the absence of the portal says.
pub(super) fn absence_help(absence: &Absence) -> &'static str {
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop, so there are no apps to show.",
        Absence::NotRunning => {
            "The screen sharing service is not running, so there are no apps to show."
        }
        Absence::NotPermitted => {
            "Apps that may share your screen can only be managed when Settings is opened from the desktop."
        }
        Absence::Unavailable => {
            "The screen sharing service could not be confirmed, so there are no apps to show."
        }
        Absence::Failed(_) => "The screen sharing service did not answer. Try again in a moment.",
    }
}

/// The apps, or the line that says none can share the screen without asking.
#[component]
pub(super) fn Listed(
    ready: ScreenSharingReady,
    book: ProgramBook,
    now: UnixSeconds,
    remove: EventHandler<ScreenSharingInput>,
) -> Element {
    let busy = if ready.removing.is_some() {
        Availability::Disabled
    } else {
        Availability::Enabled
    };
    rsx! {
        div { class: "screen-sharing", "data-screen-sharing": "list",
            if ready.grants.is_empty() {
                EmptyState {
                    title: "No apps can share your screen without asking",
                    description: "When you let an app share your screen without asking again, it appears here.",
                }
            } else {
                FieldGroup { title: Some(TITLE.to_owned()),
                    for grant in ready.grants.iter().cloned() {
                        AppLine {
                            key: "{grant.app.0}",
                            name: book.look_of(&grant.app).name,
                            line: grant_line(&grant, now),
                            grant,
                            availability: busy,
                            remove,
                        }
                    }
                }
            }
        }
    }
}

/// One app: its name, when it was allowed, and Remove.
#[component]
fn AppLine(
    grant: ScreenGrant,
    name: String,
    line: String,
    availability: Availability,
    remove: EventHandler<ScreenSharingInput>,
) -> Element {
    let app: ProgramSubject = grant.app.clone();
    rsx! {
        FieldRow {
            label: TextLine::from(name.clone()),
            help: Some(TextLine::from(line)),
            common: Common { id: Some(row_id("screen-sharing", &app.0)), ..Common::default() },
            Button {
                common: Common {
                    aria_label: Some(format!("Remove {name}")),
                    ..Common::default()
                },
                label: "Remove",
                role: ButtonRole::Destructive,
                size: ControlSize::Small,
                availability,
                onclick: move |_| remove.call(ScreenSharingInput::Remove(app.clone())),
            }
        }
    }
}
