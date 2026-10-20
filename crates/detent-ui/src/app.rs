//! The root: the environment, the frame with the page's title, and the window inside it.

use crate::accounts::AccountLink;
use crate::assistant_model::AssistantSetup;
use crate::env::Env;
use crate::files::load_all;
use crate::view::View;
use crate::window::Window;
use detent_model::{
    Arrival, Notice, arrival, assistant_model_page, is_assistant_setup, opening_page,
};
use dioxus::prelude::*;
use ds::prelude::*;
use ds_blitz::{TokioSpawner, use_window_handle};
use ds_settings::{AppName, ConfigRoot, Store, use_environment};
use std::sync::Arc;

/// The Settings app. Reads its [`Env`] from the context the launcher provides.
#[component]
pub fn App() -> Element {
    let env = use_context::<Env>();
    let catalogue = env.catalogue.clone();
    let first = opening_page(&catalogue);
    let opening = env.opening.clone();
    let page = use_signal(|| opening.page.clone().or(first).unwrap_or_default());
    let query = use_signal(|| opening.query.clone());
    let link = AccountLink::provide(page, query);
    let handle = use_window_handle();
    let files = use_signal({
        let env = env.clone();
        move || load_all(&env)
    });

    let told = env.clone();
    use_future(move || {
        let (told, catalogue) = (told.clone(), told.catalogue.clone());
        let (mut page, mut query, mut files) = (page, query, files);
        let handle = handle.clone();
        async move {
            let Some(mut notices) = told.inbox.take() else {
                return;
            };
            while let Some(notice) = notices.recv().await {
                match notice {
                    Notice::OpenPage(slug)
                        if catalogue.page(&slug).is_some() || is_assistant_setup(&slug) =>
                    {
                        query.set(String::new());
                        page.set(slug);
                    }
                    Notice::OpenPage(_) => {}
                    Notice::OpenTarget(target) => match arrival(&catalogue, &target) {
                        Some(Arrival::Page { slug, query: asked }) => {
                            query.set(asked);
                            page.set(slug);
                        }
                        Some(Arrival::Account(account)) => link.open_target(account),
                        None => {}
                    },
                    Notice::Reload => files.set(load_all(&told)),
                    Notice::OpenAccount(target) => link.open_target(target),
                    Notice::Raise(token) => {
                        if let Some(window) = &handle {
                            window.focus_with_token(token);
                        }
                    }
                }
            }
        }
    });

    let appearance = use_environment(
        Store::new(
            ConfigRoot::Scratch(env.config_home.0.clone()),
            AppName::QUIRE,
        ),
        env.system.clone(),
        Arc::new(TokioSpawner::current()),
    );
    let now = appearance();
    let view = View::of(&catalogue, &query(), &page());
    let setup = is_assistant_setup(&page());
    let title = if setup {
        assistant_model_page().label().to_owned()
    } else {
        view.current
            .as_ref()
            .and_then(|slug| catalogue.page(slug))
            .map_or("Settings", |page| page.title.as_str())
            .to_owned()
    };
    rsx! {
        Ds {
            appearance: now.settings.appearance.appearance(),
            system: now.system,
            material: Material::Window,
            tint_alpha: Some(now.tint_alpha()),
            stack: Some(now.material_stack()),
            window: WindowFrame::titlebar(title, TrafficLights::Shown),
            style { {crate::STYLE} }
            if setup {
                AssistantSetup {}
            } else {
                Window { page, query, files, view }
            }
        }
    }
}
