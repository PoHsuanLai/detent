//! One tier's menu: the row every part of the picker is made of.

use crate::key_row::row_id;
use detent_model::{Adding, Applying, Choice, Gate, ModelRow, loading_text};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_help::HelpShown;
use ds::components::fields::field_row::FieldRow;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::KeyPath;

/// What the button says while nothing is chosen.
const NOTHING_CHOSEN: &str = "Choose\u{2026}";

/// One tier's menu; while a set applies the row says "Loading <model>" and takes no input, and
/// while the add-account sheet is open no row does. A row that has chosen nothing shows no
/// selection.
#[component]
pub(crate) fn TierRow(
    row: ModelRow,
    applying: Option<Applying>,
    adding: Adding,
    onchoose: EventHandler<(KeyPath, String)>,
) -> Element {
    let here = applying.as_ref().filter(|set| set.path == row.path);
    let availability = match (&applying, here, adding) {
        (_, Some(_), _) => Availability::Busy,
        (Some(_), None, _) | (None, None, Adding::SheetOpen) => Availability::Disabled,
        (None, None, Adding::Idle) => Availability::Enabled,
    };
    let loading = here.map(|set| loading_text(&row, &set.word));
    let shown = here.map_or_else(|| row.choice(), |set| Choice::of(&set.word));
    let items = menu_items(&row);
    let path = row.path.clone();
    rsx! {
        FieldRow {
            label: TextLine::from(row.label.clone()),
            help: if row.help.is_empty() { None } else { Some(TextLine::from(row.help.clone())) },
            help_shown: HelpShown::OnRequest,
            availability,
            common: Common { id: Some(row_id("picker", &row.path.0)), ..Common::default() },
            if let Some(text) = loading {
                Label { text, role: LabelRole::Secondary, common: Common { extra_class: ds::root::pass_through::ExtraClass::parse("picker-loading").ok(), ..Common::default() } }
            }
            PopUpButton::<String> {
                items,
                value: shown.selected(),
                title: Some(NOTHING_CHOSEN.to_owned()),
                size: ControlSize::Small,
                availability: if availability == Availability::Enabled { Availability::Enabled } else { Availability::Disabled },
                onpick: move |word| onchoose.call((path.clone(), word)),
            }
        }
    }
}

/// The row's menu: the ungrouped choices first, then a header per group over its choices.
fn menu_items(row: &ModelRow) -> Vec<MenuItem<String>> {
    row.sections
        .iter()
        .flat_map(|section| {
            let header = section.group.clone().map(MenuItem::Header);
            let items = section
                .words
                .iter()
                .flat_map(|word| choice_items(row, word));
            header.into_iter().chain(items)
        })
        .collect()
}

/// The menu lines for `word`: the choice, and when it cannot be picked as it is, its reason in a
/// status line right under it. A model that needs an account stays pickable (picking it offers
/// to add one); any other reason greys the choice.
fn choice_items(row: &ModelRow, word: &str) -> Vec<MenuItem<String>> {
    let item = MenuItem::new(word.to_owned(), row.label_of(word));
    let info = |reason: &str| MenuItem::Info {
        title: reason.to_owned(),
        detail: None,
    };
    match row.gate(word) {
        Gate::Open => vec![item],
        Gate::NeedsAccount => vec![item, info(row.unavailable_reason(word).unwrap_or_default())],
        Gate::Blocked(reason) => vec![
            item.with_availability(Availability::Disabled),
            info(&reason),
        ],
    }
}
