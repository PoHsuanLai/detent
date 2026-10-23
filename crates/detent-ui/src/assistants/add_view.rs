//! The coding assistants' Add sheet, and the question before one is removed.

use detent_model::{
    AddButton, AddForm, Candidate, INTERNET_LINE, LOADING, Listing, NOTHING_NEW, RemoveQuestion,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::{Answers, ButtonRole};
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The sheet's title.
const TITLE: &str = "Add a coding assistant";

/// The sheet over the coding assistants: pick one, installed or on offer, name it, and read what
/// it reaches before Add. `typed` and `picked` hand the form back changed; `submit` asks to add it.
#[component]
pub(super) fn AddSheet(
    form: AddForm,
    picked: EventHandler<String>,
    typed: EventHandler<String>,
    submit: EventHandler<()>,
    close: EventHandler<()>,
) -> Element {
    let button = form.button();
    let loading = *form.listing() == Listing::Loading;
    let choices: Vec<Choice<String>> = form
        .entries()
        .iter()
        .map(|one| Choice::new(one.id.clone(), line_of(one)))
        .collect();
    let (add, cancel) = match button {
        AddButton::Ready => (Availability::Enabled, Availability::Enabled),
        AddButton::Busy => (Availability::Busy, Availability::Disabled),
        AddButton::Off => (Availability::Disabled, Availability::Enabled),
    };
    rsx! {
        Sheet {
            label: TITLE.to_owned(),
            width: SheetWidth::Narrow,
            onclose: close,
            div { class: "system-sheet", "data-system": "add-coding-assistant",
                Label { text: TITLE, role: LabelRole::Primary, style: LabelStyle::Headline }
                if loading {
                    Looking {}
                }
                if let Listing::Failed(why) = form.listing() {
                    Label { text: why.clone(), role: LabelRole::Secondary }
                }
                if choices.is_empty() && !loading {
                    Label { text: NOTHING_NEW, role: LabelRole::Secondary }
                }
                if !choices.is_empty() {
                    RadioGroup::<String> {
                        label: "Coding assistants",
                        choices,
                        value: form.picked_id().to_owned(),
                        onchange: move |id: String| picked.call(id),
                    }
                    TextField {
                        label: "Name",
                        value: form.label().to_owned(),
                        kind: FieldKind::Plain,
                        size: ControlSize::Small,
                        oninput: move |text: String| typed.call(text),
                        onsubmit: move |_: String| submit.call(()),
                    }
                }
                Label { text: INTERNET_LINE, role: LabelRole::Secondary }
                if let Some(line) = form.download_line() {
                    Label { text: line, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
                if let Some(words) = form.refusal.clone() {
                    Label { text: words, role: LabelRole::Secondary }
                }
                div { class: "system-sheet-buttons",
                    Button {
                        label: "Cancel",
                        common: Common { aria_label: Some("Cancel".to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        availability: cancel,
                        onclick: move |_| close.call(()),
                    }
                    Button {
                        label: "Add",
                        common: Common { aria_label: Some("Add".to_owned()), ..Common::default() },
                        answers: Answers::Return,
                        role: ButtonRole::Normal,
                        availability: add,
                        onclick: move |_| submit.call(()),
                    }
                }
            }
        }
    }
}

/// An assistant's row: its name, whether it is installed, and the line about it.
fn line_of(one: &Candidate) -> TextLine {
    let mut runs = vec![TextRun::new(one.name.clone(), RunTone::Strong)];
    if let Some(mark) = one.presence.mark() {
        runs.push(TextRun::new(format!("  {mark}"), RunTone::Faint));
    }
    if !one.description.is_empty() {
        runs.push(TextRun::new(
            format!(" \u{2014} {}", one.description),
            RunTone::Faint,
        ));
    }
    TextLine::Runs(runs)
}

/// The turning wheel and the words beside it while the list is fetched.
#[component]
fn Looking() -> Element {
    let token = use_hook(PendingToken::start);
    rsx! {
        div { class: "system-glyphs",
            ProgressIndicator {
                style: ProgressStyle::Spinner,
                progress: Progress::Unknown(Operation::Running(token)),
                size: ControlSize::Small,
                common: Common { aria_label: Some("Looking for coding assistants".to_owned()), ..Common::default() },
            }
            Label { text: LOADING, role: LabelRole::Secondary }
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
