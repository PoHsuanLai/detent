//! The forms that follow "Open…", "App Action…" and "Command…": each replaces the screen in the
//! sheet, and Choose stores the binding for the row that asked.

use super::cell::{bindings_in, store};
use super::sheet::{SheetState, Step};
use crate::copy::hot_corners::open_kind_label;
use crate::env::Env;
use crate::files::{Edit, Files};
use crate::key_row::row_id;
use detent_model::{
    Corner, CornerAsk, HotDo, HotKeys, HotKnown, HotOpenForm, HotOpenKind, KeyEntry, action_groups,
    apps_matching, binding_for, command_binding, with_binding,
};
use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::controls::segmented::Tracking;
use ds::components::lists::list::model::ListStyle;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The form for the row of `corner` with `keys` held.
#[component]
pub(super) fn AskView(
    corner: Corner,
    entry: KeyEntry,
    keys: HotKeys,
    ask: CornerAsk,
    known: HotKnown,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
    state: SheetState,
) -> Element {
    let bindings = bindings_in(&files(), &entry);
    let current = binding_for(&bindings, &keys).cloned();
    let mut step = state.step;
    let back = EventHandler::new(move |()| step.set(Step::Corners));
    let save = {
        let (entry, keys, bindings) = (entry.clone(), keys.clone(), bindings.clone());
        EventHandler::new(move |does: HotDo| {
            store(onedit, &entry, &with_binding(&bindings, &keys, Some(does)));
            step.set(Step::Corners);
        })
    };
    match ask {
        CornerAsk::Open => rsx! { OpenAsk { current, known, save, back } },
        CornerAsk::AppAction => rsx! { ActionAsk { current, known, save, back } },
        CornerAsk::Command => rsx! { CommandAsk { current, save, back } },
    }
}

/// Cancel and Choose, the sheet's two buttons; Choose waits until `ready`.
#[component]
fn ChooseButtons(ready: Availability, back: EventHandler<()>, choose: EventHandler<()>) -> Element {
    rsx! {
        div { class: "hot-corners-buttons",
            Button {
                label: "Cancel",
                common: Common { aria_label: Some("Cancel".to_owned()), ..Common::default() },
                answers: Answers::Escape,
                onclick: move |_| back.call(()),
            }
            Button {
                label: "Choose",
                common: Common { id: Some("hot-corner-choose".to_owned()), aria_label: Some("Choose".to_owned()), ..Common::default() },
                answers: Answers::Return,
                availability: ready,
                onclick: move |_| choose.call(()),
            }
        }
    }
}

/// The form's heading and the line under it.
#[component]
fn AskTitle(title: String, words: String) -> Element {
    rsx! {
        Label { text: title, role: LabelRole::Primary, style: LabelStyle::Headline }
        Label { text: words, role: LabelRole::Secondary }
    }
}

/// Choose is available once `made` holds what the form makes.
fn ready<T>(made: &Option<T>) -> Availability {
    match made {
        Some(_) => Availability::Enabled,
        None => Availability::Disabled,
    }
}

/// The command to run.
#[component]
fn CommandAsk(
    current: Option<HotDo>,
    save: EventHandler<HotDo>,
    back: EventHandler<()>,
) -> Element {
    let initial = match current {
        Some(HotDo::Command(text)) => text,
        _ => String::new(),
    };
    let mut typed = use_signal(move || initial.clone());
    let does = command_binding(&typed.read());
    let choose = {
        let does = does.clone();
        EventHandler::new(move |()| {
            if let Some(does) = does.clone() {
                save.call(does);
            }
        })
    };
    rsx! {
        AskTitle { title: "Command", words: "Run a command when the pointer rests in the corner." }
        TextField {
            label: "Command",
            value: typed(),
            placeholder: "Command to run",
            kind: FieldKind::Plain,
            size: ControlSize::Small,
            common: Common { id: Some("hot-corner-command".to_owned()), ..Common::default() },
            oninput: move |text: String| typed.set(text),
            onsubmit: move |_: String| choose.call(()),
        }
        ChooseButtons { ready: ready(&does), back, choose }
    }
}

/// The text of the search field above a list, and the field.
#[component]
fn SearchBox(label: String, query: Signal<String>) -> Element {
    let mut query = query;
    rsx! {
        TextField {
            label,
            value: query(),
            kind: FieldKind::Search,
            placeholder: "Search",
            size: ControlSize::Small,
            oninput: move |text: String| query.set(text),
        }
    }
}

/// The state of the row of `own` in a list whose chosen row is `chosen`.
fn row_state<T: PartialEq>(chosen: &Option<T>, own: &T) -> RowState {
    RowState {
        selection: match chosen {
            Some(chosen) if chosen == own => Selection::Selected,
            _ => Selection::Unselected,
        },
        ..RowState::default()
    }
}

