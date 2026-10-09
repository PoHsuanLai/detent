//! One place the built-in assistant may run: its name and switch, and under an Advanced
//! disclosure its model and, for a cloud account, "Ask me each time".

use detent_model::ModelOffer;
use dioxus::prelude::*;
use docket_settings::ModelChoice;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::disclosure::Disclosure;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The model choice that lets the assistant decide.
const AUTOMATIC: &str = "";

/// What the row's switch is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Switch {
    /// This computer: always on, so there is no switch.
    Always,
    /// A switch, with its state.
    Is(Check),
}

/// What a row shows.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlaceView {
    /// The place's name.
    pub name: String,
    /// The line under the name.
    pub detail: Option<String>,
    /// Its switch.
    pub switch: Switch,
    /// The model picked, if any.
    pub model: Option<ModelChoice>,
    /// The models the place offers.
    pub offers: Vec<ModelOffer>,
    /// "Ask me each time": its state for a cloud account, none for any other place.
    pub ask: Option<Check>,
    /// Whether it answers now, for one of the person's computers.
    pub standing: Option<&'static str>,
}

/// What a row reports.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct PlaceHandlers {
    /// The switch was flipped.
    pub switched: EventHandler<Check>,
    /// "Ask me each time" was flipped.
    pub asked: EventHandler<Check>,
    /// A model was picked; none is Automatic.
    pub picked: EventHandler<Option<ModelChoice>>,
    /// Remove was pressed, for one of the person's computers.
    pub removed: Option<EventHandler<()>>,
}

/// A row with what it reports.
#[derive(Clone, PartialEq)]
pub(crate) struct PlaceEntry {
    /// What the row shows.
    pub view: PlaceView,
    /// What it reports.
    pub on: PlaceHandlers,
}

fn items_of(view: &PlaceView) -> Vec<MenuItem<String>> {
    let mut items = vec![MenuItem::new(AUTOMATIC.to_owned(), "Automatic")];
    items.extend(
        view.offers
            .iter()
            .map(|offer| MenuItem::new(offer.choice.0.clone(), offer.label.clone())),
    );
    let kept = view.model.as_ref().map(|model| &model.0);
    if let Some(word) = kept.filter(|word| !view.offers.iter().any(|o| &o.choice.0 == *word)) {
        let name = word
            .rsplit_once('/')
            .map_or(word.as_str(), |(_, name)| name);
        items.push(MenuItem::new(word.clone(), name.to_owned()));
    }
    items
}

/// A place's row and its Advanced disclosure.
#[component]
pub(crate) fn PlaceRow(view: PlaceView, on: PlaceHandlers) -> Element {
    let mut shown = use_signal(|| Shown::Hidden);
    let items = items_of(&view);
    let current = view
        .model
        .as_ref()
        .map_or_else(|| AUTOMATIC.to_owned(), |model| model.0.clone());
    let ask = view.ask;
    let ask_availability = match view.switch {
        Switch::Is(Check::Off) => Availability::Disabled,
        Switch::Always | Switch::Is(_) => Availability::Enabled,
    };
    rsx! {
        FieldRow {
            label: TextLine::from(view.name.clone()),
            help: view.detail.clone().map(TextLine::from),
            match view.switch {
                Switch::Always => rsx! {
                    Label { text: "Always on", role: LabelRole::Secondary, style: LabelStyle::Footnote }
                },
                Switch::Is(value) => rsx! {
                    Toggle {
                        label: view.name.clone(),
                        value,
                        onchange: move |next| on.switched.call(next),
                    }
                },
            }
            if let Some(standing) = view.standing {
                Label { text: standing, role: LabelRole::Secondary, style: LabelStyle::Footnote }
            }
            if let Some(removed) = on.removed {
                Button {
                    common: Common { aria_label: Some(format!("Remove {}", view.name)), ..Common::default() },
                    label: "Remove",
                    size: ControlSize::Small,
                    onclick: move |_| removed.call(()),
                }
            }
        }
        Disclosure {
            label: Some("Advanced".to_owned()),
            common: Common {
                aria_label: Some(format!("Advanced, {}", view.name)),
                ..Common::default()
            },
            shown: shown(),
            on_toggle: move |next| shown.set(next),
            FieldGroup {
                FieldRow { label: TextLine::from("Model"),
                    PopUpButton::<String> {
                        items,
                        value: Some(current),
                        size: ControlSize::Small,
                        onpick: move |word: String| {
                            on.picked.call((!word.is_empty()).then_some(ModelChoice(word)));
                        },
                    }
                }
                if let Some(value) = ask {
                    FieldRow {
                        label: TextLine::from("Ask me each time"),
                        help: Some(TextLine::from("The assistant asks before it uses this account.")),
                        Toggle {
                            label: "Ask me each time",
                            value,
                            availability: ask_availability,
                            onchange: move |next| on.asked.call(next),
                        }
                    }
                }
            }
        }
    }
}
