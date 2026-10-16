//! The Software Update page: the version row with its buttons, the progress bar, when the last
//! check was, the daily check, and the release notes in their sheet.

use super::drive::use_updates;
use crate::env::Env;
use detent_model::{
    ButtonActivity, NotesSheet, UpdateButton, UpdateMeter, UpdatesInput, UpdatesPage, UpdatesShown,
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
    let env = use_context::<Env>();
    let standing = use_updates();
    let shown = updates_shown(&standing.state.read(), env.updates.now());
    match shown {
        UpdatesShown::Looking => rsx! { div { class: "updates", "data-updates": "looking" } },
        UpdatesShown::Unavailable(words) => rsx! {
            div { class: "updates", "data-updates": "unavailable",
                FieldGroup {
                    FieldRow {
                        label: TextLine::from("Software Update"),
                        help: Some(TextLine::from(words)),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        UpdatesShown::Page(page) => {
            let press = EventHandler::new(move |input: UpdatesInput| standing.feed(input));
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
                    label: TextLine::from(page.title.clone()),
                    help: Some(TextLine::from(page.detail.clone())),
                    for (at, button) in page.buttons.iter().copied().enumerate() {
                        ActionButton { key: "{at}", button, press }
                    }
                }
            }
            if let Some(words) = page.notice {
                div { class: "updates-notice",
                    Label { text: words, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
            }
            Meter { meter: page.meter.clone() }
            if let Some(line) = page.checked.clone() {
                div { class: "updates-note",
                    Label { text: line, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                }
            }
            FieldGroup {
                FieldRow {
                    label: TextLine::from("Automatic updates"),
                    help: Some(TextLine::from(page.automatic)),
                    Label { text: "Daily", role: LabelRole::Secondary }
                }
            }
            if let Some(sheet) = page.notes.clone() {
                NotesView { sheet, press }
            }
        }
    }
}

#[component]
fn ActionButton(button: UpdateButton, press: EventHandler<UpdatesInput>) -> Element {
    let (bezel, availability) = match button {
        UpdateButton::Check(ButtonActivity::Busy) => (Bezel::Push, Availability::Busy),
        UpdateButton::MoreInfo => (Bezel::Inline, Availability::Enabled),
        _ => (Bezel::Push, Availability::Enabled),
    };
    rsx! {
        Button {
            label: button.label(),
            bezel,
            availability,
            common: Common { aria_label: Some(button.label().to_owned()), ..Common::default() },
            onclick: move |_| press.call(button.press()),
        }
    }
}

/// The bar under the version row: how far a download has come, or work with no end in sight.
#[component]
fn Meter(meter: UpdateMeter) -> Element {
    match meter {
        UpdateMeter::None => rsx! {},
        UpdateMeter::Known { share, words } => rsx! {
            div { class: "updates-meter",
                ProgressIndicator {
                    style: ProgressStyle::Bar,
                    progress: Progress::Known(Fraction(u16::try_from(share.0).unwrap_or(1000))),
                    common: Common { aria_label: Some("Download progress".to_owned()), ..Common::default() },
                }
                if let Some(words) = words {
                    Label { text: words, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
            }
        },
        UpdateMeter::Pending => rsx! {
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
            common: Common { aria_label: Some("Installing".to_owned()), ..Common::default() },
        }
    }
}

/// What the new version changes, in a sheet with Done.
#[component]
fn NotesView(sheet: NotesSheet, press: EventHandler<UpdatesInput>) -> Element {
    let close = EventHandler::new(move |()| press.call(UpdatesInput::CloseInfo));
    let lines: Vec<String> = sheet
        .body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    rsx! {
        Sheet {
            label: sheet.title.clone(),
            width: SheetWidth::Regular,
            onclose: close,
            on_return: close,
            div { class: "updates-sheet", "data-updates": "notes",
                Label { text: sheet.title.clone(), role: LabelRole::Primary, style: LabelStyle::Headline }
                for (at, line) in lines.into_iter().enumerate() {
                    Label { key: "{at}", text: line, role: LabelRole::Secondary }
                }
                div { class: "updates-sheet-buttons",
                    Button {
                        label: "Done",
                        answers: Answers::Return,
                        onclick: move |_| close.call(()),
                    }
                }
            }
        }
    }
}