/// The action to run, from the installed apps' actions grouped by app.
#[component]
fn ActionAsk(
    current: Option<HotDo>,
    known: HotKnown,
    save: EventHandler<HotDo>,
    back: EventHandler<()>,
) -> Element {
    let initial = match current {
        Some(HotDo::AppAction { app, action }) => Some((app, action)),
        _ => None,
    };
    let query = use_signal(String::new);
    let mut chosen = use_signal(move || initial.clone());
    let groups = action_groups(&known, &query.read());
    let picked = chosen();
    let mut items: Vec<ListItem<String>> = Vec::new();
    for group in &groups {
        items.push(ListItem::heading(
            format!("app:{}", group.app),
            rsx! { SectionHeader { title: group.title.clone() } },
        ));
        for action in &group.actions {
            let key = format!("{}|{}", action.app, action.action);
            let own = (action.app.clone(), action.action.clone());
            let state = row_state(&picked, &own);
            items.push(ListItem::row(
                key.clone(),
                action.label.clone(),
                rsx! {
                    Row {
                        title: action.label.clone(),
                        state,
                        common: Common { id: Some(row_id("hot-action", &key)), ..Common::default() },
                        onclick: move |_| chosen.set(Some(own.clone())),
                    }
                },
            ));
        }
    }
    let cursor = picked
        .as_ref()
        .map(|(app, action)| format!("{app}|{action}"));
    let words = if known.actions.is_empty() {
        "No installed app offers an action that needs nothing more."
    } else if groups.is_empty() {
        "Nothing matches."
    } else {
        ""
    };
    let choose = EventHandler::new(move |()| {
        if let Some((app, action)) = chosen() {
            save.call(HotDo::AppAction { app, action });
        }
    });
    rsx! {
        AskTitle { title: "App Action", words: "Run an action of an app when the pointer rests in the corner." }
        SearchBox { label: "Search actions", query }
        div { class: "hot-corners-list",
            List::<String> {
                label: "Actions",
                items,
                style: ListStyle::Grouped,
                cursor,
            }
        }
        if !words.is_empty() {
            Label { text: words, role: LabelRole::Secondary, style: LabelStyle::Footnote }
        }
        ChooseButtons { ready: ready(&picked), back, choose }
    }
}

/// What to open: an app from the installed ones, a file or a folder from the chooser, or a web
/// address.
#[component]
fn OpenAsk(
    current: Option<HotDo>,
    known: HotKnown,
    save: EventHandler<HotDo>,
    back: EventHandler<()>,
) -> Element {
    let env = use_context::<Env>();
    let initial = match current {
        Some(HotDo::Open(item)) => HotOpenForm::of_item(&item),
        _ => HotOpenForm::default(),
    };
    let mut form = use_signal(move || initial.clone());
    let query = use_signal(String::new);
    let shown = form();
    let body = match shown.kind {
        HotOpenKind::App => {
            let chosen_app = Some(shown.app.clone()).filter(|id| !id.is_empty());
            let rows = apps_matching(&known.apps, &query.read());
            let items: Vec<ListItem<String>> = rows
                .iter()
                .map(|app| {
                    let id = app.id.clone();
                    let state = row_state(&chosen_app, &app.id);
                    ListItem::row(
                        app.id.clone(),
                        app.name.clone(),
                        rsx! {
                            Row {
                                title: app.name.clone(),
                                state,
                                common: Common { id: Some(row_id("hot-app", &app.id)), ..Common::default() },
                                onclick: move |_| form.write().app = id.clone(),
                            }
                        },
                    )
                })
                .collect();
            let cursor = (!shown.app.is_empty()).then(|| shown.app.clone());
            rsx! {
                SearchBox { label: "Search apps", query }
                div { class: "hot-corners-list",
                    List::<String> { label: "Apps", items, style: ListStyle::Grouped, cursor }
                }
            }
        }
        kind @ (HotOpenKind::File | HotOpenKind::Folder) => {
            let corners = env.client.corners();
            let words = if shown.path.is_empty() {
                "Nothing chosen yet.".to_owned()
            } else {
                shown.path.clone()
            };
            rsx! {
                div { class: "hot-corners-row",
                    Label { text: words, role: LabelRole::Secondary }
                    Button {
                        label: format!("Choose {}\u{2026}", open_kind_label(kind)),
                        common: Common { id: Some("hot-corner-chooser".to_owned()), ..Common::default() },
                        onclick: move |_| {
                            let corners = corners.clone();
                            spawn(async move {
                                if let Some(path) = corners.choose(kind).await {
                                    form.write().path = path;
                                }
                            });
                        },
                    }
                }
            }
        }
        HotOpenKind::Web => rsx! {
            TextField {
                label: "Web Address",
                value: shown.web.clone(),
                placeholder: "example.org",
                kind: FieldKind::Plain,
                size: ControlSize::Small,
                common: Common { id: Some("hot-corner-web".to_owned()), ..Common::default() },
                oninput: move |text: String| form.write().web = text,
            }
        },
    };
    let ready = ready(&shown.item());
    let choose = EventHandler::new(move |()| {
        if let Some(does) = form.peek().binding() {
            save.call(does);
        }
    });
    rsx! {
        AskTitle { title: "Open", words: "Open an app, a file, a folder or a web address when the pointer rests in the corner." }
        SegmentedControl::<HotOpenKind> {
            label: "What to open",
            choices: Choice::pairs(HotOpenKind::ALL.map(|kind| (kind, open_kind_label(kind)))),
            tracking: Tracking::SelectOne(shown.kind),
            size: ControlSize::Small,
            onchange: move |kind: HotOpenKind| {
                let mut form = form.write();
                form.kind = kind;
                form.path.clear();
            },
        }
        {body}
        ChooseButtons { ready, back, choose }
    }
}
