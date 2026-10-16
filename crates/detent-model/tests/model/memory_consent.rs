//! The loose memory settings over almanac's real schema: which keys can wait, the pairs a save
//! confirms, the words of the note, and the state's bookkeeping.

use almanac_core::WaitingSetting;
use detent_model::{
    Catalogue, Confinement, ConsentState, KeyEntry, SettingPair, Trouble, discover,
    holds_memory_settings, is_memory_setting, tells_confinement, to_confirm, waiting_note,
};
use detent_testkit::{Scratch, seed_schemas};
use ds_settings::schema::KeyPath;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");
const AT_REST: &str = "memory.files.at_rest";
const SEARCH_DAYS: &str = "memory.retention.search_days";
const APPLY: &str = "memory.consolidation.apply";

fn real(test: &str) -> Catalogue {
    let scratch = Scratch::under(TMP, test);
    Catalogue::build(&discover(&[seed_schemas(scratch.path())]).programs)
}

fn entry(catalogue: &Catalogue, path: &str) -> KeyEntry {
    catalogue
        .entry(&KeyPath(path.to_owned()))
        .unwrap_or_else(|| panic!("{path} is in the catalogue"))
        .clone()
}

fn waits(key: &str, waiting: &str, shipped: &str) -> WaitingSetting {
    WaitingSetting {
        key: key.to_owned(),
        waiting: waiting.to_owned(),
        shipped: shipped.to_owned(),
    }
}

