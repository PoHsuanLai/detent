//! quire's coherence rules (CONSUMING.md section 5) on this app: layout-only CSS, and markup that
//! is only quire's components. The markup is a server render of every page of the real schemas.

use crate::support::Sandbox;
use detent_model::PageSlug;
use detent_ui::{App, Env, Opening, STYLE};
use dioxus::prelude::VirtualDom;
use ds_lint::{LintConfig, Profile, assert_clean, markup};

/// A server render of the app opened on `page`, inside the runtime the app's watches need.
fn render(env: Env) -> String {
    let _runtime = ds_blitz::enter_runtime();
    let mut dom = VirtualDom::new(App);
    dom.provide_root_context(env);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn css() -> String {
    format!("{}\n{STYLE}", ds::stylesheet())
}

fn pages(sandbox: &Sandbox) -> Vec<PageSlug> {
    sandbox
        .env(Opening::default(), ds::prelude::Scheme::Light)
        .catalogue
        .pages
        .iter()
        .map(|page| page.slug.clone())
        .collect()
}

fn render_page(sandbox: &Sandbox, page: &PageSlug) -> String {
    let opening = Opening {
        page: Some(page.clone()),
        query: String::new(),
    };
    let html = render(sandbox.env(opening, ds::prelude::Scheme::Light));
    assert!(
        html.contains(&format!("data-page=\"{}\"", page.0)),
        "the page was not rendered, so there is nothing to lint: {html}"
    );
    html
}

/// Rule 1: the stylesheet is layout only, under the strictest profile.
#[test]
fn our_stylesheet_has_no_colours_or_raw_values() {
    assert_clean(
        STYLE,
        &LintConfig {
            profile: Profile::Strict,
            ..LintConfig::new(&ds::kits())
        },
    );
}

/// The whole lint, every rule the markup lint has, on every page.
#[test]
fn the_markup_lint_is_clean_on_every_page() {
    let sandbox = Sandbox::new("coherence-all");
    for page in pages(&sandbox) {
        let html = render_page(&sandbox, &page);
        let offences = markup(&html, &css(), &LintConfig::new(&ds::kits()));
        assert!(offences.is_empty(), "{}: {offences:#?}", page.0);
    }
}
