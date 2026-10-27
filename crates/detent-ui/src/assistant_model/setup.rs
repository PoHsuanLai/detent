//! The first-run step: a hero mark, one sentence, the picker's row, Skip and Continue. Choosing
//! writes at once, as everywhere in Settings; Skip leaves without writing and Continue leaves
//! once a model is chosen. Either closes the window.

use super::picker::AssistantPicker;
use crate::env::Env;
use crate::inference::{Picker, use_picker};
use crate::live::start;
use detent_model::{Clearing, PageKind, SetupAction, SetupAvailability, setup_availability};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button::Button;
use ds::components::controls::button_model::Answers;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;
use ds::style::icon::family::PlateFamily;
use ds::style::tokens::control_size::ControlSize;
use ds_blitz::{AppHandle, use_app_handle};

/// Said once, at the top.
const HEADING: &str = "Choose a Model for Your Assistant";

/// The step, under the frame: the picker's row and the two buttons.
#[component]
pub(crate) fn AssistantSetup() -> Element {
    let env = use_context::<Env>();
    let app = use_app_handle();
    let Picker { state, ask } = use_picker();
    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
            async move { start(&bus, PageKind::AssistantModel.modules_read()).await }
        }
    });
    let now = state();
    let skip = availability_of(setup_availability(&now, SetupAction::Skip));
    let next = availability_of(setup_availability(&now, SetupAction::Continue));
    let (skip_app, next_app) = (app.clone(), app);
    rsx! {
        div { class: "assistant-setup", "data-assistant": "setup",
            IconView {
                source: IconSource::Glyph(Icon::Sparkles),
                size: IconSize::Tile48,
                plate: Some(PlateFamily::Violet),
            }
            Label { text: HEADING, role: LabelRole::Primary, style: LabelStyle::Title }
            div { class: "assistant-setup-card",
                AssistantPicker { state: now, clearing: Clearing::Withheld, onask: ask }
            }
            div { class: "assistant-setup-buttons",
                Button {
                    label: "Skip",
                    answers: Answers::Escape,
                    size: ControlSize::Large,
                    availability: skip,
                    common: Common { extra_class: ExtraClass::parse("assistant-skip").ok(), ..Common::default() },
                    onclick: move |_| leave(skip_app.as_ref()),
                }
                Button {
                    label: "Continue",
                    answers: Answers::Return,
                    size: ControlSize::Large,
                    availability: next,
                    common: Common { extra_class: ExtraClass::parse("assistant-continue").ok(), ..Common::default() },
                    onclick: move |_| leave(next_app.as_ref()),
                }
            }
        }
    }
}

/// A button that can be pressed now is enabled.
fn availability_of(availability: SetupAvailability) -> Availability {
    match availability {
        SetupAvailability::Now => Availability::Enabled,
        SetupAvailability::NotYet => Availability::Disabled,
    }
}

/// Closes the window. `app` is the running app, absent in the harness, where there is no window
/// to close.
fn leave(app: Option<&AppHandle>) {
    if let Some(app) = app {
        // The loop having ended already is the same as the window being closed.
        let _ended = app.quit();
    }
}
