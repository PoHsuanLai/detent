//! The list: a card for each program with its icon and name, and a row for each power with the
//! standing it is at. Only the person's own answers can be changed; an administrator's says so.

use crate::copy::privacy::{SET_BY_ADMINISTRATOR, power_label, program_name, standing_label};
use crate::key_row::row_id;
use detent_model::{
    CellKind, PowerCell, PowerStanding, PrivacyInput, PrivacyProgram, PrivacyReady, ProgramBook,
    ProgramLook, ProgramSubject,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::icon::url::IconUrl;
use ds::style::tokens::control_size::ControlSize;

/// The line under the list.
const NOT_LISTED: &str = crate::copy::privacy::OTHER_PROGRAMS_ASK_THE_FIRST_TIME;

/// The programs, or the line that says none has asked.
#[component]
pub(super) fn Listed(
    ready: PrivacyReady,
    book: ProgramBook,
    pick: EventHandler<PrivacyInput>,
) -> Element {
    let PrivacyReady { pane, pending } = ready;
    let busy = if pending.is_some() {
        Availability::Disabled
    } else {
        Availability::Enabled
    };
    rsx! {
        div { class: "privacy", "data-privacy": "list",
            if pane.programs.is_empty() {
                EmptyState {
                    title: crate::copy::privacy::NOTHING_HAS_ASKED_YET,
                    description: crate::copy::privacy::PROGRAMS_THAT_ASK_TO_SEE_YOUR,
                }
            } else {
                for program in pane.programs.iter().cloned() {
                    ProgramCard {
                        key: "{program.subject.0}",
                        look: book.look_of(&program.subject),
                        program,
                        availability: busy,
                        pick,
                    }
                }
                Label { text: NOT_LISTED, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
            }
        }
    }
}

/// A program's icon beside its name, over the card of its powers.
#[component]
fn ProgramCard(
    program: PrivacyProgram,
    look: ProgramLook,
    availability: Availability,
    pick: EventHandler<PrivacyInput>,
) -> Element {
    rsx! {
        div {
            class: "privacy-program",
            "data-program": "{program.subject.0}",
            div { class: "privacy-program-head",
                ProgramIcon { look: look.clone() }
                div { class: "privacy-program-name",
                    Label { text: TextLine::from(program_name(&look)), role: LabelRole::Primary, style: LabelStyle::Headline }
                    if let Some(place) = look.place.clone() {
                        Label {
                            text: TextLine::from(crate::copy::privacy::in_folder(&place)),
                            role: LabelRole::Tertiary,
                            style: LabelStyle::Footnote,
                        }
                    }
                }
            }
            FieldGroup {
                for cell in program.cells.iter().cloned() {
                    PowerLine {
                        key: "{cell.power.word()}",
                        cell,
                        subject: program.subject.clone(),
                        availability,
                        pick,
                    }
                }
            }
        }
    }
}

/// The program's icon file, or a plain window when its entry has none.
#[component]
fn ProgramIcon(look: ProgramLook) -> Element {
    let source = look
        .icon
        .as_deref()
        .and_then(|file| IconUrl::file(file).ok())
        .map_or(IconSource::Glyph(Icon::Window), |url| {
            IconSource::Image(ExternalIcon {
                url,
                size: IconSize::Large,
            })
        });
    rsx! {
        IconView { source, size: IconSize::Large }
    }
}

/// One power: its name, and where it stands as a pop-up when the person can change it, as words
/// when nobody can.
#[component]
fn PowerLine(
    cell: PowerCell,
    subject: ProgramSubject,
    availability: Availability,
    pick: EventHandler<PrivacyInput>,
) -> Element {
    let id = row_id("privacy", &format!("{} {}", subject.0, cell.power.word()));
    let locked = cell.locked();
    let standing = cell.standing;
    let power = cell.power;
    rsx! {
        FieldRow {
            label: TextLine::from(power_label(power)),
            help: locked.then(|| TextLine::from(SET_BY_ADMINISTRATOR)),
            availability: if locked { Availability::Disabled } else { availability },
            common: Common { id: Some(id), ..Common::default() },
            if matches!(cell.kind, CellKind::Remembered { .. }) {
                PopUpButton::<String> {
                    items: PowerStanding::CHOICES
                        .iter()
                        .map(|choice| MenuItem::new(standing_label(*choice).to_owned(), standing_label(*choice)))
                        .collect::<Vec<_>>(),
                    value: Some(standing_label(standing).to_owned()),
                    size: ControlSize::Small,
                    availability,
                    onpick: move |word: String| {
                        if let Some(standing) = PowerStanding::CHOICES
                            .into_iter()
                            .find(|choice| standing_label(*choice) == word)
                        {
                            pick.call(PrivacyInput::Pick {
                                subject: subject.clone(),
                                power,
                                standing,
                            });
                        }
                    },
                }
            } else {
                Label { text: standing_label(standing), role: LabelRole::Secondary }
            }
        }
    }
}
