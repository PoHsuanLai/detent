//! The places' transitions: nothing turns a cloud account on without the person's yes.

use super::{Confirm, PlaceRef, PlacesEffect, PlacesInput, PlacesState, known_of, places_step};
use crate::accounts::AccountsPane;
use crate::computers::{ComputerRow, Standing};
use docket_settings::{
    CloudAccount, CloudAllowed, Known, KnownPlace, ModelChoice, OwnComputer, PlaceName, Places,
    Toggle,
};
use porter_infer::ComputerName;

fn name(text: &str) -> PlaceName {
    PlaceName(text.to_owned())
}

fn state() -> PlacesState {
    PlacesState::of(Places {
        this_computer: None,
        own: vec![OwnComputer {
            name: name("lab"),
            allowed: Toggle::On,
            model: None,
        }],
        cloud: vec![CloudAccount {
            name: name("me@acme.example"),
            provider: "Acme".to_owned(),
            allowed: CloudAllowed::Off,
            model: None,
        }],
    })
}

fn cloud(state: &PlacesState) -> &CloudAccount {
    &state.places.cloud[0]
}

fn written(effects: &[PlacesEffect]) -> Option<&Places> {
    effects.iter().map(|PlacesEffect::Write(p)| p).next()
}

#[test]
fn a_cloud_account_is_turned_on_only_after_the_person_says_yes() {
    let switch = PlacesInput::CloudSwitched {
        name: name("me@acme.example"),
        to: Toggle::On,
    };
    // The switch asks first and writes nothing.
    let (asked, effects) = places_step(state(), switch);
    assert_eq!(asked.confirming, Some(name("me@acme.example")));
    assert_eq!(cloud(&asked).allowed, CloudAllowed::Off);
    assert!(effects.is_empty());
    // Yes turns it on and writes it.
    let (next, effects) = places_step(asked.clone(), PlacesInput::Answered(Confirm::Yes));
    assert_eq!(cloud(&next).allowed, CloudAllowed::On);
    assert_eq!(next.confirming, None);
    assert_eq!(written(&effects), Some(&next.places));
    // No leaves it off and writes nothing.
    let (next, effects) = places_step(asked, PlacesInput::Answered(Confirm::No));
    assert_eq!(cloud(&next).allowed, CloudAllowed::Off);
    assert_eq!(next.confirming, None);
    assert!(effects.is_empty());
    // An answer with nothing asked changes nothing.
    let (next, effects) = places_step(state(), PlacesInput::Answered(Confirm::Yes));
    assert_eq!(next, state());
    assert!(effects.is_empty());
}

#[test]
fn turning_a_cloud_account_off_needs_no_question() {
    let on = {
        let mut on = state();
        on.places.cloud[0].allowed = CloudAllowed::On;
        on
    };
    let (next, effects) = places_step(
        on,
        PlacesInput::CloudSwitched {
            name: name("me@acme.example"),
            to: Toggle::Off,
        },
    );
    assert_eq!(cloud(&next).allowed, CloudAllowed::Off);
    assert_eq!(written(&effects), Some(&next.places));
}

#[test]
fn ask_each_time_applies_only_to_an_account_that_is_on() {
    let ask = |from: &PlacesState, to| {
        places_step(
            from.clone(),
            PlacesInput::AskEachTime {
                name: name("me@acme.example"),
                to,
            },
        )
    };
    let (off, effects) = ask(&state(), Toggle::On);
    assert_eq!(
        cloud(&off).allowed,
        CloudAllowed::Off,
        "an off account stays off"
    );
    assert!(effects.is_empty());
    let mut on = state();
    on.places.cloud[0].allowed = CloudAllowed::On;
    let (asking, _) = ask(&on, Toggle::On);
    assert_eq!(cloud(&asking).allowed, CloudAllowed::AskEachTime);
    let (back, _) = ask(&asking, Toggle::Off);
    assert_eq!(cloud(&back).allowed, CloudAllowed::On);
}

#[test]
fn a_computer_of_the_persons_switches_and_writes() {
    let (next, effects) = places_step(
        state(),
        PlacesInput::OwnSwitched {
            name: name("lab"),
            to: Toggle::Off,
        },
    );
    assert_eq!(next.places.own[0].allowed, Toggle::Off);
    assert_eq!(written(&effects), Some(&next.places));
}

