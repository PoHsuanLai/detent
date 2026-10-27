//! The coding assistants' Add sheet, and the question before one is removed.

use crate::copy::coding_agents as copy;
use detent_model::{AddAction, AddForm, Candidate, Listing, RemoveAnswer, RemoveQuestion};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::{Answers, ButtonRole};
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The sheet's title.
const TITLE: &str = crate::copy::assistants::ADD_A_CODING_ASSISTANT;

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
        AddAction::Ready => (Availability::Enabled, Availability::Enabled),
        AddAction::Busy => (Availability::Busy, Availability::Disabled),
        AddAction::Off => (Availability::Disabled, Availability::Enabled),
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
                if let Listing::Failed(said) = form.listing() {
                    Label { text: copy::list_failure(said.as_deref()), role: LabelRole::Secondary }
                }
                if choices.is_empty() && !loading {
                    Label { text: copy::NOTHING_NEW, role: LabelRole::Secondary }
                }
                if !choices.is_empty() {
                    RadioGroup::<String> {
                        label: crate::copy::assistants::CODING_ASSISTANTS,
                        choices,
                        value: form.picked_id().to_owned(),
                        onchange: move |id: String| picked.call(id),
                    }
                    TextField {
                        label: crate::copy::common::NAME,
                        value: form.label().to_owned(),
                        kind: FieldKind::Plain,
                        size: ControlSize::Small,
                        oninput: move |text: String| typed.call(text),
                        onsubmit: move |_: String| submit.call(()),
                    }
                }
                Label { text: copy::INTERNET_LINE, role: LabelRole::Secondary }
                if let Some(one) = form.download_of() {
                    Label { text: copy::downloading(&one.name), role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
                if let Some(refusal) = form.refusal.as_ref() {
                    Label { text: copy::add_refusal(refusal), role: LabelRole::Secondary }
                }
                div { class: "system-sheet-buttons",
                    Button {
                        label: crate::copy::common::CANCEL,
                        common: Common { aria_label: Some(crate::copy::common::CANCEL.to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        availability: cancel,
                        onclick: move |_| close.call(()),
                    }
                    Button {
                        label: crate::copy::common::ADD,
                        common: Common { aria_label: Some(crate::copy::common::ADD.to_owned()), ..Common::default() },
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
    if let Some(mark) = copy::presence_mark(one.presence) {
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
                common: Common { aria_label: Some(crate::copy::assistants::LOOKING_FOR_CODING_ASSISTANTS.to_owned()), ..Common::default() },
            }
            Label { text: copy::LOADING, role: LabelRole::Secondary }
        }
    }
}

/// Asks before an assistant is taken off the list; Cancel is the one Return presses.
#[component]
pub(super) fn RemoveAlert(question: RemoveQuestion, answer: EventHandler<RemoveAnswer>) -> Element {
    rsx! {
        Alert {
            title: copy::remove_title(&question),
            message: Some(TextLine::from(copy::remove_message())),
            style: AlertStyle::Critical,
            buttons: vec![
                AlertButton::new(crate::copy::common::CANCEL, AlertRole::Cancel, EventHandler::new(move |()| answer.call(RemoveAnswer::Keep))),
                AlertButton::new(crate::copy::common::REMOVE, AlertRole::Destructive, EventHandler::new(move |()| answer.call(RemoveAnswer::Remove))),
            ],
        }
    }
}