fn pair(key: &str, value: &str) -> SettingPair {
    SettingPair {
        key: key.to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn only_almanacs_keys_are_memory_settings_and_only_their_pages_hold_them() {
    let catalogue = real("consent-which-keys");
    for (path, want) in [
        (AT_REST, true),
        (SEARCH_DAYS, true),
        ("dock.magnification", false),
    ] {
        assert_eq!(is_memory_setting(&entry(&catalogue, path)), want, "{path}");
    }
    let pages: Vec<(&str, bool)> = catalogue
        .pages
        .iter()
        .map(|page| (page.slug.0.as_str(), holds_memory_settings(page)))
        .collect();
    assert!(pages.contains(&("intelligence", true)), "{pages:?}");
    assert!(pages.contains(&("appearance", false)), "{pages:?}");
}

#[test]
fn a_saved_value_is_confirmed_only_when_memory_says_that_value_waits() {
    let waiting = [
        waits(AT_REST, "plain", "sealed"),
        waits(SEARCH_DAYS, "7", "30"),
    ];
    // `(case, what the person set, what is confirmed)`.
    let cases: &[(&str, Vec<SettingPair>, Vec<SettingPair>)] = &[
        (
            "a loose word",
            vec![pair(AT_REST, "plain")],
            vec![pair(AT_REST, "plain")],
        ),
        (
            "a loose number",
            vec![pair(SEARCH_DAYS, "7")],
            vec![pair(SEARCH_DAYS, "7")],
        ),
        ("a plain value", vec![pair(SEARCH_DAYS, "60")], vec![]),
        (
            "a key that does not wait",
            vec![pair(APPLY, "review")],
            vec![],
        ),
        (
            "only the pairs that wait",
            vec![pair(AT_REST, "plain"), pair(APPLY, "review")],
            vec![pair(AT_REST, "plain")],
        ),
    ];
    for (name, set, want) in cases {
        assert_eq!(&to_confirm(&waiting, set), want, "{name}");
    }
    assert_eq!(
        to_confirm(&[], &[pair(AT_REST, "plain")]),
        vec![],
        "nothing waits"
    );
}

#[test]
fn a_saved_value_is_named_the_way_memory_names_it() {
    let catalogue = real("consent-pair");
    let cases = [
        (AT_REST, toml::Value::String("plain".to_owned()), "plain"),
        (SEARCH_DAYS, toml::Value::Integer(7), "7"),
    ];
    for (path, value, text) in cases {
        assert_eq!(
            SettingPair::of(&entry(&catalogue, path), &value),
            pair(path, text),
            "{path}"
        );
    }
}

#[test]
fn the_note_says_what_memory_keeps_meanwhile_in_the_rows_own_words() {
    let catalogue = real("consent-note");
    // `(key, shipped value, the words)`.
    let cases = [
        (AT_REST, "plain", "sealed", "\u{201c}Private\u{201d}"),
        (APPLY, "review", "auto", "\u{201c}Right away\u{201d}"),
        (SEARCH_DAYS, "7", "30", "30 days"),
    ];
    for (path, waiting, shipped, kept) in cases {
        assert_eq!(
            waiting_note(&entry(&catalogue, path), &waits(path, waiting, shipped)),
            format!("Waits for you to confirm. Until then, memory keeps {kept}."),
            "{path}"
        );
    }
}

#[test]
fn a_failure_stays_while_its_key_waits_and_goes_when_confirmed_or_gone() {
    let keys = [AT_REST.to_owned()];
    let one = || {
        vec![
            waits(AT_REST, "plain", "sealed"),
            waits(SEARCH_DAYS, "7", "30"),
        ]
    };
    let troubled = ConsentState::default()
        .listed(one())
        .troubled(&keys, Trouble::Invalid);
    assert_eq!(troubled.trouble_for(AT_REST), Some(Trouble::Invalid));
    assert_eq!(troubled.trouble_for(SEARCH_DAYS), None);

    let again = troubled.clone().listed(one());
    assert_eq!(
        again.trouble_for(AT_REST),
        Some(Trouble::Invalid),
        "still waits"
    );
    assert_eq!(
        troubled
            .clone()
            .listed(vec![waits(SEARCH_DAYS, "7", "30")])
            .trouble_for(AT_REST),
        None,
        "no longer waits"
    );
    assert_eq!(
        troubled.cleared(&keys).trouble_for(AT_REST),
        None,
        "confirmed"
    );
    assert!(again.waiting_for(SEARCH_DAYS).is_some());
    assert!(again.waiting_for(APPLY).is_none());
}

#[test]
fn memorys_confinement_word_is_parsed_once_and_has_a_line_only_when_there_is_something_to_say() {
    // `(the word on the bus, what it parses to, the line shown)`.
    const CASES: &[(&str, Confinement, Option<&str>)] = &[
        (
            "confined",
            Confinement::Confined,
            Some("Memory keeps itself to its own folders"),
        ),
        (
            "partial",
            Confinement::Partial,
            Some(
                "Memory keeps itself to its own folders, though this computer can only enforce part of that",
            ),
        ),
        (
            "not-available",
            Confinement::NotAvailable,
            Some("This computer can't keep memory to its own folders"),
        ),
        ("off", Confinement::Off, None),
        ("somewhere-new", Confinement::Unknown, None),
        ("Confined", Confinement::Unknown, None),
        ("", Confinement::Unknown, None),
    ];
    for (word, parsed, line) in CASES {
        assert_eq!(Confinement::from_word(word), *parsed, "{word:?}");
        assert_eq!(parsed.line(), *line, "{word:?}");
    }
    assert_eq!(
        Confinement::default(),
        Confinement::Unknown,
        "before a read"
    );
}

#[test]
fn the_state_keeps_the_last_confinement_read_and_a_new_list_does_not_lose_it() {
    let state = ConsentState::default();
    assert_eq!(state.confinement(), Confinement::Unknown);
    let state = state
        .confined(Confinement::Partial)
        .listed(vec![waits(AT_REST, "plain", "sealed")])
        .troubled(&[AT_REST.to_owned()], Trouble::Failed)
        .cleared(&[AT_REST.to_owned()]);
    assert_eq!(state.confinement(), Confinement::Partial);
}

#[test]
fn only_the_memory_section_of_a_page_with_memory_settings_tells_the_confinement() {
    let catalogue = real("consent-confinement-section");
    let sections: Vec<(&str, bool)> = catalogue
        .pages
        .iter()
        .flat_map(|page| page.sections.iter().chain(&page.advanced))
        .map(|section| (section.title.as_str(), tells_confinement(section)))
        .collect();
    assert!(sections.contains(&("Memory", true)), "{sections:?}");
    assert!(sections.contains(&("History", false)), "{sections:?}");
}
