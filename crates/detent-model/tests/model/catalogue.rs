//! Grouping, ordering and search over the real schemas (`fixtures/schemas`).

use detent_model::{
    Catalogue, Found, PageSlug, Rank, Search, SectionModel, discover, landing, search,
    visible_pages,
};
use detent_testkit::{Scratch, seed_schemas};
use ds_settings::schema::Exposure;

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
fn pages_come_in_the_page_tables_order_and_only_when_they_have_keys() {
    let slugs: Vec<String> = real("pages_come_in_the_page_tables_order")
        .pages
        .iter()
        .map(|p| p.slug.0.clone())
        .collect();
    assert_eq!(
        slugs,
        [
            "wifi",
            "bluetooth",
            "notifications",
            "appearance",
            "intelligence",
            "assistant-model",
            "privacy",
            "dock",
            "displays",
            "wallpaper",
            "spaces",
            "power",
            "software-update",
            "accounts",
            "keyboard_and_shortcuts",
            "mouse_and_gestures"
        ],
        "no schema names the Apps page, so it is not listed; sill's wallpaper.picture lists Wallpaper; inferd's and docket's name Intelligence; Accounts, Privacy, Assistant Model, Software Update, Wi-Fi, Bluetooth and Displays are live panes and always there"
    );
}

#[test]
fn a_page_lists_its_basic_sections_and_one_advanced_group_after_them() {
    let catalogue = real("a_page_lists_its_basic_sections_and_one_advanced_group_after_them");
    let appearance = catalogue.page(&slug("appearance")).expect("appearance");
    let titles = |sections: &[SectionModel]| -> Vec<String> {
        sections.iter().map(|s| s.title.clone()).collect()
    };
    assert_eq!(
        titles(&appearance.sections),
        ["Appearance", "Icons", "Time"],
        "only sections with a basic key are headers of their own"
    );
    let advanced = titles(&appearance.advanced);
    assert!(advanced.contains(&"Bar".to_owned()), "{advanced:?}");
    assert!(
        appearance
            .sections
            .iter()
            .flat_map(|s| &s.keys)
            .all(|e| e.spec.exposure == Exposure::Basic)
    );
    assert!(
        appearance
            .advanced
            .iter()
            .flat_map(|s| &s.keys)
            .all(|e| e.spec.exposure == Exposure::Advanced)
    );

    let dock = catalogue.page(&slug("dock")).expect("dock");
    let paths = |sections: &[SectionModel], title: &str| -> Vec<String> {
        sections
            .iter()
            .find(|s| s.title == title)
            .map(|s| s.keys.iter().map(|e| e.spec.path.0.clone()).collect())
            .unwrap_or_default()
    };
    assert_eq!(
        paths(&dock.sections, "Magnification"),
        ["dock.magnification"]
    );
    let advanced = paths(&dock.advanced, "Magnification");
    assert!(
        advanced.contains(&"dock.magnified_size_px".to_owned()),
        "{advanced:?}"
    );
}

#[test]
fn mouse_and_gestures_holds_palmrests_keys_with_its_advanced_ones_apart() {
    let catalogue = real("mouse_and_gestures_holds_palmrests_keys");
    let page = catalogue
        .page(&slug("mouse_and_gestures"))
        .expect("the page");
    let basic: Vec<&str> = page
        .sections
        .iter()
        .flat_map(|s| &s.keys)
        .map(|e| e.spec.path.0.as_str())
        .collect();
    assert!(basic.contains(&"rejection"), "{basic:?}");
    assert!(basic.contains(&"foreign_output"), "{basic:?}");
    assert!(
        basic.contains(&"scroll.natural"),
        "sill's key sits beside palmrest's"
    );
    assert!(
        !basic.contains(&"palm_rejection.click_guard_ms"),
        "{basic:?}"
    );
    let advanced: Vec<&str> = page
        .advanced
        .iter()
        .flat_map(|s| &s.keys)
        .map(|e| e.spec.path.0.as_str())
        .collect();
    assert!(
        advanced.contains(&"palm_rejection.click_guard_ms"),
        "{advanced:?}"
    );
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
    assert_eq!(
        files,
        [
            "almanac/settings.toml",
            "cua/settings.toml",
            "docket/settings.toml",
            "inferd/inferd.toml",
            "palmrest/gestures.toml",
            "quire/appearance.toml",
            "sill/settings.toml"
        ]
    );
}

#[test]
fn a_query_resolves_to_the_key_the_schemas_word_says() {
    const CASES: &[(&str, &str, &str)] = &[
        ("dark mode", "appearance.theme", "appearance"),
        ("DARK MODE", "appearance.theme", "appearance"),
        ("magnification", "dock.magnification", "dock"),
        ("natural scrolling", "scroll.natural", "mouse_and_gestures"),
        ("do not disturb", "notifications.dnd", "notifications"),
        ("palm rejection", "rejection", "mouse_and_gestures"),
        ("reduce motion", "appearance.motion_level", "appearance"),
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
    assert_eq!(
        search(&catalogue, "zzzzqqqq"),
        Search::Results(Found::default())
    );
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
        slug("wifi"),
        "an unknown page falls back to the first listed"
    );
}

#[test]
fn a_page_is_found_by_the_start_of_its_name_or_of_a_word_of_it() {
    let catalogue = real("a_page_is_found_by_the_start_of_its_name_or_of_a_word_of_it");
    let listed = |query: &str| -> Vec<String> {
        visible_pages(&catalogue, &search(&catalogue, query))
            .iter()
            .map(|page| page.slug.0.clone())
            .collect()
    };
    // Accounts is accountd's live pane: no key of it says "acc", the page's name does.
    for query in ["acc", "Account", "accounts", "  ACC "] {
        assert!(
            listed(query).contains(&"accounts".to_owned()),
            "{query:?} drops the Accounts page"
        );
    }
    assert_eq!(listed("mouse gest"), ["mouse_and_gestures"]);
    assert!(listed("gest").contains(&"mouse_and_gestures".to_owned()));
    // Only the start of a word counts, and every word of the query must.
    assert!(!listed("ccount").contains(&"accounts".to_owned()));
    assert!(!listed("acc zzzz").contains(&"accounts".to_owned()));
    // Typing the page's name shows that page.
    let found = search(&catalogue, "acc");
    let visible = visible_pages(&catalogue, &found);
    let shown = landing(&visible, &found, &slug("nowhere")).expect("a page");
    assert_eq!(shown.slug, slug("accounts"));
    assert_eq!(found.first_page(), Some(&slug("accounts")));
}
