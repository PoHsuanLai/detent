//! The sheet before a Space is removed.

use crate::copy::{
    HISTORY_CHOICES, HISTORY_NOTE, flip_label, history_label, removal_body, removal_title,
};
use almanac_core::HistoryFate;
use detent_model::{RemovalAsk, RemovalInput};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::{Answers, Bezel, ButtonRole};
use ds::components::controls::choice::Choice;
use ds::components::controls::radio_group::RadioGroup;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;

/// What the sheet reports.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct SheetHandlers {
    /// A choice changed.
    pub changed: EventHandler<RemovalInput>,
    /// Remove was pressed.
    pub confirm: EventHandler<()>,
    /// Cancel was pressed, or Escape.
    pub cancel: EventHandler<()>,
}

/// The sheet over `ask`: the title, what becomes of the memories, what becomes of the history,
/// and Remove and Cancel.
#[component]
pub(super) fn RemovalSheetView(ask: RemovalAsk, on: SheetHandlers) -> Element {
    let role = if ask.destroys() {
        ButtonRole::Destructive
    } else {
        ButtonRole::Normal
    };
    let choices: Vec<Choice<HistoryFate>> = HISTORY_CHOICES
        .iter()
        .map(|fate| Choice::new(*fate, history_label(*fate)))
        .collect();
    rsx! {
        Sheet {
            label: removal_title(&ask.name),
            width: SheetWidth::Narrow,
            onclose: move |()| on.cancel.call(()),
            on_return: move |()| on.confirm.call(()),
            common: Common { id: Some("remove-space".to_owned()), ..Common::default() },
            div { class: "space-sheet", "data-space-sheet": "remove",
                Label { text: removal_title(&ask.name), role: LabelRole::Primary, style: LabelStyle::Headline }
                Label { text: removal_body(ask.memories), role: LabelRole::Secondary }
                Button {
                    label: flip_label(ask.memories),
                    bezel: Bezel::Inline,
                    onclick: move |_| on.changed.call(RemovalInput::FlipMemories),
                }
                RadioGroup::<HistoryFate> {
                    label: "History",
                    choices,
                    value: ask.history,
                    onchange: move |fate| on.changed.call(RemovalInput::History(fate)),
                }
                Label { text: HISTORY_NOTE, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                div { class: "space-sheet-buttons",
                    Button {
                        label: "Cancel",
                        answers: Answers::Escape,
                        onclick: move |_| on.cancel.call(()),
                    }
                    Button {
                        label: "Remove",
                        role,
                        answers: Answers::Return,
                        onclick: move |_| on.confirm.call(()),
                    }
                }
            }
        }
    }
}
