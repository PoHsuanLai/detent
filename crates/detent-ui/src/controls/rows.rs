//! A list of tables: a line per row with a cell per column, a Remove button on each, and a line
//! to add one. An edit that does not fit its column is not stored.

use detent_model::external::toml;
use detent_model::{Columns, humanize};
use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, ButtonRole};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::ColumnKind;

/// The cells of every row, and the rows each change would make.
#[derive(Clone, PartialEq)]
struct Table {
    columns: Columns,
    cells: Vec<Vec<String>>,
}

impl Table {
    /// The rows with the cell at `(row, column)` set to `text`.
    fn with_cell(&self, row: usize, column: usize, text: &str) -> Vec<Vec<String>> {
        let mut next = self.cells.clone();
        if let Some(cell) = next.get_mut(row).and_then(|cells| cells.get_mut(column)) {
            *cell = text.to_owned();
        }
        next
    }

    /// The rows without `row`.
    fn without(&self, row: usize) -> Vec<Vec<String>> {
        let mut next = self.cells.clone();
        if row < next.len() {
            next.remove(row);
        }
        next
    }

    /// The rows with `row` after them.
    fn with_row(&self, row: Vec<String>) -> Vec<Vec<String>> {
        let mut next = self.cells.clone();
        next.push(row);
        next
    }

    /// Whether `row` has a fitting cell in every column.
    fn fits(&self, row: &[String]) -> bool {
        row.len() == self.columns.all().len()
            && row
                .iter()
                .enumerate()
                .all(|(at, text)| self.columns.fits(at, text))
    }
}

/// One cell: a text field or a pop-up of the column's words, committing through `oncommit`.
#[component]
fn Cell(columns: Columns, at: usize, text: String, oncommit: EventHandler<String>) -> Element {
    let Some(column) = columns.all().get(at).cloned() else {
        return rsx! {};
    };
    let label = column.label.0.clone();
    match column.kind {
        ColumnKind::Choice { variants } => {
            let items: Vec<MenuItem<String>> = variants
                .iter()
                .map(|word| MenuItem::new(word.clone(), humanize(word)))
                .collect();
            rsx! {
                PopUpButton::<String> {
                    items,
                    value: Some(text),
                    size: ControlSize::Small,
                    onpick: move |word| oncommit.call(word),
                }
            }
        }
        ColumnKind::Text { .. } => {
            let mut draft = use_signal(|| text.clone());
            let stored = text.clone();
            rsx! {
                TextField {
                    label,
                    value: draft(),
                    size: ControlSize::Small,
                    oninput: move |next| draft.set(next),
                    onchange: move |next: String| {
                        if columns.fits(at, &next) {
                            oncommit.call(next);
                        } else {
                            draft.set(stored.clone());
                        }
                    },
                }
            }
        }
    }
}

/// The rows editor of a key whose columns are `columns`, holding `value`.
#[component]
pub fn RowsControl(
    columns: Columns,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let table = Table {
        cells: columns.cells(&value).unwrap_or_default(),
        columns: columns.clone(),
    };
    let mut adding = use_signal(|| columns.blank());
    let send = {
        let columns = columns.clone();
        EventHandler::new(move |cells: Vec<Vec<String>>| {
            if let Some(next) = columns.value_of(&cells) {
                onchange.call(next);
            }
        })
    };
    let draft = adding();
    let addable = table.fits(&draft);
    rsx! {
        div { class: "key-rows",
            for (row, cells) in table.cells.iter().cloned().enumerate() {
                div { key: "{row}:{cells.join(\"|\")}", class: "key-rows-row", "data-row": "{row}",
                    for (at, column) in columns.all().iter().enumerate() {
                        Cell {
                            key: "{column.name.0}",
                            text: cells[at].clone(),
                            columns: columns.clone(),
                            at,
                            oncommit: {
                                let table = table.clone();
                                move |text: String| send.call(table.with_cell(row, at, &text))
                            },
                        }
                    }
                    Button {
                        label: crate::copy::common::REMOVE,
                        bezel: Bezel::Inline,
                        role: ButtonRole::Destructive,
                        size: ControlSize::Small,
                        onclick: {
                            let table = table.clone();
                            move |_| send.call(table.without(row))
                        },
                    }
                }
            }
            div { class: "key-rows-new",
                for (at, column) in columns.all().iter().enumerate() {
                    Cell {
                        key: "{column.name.0}:{draft[at]}",
                        text: draft[at].clone(),
                        columns: columns.clone(),
                        at,
                        oncommit: {
                            let draft = draft.clone();
                            move |text: String| {
                                let mut next = draft.clone();
                                next[at] = text;
                                adding.set(next);
                            }
                        },
                    }
                }
                Button {
                    label: crate::copy::common::ADD,
                    size: ControlSize::Small,
                    availability: if addable { Availability::Enabled } else { Availability::Disabled },
                    onclick: {
                        let table = table.clone();
                        let blank = columns.blank();
                        move |_| {
                            send.call(table.with_row(adding.peek().clone()));
                            adding.set(blank.clone());
                        }
                    },
                }
            }
        }
    }
}
