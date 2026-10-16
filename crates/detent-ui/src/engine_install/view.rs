//! The sheet: "Download llama.cpp?", who it is for, "Not Now" and "Download", then the progress in
//! place, then what the download came to.

use super::board::Board;
use detent_model::{
    EngineSheet, EngineSheetInput, INSTALL_FOR, InstallScope, SheetButton, SheetWords, Waiting,
    sheet_words,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The word of a scope in the pop-up button.
fn word_of(scope: InstallScope) -> &'static str {
    match scope {
        InstallScope::AllUsers => "all_users",
        InstallScope::JustMe => "just_me",
    }
}

/// The scope a word of the pop-up button names.
fn scope_of(word: &str) -> InstallScope {
    match word {
        "all_users" => InstallScope::AllUsers,
        _ => InstallScope::JustMe,
    }
}

/// The sheet, when the board says there is one; nothing otherwise (and nothing on a page that has
/// no board).
#[component]
pub(crate) fn EngineSheetView() -> Element {
    let Some(board) = try_use_context::<Board>() else {
        return rsx! {};
    };
    let sheet = board.state.read().sheet.clone();
    let Some(words) = sheet_words(&sheet) else {
        return rsx! {};
    };
    let kind = kind_of(&sheet);
    // Escape does what the leaving button does, and Return what the going-on button does.
    let leave = words
        .secondary
        .or(words.primary)
        .map(|button| button.press.input());
    let go = words.primary.map(|button| button.press.input());
    let ask = board.ask;
    let close = EventHandler::new(move |()| {
        if let Some(input) = leave.clone() {
            ask.call(input);
        }
    });
    let enter = EventHandler::new(move |()| {
        if let Some(input) = go.clone() {
            ask.call(input);
        }
    });
    rsx! {
        Sheet {
            label: words.title.clone(),
            width: SheetWidth::Regular,
            onclose: close,
            on_return: enter,
            div { class: "engine-sheet", "data-engine-sheet": kind,
                Label { text: words.title.clone(), role: LabelRole::Primary, style: LabelStyle::Headline }
                if !words.message.is_empty() {
                    Label { text: words.message.clone(), role: LabelRole::Secondary }
                }
                Scopes { words: words.clone(), ask }
                Meter { words: words.clone() }
                div { class: "engine-sheet-buttons",
                    if let Some(button) = words.secondary {
                        Pressed { button, answers: Answers::Escape, ask }
                    }
                    if let Some(button) = words.primary {
                        Pressed { button, answers: Answers::Return, ask }
                    }
                }
            }
        }
    }
}

/// Which of the sheet's states this is, for a test to find.
fn kind_of(sheet: &EngineSheet) -> &'static str {
    match sheet {
        EngineSheet::Closed => "closed",
        EngineSheet::Asking { .. } => "asking",
        EngineSheet::Pointing { .. } => "pointing",
        EngineSheet::Working { .. } => "working",
        EngineSheet::Done { .. } => "done",
        EngineSheet::Failed { .. } => "failed",
    }
}

/// "Install for", when there is more than one way.
#[component]
fn Scopes(words: SheetWords, ask: EventHandler<EngineSheetInput>) -> Element {
    if words.scopes.is_empty() {
        return rsx! {};
    }
    let items: Vec<MenuItem<String>> = words
        .scopes
        .iter()
        .map(|scope| MenuItem::new(word_of(*scope).to_owned(), scope.label().to_owned()))
        .collect();
    rsx! {
        FieldGroup {
            FieldRow {
                label: TextLine::from(INSTALL_FOR),
                common: Common { id: Some("engine-sheet-scope".to_owned()), ..Common::default() },
                PopUpButton::<String> {
                    items,
                    value: Some(word_of(words.scope).to_owned()),
                    size: ControlSize::Small,
                    onpick: move |word: String| ask.call(EngineSheetInput::Scope(scope_of(&word))),
                }
            }
        }
    }
}

/// The bar of a download, or of work with no end in sight.
#[component]
fn Meter(words: SheetWords) -> Element {
    match (words.progress, words.waiting) {
        (Some(permille), _) => rsx! {
            ProgressIndicator {
                style: ProgressStyle::Bar,
                progress: Progress::Known(Fraction(u16::try_from(permille.min(1000)).unwrap_or(1000))),
                common: Common { aria_label: Some("Download progress".to_owned()), ..Common::default() },
            }
        },
        (None, Waiting::Yes) => rsx! { Pending {} },
        (None, Waiting::No) => rsx! {},
    }
}

#[component]
fn Pending() -> Element {
    let token = use_hook(PendingToken::start);
    rsx! {
        ProgressIndicator {
            style: ProgressStyle::Bar,
            progress: Progress::Unknown(Operation::Running(token)),
            common: Common { aria_label: Some("Checking".to_owned()), ..Common::default() },
        }
    }
}

/// One of the sheet's buttons.
#[component]
fn Pressed(button: SheetButton, answers: Answers, ask: EventHandler<EngineSheetInput>) -> Element {
    rsx! {
        Button {
            label: button.label,
            answers,
            onclick: move |_| ask.call(button.press.input()),
        }
    }
}
