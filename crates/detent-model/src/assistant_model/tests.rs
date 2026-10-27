use super::{
    ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, SetupAction, SetupAvailability,
    assistant_model_page, assistant_of, is_assistant_setup, setup_availability,
};
use crate::accounts::AddEnd;
use crate::catalogue::Catalogue;
use crate::page::PageSlug;
use crate::page_kind::PageKind;
use crate::picker::{Described, PickerInput, PickerState, step};
use ds_settings::schema::{
    AgentSetting, ChoiceWord, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section,
    UnavailableReason, WordLabels,
};
use std::collections::BTreeMap;

fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

fn row(key: &str, gated: &[&str]) -> KeySpec {
    KeySpec {
        control: None,
        path: path(key),
        kind: KeyKind::Menu {
            variants: ["", "auto", "local/qwen", "anthropic/claude"]
                .map(str::to_owned)
                .to_vec(),
        },
        default: toml::Value::String(String::new()),
        label: Label("Balanced".to_owned()),
        help: Help(String::new()),
        page: Page::Intelligence,
        section: Section("Models / Text".to_owned()),
        exposure: Exposure::Basic,
        labels: WordLabels::default(),
        agent: AgentSetting::HandsOff,
        unavailable: gated
            .iter()
            .map(|word| {
                (
                    ChoiceWord((*word).to_owned()),
                    UnavailableReason("Add an account to use".to_owned()),
                )
            })
            .collect(),
        groups: BTreeMap::new(),
        places: BTreeMap::new(),
        note: None,
    }
}

/// A picker whose text slot lists a row per tier; the balanced one holds `value`.
fn picker(value: &str) -> PickerState {
    let described = Described {
        keys: vec![
            row("ai.model.text.fast", &[]),
            row("ai.model.text.balanced", &["anthropic/claude"]),
            row("ai.model.computer_use.balanced", &[]),
        ],
        values: BTreeMap::from([
            (path("ai.model.text.fast"), "local/qwen".to_owned()),
            (path("ai.model.text.balanced"), value.to_owned()),
        ]),
        ..Default::default()
    };
    step(PickerState::Looking, PickerInput::Found(Ok(described))).0
}

/// `picker`, with the text slot's Automatic list holding `held`.
fn picker_with_pool(value: &str, held: &[&str]) -> PickerState {
    let mut pool = row("ai.auto.pool.text", &[]);
    pool.kind = KeyKind::List(Box::new(pool.kind));
    let mut described = Described {
        keys: vec![row("ai.model.text.balanced", &[]), pool],
        values: BTreeMap::from([(path("ai.model.text.balanced"), value.to_owned())]),
        ..Default::default()
    };
    described.lists.insert(
        path("ai.auto.pool.text"),
        held.iter().map(|word| (*word).to_owned()).collect(),
    );
    step(PickerState::Looking, PickerInput::Found(Ok(described))).0
}

fn chose(state: PickerState, word: &str) -> PickerState {
    step(
        state,
        PickerInput::Chose {
            path: path("ai.model.text.balanced"),
            word: word.to_owned(),
        },
    )
    .0
}

#[test]
fn the_assistants_row_is_the_text_slots_balanced_tier() {
    let state = picker("auto");
    assert_eq!(
        assistant_of(&state).map(|row| row.path.0.as_str()),
        Some("ai.model.text.balanced"),
        "not the fast tier, not another slot's balanced one"
    );
    let older = step(
        PickerState::Looking,
        PickerInput::Found(Ok(Described {
            keys: vec![row("ai.model.llm.balanced", &[])],
            values: BTreeMap::new(),
            ..Default::default()
        })),
    )
    .0;
    assert_eq!(
        assistant_of(&older).map(|row| row.path.0.as_str()),
        Some("ai.model.llm.balanced"),
        "an older module's name for the text slot"
    );
    let without = step(
        PickerState::Looking,
        PickerInput::Found(Ok(Described {
            keys: vec![row("ai.model.text.fast", &[])],
            values: BTreeMap::new(),
            ..Default::default()
        })),
    )
    .0;
    assert_eq!(assistant_of(&without), None);
    assert_eq!(assistant_of(&PickerState::Looking), None);
}

#[test]
fn a_model_nobody_chose_shows_as_nothing_chosen() {
    let state = picker("");
    let row = assistant_of(&state).expect("the assistant's row");
    assert_eq!(row.choice(), crate::picker::Choice::Nothing);
    assert_eq!(row.choice().selected(), None, "no item is marked");
}

