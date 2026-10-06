//! The model picker on the Intelligence page: a card per kind of work, a row per tier, and the
//! footer that says why the last answer came from the model it did.

use super::io::{next_change, open, read, set};
use crate::env::Env;
use crate::files::{Files, value_of};
use detent_model::{
    Absence, KindRows, ModelRow, PickerEffect, PickerInput, PickerState, Reason, loading_text, step,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::live::LiveClient;
use ds_settings::schema::KeyPath;

/// The inferd row that turns the footer on.
const SHOW_REASON: &str = "ai.auto.show_reason";

/// What the "Automatic" choice of every row does.
const AUTOMATIC: &str = "Automatic uses a model that's already running on this computer when it \
    can handle the request. Otherwise it loads one that can, or uses an online model where your \
    privacy settings allow it.";

/// Whether the footer is shown: `ai.auto.show_reason`, on when no schema or file says otherwise
/// (inferd's own default).
pub(crate) fn show_reason(env: &Env, files: &Files) -> Shown {
    let on = env
        .catalogue
        .entries()
        .map(|(_, entry)| entry)
        .find(|entry| entry.spec.path.0 == SHOW_REASON)
        .is_none_or(|entry| value_of(files, entry).as_str() == Some("on"));
    if on { Shown::Visible } else { Shown::Hidden }
}

/// Applies `input` to the picker and returns the effects it asks for.
fn send(state: &mut Signal<PickerState>, input: PickerInput) -> Vec<PickerEffect> {
    let (next, effects) = step(state.peek().clone(), input);
    state.set(next);
    effects
}

/// inferd's model rows, or why they are not there. `reasons` is whether the footer is shown.
#[component]
pub fn ModelPicker(reasons: Shown) -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let mut state = use_signal(PickerState::default);
    let mut client = use_signal(|| None::<LiveClient>);
    let mut reason = use_signal(|| None::<Reason>);

    let look = env.clone();
    use_future(move || {
        let bus = look.bus.clone();
        async move {
            let found = match open(&bus).await {
                Ok(opened) => read(&opened).await.map(|described| (opened, described)),
                Err(absence) => Err(absence),
            };
            match found {
                Ok((opened, described)) => {
                    send(&mut state, PickerInput::Found(Ok(described)));
                    client.set(Some(opened.clone()));
                    if let Ok(mut changes) = opened.changes().await {
                        while let Some((path, word)) = next_change(&mut changes).await {
                            send(&mut state, PickerInput::Changed { path, word });
                        }
                    }
                }
                Err(absence) => {
                    send(&mut state, PickerInput::Found(Err(absence)));
                }
            }
        }
    });

    let feed = env.reasons.clone();
    use_future(move || {
        let taken = feed.take();
        async move {
            let Some(mut events) = taken else { return };
            while let Some(event) = events.recv().await {
                if let Some(next) = Reason::of(&event) {
                    reason.set(Some(next));
                }
            }
        }
    });

    let choose = EventHandler::new(move |(path, word): (KeyPath, String)| {
        let effects = send(&mut state, PickerInput::Chose { path, word });
        for effect in effects {
            match effect {
                PickerEffect::Set { path, word } => {
                    let Some(opened) = client.peek().clone() else {
                        continue;
                    };
                    spawn(async move {
                        let input = match set(&opened, &path, &word).await {
                            Ok(()) => PickerInput::Applied(path),
                            Err(why) => PickerInput::Refused { path, why },
                        };
                        for effect in send(&mut state, input) {
                            if let PickerEffect::Tell(text) = effect {
                                toasts.push(text, None);
                            }
                        }
                    });
                }
                PickerEffect::Tell(text) => toasts.push(text, None),
            }
        }
    });

    match state() {
        PickerState::Probing => rsx! {},
        PickerState::Unavailable(absence) => rsx! { Unavailable { absence } },
        PickerState::Ready(ready) => {
            let applying = ready.applying.clone();
            rsx! {
                div { class: "picker", "data-picker": "ready",
                    for kind in ready.kinds.iter().cloned() {
                        KindCard { key: "{kind.kind}", kind, applying: applying.clone(), onchoose: choose }
                    }
                    div { class: "picker-auto",
                        Label { text: AUTOMATIC, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                    }
                    if reasons == Shown::Visible {
                        ReasonFooter { reason: reason() }
                    }
                }
            }
        }
    }
}

/// What the absence of the picker says.
fn absence_help(absence: &Absence) -> String {
    match absence {
        Absence::NoBus => "Settings is not connected to the session bus.".to_owned(),
        Absence::NotRunning => {
            "inferd is not running, so there are no models to choose from.".to_owned()
        }
        Absence::NotPermitted => {
            "inferd does not list Settings among its callers (settings in inferd.toml).".to_owned()
        }
        Absence::Failed(why) => format!("inferd could not be asked: {why}"),
    }
}

/// The picker's place when inferd cannot be reached: one row that says so, blocking nothing.
#[component]
fn Unavailable(absence: Absence) -> Element {
    rsx! {
        div { class: "picker", "data-picker": "unavailable",
            FieldGroup { title: Some("Model choice".to_owned()),
                FieldRow {
                    label: TextLine::from("Models"),
                    help: Some(TextLine::from(absence_help(&absence))),
                    availability: Availability::Disabled,
                    Label { text: "Unavailable", role: LabelRole::Secondary }
                }
            }
        }
    }
}

/// One kind of work: a card of its tiers.
#[component]
fn KindCard(
    kind: KindRows,
    applying: Option<detent_model::Applying>,
    onchoose: EventHandler<(KeyPath, String)>,
) -> Element {
    rsx! {
        div { class: "picker-kind",
            FieldGroup { title: Some(kind.label.clone()),
                for row in kind.rows.iter().cloned() {
                    TierRow { key: "{row.path.0}", applying: applying.clone(), row, onchoose }
                }
            }
        }
    }
}

/// One tier's menu; while a set applies the row says "Loading <model>" and takes no input.
#[component]
fn TierRow(
    row: ModelRow,
    applying: Option<detent_model::Applying>,
    onchoose: EventHandler<(KeyPath, String)>,
) -> Element {
    let here = applying.as_ref().filter(|set| set.path == row.path);
    let availability = match (&applying, here) {
        (_, Some(_)) => Availability::Busy,
        (Some(_), None) => Availability::Disabled,
        (None, None) => Availability::Enabled,
    };
    let loading = here.map(|set| loading_text(&row, &set.word));
    let shown = here.map_or_else(|| row.value.clone(), |set| set.word.clone());
    let items: Vec<MenuItem<String>> = row
        .words
        .iter()
        .map(|word| MenuItem::new(word.clone(), row.label_of(word)))
        .collect();
    let path = row.path.clone();
    rsx! {
        FieldRow {
            label: TextLine::from(row.tier),
            availability,
            common: Common { id: None, ..Common::default() },
            if let Some(text) = loading {
                Label { text, role: LabelRole::Secondary, common: Common { extra_class: ds::root::pass_through::ExtraClass::parse("picker-loading").ok(), ..Common::default() } }
            }
            PopUpButton::<String> {
                items,
                value: Some(shown),
                size: ControlSize::Small,
                availability: if availability == Availability::Enabled { Availability::Enabled } else { Availability::Disabled },
                onpick: move |word| onchoose.call((path.clone(), word)),
            }
        }
    }
}

/// The line under the picker: why the last model answered, or what the line is for.
#[component]
fn ReasonFooter(reason: Option<Reason>) -> Element {
    let text = reason.map_or_else(
        || "Each answer says which model gave it and why.".to_owned(),
        |reason| reason.line(),
    );
    rsx! {
        div { class: "picker-reason",
            Label { text, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
        }
    }
}
