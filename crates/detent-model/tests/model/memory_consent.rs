//! The loose memory settings over almanac's real schema: which keys can wait, the pairs a save
//! confirms, the value the note names, and the state's bookkeeping.

use almanac_core::WaitingSetting;
use detent_model::{
    Catalogue, Confinement, ConsentState, KeptValue, KeyEntry, SettingPair, Trouble, discover,
    holds_memory_settings, is_memory_setting, kept_value, tells_confinement, to_confirm,
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
fn the_value_memory_keeps_meanwhile_is_read_in_the_rows_own_words() {
    let catalogue = real("consent-note");
    // `(key, waiting value, shipped value, what the row reads it as)`.
    let cases = [
        (
            AT_REST,
            "plain",
            "sealed",
            KeptValue::Named("Private".to_owned()),
        ),
        (
            APPLY,
            "review",
            "auto",
            KeptValue::Named("Right away".to_owned()),
        ),
        (
            SEARCH_DAYS,
            "7",
            "30",
            KeptValue::Measured("30 days".to_owned()),
        ),
    ];
    for (path, waiting, shipped, kept) in cases {
        assert_eq!(
            kept_value(&entry(&catalogue, path), &waits(path, waiting, shipped)),
            kept,
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
fn memorys_confinement_word_is_parsed_once() {
    // `(the word on the bus, what it parses to)`.
    const CASES: &[(&str, Confinement)] = &[
        ("confined", Confinement::Confined),
        ("partial", Confinement::Partial),
        ("not-available", Confinement::NotAvailable),
        ("off", Confinement::Off),
        ("somewhere-new", Confinement::Unknown),
        ("Confined", Confinement::Unknown),
        ("", Confinement::Unknown),
    ];
    for (word, parsed) in CASES {
        assert_eq!(Confinement::from_word(word), *parsed, "{word:?}");
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