#[test]
fn the_setup_buttons_follow_what_is_chosen() {
    let waiting_on_account = chose(picker("auto"), "anthropic/claude");
    let writing = chose(picker("auto"), "local/qwen");
    let no_assistant_row = step(
        PickerState::Looking,
        PickerInput::Found(Ok(Described {
            keys: vec![row("ai.model.text.fast", &[])],
            values: BTreeMap::new(),
            ..Default::default()
        })),
    )
    .0;
    // `(case, the picker, Skip, Continue)`.
    let cases = [
        (
            "still looking for the module",
            PickerState::Looking,
            SetupAvailability::Now,
            SetupAvailability::NotYet,
        ),
        (
            "the module is not there",
            PickerState::Unavailable(crate::Absence::NotRunning),
            SetupAvailability::Now,
            SetupAvailability::NotYet,
        ),
        (
            "nothing chosen yet",
            picker(""),
            SetupAvailability::Now,
            SetupAvailability::NotYet,
        ),
        (
            "Automatic with no model to choose from",
            picker("auto"),
            SetupAvailability::Now,
            SetupAvailability::NotYet,
        ),
        (
            "Automatic with a model to choose from",
            picker_with_pool("auto", &["local/qwen"]),
            SetupAvailability::Now,
            SetupAvailability::Now,
        ),
        (
            "a model is chosen",
            picker("local/qwen"),
            SetupAvailability::Now,
            SetupAvailability::Now,
        ),
        (
            "the pick is still being written",
            writing,
            SetupAvailability::Now,
            SetupAvailability::NotYet,
        ),
        (
            "the pick waits on an account",
            waiting_on_account,
            SetupAvailability::Now,
            SetupAvailability::NotYet,
        ),
        (
            "the module lists no row for the assistant",
            no_assistant_row,
            SetupAvailability::Now,
            SetupAvailability::NotYet,
        ),
    ];
    for (case, state, skip, next) in cases {
        assert_eq!(
            setup_availability(&state, SetupAction::Skip),
            skip,
            "{case}: Skip"
        );
        assert_eq!(
            setup_availability(&state, SetupAction::Continue),
            next,
            "{case}: Continue"
        );
    }
}

#[test]
fn continue_opens_once_the_pick_is_written_and_stays_shut_if_it_is_not() {
    let writing = chose(picker(""), "local/qwen");
    assert_eq!(
        setup_availability(&writing, SetupAction::Continue),
        SetupAvailability::NotYet,
        "nothing is chosen until the module has taken it"
    );
    let (done, _) = step(
        writing,
        PickerInput::Applied(path("ai.model.text.balanced")),
    );
    assert_eq!(
        setup_availability(&done, SetupAction::Continue),
        SetupAvailability::Now
    );

    let (refused, _) = step(
        chose(picker(""), "local/qwen"),
        PickerInput::Refused {
            path: path("ai.model.text.balanced"),
            why: "no".to_owned(),
        },
    );
    assert_eq!(
        setup_availability(&refused, SetupAction::Continue),
        SetupAvailability::NotYet,
        "a refused pick leaves nothing chosen"
    );

    let (closed, _) = step(
        chose(picker(""), "anthropic/claude"),
        PickerInput::AccountSheetEnded(AddEnd::Closed),
    );
    assert_eq!(
        setup_availability(&closed, SetupAction::Continue),
        SetupAvailability::NotYet,
        "closing the account sheet chooses nothing"
    );
}

#[test]
fn the_page_has_its_own_slug_label_and_place_and_the_setup_step_is_never_listed() {
    let page = assistant_model_page();
    assert_eq!(PageKind::of(&page), PageKind::AssistantModel);
    assert_eq!(
        PageKind::of(&Page::App("anyview".to_owned())),
        PageKind::Program
    );
    assert_eq!(PageKind::of(&Page::Intelligence), PageKind::Intelligence);
    assert_eq!(PageSlug::of(&page).0, ASSISTANT_MODEL_SLUG);
    assert_eq!(page.label(), "Assistant Model");

    assert!(is_assistant_setup(&PageSlug(
        ASSISTANT_SETUP_SLUG.to_owned()
    )));
    assert!(!is_assistant_setup(&PageSlug(
        ASSISTANT_MODEL_SLUG.to_owned()
    )));
    let catalogue = Catalogue::build(&[]);
    assert!(
        catalogue
            .page(&PageSlug(ASSISTANT_SETUP_SLUG.to_owned()))
            .is_none(),
        "the step is a dialogue, not a page of the sidebar"
    );
    let listed: Vec<&str> = catalogue.pages.iter().map(|p| p.slug.0.as_str()).collect();
    assert_eq!(
        listed,
        [
            "wifi",
            "bluetooth",
            "extensions",
            ASSISTANT_MODEL_SLUG,
            "privacy",
            "displays",
            "software-update",
            "accounts"
        ]
    );
}

#[test]
fn opening_local_writes_nothing_so_skipping_leaves_the_model_unset() {
    let (open, effects) = step(
        picker(""),
        PickerInput::Chose {
            path: path("ai.model.text.balanced"),
            word: "on-this-computer".to_owned(),
        },
    );
    assert_eq!(effects, [], "Local is a question, not a model");
    let row = assistant_of(&open).expect("the assistant's row");
    assert_eq!(row.value, "", "still unset");
    assert_eq!(
        setup_availability(&open, SetupAction::Skip),
        SetupAvailability::Now
    );
    assert_eq!(
        setup_availability(&open, SetupAction::Continue),
        SetupAvailability::NotYet,
        "Local is not a model until one is chosen"
    );
}
