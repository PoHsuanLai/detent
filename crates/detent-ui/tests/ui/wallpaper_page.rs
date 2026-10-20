//! The Wallpaper page: the key `wallpaper.picture` drawn as a grid of pictures, over a fake
//! list and a fake chooser.

use crate::support::Sandbox;
use crate::support::drive::{click, page_of, title_of};
use crate::support::pictures::Fake;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_settings::schema::{
    AppId, ControlName, Exposure, FilePath, Help, KeyKind, KeyPath, KeySpec, Label, Page, Schema,
    Section,
};

const FILE: &str = "sill/settings.toml";

/// A stand-in for sill's schema with only the wallpaper's key (sill's fixture predates it).
fn sill() -> Schema {
    Schema {
        pages: Vec::new(),
        app: AppId("sill".to_owned()),
        file: FilePath(FILE.to_owned()),
        foreign: Default::default(),
        version: 1,
        key: vec![KeySpec {
            control: Some(ControlName("picture".to_owned())),
            path: KeyPath("wallpaper.picture".to_owned()),
            kind: KeyKind::Text,
            default: toml::Value::String(String::new()),
            label: Label("Picture".to_owned()),
            help: Help("The picture on the desktop.".to_owned()),
            page: Page::declared("wallpaper"),
            section: Section("Wallpaper".to_owned()),
            exposure: Exposure::Basic,
            labels: Default::default(),
            agent: Default::default(),
            unavailable: Default::default(),
            groups: Default::default(),
            places: Default::default(),
            note: None,
        }],
    }
}

fn opening() -> Opening {
    Opening {
        page: Some(PageSlug("wallpaper".to_owned())),
        query: String::new(),
    }
}

/// A world with only the wallpaper key, working through `fake`.
fn world(name: &str, fake: &Fake) -> Sandbox {
    let sandbox = Sandbox::new(name).with_pictures(fake.pictures());
    sandbox.remove_schema("sill");
    sandbox.install_schema(&sill());
    sandbox
}

/// The second picture's button.
const SECOND: &str = ".ds-radio-group-item:nth-child(2)";

#[test]
fn the_page_offers_the_pictures_and_clicking_one_writes_its_path() {
    let fake = Fake::with(&["/pics/sky.jpg", "/pics/dune.png"]);
    let sandbox = world("wallpaper-pick", &fake);
    let mut harness = sandbox.harness(opening());
    assert_eq!(page_of(&harness).as_deref(), Some("wallpaper"));
    assert_eq!(title_of(&harness).as_deref(), Some("Wallpaper"));

    let html = harness.html();
    for words in ["dune", "sky", "Add Picture\u{2026}"] {
        assert!(html.contains(words), "{words} is on the page:\n{html}");
    }
    assert_eq!(harness.count(".ds-radio-group-item"), 2);
    assert!(
        !html.contains("aria-checked=\"true\""),
        "none is current yet"
    );

    assert_eq!(sandbox.read(FILE), None, "looking writes nothing");
    click(&mut harness, SECOND);
    let text = sandbox.read(FILE).expect("the file was written");
    assert!(text.contains("picture = \"/pics/sky.jpg\""), "{text}");
    assert_eq!(
        harness.attr(SECOND, "aria-checked").as_deref(),
        Some("true"),
        "the picked one is checked"
    );
}

#[test]
fn add_picture_asks_for_one_and_writes_the_path_it_was_given() {
    let fake = Fake::with(&["/pics/dune.png"]);
    fake.will_choose("/home/p/mine.png");
    let sandbox = world("wallpaper-add", &fake);
    let mut harness = sandbox.harness(opening());

    click(&mut harness, ".key-pictures .ds-button");
    assert_eq!(fake.asked(), 1, "the person was asked once");
    let text = sandbox.read(FILE).expect("the file was written");
    assert!(text.contains("picture = \"/home/p/mine.png\""), "{text}");
    assert!(
        harness.html().contains("mine"),
        "the new picture is in the grid"
    );
    assert_eq!(harness.count(".ds-radio-group-item"), 2);
}

#[test]
fn a_closed_chooser_writes_nothing() {
    let fake = Fake::with(&["/pics/dune.png"]);
    let sandbox = world("wallpaper-closed", &fake);
    let mut harness = sandbox.harness(opening());

    click(&mut harness, ".key-pictures .ds-button");
    assert_eq!(fake.asked(), 1);
    assert_eq!(sandbox.read(FILE), None);
}
