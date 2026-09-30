//! Grouping, ordering and search over the real schemas (`fixtures/schemas`).

use detent_model::{Catalogue, PageSlug, Rank, Search, discover, landing, search, visible_pages};
use detent_testkit::{Scratch, seed_schemas};

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

/// The catalogue of the fixtures, seeded in a scratch directory of the test's own name.
fn real(test: &str) -> Catalogue {
    let scratch = Scratch::under(TMP, test);
    let found = discover(&[seed_schemas(scratch.path())]);
    Catalogue::build(&found.programs)
}

fn slug(name: &str) -> PageSlug {
    PageSlug(name.to_owned())
}

#[test]
fn pages_come_in_the_fixed_order_and_only_when_they_have_keys() {
    let slugs: Vec<String> = real("pages_come_in_the_fixed_order")
        .pages
        .iter()
        .map(|p| p.slug.0.clone())
        .collect();
    assert_eq!(
        slugs,
        [
            "appearance",
            "dock",
            "mouse_and_gestures",
            "keyboard_and_shortcuts",
            "notifications",
            "spaces",
            "accounts",
            "power"
        ],
        "no schema names the Apps page, so it is not listed"
    );
}

#[test]
fn sections_with_a_basic_key_come_first_and_keys_split_by_exposure() {
    let catalogue = real("sections_with_a_basic_key_come_first_and_keys_split_by_exposure");
    let appearance = catalogue.page(&slug("appearance")).expect("appearance");
    let titles: Vec<&str> = appearance
        .sections
        .iter()
        .map(|s| s.title.as_str())
        .collect();
    assert_eq!(
        &titles[..2],
        ["Appearance", "Icons"],
        "quire's basic sections lead"
    );
    let first_advanced_only = appearance
        .sections
        .iter()
        .position(|s| s.basic.is_empty())
        .expect("sill's bar sections are advanced only");
    assert_eq!(first_advanced_only, 2);
    assert!(
        appearance.sections[first_advanced_only..]
            .iter()
            .all(|s| s.basic.is_empty() && !s.advanced.is_empty()),
        "advanced-only sections all follow the others"
    );

    let dock = catalogue.page(&slug("dock")).expect("dock");
    let magnification = dock
        .sections
        .iter()
        .find(|s| s.title == "Magnification")
        .expect("Magnification section");
    let basic: Vec<&str> = magnification
        .basic
        .iter()
        .map(|e| e.spec.path.0.as_str())
        .collect();
    let advanced: Vec<&str> = magnification
        .advanced
        .iter()
        .map(|e| e.spec.path.0.as_str())
        .collect();
    assert_eq!(basic, ["dock.magnification"]);
    assert!(advanced.contains(&"dock.magnified_size_px"), "{advanced:?}");
}

#[test]
fn every_key_of_every_schema_is_on_exactly_one_page() {
    let scratch = Scratch::under(TMP, "catalogue-count");
    let found = discover(&[seed_schemas(scratch.path())]);
    let total: usize = found.programs.iter().map(|p| p.keys.len()).sum();
    let catalogue = Catalogue::build(&found.programs);
    assert_eq!(catalogue.entries().count(), total);
    let files: Vec<&str> = catalogue
        .files
        .iter()
        .filter_map(|f| f.file.relative().to_str())
        .collect();
    assert_eq!(files, ["quire/appearance.toml", "sill/settings.toml"]);
}

#[test]
fn a_query_resolves_to_the_key_the_schemas_word_says() {
    const CASES: &[(&str, &str, &str)] = &[
        ("dark mode", "appearance.theme", "appearance"),
        ("DARK MODE", "appearance.theme", "appearance"),
        ("magnification", "dock.magnification", "dock"),
        ("natural scrolling", "scroll.natural", "mouse_and_gestures"),
        ("do not disturb", "notifications.dnd", "notifications"),
    ];
    let catalogue = real("a_query_resolves_to_the_key_the_schemas_word_says");
    for (query, path, page) in CASES {
        let found = search(&catalogue, query);
        let first = found
            .first()
            .unwrap_or_else(|| panic!("{query:?} found nothing"));
        assert_eq!(
            (first.path.0.as_str(), first.page.0.as_str()),
            (*path, *page),
            "{query:?}"
        );
    }
    assert_eq!(
        search(&catalogue, "dark mode").hits().len(),
        1,
        "one key says dark mode"
    );
}

#[test]
fn an_empty_query_is_idle_and_a_stranger_finds_nothing() {
    let catalogue = real("an_empty_query_is_idle_and_a_stranger_finds_nothing");
    assert_eq!(search(&catalogue, "   "), Search::Idle);
    assert_eq!(search(&catalogue, "zzzzqqqq"), Search::Results(Vec::new()));
}

#[test]
fn a_label_match_outranks_a_help_match() {
    let catalogue = real("a_label_match_outranks_a_help_match");
    let found = search(&catalogue, "size");
    let ranks: Vec<_> = found.hits().iter().map(|h| h.rank).collect();
    let mut sorted = ranks.clone();
    sorted.sort();
    assert_eq!(ranks, sorted, "hits are best first");
    assert_eq!(
        ranks.first(),
        Some(&Rank::Label),
        "a key labelled size leads"
    );
    assert!(
        ranks.contains(&Rank::Help),
        "keys that only mention size in their help follow"
    );
}

#[test]
fn a_search_narrows_the_sidebar_and_picks_the_page_to_show() {
    let catalogue = real("a_search_narrows_the_sidebar_and_picks_the_page_to_show");
    let found = search(&catalogue, "magnification");
    let visible = visible_pages(&catalogue, &found);
    assert_eq!(
        visible
            .iter()
            .map(|p| p.slug.0.as_str())
            .collect::<Vec<_>>(),
        ["dock"]
    );
    let shown = landing(&visible, &found, &slug("appearance")).expect("a page");
    assert_eq!(
        shown.slug,
        slug("dock"),
        "the chosen page has no hit, so the best hit's page shows"
    );

    let idle = search(&catalogue, "");
    let all = visible_pages(&catalogue, &idle);
    assert_eq!(all.len(), catalogue.pages.len());
    let kept = landing(&all, &idle, &slug("power")).expect("a page");
    assert_eq!(
        kept.slug,
        slug("power"),
        "with no search the chosen page stays"
    );
    let unknown = landing(&all, &idle, &slug("nowhere")).expect("a page");
    assert_eq!(
        unknown.slug,
        slug("appearance"),
        "an unknown page falls back to the first"
    );
}
