//! The Wallpaper page: the key `wallpaper.picture` drawn as a grid of pictures, over a fake
//! list and a fake chooser.

use crate::support::Sandbox;
use crate::support::drive::{click, page_of, title_of};
use crate::support::pictures::Fake;
use detent_model::{AddOutcome, PageSlug};
use detent_ui::Opening;
use ds_settings::schema::{
    AppId, ControlName, Exposure, FilePath, Help, KeyKind, KeyPath, KeySpec, Label, Page, Schema,
    Section,
};

const FILE: &str = "sill/settings.toml";

/// A stand-in for sill's schema with only the wallpaper's key (sill's fixture predates it).
fn sill() -> Schema {
    Schema {
        footnotes: Vec::new(),
        pages: Vec::new(),
        app: AppId("sill".to_owned()),
        file: FilePath(FILE.to_owned()),
        foreign: Default::default(),
        version: 1,
        key: vec![KeySpec {
            ends: None,
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

/// A section's tiles, by the heading above them.
fn tiles_of(heading: &str) -> String {
    format!("[*|aria-label=\"{heading}\"] .ds-cv-item")
}

#[test]
fn the_page_offers_the_pictures_in_two_sections_and_clicking_one_writes_its_path() {
    let fake = Fake::with(&["/pics/sky.jpg", "/pics/dune.png"]).and_added("/mine/home.png");
    let sandbox = world("wallpaper-pick", &fake);
    let mut harness = sandbox.harness(opening());
    assert_eq!(page_of(&harness).as_deref(), Some("wallpaper"));
    assert_eq!(title_of(&harness).as_deref(), Some("Wallpaper"));

    let html = harness.html();
    for words in [
        "Your Pictures",
        "Desktop Pictures",
        "No picture chosen",
        "dune",
        "sky",
        "home",
        "Add Picture\u{2026}",
    ] {
        assert!(html.contains(words), "{words} is on the page:\n{html}");
    }
    assert_eq!(harness.count(&tiles_of("Your Pictures")), 1);
    assert_eq!(harness.count(&tiles_of("Desktop Pictures")), 2);
    assert_eq!(
        harness.count(".ds-cv-item[*|aria-selected=\"true\"]"),
        0,
        "none is current yet"
    );

    assert_eq!(sandbox.read(FILE), None, "looking writes nothing");
    click(
        &mut harness,
        &format!("{}:nth-child(2)", tiles_of("Desktop Pictures")),
    );
    let text = sandbox.read(FILE).expect("the file was written");
    assert!(text.contains("picture = \"/pics/sky.jpg\""), "{text}");
    assert_eq!(
        harness.count(&format!(
            "{}[*|aria-selected=\"true\"]",
            tiles_of("Desktop Pictures")
        )),
        1,
        "the picked one is selected in its own section only"
    );
    assert_eq!(
        harness.count(&format!(
            "{}[*|aria-selected=\"true\"]",
            tiles_of("Your Pictures")
        )),
        0
    );
}

#[test]
fn add_picture_keeps_the_choice_makes_it_the_wallpaper_and_shows_it_under_your_pictures() {
    let fake = Fake::with(&["/pics/dune.png"]);
    fake.will_choose("/data/pictures/mine.png");
    let sandbox = world("wallpaper-add", &fake);
    let mut harness = sandbox.harness(opening());
    assert!(!harness.html().contains("Your Pictures"), "none added yet");

    click(&mut harness, ".key-pictures .ds-button");
    assert_eq!(fake.asked(), 1, "the person was asked once");
    let text = sandbox.read(FILE).expect("the file was written");
    assert!(
        text.contains("picture = \"/data/pictures/mine.png\""),
        "{text}"
    );
    let html = harness.html();
    assert!(html.contains("Your Pictures"), "{html}");
    assert_eq!(harness.count(&tiles_of("Your Pictures")), 1);
    assert_eq!(
        harness.count(&format!(
            "{}[*|aria-selected=\"true\"]",
            tiles_of("Your Pictures")
        )),
        1,
        "the new picture is the current one"
    );
    assert!(
        html.contains(&Fake::small_of("/data/pictures/mine.png")),
        "its small picture is made and drawn:\n{html}"
    );
}

#[test]
fn a_closed_chooser_writes_and_says_nothing_and_a_refusal_says_why() {
    let fake = Fake::with(&["/pics/dune.png"]);
    let sandbox = world("wallpaper-refused", &fake);
    let mut harness = sandbox.harness(opening());

    click(&mut harness, ".key-pictures .ds-button");
    assert_eq!(fake.asked(), 1);
    assert_eq!(sandbox.read(FILE), None);
    assert_eq!(
        harness.count(".key-pictures-told"),
        0,
        "closing is not a fault"
    );

    fake.will_end(AddOutcome::NotAPicture("IMG_1.heic".to_owned()));
    click(&mut harness, ".key-pictures .ds-button");
    assert_eq!(sandbox.read(FILE), None);
    let html = harness.html();
    assert!(
        html.contains("IMG_1.heic") && html.contains("PNG, JPEG or WebP"),
        "the refusal is told in plain words:\n{html}"
    );

    fake.will_choose("/data/pictures/ok.png");
    click(&mut harness, ".key-pictures .ds-button");
    assert_eq!(
        harness.count(".key-pictures-told"),
        0,
        "a success clears the words"
    );
}

#[test]
fn each_tile_draws_the_small_picture_made_from_its_own_preview_and_a_failed_one_is_a_glyph() {
    let fake = Fake::with(&["/pics/sky.jpg", "/pics/huge.png"])
        .and_packaged(
            "/w/Dune/contents/images/5120x2880.png",
            "Dune Dusk",
            "/w/Dune/contents/screenshot.png",
        )
        .cannot_shrink("/pics/huge.png");
    let sandbox = world("wallpaper-small", &fake);
    let harness = sandbox.harness(opening());
    let html = harness.html();

    // The package is one tile, titled by the package, drawn from its screenshot.
    assert_eq!(harness.count(&tiles_of("Desktop Pictures")), 3);
    assert!(html.contains("Dune Dusk"), "{html}");
    for source in ["/pics/sky.jpg", "/w/Dune/contents/screenshot.png"] {
        let small = Fake::small_of(source);
        assert!(
            html.contains(&small),
            "{source} is drawn from {small}:\n{html}"
        );
    }
    assert_eq!(
        fake.made().len(),
        3,
        "a small picture was asked for once per tile: {:?}",
        fake.made()
    );
    assert!(
        !html.contains(&Fake::small_of("/pics/huge.png")),
        "a picture that cannot be made small has no picture, only its name:\n{html}"
    );
    assert!(html.contains("huge"), "but it is still a tile");
}
