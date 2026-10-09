//! The assistant's places in docket's settings file: written one key at a time, read back as
//! docket reads them, and every other key of the file keeps its text.

use detent_files::{ConfigHome, load_places, save_places};
use detent_testkit::Scratch;
use docket_settings::{
    CloudAccount, CloudAllowed, ModelChoice, OwnComputer, PlaceName, Places, Toggle,
};

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

fn places() -> Places {
    Places {
        this_computer: Some(ModelChoice("small".to_owned())),
        own: vec![OwnComputer {
            name: PlaceName("lab server".to_owned()),
            allowed: Toggle::On,
            model: None,
        }],
        cloud: vec![CloudAccount {
            name: PlaceName("me@acme.example".to_owned()),
            provider: "Acme".to_owned(),
            allowed: CloudAllowed::AskEachTime,
            model: Some(ModelChoice("acme/big".to_owned())),
        }],
    }
}

#[test]
fn what_is_saved_reads_back_as_docket_reads_it() {
    let scratch = Scratch::under(TMP, "places-round-trip");
    let home = ConfigHome(scratch.path().to_path_buf());
    save_places(&home, &places()).expect("saved");
    let loaded = load_places(&home);
    assert_eq!(loaded.places, places());
    assert!(loaded.fallbacks.is_empty());
}

#[test]
fn every_other_key_of_the_file_keeps_its_text() {
    let scratch = Scratch::under(TMP, "places-other-keys");
    let home = ConfigHome(scratch.path().to_path_buf());
    let file = scratch.join("docket/settings.toml");
    std::fs::create_dir_all(file.parent().expect("a parent")).expect("dir");
    let before =
        "# mine\n[agent]\nstrictness = \"strict\"   # careful\n\n[assistant]\nnote = \"kept\"\n";
    std::fs::write(&file, before).expect("written");
    save_places(&home, &places()).expect("saved");
    let after = std::fs::read_to_string(&file).expect("read");
    for kept in [
        "# mine",
        "strictness = \"strict\"   # careful",
        "note = \"kept\"",
    ] {
        assert!(after.contains(kept), "{kept} in\n{after}");
    }
    assert_eq!(load_places(&home).places, places());
}

#[test]
fn no_file_is_a_machine_with_this_computer_only() {
    let scratch = Scratch::under(TMP, "places-no-file");
    let home = ConfigHome(scratch.path().to_path_buf());
    assert_eq!(load_places(&home).places, Places::none());
}
