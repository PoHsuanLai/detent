//! The list: a card of rows for each kind of thing the Extensions provide, each row with its
//! name, what it says of itself and a switch. An Extension that cannot be used has the reason
//! where the line would be, and no switch.

use super::link::ExtensionLink;
use crate::copy::{
    EXTENSIONS_EMPTY_HELP, EXTENSIONS_EMPTY_TITLE, NOT_AVAILABLE, extension_line, group_title,
};
use crate::env::Env;
use crate::key_row::{HIT_CLASS, Match, row_id};
use detent_model::{ExtensionPower, ExtensionRow};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;

/// The Extensions installed, read when the page is shown and again after each switch.
#[component]
pub(crate) fn ExtensionsPane() -> Element {
    let env = use_context::<Env>();
    let link = use_context::<ExtensionLink>();
    let toasts = use_toasts();
    let mut list = use_signal({
        let extensions = env.extensions.clone();
        move || extensions.read().list
    });
    // The link's request is taken once: the row stays marked while the page is open.
    let mut marked = use_signal(|| None::<String>);
    use_effect(move || {
        if let Some(id) = link.wanted() {
            marked.set(Some(id));
            link.clear();
        }
    });

    let switch = EventHandler::new(move |(id, power): (String, ExtensionPower)| {
        match env.extensions.set_power(&id, power) {
            Ok(()) => list.set(env.extensions.read().list),
            Err(error) => toasts.push(error.to_string(), None),
        }
    });

    let shown = list();
    if shown.is_empty() {
        return rsx! {
            div { class: "extensions", "data-extensions": "empty",
                EmptyState { title: EXTENSIONS_EMPTY_TITLE, description: EXTENSIONS_EMPTY_HELP }
            }
        };
    }
    rsx! {
        div { class: "extensions", "data-extensions": "list",
            for group in shown.groups {
                FieldGroup { key: "{group.kind:?}", title: Some(group_title(group.kind).to_owned()),
                    for row in group.rows {
                        ExtensionLine {
                            key: "{row.id}",
                            found: if marked().as_deref() == Some(row.id.as_str()) {
                                Match::Hit
                            } else {
                                Match::Miss
                            },
                            row,
                            onswitch: switch,
                        }
                    }
                }
            }
        }
    }
}

/// One Extension: its name and line, and the switch (or why there is none).
#[component]
fn ExtensionLine(
    row: ExtensionRow,
    found: Match,
    onswitch: EventHandler<(String, ExtensionPower)>,
) -> Element {
    let id = row.id.clone();
    let name = row.name.clone();
    let common = Common {
        id: Some(row_id("extension", &row.id)),
        extra_class: match found {
            Match::Hit => ExtraClass::parse(HIT_CLASS).ok(),
            Match::Miss => None,
        },
        ..Common::default()
    };
    rsx! {
        FieldRow {
            label: TextLine::from(row.name.clone()),
            help: extension_line(&row).map(|line| TextLine::from(line.to_owned())),
            common,
            match row.power() {
                Some(power) => rsx! {
                    Toggle {
                        label: name,
                        value: match power {
                            ExtensionPower::On => Check::On,
                            ExtensionPower::Off => Check::Off,
                        },
                        onchange: move |next| {
                            let wanted = match next {
                                Check::On => ExtensionPower::On,
                                Check::Off | Check::Mixed => ExtensionPower::Off,
                            };
                            onswitch.call((id.clone(), wanted));
                        },
                    }
                },
                None if row.is_unusable() => rsx! {
                    Label { text: NOT_AVAILABLE, role: LabelRole::Secondary }
                },
                None => rsx! {},
            }
        }
    }
}
