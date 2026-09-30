//! The root: the environment, the frame with the page's title, and the window inside it.

use crate::env::Env;
use crate::files::load_all;
use crate::view::View;
use crate::window::Window;
use dioxus::prelude::*;
use ds::prelude::*;
use ds_blitz::TokioSpawner;
use ds_settings::{AppName, ConfigRoot, Store, use_environment};
use std::sync::Arc;

/// The Settings app. Reads its [`Env`] from the context the launcher provides.
#[component]
pub fn App() -> Element {
    let env = use_context::<Env>();
    let catalogue = env.catalogue.clone();
    let first = catalogue.pages.first().map(|page| page.slug.clone());
    let opening = env.opening.clone();
    let page = use_signal(|| opening.page.clone().or(first).unwrap_or_default());
    let query = use_signal(|| opening.query.clone());
    let files = use_signal({
        let env = env.clone();
        move || load_all(&env)
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
    let title = view
        .current
        .as_ref()
        .and_then(|slug| catalogue.page(slug))
        .map_or("Settings", |page| page.page.label())
        .to_owned();
    rsx! {
        Ds {
            appearance: now.settings.appearance.appearance(),
            system: now.system,
            material: Material::Window,
            tint_alpha: Some(now.tint_alpha()),
            stack: Some(now.material_stack()),
            window: WindowFrame::titlebar(title, TrafficLights::Shown),
            style { {crate::STYLE} }
            Window { page, query, files, view }
        }
    }
}
