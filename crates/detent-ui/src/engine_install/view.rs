//! The sheet: "Download llama.cpp?", who it is for, "Not Now" and "Download", then the progress in
//! place, then what the download came to.

use super::board::Board;
use crate::copy;
use detent_model::{
    EngineAction, EngineBrief, EngineOffer, EngineOfferInput, InstallScope, Waiting, engine_brief,
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
    let offer = board.state.read().offer.clone();
    let Some(brief) = engine_brief(&offer) else {
        return rsx! {};
    };
    let kind = kind_of(&offer);
    let name = brief.engine.name.as_str();
    let title = copy::engine_install::title(name, &brief.matter);
    let message = copy::engine_install::message(name, &brief.matter);
    // Escape does what the leaving button does, and Return what the going-on button does.
    let leave = brief.secondary.or(brief.primary).map(EngineAction::input);
    let go = brief.primary.map(EngineAction::input);
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
            label: title.clone(),
            width: SheetWidth::Regular,
            onclose: close,
            on_return: enter,
            div { class: "engine-sheet", "data-engine-sheet": kind,
                Label { text: title.clone(), role: LabelRole::Primary, style: LabelStyle::Headline }
                if !message.is_empty() {
                    Label { text: message.clone(), role: LabelRole::Secondary }
                }
                Scopes { brief: brief.clone(), ask }
                Meter { brief: brief.clone() }
                div { class: "engine-sheet-buttons",
                    if let Some(action) = brief.secondary {
                        ActionButton { action, answers: Answers::Escape, ask }
                    }
                    if let Some(action) = brief.primary {
                        ActionButton { action, answers: Answers::Return, ask }
                    }
                }
            }
        }
    }
}

/// Which of the sheet's states this is, for a test to find.
fn kind_of(offer: &EngineOffer) -> &'static str {
    match offer {
        EngineOffer::Idle => "closed",
        EngineOffer::Asking { .. } => "asking",
        EngineOffer::Pointing { .. } => "pointing",
        EngineOffer::Working { .. } => "working",
        EngineOffer::Done { .. } => "done",
        EngineOffer::Failed { .. } => "failed",
    }
}

/// "Install for", when there is more than one way.
#[component]
fn Scopes(brief: EngineBrief, ask: EventHandler<EngineOfferInput>) -> Element {
    if brief.scopes.is_empty() {
        return rsx! {};
    }
    let items: Vec<MenuItem<String>> = brief
        .scopes
        .iter()
        .map(|scope| {
            MenuItem::new(
                word_of(*scope).to_owned(),
                copy::engine_install::scope(*scope).to_owned(),
            )
        })
        .collect();
    rsx! {
        FieldGroup {
            FieldRow {
                label: TextLine::from(copy::engine_install::install_for()),
                common: Common { id: Some("engine-sheet-scope".to_owned()), ..Common::default() },
                PopUpButton::<String> {
                    items,
                    value: Some(word_of(brief.scope).to_owned()),
                    size: ControlSize::Small,
                    onpick: move |word: String| ask.call(EngineOfferInput::Scope(scope_of(&word))),
                }
            }
        }
    }
}

/// The bar of a download, or of work with no end in sight.
#[component]
fn Meter(brief: EngineBrief) -> Element {
    match (brief.progress, brief.waiting) {
        (Some(permille), _) => rsx! {
            ProgressIndicator {
                style: ProgressStyle::Bar,
                progress: Progress::Known(Fraction(u16::try_from(permille.min(1000)).unwrap_or(1000))),
                common: Common { aria_label: Some(crate::copy::common::DOWNLOAD_PROGRESS.to_owned()), ..Common::default() },
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
            common: Common { aria_label: Some(crate::copy::engine_install::CHECKING.to_owned()), ..Common::default() },
        }
    }
}

/// One of the sheet's buttons.
#[component]
fn ActionButton(
    action: EngineAction,
    answers: Answers,
    ask: EventHandler<EngineOfferInput>,
) -> Element {
    rsx! {
        Button {
            label: copy::engine_install::action(action),
            answers,
            onclick: move |_| ask.call(action.input()),
        }
    }
}
