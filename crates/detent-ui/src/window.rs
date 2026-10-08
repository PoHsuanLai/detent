//! The window under the frame: the sidebar with its search field, and the page beside it.

use crate::env::Env;
use crate::files::{Edit, Files, apply};
use crate::page_view::PageView;
use crate::sidebar::sidebar_items;
use crate::view::{Searching, View};
use detent_model::{PageSlug, search};
use dioxus::prelude::*;
use ds::components::chrome::sidebar::Sidebar;
use ds::components::chrome::sidebar_model::SidebarSection;
use ds::components::chrome::split_view::model::{PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::components::overlays::empty_state::EmptyForm;
use ds::prelude::*;
use ds::style::tokens::control_size::{ControlSize, SidebarSize};

/// The sidebar, the search field and the selected page. Sits under the `Ds` root so the toasts
/// that report a failed write have somewhere to mount.
#[component]
pub fn Window(
    page: Signal<PageSlug>,
    query: Signal<String>,
    files: Signal<Files>,
    view: View,
) -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let mut sidebar = use_signal(|| Shown::Visible);

    let write_env = env.clone();
    let onedit = EventHandler::new(move |edit: Edit| {
        let written = apply(&write_env, &files.peek(), &edit);
        match written {
            Ok(next) => files.set(next),
            Err(error) => toasts.push(error.to_string(), None),
        }
    });

    let search_env = env.clone();
    let find = move |next: String| {
        if let Some(shown) = search(&search_env.catalogue, &next).first_page() {
            page.set(shown.clone());
        }
        query.set(next);
    };

    let side = rsx! {
        Sidebar::<String> {
            label: "Settings",
            size: SidebarSize::Medium,
            cursor: view.current.as_ref().map(|slug| slug.0.clone()),
            sections: vec![SidebarSection::List(sidebar_items(&env.catalogue, &view.listed, view.current.as_ref(), page))],
            onselect: move |slug: String| page.set(PageSlug(slug)),
            header: rsx! {
                TextField {
                    label: "Search",
                    value: query(),
                    kind: FieldKind::Search,
                    placeholder: "Search",
                    size: ControlSize::Regular,
                    oninput: find,
                }
            },
        }
    };
    let hits: Vec<_> = view.hits.iter().map(|hit| hit.path.clone()).collect();
    rsx! {
        div { class: "window",
            SplitView {
                label: "Settings",
                panes: vec![SplitPane::new(PaneSpec::SIDEBAR, side).shown(sidebar())],
                on_shown: move |(_, shown)| sidebar.set(shown),
                div { class: "content",
                    match (&view.current, view.searching) {
                        (Some(slug), _) => rsx! {
                            PageView { slug: slug.clone(), hits, files, onedit }
                        },
                        (None, Searching::Yes) => rsx! {
                            EmptyState {
                                form: EmptyForm::NoResults,
                                title: "No Results",
                                description: "Nothing in Settings matches that search.",
                            }
                        },
                        (None, Searching::No) => rsx! {
                            EmptyState {
                                title: "No Settings",
                                description: "No installed program ships a settings schema.",
                            }
                        },
                    }
                }
            }
        }
    }
}