#[test]
fn a_model_is_kept_for_the_place_it_was_picked_for() {
    let pick = |place| PlacesInput::Model {
        place,
        model: Some(ModelChoice("small".to_owned())),
    };
    let (next, _) = places_step(state(), pick(PlaceRef::ThisComputer));
    assert_eq!(
        next.places.this_computer,
        Some(ModelChoice("small".to_owned()))
    );
    let (next, _) = places_step(next, pick(PlaceRef::Cloud(name("me@acme.example"))));
    assert_eq!(cloud(&next).model, Some(ModelChoice("small".to_owned())));
    assert_eq!(next.places.own[0].model, None);
}

#[test]
fn reading_adds_known_places_off_and_never_turns_one_on() {
    let read = Places::none();
    let known = Known(vec![KnownPlace::Cloud(
        name("new@acme.example"),
        "Acme".to_owned(),
    )]);
    let (next, effects) = places_step(
        PlacesState::of(Places::none()),
        PlacesInput::Read(read, known),
    );
    assert_eq!(next.places.cloud.len(), 1);
    assert_eq!(next.places.cloud[0].allowed, CloudAllowed::Off);
    assert!(effects.is_empty());
}

#[test]
fn the_question_names_the_account_and_the_provider_in_dockets_words() {
    let (asked, _) = places_step(
        state(),
        PlacesInput::CloudSwitched {
            name: name("me@acme.example"),
            to: Toggle::On,
        },
    );
    assert_eq!(
        asked.question(),
        Some((
            "Turn on me@acme.example?".to_owned(),
            "Requests the assistant can't do on your computers may be sent to Acme.".to_owned()
        ))
    );
    assert_eq!(state().question(), None);
}

mod offers {
    use super::name;
    use crate::accounts::{Account, AccountKey, AccountSpend, AccountStatus, AccountsPane, Place};
    use crate::assistants::{PlaceRef, offers_for};
    use crate::picker::{Busy, ChoiceSection, KindRows, ModelRow, Ready};
    use ds_settings::schema::{KeyPath, WordLabels};
    use porter_core::sheet::ProviderGroup;
    use std::collections::BTreeMap;

    fn picker(words: &[&str]) -> Ready {
        Ready {
            kinds: vec![KindRows {
                area: None,
                label: "Text".to_owned(),
                rows: vec![ModelRow {
                    path: KeyPath("ai.model.text.fast".to_owned()),
                    label: "Fast".to_owned(),
                    help: String::new(),
                    sections: vec![ChoiceSection {
                        group: None,
                        words: words.iter().map(|w| (*w).to_owned()).collect(),
                    }],
                    labels: WordLabels(
                        words
                            .iter()
                            .map(|w| ((*w).to_owned(), format!("model of {w}")))
                            .collect(),
                    ),
                    unavailable: BTreeMap::new(),
                    places: BTreeMap::new(),
                    note: None,
                    value: String::new(),
                    pool: Default::default(),
                    local: Default::default(),
                }],
            }],
            busy: Busy::Idle,
        }
    }

    fn pane() -> AccountsPane {
        let mut pane = AccountsPane::default();
        pane.accounts.push(Account {
            key: AccountKey("acme".to_owned()),
            module: crate::accounts::ModuleId::Accountd,
            label: "me@acme.example".to_owned(),
            status: AccountStatus::Ok,
            place: Place::Elsewhere,
            expires: None,
            sign_in: None,
            provider: None,
            group: ProviderGroup::Intelligence,
            services: Vec::new(),
            sync: Vec::new(),
            show_in_files: None,
            permissions: Vec::new(),
            reauth: None,
            sign_out: None,
            remove: None,
            used_by: Vec::new(),
            spend: AccountSpend::default(),
        });
        pane
    }

    #[test]
    fn a_cloud_place_offers_only_the_models_of_its_own_account() {
        let ready = picker(&["acme/big", "other/small", "acme/big"]);
        let offers = offers_for(&PlaceRef::Cloud(name("me@acme.example")), &pane(), &ready);
        let words: Vec<&str> = offers.iter().map(|o| o.choice.0.as_str()).collect();
        assert_eq!(words, ["acme/big"], "once, and not another account's");
        assert_eq!(offers[0].label, "model of acme/big");
    }

    #[test]
    fn a_computer_of_the_persons_offers_none_for_there_is_no_list_of_them() {
        let ready = picker(&["acme/big"]);
        assert!(offers_for(&PlaceRef::Own(name("lab")), &pane(), &ready).is_empty());
    }
}

#[test]
fn the_computers_the_service_lists_are_known_places() {
    let lab = ComputerRow {
        computer: ComputerName::parse("lab").expect("a name"),
        name: "lab".to_owned(),
        models: vec![],
        standing: Standing::Silent,
    };
    assert_eq!(
        known_of(&AccountsPane::default(), &[lab]),
        Known(vec![KnownPlace::OwnComputer(name("lab"))])
    );
}
