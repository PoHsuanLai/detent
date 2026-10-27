//! One corner of the screen picture: a pop-up button for each set of keys the corner has a row
//! for, and the way to add another.

use super::sheet::{SheetState, Step};
use crate::controls::FileOnlyControl;
use crate::copy::hot_corners::{
    add_fault, choice_label, corner_label, corner_name, keys_glyphs, keys_names, modifier_glyph,
    modifier_name,
};
use crate::files::{Edit, Files, value_of, written_value};
use crate::key_row::row_id;
use detent_model::{
    Corner, CornerChoice, CornerMenuLine, HotBinding, HotKeys, HotKnown, HotModifier, HotPicked,
    KeyEntry, add_row, binding_for, bindings_of, corner_menu, corner_picked, corner_shown,
    corner_value, remove_row, shown_rows, with_binding,
};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::controls::checkbox::Checkbox;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The bindings of `entry`: its list, or the old word a file written before the list holds.
pub(super) fn bindings_in(files: &Files, entry: &KeyEntry) -> Vec<HotBinding> {
    bindings_of(&value_of(files, entry))
}

/// Stores `bindings` as `entry`'s value.
pub(super) fn store(onedit: EventHandler<Edit>, entry: &KeyEntry, bindings: &[HotBinding]) {
    onedit.call(Edit {
        entry: entry.clone(),
        value: corner_value(bindings),
    });
}

/// The pop-up's lines: the choices and the rules between them.
fn menu_items() -> Vec<MenuItem<CornerChoice>> {
    corner_menu()
        .into_iter()
        .map(|line| match line {
            CornerMenuLine::Choice(choice) => MenuItem::new(choice, choice_label(choice)),
            CornerMenuLine::Separator => MenuItem::Separator,
        })
        .collect()
}

/// The classes that put a cell in its corner of the screen.
fn cell_class(corner: Corner) -> &'static str {
    match corner {
        Corner::TopLeft => "hot-corners-cell",
        Corner::TopRight => "hot-corners-cell hot-corners-right",
        Corner::BottomLeft => "hot-corners-cell hot-corners-bottom",
        Corner::BottomRight => "hot-corners-cell hot-corners-right hot-corners-bottom",
    }
}

/// The id of the pop-up for the row of `corner` with `keys` held.
fn pop_up_id(corner: Corner, keys: &HotKeys) -> String {
    row_id(
        "hot-corner",
        &format!("{} {}", corner.path(), keys.words().join(" ")),
    )
}

/// A corner: its rows, then "Add with Keys…". A value the sheet cannot hold is shown as it is.
#[component]
pub(super) fn CornerCell(
    corner: Corner,
    entry: KeyEntry,
    known: HotKnown,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
    state: SheetState,
) -> Element {
    if let Some(value) = written_value(&files(), &entry) {
        return rsx! {
            div { class: cell_class(corner), "aria-label": corner_name(corner), "data-corner": corner.path(),
                FileOnlyControl { value }
            }
        };
    }
    let bindings = bindings_in(&files(), &entry);
    let rows: Vec<(String, HotKeys)> = shown_rows(&bindings, &state.extra_of(corner))
        .into_iter()
        .map(|keys| (pop_up_id(corner, &keys), keys))
        .collect();
    rsx! {
        div { class: cell_class(corner), "aria-label": corner_name(corner), "data-corner": corner.path(),
            for (id, keys) in rows {
                CornerRow {
                    key: "{id}",
                    corner,
                    entry: entry.clone(),
                    keys,
                    bindings: bindings.clone(),
                    known: known.clone(),
                    onedit,
                    state,
                }
            }
            AddWithKeys { corner, bindings: bindings.clone(), state }
        }
    }
}

