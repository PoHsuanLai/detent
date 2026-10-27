//! The sheet: the screen with a cell at each corner, or the form that follows a pick, and Done.

use super::ask::AskView;
use super::cell::CornerCell;
use crate::env::Env;
use crate::files::{Edit, Files};
use crate::privacy::use_program_book;
use detent_model::{Corner, CornerAsk, HotActionChoice, HotApp, HotKeys, HotKnown, KeyEntry};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;

/// What the sheet shows.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Step {
    /// The screen and its four corners.
    Corners,
    /// The form that follows a pick of "Open…", "App Action…" or "Command…", for the row of
    /// `corner` with `keys` held.
    Ask {
        corner: Corner,
        keys: HotKeys,
        ask: CornerAsk,
    },
}

/// What the sheet remembers while it is open.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct SheetState {
    /// What is shown.
    pub step: Signal<Step>,
    /// The rows the person added that hold no binding yet, by corner.
    pub added: Signal<Vec<(Corner, HotKeys)>>,
    /// The corner whose "Add with Keys…" is open, and the keys ticked so far.
    pub adding: Signal<Option<(Corner, HotKeys)>>,
    /// Why the last add came to nothing.
    pub told: Signal<Option<String>>,
}

impl SheetState {
    /// The rows added to `corner`.
    pub(super) fn extra_of(&self, corner: Corner) -> Vec<HotKeys> {
        self.added
            .read()
            .iter()
            .filter(|(own, _)| *own == corner)
            .map(|(_, keys)| keys.clone())
            .collect()
    }

    /// Makes `rows` the rows added to `corner`, and leaves the other corners' as they are.
    pub(super) fn set_extra(&mut self, corner: Corner, rows: Vec<HotKeys>) {
        let others = self
            .added
            .read()
            .iter()
            .filter(|(own, _)| *own != corner)
            .cloned()
            .collect::<Vec<_>>();
        self.added.set(
            others
                .into_iter()
                .chain(rows.into_iter().map(|keys| (corner, keys)))
                .collect(),
        );
    }
}

fn use_sheet_state() -> SheetState {
    SheetState {
        step: use_signal(|| Step::Corners),
        added: use_signal(Vec::new),
        adding: use_signal(|| None),
        told: use_signal(|| None),
    }
}

/// The corner keys as `(corner, key)`, in the order the corners sit on the screen.
fn corners_of(entries: &[KeyEntry]) -> Vec<(Corner, Option<KeyEntry>)> {
    Corner::ALL
        .into_iter()
        .map(|corner| {
            let entry = entries
                .iter()
                .find(|entry| entry.spec.path.0 == corner.path())
                .cloned();
            (corner, entry)
        })
        .collect()
}

/// The sheet.
#[component]
pub(super) fn HotCornersSheet(
    entries: Vec<KeyEntry>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
    onclose: EventHandler<()>,
) -> Element {
    let env = use_context::<Env>();
    let state = use_sheet_state();
    let book = use_program_book();
    let mut actions = use_signal(Vec::<HotActionChoice>::new);
    use_future({
        let corners = env.client.corners();
        move || {
            let corners = corners.clone();
            async move { actions.set(corners.actions().await) }
        }
    });
    let known = HotKnown {
        apps: book
            .read()
            .apps
            .iter()
            .map(|app| HotApp {
                id: app.id.clone(),
                name: app.name.clone(),
            })
            .collect(),
        actions: actions(),
    };
    let step = state.step.read().clone();
    let told = state.told.read().clone();
    let placed = corners_of(&entries);
    let body = match step {
        Step::Corners => rsx! {
            div { class: "hot-corners-screen",
                for (corner, entry) in placed.clone() {
                    if let Some(entry) = entry {
                        CornerCell { key: "{entry.spec.path.0}", corner, entry, known: known.clone(), files, onedit, state }
                    } else {
                        div { key: "{corner.path()}" }
                    }
                }
            }
            if let Some(words) = told {
                Label { text: words, role: LabelRole::Secondary, style: LabelStyle::Footnote }
            }
            Label { text: crate::copy::hot_corners::MOVE_THE_POINTER_INTO_A_CORNER, role: LabelRole::Secondary, style: LabelStyle::Footnote }
            div { class: "hot-corners-buttons",
                Button {
                    label: crate::copy::common::DONE,
                    common: Common { aria_label: Some(crate::copy::common::DONE.to_owned()), ..Common::default() },
                    answers: Answers::Return,
                    onclick: move |_| onclose.call(()),
                }
            }
        },
        Step::Ask { corner, keys, ask } => {
            let entry = placed
                .iter()
                .find(|(own, _)| *own == corner)
                .and_then(|(_, entry)| entry.clone());
            match entry {
                Some(entry) => rsx! {
                    AskView { corner, entry, keys, ask, known: known.clone(), files, onedit, state }
                },
                None => rsx! {},
            }
        }
    };
    let on_return = matches!(*state.step.read(), Step::Corners).then_some(onclose);
    rsx! {
        Sheet {
            label: crate::copy::hot_corners::HOT_CORNERS,
            width: SheetWidth::Regular,
            onclose,
            on_return,
            div { class: "hot-corners-sheet", {body} }
        }
    }
}
