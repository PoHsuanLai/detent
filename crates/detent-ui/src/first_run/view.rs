//! The first-run window: a hero mark, one sentence, the switch, the assistant's model, Done.

use super::io::save;
use crate::assistant_model::AssistantPicker;
use crate::env::Env;
use crate::inference::{Picker, use_picker};
use crate::live::start;
use detent_model::{
    Clearing, FirstRun, FirstRunEffect, FirstRunInput, LocalOnly, PickerInput, PickerState, Stage,
    assistant_model_page, first_run_step, modules_read,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button::Button;
use ds::components::controls::button_model::Answers;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;
use ds::stack::toast_hub::ToastHub;
use ds::style::icon::family::PlateFamily;
use ds::style::tokens::control_size::ControlSize;
use ds_blitz::{AppHandle, TokioSpawner, use_app_handle};
use ds_settings::{AppName, ConfigRoot, Store, use_environment};
use std::sync::Arc;

/// What the model is for, and that nothing is chosen for the person.
const MODEL_NOTE: &str = "Choose a model for your assistant, or do it later in Settings.";

fn send(state: &mut Signal<FirstRun>, input: FirstRunInput) -> Vec<FirstRunEffect> {
    let (next, effects) = first_run_step(state.peek().clone(), input);
    state.set(next);
    effects
}

/// The first-run window's root. Reads its [`Env`] from the context the launcher provides.
#[component]
pub fn FirstRunApp() -> Element {
    let env = use_context::<Env>();
    let appearance = use_environment(
        Store::new(
            ConfigRoot::Scratch(env.config_home.0.clone()),
            AppName::QUIRE,
        ),
        env.system.clone(),
        Arc::new(TokioSpawner::current()),
    );
    let now = appearance();
    rsx! {
        Ds {
            appearance: now.settings.appearance.appearance(),
            system: now.system,
            material: Material::Window,
            tint_alpha: Some(now.tint_alpha()),
            stack: Some(now.material_stack()),
            window: WindowFrame::titlebar("Welcome".to_owned(), TrafficLights::Shown),
            style { {crate::STYLE} }
            FirstRunWindow {}
        }
    }
}

/// The window under the frame (the toasts that report a failed write mount in the frame): what
/// the person chose, the assistant's model row, and the write Done makes.
#[component]
fn FirstRunWindow() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let app = use_app_handle();
    let mut state = use_signal(FirstRun::default);
    let Picker { state: picker, ask } = use_picker();
    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
            async move { start(&bus, modules_read(&assistant_model_page())).await }
        }
    });

    let apply = EventHandler::new(move |input: FirstRunInput| {
        for effect in send(&mut state, input) {
            match effect {
                FirstRunEffect::Save { local_only } => {
                    let answer = match save(&env, local_only) {
                        Ok(()) => FirstRunInput::Saved,
                        Err(why) => FirstRunInput::Failed(why),
                    };
                    for effect in send(&mut state, answer) {
                        react(effect, toasts, app.as_ref());
                    }
                }
                other => react(other, toasts, app.as_ref()),
            }
        }
    });
    rsx! { Welcome { state: state(), picker: picker(), onchange: apply, onask: ask } }
}

/// What the window does for an effect that is not a write. `app` is the running app, absent in
/// the harness, where there is no window to close.
fn react(effect: FirstRunEffect, toasts: ToastHub, app: Option<&AppHandle>) {
    match effect {
        FirstRunEffect::Tell(text) => toasts.push(text, None),
        FirstRunEffect::Close => {
            if let Some(app) = app {
                // The loop having ended already is the same as the window being closed.
                let _ended = app.quit();
            }
        }
        FirstRunEffect::Save { .. } => {}
    }
}

#[component]
fn Welcome(
    state: FirstRun,
    picker: PickerState,
    onchange: EventHandler<FirstRunInput>,
    onask: EventHandler<PickerInput>,
) -> Element {
    let busy = if state.stage == Stage::Asking {
        Availability::Enabled
    } else {
        Availability::Busy
    };
    let stage = format!("{:?}", state.stage).to_lowercase();
    let on = if state.local_only == LocalOnly::On {
        Check::On
    } else {
        Check::Off
    };
    rsx! {
        div { class: "first-run", "data-first-run": "{stage}",
            IconView {
                source: IconSource::Glyph(Icon::Sparkles),
                size: IconSize::Tile48,
                plate: Some(PlateFamily::Violet),
            }
            Label { text: "AI on this computer", role: LabelRole::Primary, style: LabelStyle::Title }
            Label {
                text: "Apps can ask a model for help. By default it runs here and nothing leaves this computer.",
                role: LabelRole::Secondary,
            }
            FieldGroup {
                common: Common { extra_class: ExtraClass::parse("first-run-card").ok(), ..Common::default() },
                FieldRow {
                    label: TextLine::from("Keep AI on this computer"),
                    help: Some(TextLine::from("Nothing is sent to online accounts.")),
                    availability: busy,
                    Toggle {
                        label: "Keep AI on this computer",
                        value: on,
                        onchange: move |next| {
                            onchange.call(FirstRunInput::Kept(match next {
                                Check::On => LocalOnly::On,
                                Check::Off | Check::Mixed => LocalOnly::Off,
                            }));
                        },
                    }
                }
            }
            Label { text: MODEL_NOTE, role: LabelRole::Secondary }
            AssistantPicker { state: picker, clearing: Clearing::Withheld, onask }
            Button {
                label: "Done",
                answers: Answers::Return,
                size: ControlSize::Large,
                availability: busy,
                onclick: move |_| onchange.call(FirstRunInput::Finish),
            }
        }
    }
}