/// The pop-up for one set of keys, with the keys beside it and a way to remove the row when
/// some are held.
#[component]
fn CornerRow(
    corner: Corner,
    entry: KeyEntry,
    keys: HotKeys,
    bindings: Vec<HotBinding>,
    known: HotKnown,
    onedit: EventHandler<Edit>,
    state: SheetState,
) -> Element {
    let shown = corner_shown(binding_for(&bindings, &keys));
    let title = shown.says.as_ref().map(|does| corner_label(does, &known));
    let pick = {
        let (entry, keys, bindings) = (entry.clone(), keys.clone(), bindings.clone());
        let mut step = state.step;
        let mut told = state.told;
        move |choice: CornerChoice| match corner_picked(choice) {
            HotPicked::Set(does) => {
                told.set(None);
                store(onedit, &entry, &with_binding(&bindings, &keys, does));
            }
            HotPicked::Ask(ask) => step.set(Step::Ask {
                corner,
                keys: keys.clone(),
                ask,
            }),
        }
    };
    let remove = {
        let (entry, keys, bindings) = (entry.clone(), keys.clone(), bindings.clone());
        let mut state = state;
        EventHandler::new(move |()| {
            let (left, rows) = remove_row(&bindings, &state.extra_of(corner), &keys);
            store(onedit, &entry, &left);
            state.set_extra(corner, rows);
        })
    };
    let names = keys_names(&keys);
    rsx! {
        div { class: "hot-corners-row",
            if !keys.is_empty() {
                div { class: "hot-corners-keys", "aria-label": "{names}",
                    Label { text: keys_glyphs(&keys), role: LabelRole::Secondary }
                }
            }
            PopUpButton::<CornerChoice> {
                items: menu_items(),
                value: shown.value,
                title,
                size: ControlSize::Small,
                common: Common { id: Some(pop_up_id(corner, &keys)), ..Common::default() },
                onpick: pick,
            }
            if !keys.is_empty() {
                Button {
                    label: crate::copy::common::REMOVE,
                    size: ControlSize::Small,
                    common: Common { aria_label: Some(crate::copy::hot_corners::remove_the_row_for(&names)), ..Common::default() },
                    onclick: move |_| remove.call(()),
                }
            }
        }
    }
}

/// "Add with Keys…", and, once pressed, the four keys to tick and the buttons that add the row
/// or give up.
#[component]
fn AddWithKeys(corner: Corner, bindings: Vec<HotBinding>, state: SheetState) -> Element {
    let ticking = state
        .adding
        .read()
        .clone()
        .filter(|(own, _)| *own == corner)
        .map(|(_, keys)| keys);
    let mut adding = state.adding;
    let mut told = state.told;
    let Some(keys) = ticking else {
        return rsx! {
            div { class: "hot-corners-row",
                Button {
                    label: crate::copy::hot_corners::ADD_WITH_KEYS_ELLIPSIS,
                    size: ControlSize::Small,
                    common: Common { id: Some(row_id("hot-corner-add", corner.path())), ..Common::default() },
                    onclick: move |_| {
                        told.set(None);
                        adding.set(Some((corner, HotKeys::default())));
                    },
                }
            }
        };
    };
    let add = {
        let keys = keys.clone();
        let mut state = state;
        EventHandler::new(move |()| {
            match add_row(&bindings, &state.extra_of(corner), keys.clone()) {
                Ok(rows) => {
                    state.set_extra(corner, rows);
                    state.told.set(None);
                    state.adding.set(None);
                }
                Err(fault) => state.told.set(Some(add_fault(fault).to_owned())),
            }
        })
    };
    rsx! {
        div { class: "hot-corners-keys",
            for (word, modifier) in HotModifier::ALL.map(|modifier| (modifier.word(), modifier)) {
                Checkbox {
                    key: "{word}",
                    label: format!("{} {}", modifier_glyph(modifier), modifier_name(modifier)),
                    value: if keys.holds(modifier) { Check::On } else { Check::Off },
                    size: ControlSize::Small,
                    onchange: {
                        let keys = keys.clone();
                        move |_| adding.set(Some((corner, keys.clone().toggled(modifier))))
                    },
                }
            }
        }
        div { class: "hot-corners-row",
            Button {
                label: crate::copy::common::CANCEL,
                size: ControlSize::Small,
                onclick: move |_| adding.set(None),
            }
            Button {
                label: crate::copy::common::ADD,
                size: ControlSize::Small,
                common: Common { id: Some(row_id("hot-corner-add-keys", corner.path())), ..Common::default() },
                onclick: move |_| add.call(()),
            }
        }
    }
}
