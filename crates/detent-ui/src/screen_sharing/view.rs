//! The list: a row for each app with its name, when it was allowed, and Remove; or the line that
//! says no app is.

use crate::copy::privacy::program_name;
use crate::copy::screen_sharing::grant_line;
use crate::key_row::row_id;
use detent_model::{
    ProgramBook, ProgramSubject, ScreenGrant, ScreenSharingInput, ScreenSharingReady, UnixSeconds,
};
use dioxus::prelude::*;
use ds::components::controls::button_model::ButtonRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

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
                    title: crate::copy::screen_sharing::NO_APPS_CAN_SHARE_YOUR_SCREEN,
                    description: crate::copy::screen_sharing::NO_REMEMBERED_APPS_HINT,
                }
            } else {
                FieldGroup { title: Some(crate::copy::screen_sharing::SCREEN_SHARING.to_owned()),
                    for grant in ready.grants.iter().cloned() {
                        AppLine {
                            key: "{grant.app.0}",
                            name: program_name(&book.look_of(&grant.app)),
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
                    aria_label: Some(crate::copy::screen_sharing::remove(&name)),
                    ..Common::default()
                },
                label: crate::copy::common::REMOVE,
                role: ButtonRole::Destructive,
                size: ControlSize::Small,
                availability,
                onclick: move |_| remove.call(ScreenSharingInput::Remove(app.clone())),
            }
        }
    }
}
