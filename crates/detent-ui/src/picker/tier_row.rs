//! One tier's menu: the row every part of the picker is made of, and what opens under it when
//! the choice asks more (Local's engine and model, Automatic's list).

use super::local_rows::LocalRows;
use super::note::NoteLine;
use super::pool_editor::PoolEditor;
use crate::copy::picker as copy;
use crate::key_row::row_id;
use detent_model::{
    Adding, Applying, Choice, Clearing, Gate, MenuLine, ModelRow, PickerInput, loading_of,
};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_help::HelpShown;
use ds::components::fields::field_row::FieldRow;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;
use ds::style::tokens::control_size::ControlSize;

/// What the button says while nothing is chosen.
pub(super) const NOTHING_CHOSEN: &str = crate::copy::common::CHOOSE_ELLIPSIS;

/// One tier's menu; while a set applies the row says "Loading <model>" and takes no input, and
/// while the add-account sheet is open no row does. A row that has chosen nothing shows no
/// selection.
#[component]
pub(crate) fn TierRow(
    row: ModelRow,
    applying: Option<Applying>,
    adding: Adding,
    clearing: Clearing,
    onask: EventHandler<PickerInput>,
) -> Element {
    let here = applying.as_ref().filter(|set| set.path == row.path);
    let availability = match (&applying, here, adding) {
        (_, Some(_), _) => Availability::Busy,
        (Some(_), None, _) | (None, None, Adding::SheetOpen) => Availability::Disabled,
        (None, None, Adding::Idle) => Availability::Enabled,
    };
    let loading = here.map(|set| copy::loading(&loading_of(&row, &set.word)));
    let shown = here.map_or_else(|| row.choice(), |set| row.choice_when(&set.word));
    let calm = if availability == Availability::Enabled {
        Availability::Enabled
    } else {
        Availability::Disabled
    };
    let items = menu_items(&row, clearing);
    let path = row.path.clone();
    let local = row.local_view();
    let note = row.unready_gap().filter(|_| local.is_none()).map(copy::gap);
    rsx! {
        FieldRow {
            label: TextLine::from(row.label.clone()),
            help: if row.help.is_empty() { None } else { Some(TextLine::from(row.help.clone())) },
            help_shown: HelpShown::OnRequest,
            availability,
            common: Common { id: Some(row_id("picker", &row.path.0)), ..Common::default() },
            if let Some(text) = loading {
                Label { text, role: LabelRole::Secondary, common: Common { extra_class: ExtraClass::parse("picker-loading").ok(), ..Common::default() } }
            }
            PopUpButton::<String> {
                items,
                value: shown.selected(),
                title: Some(NOTHING_CHOSEN.to_owned()),
                size: ControlSize::Small,
                availability: calm,
                onpick: move |word| onask.call(PickerInput::Chose { path: path.clone(), word }),
            }
        }
        if let Some(text) = row.row_note().map(str::to_owned) {
            NoteLine { text, kind: "row" }
        }
        if let Some(view) = local {
            LocalRows { path: row.path.clone(), view, availability: calm, onask }
        }
        if let Choice::Automatic { pool } = &shown {
            PoolEditor { pool: pool.clone(), availability: calm, onask }
            if let Some(text) = note {
                NoteLine { text, kind: "unready" }
            }
        }
    }
}

/// The row's menu: the ungrouped choices first, then a header per group over its choices.
fn menu_items(row: &ModelRow, clearing: Clearing) -> Vec<MenuItem<String>> {
    row.menu(clearing)
        .into_iter()
        .flat_map(|line| match line {
            MenuLine::Header(name) => vec![MenuItem::Header(name)],
            MenuLine::Agents => vec![MenuItem::Header(copy::CODING_AGENTS.to_owned())],
            MenuLine::Pick { word, label, gate } => {
                pick_items(&word, &copy::pick_label(&label), &gate)
            }
        })
        .collect()
}

/// The menu lines for `word`: the choice, and when it cannot be picked as it is, its reason in a
/// status line right under it. A model that needs an account stays pickable (picking it offers
/// to add one); any other reason greys the choice.
pub(super) fn pick_items(word: &str, label: &str, gate: &Gate) -> Vec<MenuItem<String>> {
    let item = MenuItem::new(word.to_owned(), label.to_owned());
    let info = |reason: &str| MenuItem::Info {
        title: reason.to_owned(),
        detail: None,
    };
    match gate {
        Gate::Open => vec![item],
        Gate::NeedsAccount => vec![item, info(&copy::gate_note(gate).unwrap_or_default())],
        Gate::Blocked(reason) => vec![
            item.with_availability(Availability::Disabled),
            info(&copy::blocked_line(reason)),
        ],
        // A coding agent while they are off: pickable, and picking asks to turn them on.
        Gate::TurnOn => vec![item, info(&copy::gate_note(gate).unwrap_or_default())],
    }
}
