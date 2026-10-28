//! The Software Update page: the version row with its buttons, the progress bar, when the last
//! check was, the daily check, and the release notes in their sheet.

use crate::copy;
use crate::copy::absence::Subject;
use crate::icons::tile_of;
use crate::live::use_live;
use crate::system::Absent;
use detent_client::Client;
use detent_model::{
    PageKind, ReleaseNotes, UpdateAction, UpdateMeter, UpdatesInput, UpdatesPage, UpdatesShown,
    updates_shown,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::{Answers, Bezel};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;

/// The page: what the update service says and the buttons that move it, or why there is none.
#[component]
pub(crate) fn SoftwareUpdatePane() -> Element {
    let updates = use_live(Client::updates);
    let shown = updates_shown(&updates.state.read(), updates.pane.now());
    match shown {
        UpdatesShown::Looking => rsx! { div { class: "updates", "data-updates": "looking" } },
        UpdatesShown::Unavailable(absence) => rsx! {
            div { class: "updates", "data-updates": "unavailable",
                Absent {
                    face: Some(tile_of(PageKind::SoftwareUpdate)),
                    title: crate::copy::updates::SOFTWARE_UPDATE,
                    words: copy::absence::absence(Subject::Updates, absence),
                }
            }
        },
        UpdatesShown::Ready(page) => {
            let press = EventHandler::new(move |input: UpdatesInput| {
                spawn(updates.pane.send(input));
            });
            rsx! { Listed { page, press } }
        }
    }
}

#[component]
fn Listed(page: UpdatesPage, press: EventHandler<UpdatesInput>) -> Element {
    rsx! {
        div { class: "updates", "data-updates": "page",
            FieldGroup {
                FieldRow {
                    label: TextLine::from(copy::updates::title(page.version.as_deref())),
                    help: Some(TextLine::from(copy::updates::standing(&page.standing))),
                    for (at, action) in page.actions.iter().copied().enumerate() {
                        ActionButton { key: "{at}", action, press }
                    }
                }
            }
            if let Some(words) = copy::updates::notice(page.notice) {
                div { class: "updates-notice",
                    Label { text: words, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
            }
            Meter { meter: page.meter.clone() }
            if let Some(line) = page.checked.map(copy::updates::checked) {
                div { class: "updates-note",
                    Label { text: line, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                }
            }
            FieldGroup {
                FieldRow {
                    label: TextLine::from(crate::copy::updates::AUTOMATIC_UPDATES),
                    help: Some(TextLine::from(copy::updates::automatic())),
                    Label { text: "Daily", role: LabelRole::Secondary }
                }
            }
            if let Some(notes) = page.notes.clone() {
                NotesView { notes, press }
            }
        }
    }
}

#[component]
fn ActionButton(action: UpdateAction, press: EventHandler<UpdatesInput>) -> Element {
    let (bezel, availability) = match action {
        UpdateAction::Check { running: true } => (Bezel::Push, Availability::Busy),
        UpdateAction::MoreInfo => (Bezel::Inline, Availability::Enabled),
        _ => (Bezel::Push, Availability::Enabled),
    };
    let label = copy::updates::action(action);
    rsx! {
        Button {
            label,
            bezel,
            availability,
            common: Common { aria_label: Some(label.to_owned()), ..Common::default() },
            onclick: move |_| press.call(action.input()),
        }
    }
}

/// The bar under the version row: how far a download has come, or work with no end in sight.
#[component]
fn Meter(meter: Option<UpdateMeter>) -> Element {
    match meter {
        None => rsx! {},
        Some(UpdateMeter::Known { share, done }) => rsx! {
            div { class: "updates-meter",
                ProgressIndicator {
                    style: ProgressStyle::Bar,
                    progress: Progress::Known(Fraction(u16::try_from(share.0).unwrap_or(1000))),
                    common: Common { aria_label: Some(crate::copy::common::DOWNLOAD_PROGRESS.to_owned()), ..Common::default() },
                }
                if let Some(done) = done {
                    Label { text: copy::updates::downloaded(done), role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
            }
        },
        Some(UpdateMeter::Pending) => rsx! {
            div { class: "updates-meter", PendingBar {} }
        },
    }
}

#[component]
fn PendingBar() -> Element {
    let token = use_hook(PendingToken::start);
    rsx! {
        ProgressIndicator {
            style: ProgressStyle::Bar,
            progress: Progress::Unknown(Operation::Running(token)),
            common: Common { aria_label: Some(crate::copy::updates::INSTALLING.to_owned()), ..Common::default() },
        }
    }
}

/// What the new version changes, in a sheet with Done.
#[component]
fn NotesView(notes: ReleaseNotes, press: EventHandler<UpdatesInput>) -> Element {
    let close = EventHandler::new(move |()| press.call(UpdatesInput::CloseInfo));
    let title = copy::updates::title(Some(&notes.version));
    let lines: Vec<String> = copy::updates::notes_body(&notes)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    rsx! {
        Sheet {
            label: title.clone(),
            width: SheetWidth::Regular,
            onclose: close,
            on_return: close,
            div { class: "updates-sheet", "data-updates": "notes",
                Label { text: title.clone(), role: LabelRole::Primary, style: LabelStyle::Headline }
                for (at, line) in lines.into_iter().enumerate() {
                    Label { key: "{at}", text: line, role: LabelRole::Secondary }
                }
                div { class: "updates-sheet-buttons",
                    Button {
                        label: crate::copy::common::DONE,
                        answers: Answers::Return,
                        onclick: move |_| close.call(()),
                    }
                }
            }
        }
    }
}
