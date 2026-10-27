//! The list: a card per group, Reset All, what in the files could not be used, and the question
//! before a chord another action has is taken over.

use super::pane::{Pane, use_pane};
use super::row_view::ShortcutLine;
use crate::copy::sharing::CANCEL;
use crate::copy::shortcuts::{REPLACE, group_title, problem_line, replace_words};
use detent_model::{ShortcutSection, ShortcutsInput, replace_question};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::FieldGroup;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The desktop's shortcuts.
#[component]
pub(crate) fn ShortcutsPane() -> Element {
    let pane = use_pane();
    use_future({
        let pane = pane.clone();
        move || pane.clone().follow()
    });
    let state = pane.state.read().clone();
    let drive = EventHandler::new({
        let pane = pane.clone();
        move |input: ShortcutsInput| pane.feed(input)
    });
    let reset = if state.has_own() {
        Availability::Enabled
    } else {
        Availability::Disabled
    };
    rsx! {
        div { class: "shortcuts", "data-shortcuts": "list",
            for section in state.sections() {
                Group { key: "{group_title(section.group)}", section, drive }
            }
            Footer { problems: state.problems().iter().map(problem_line).collect(), reset, drive }
            {question(&pane)}
        }
    }
}

#[component]
fn Group(section: ShortcutSection, drive: EventHandler<ShortcutsInput>) -> Element {
    rsx! {
        FieldGroup { title: Some(group_title(section.group).to_owned()),
            for row in section.rows {
                ShortcutLine { key: "{row.action.slug()}", row, drive }
            }
        }
    }
}

/// Reset All, and the lines of the files that could not be used.
#[component]
fn Footer(
    problems: Vec<String>,
    reset: Availability,
    drive: EventHandler<ShortcutsInput>,
) -> Element {
    rsx! {
        div { class: "shortcuts-footer",
            for problem in problems {
                Label { key: "{problem}", text: problem.clone(), role: LabelRole::Secondary, style: LabelStyle::Footnote }
            }
            Button {
                common: Common {
                    aria_label: Some(crate::copy::shortcuts::RESET_ALL_SHORTCUTS_TO_STANDARD.to_owned()),
                    ..Common::default()
                },
                label: crate::copy::shortcuts::RESET_ALL,
                size: ControlSize::Small,
                availability: reset,
                onclick: move |_| drive.call(ShortcutsInput::ResetAll),
            }
        }
    }
}

/// The question while one is asked.
fn question(pane: &Pane) -> Element {
    let Some(question) = replace_question(&pane.state.read()) else {
        return rsx! {};
    };
    let (title, message) = replace_words(&question);
    let (replace, cancel) = (pane.clone(), pane.clone());
    rsx! {
        Alert {
            title,
            message: Some(TextLine::from(message)),
            style: AlertStyle::Informational,
            buttons: vec![
                AlertButton::new(
                    REPLACE,
                    AlertRole::Normal,
                    EventHandler::new(move |()| replace.feed(ShortcutsInput::Replace)),
                ),
                AlertButton::new(
                    CANCEL,
                    AlertRole::Cancel,
                    EventHandler::new(move |()| cancel.feed(ShortcutsInput::Cancel)),
                ),
            ],
        }
    }
}
