//! The first-run window: a hero mark, one sentence, the switch, the runtime menu, Done.

use super::io::save;
use crate::env::Env;
use crate::inference::{Found, use_runtimes};
use detent_model::{
    AccountKey, FirstRun, FirstRunEffect, FirstRunInput, LocalOnly, Runtime, Stage, first_run_step,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button::Button;
use ds::components::controls::button_model::Answers;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;
use ds::stack::toast_hub::ToastHub;
use ds::style::icon::family::PlateFamily;
use ds::style::tokens::control_size::ControlSize;
use ds_blitz::{AppHandle, TokioSpawner, use_app_handle};
use ds_settings::{AppName, ConfigRoot, Store, use_environment};
use std::sync::Arc;

/// The menu word that leaves the text slot to Automatic.
const AUTOMATIC: &str = "";

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
/// accountd lists, what the person chose, and the writes Done makes.
#[component]
fn FirstRunWindow() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let app = use_app_handle();
    let mut state = use_signal(FirstRun::default);
    let found = use_runtimes();

    use_effect(move || {
        if let Found::Runtimes(runtimes) = found() {
            send(&mut state, FirstRunInput::Found(runtimes));
        }
    });

    let apply = EventHandler::new(move |input: FirstRunInput| {
        for effect in send(&mut state, input) {
            match effect {
                FirstRunEffect::Save {
                    local_only,
                    runtime,
                } => {
                    let env = env.clone();
                    let app = app.clone();
                    spawn(async move {
                        let answer = match save(&env, local_only, runtime).await {
                            Ok(()) => FirstRunInput::Saved,
                            Err(why) => FirstRunInput::Failed(why),
                        };
                        for effect in send(&mut state, answer) {
                            react(effect, toasts, app.as_ref());
                        }
                    });
                }
                other => react(other, toasts, app.as_ref()),
            }
        }
    });
    rsx! { Welcome { state: state(), onchange: apply } }
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
fn Welcome(state: FirstRun, onchange: EventHandler<FirstRunInput>) -> Element {
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
                text: "Apps can ask for help from a model. By default the model runs here and nothing leaves this computer. You can change this in Settings any time.",
                role: LabelRole::Secondary,
            }
            FieldGroup {
                common: Common { extra_class: ExtraClass::parse("first-run-card").ok(), ..Common::default() },
                FieldRow {
                    label: TextLine::from("Keep AI on this computer"),
                    help: Some(TextLine::from("While on, no request goes to a cloud account.")),
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
                if !state.runtimes.is_empty() {
                    FieldRow {
                        label: TextLine::from("Local model runtime"),
                        help: Some(TextLine::from("Used for text. Automatic picks a model for each request.")),
                        availability: busy,
                        PopUpButton::<String> {
                            items: runtime_items(&state.runtimes),
                            value: Some(state.picked.as_ref().map_or_else(|| AUTOMATIC.to_owned(), |key| key.0.clone())),
                            size: ControlSize::Small,
                            availability: busy,
                            onpick: move |word: String| {
                                let key = (!word.is_empty()).then_some(AccountKey(word));
                                onchange.call(FirstRunInput::Picked(key));
                            },
                        }
                    }
                }
            }
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

/// Automatic, then each runtime found; one that is not running is greyed.
fn runtime_items(runtimes: &[Runtime]) -> Vec<MenuItem<String>> {
    std::iter::once(MenuItem::new(AUTOMATIC.to_owned(), "Automatic".to_owned()))
        .chain(runtimes.iter().map(|runtime| {
            let item = MenuItem::new(runtime.key.0.clone(), runtime.label.clone());
            if runtime.is_running() {
                item
            } else {
                item.with_availability(Availability::Disabled)
            }
        }))
        .collect()
}
