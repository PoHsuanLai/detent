//! The coding assistants' Add sheet, and the question before one is removed.

use detent_model::{AddForm, INTERNET_LINE, RemoveQuestion};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::{Answers, ButtonRole};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The sheet's title.
const TITLE: &str = "Add a coding assistant";

/// The sheet over the coding assistants: pick one that is installed, name it, and read what it
/// reaches before Add. `typed` and `picked` hand the form back changed; `submit` asks to write it.
#[component]
pub(super) fn AddSheet(
    form: AddForm,
    picked: EventHandler<String>,
    typed: EventHandler<String>,
    submit: EventHandler<()>,
    close: EventHandler<()>,
) -> Element {
    let entries = form.entries();
    let items: Vec<MenuItem<String>> = entries
        .iter()
        .map(|(id, name)| MenuItem::new(id.clone(), name.clone()))
        .collect();
    rsx! {
        Sheet {
            label: TITLE.to_owned(),
            width: SheetWidth::Narrow,
            onclose: close,
            div { class: "system-sheet", "data-system": "add-coding-assistant",
                Label { text: TITLE, role: LabelRole::Primary, style: LabelStyle::Headline }
                if entries.len() > 1 {
                    PopUpButton::<String> {
                        items,
                        value: Some(form.picked_id().to_owned()),
                        size: ControlSize::Small,
                        onpick: move |id: String| picked.call(id),
                    }
                }
                TextField {
                    label: "Name",
                    value: form.label().to_owned(),
                    kind: FieldKind::Plain,
                    size: ControlSize::Small,
                    oninput: move |text: String| typed.call(text),
                    onsubmit: move |_: String| submit.call(()),
                }
                Label { text: INTERNET_LINE, role: LabelRole::Secondary }
                if let Some(words) = form.refusal {
                    Label { text: words, role: LabelRole::Secondary }
                }
                div { class: "system-sheet-buttons",
                    Button {
                        label: "Cancel",
                        common: Common { aria_label: Some("Cancel".to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        onclick: move |_| close.call(()),
                    }
                    Button {
                        label: "Add",
                        common: Common { aria_label: Some("Add".to_owned()), ..Common::default() },
                        answers: Answers::Return,
                        role: ButtonRole::Normal,
                        onclick: move |_| submit.call(()),
                    }
                }
            }
        }
    }
}

/// Asks before an assistant is taken off the list; Cancel is the one Return presses.
#[component]
pub(super) fn RemoveAlert(question: RemoveQuestion, answer: EventHandler<bool>) -> Element {
    rsx! {
        Alert {
            title: question.title(),
            message: Some(TextLine::from(question.message())),
            style: AlertStyle::Critical,
            buttons: vec![
                AlertButton::new("Cancel", AlertRole::Cancel, EventHandler::new(move |()| answer.call(false))),
                AlertButton::new("Remove", AlertRole::Destructive, EventHandler::new(move |()| answer.call(true))),
            ],
        }
    }
}
