//! The Intelligence page over the schemas docket, almanac and cua ship: each row in the section
//! its own `section` names, the Advanced ones as sheet rows, a write in the program's own
//! file, and cua's list of apps edited as rows.

use crate::support::Sandbox;
use crate::support::drive::{click, scroll_into_view};
use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::{Point, ShortcutKey};
use ds_harness::{Driver, Harness, Input, Query};
use std::time::Duration;

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

/// The page's markup with entities for an apostrophe read back, cut where the Advanced
/// group starts: what is drawn before it, and what it holds.
fn basic_and_advanced(harness: &Harness) -> (String, String) {
    let html = harness.html().replace("&#39;", "'");
    let cut = html.find("class=\"advanced\"").expect("the advanced group");
    (html[..cut].to_owned(), html[cut..].to_owned())
}

/// Each of `texts` is on the page after the one before it.
fn in_order(html: &str, texts: &[&str]) {
    let mut from = 0;
    for text in texts {
        let at = html[from..]
            .find(&format!(">{text}<"))
            .unwrap_or_else(|| panic!("{text:?} is not drawn after byte {from}"));
        from += at;
    }
}

/// `(section, the rows it shows before Advanced)`, in the order the page draws them.
const BASIC: &[(&str, &[&str])] = &[
    (
        "Models",
        &[
            "Keep AI on this computer",
            "Count my other machines as this computer",
            "Let my other computers use this computer's models",
        ],
    ),
    ("Memory", &["Memory files", "Tidy memory"]),
    (
        "Computer use",
        &[
            "Let the assistant use my apps for me",
            "Apps it may use",
            "Where it works",
            "Pause when I touch the window",
        ],
    ),
    (
        "Assistant",
        &["Asks first", "Other apps", "Coding helpers", "Undo history"],
    ),
];

/// The sections the advanced rows are grouped into: each is one sheet row, "<Section>...".
const ADVANCED: &[&str] = &[
    "Assistant",
    "Limits",
    "Memory",
    "History",
    "Run limits",
    "Timing",
    "Window detail",
    "Automatic",
    "Structured output",
];

#[test]
fn each_basic_row_is_drawn_in_the_section_its_schema_names() {
    let sandbox = Sandbox::new("programs-basic");
    let harness = sandbox.harness(intelligence());
    let (basic, _) = basic_and_advanced(&harness);
    for (section, rows) in BASIC {
        let mut texts = vec![*section];
        texts.extend(rows.iter().copied());
        in_order(&basic, &texts);
    }
    let sections: Vec<&str> = BASIC.iter().map(|(section, _)| *section).collect();
    in_order(&basic, &sections);
}

#[test]
fn the_advanced_sections_are_sheet_rows_and_their_keys_are_not_among_the_basic_ones() {
    let sandbox = Sandbox::new("programs-advanced");
    let harness = sandbox.harness(intelligence());
    assert_eq!(
        harness.count(".advanced .ds-disclosure"),
        0,
        "no disclosure"
    );
    assert_eq!(
        harness.count(".advanced .ds-field-row"),
        ADVANCED.len(),
        "one row per advanced section"
    );
    assert_eq!(harness.count(".ds-sheet"), 0, "closed until opened");
    let (basic, advanced) = basic_and_advanced(&harness);
    for section in ADVANCED {
        let row = format!(">{section}\u{2026}<");
        assert!(advanced.contains(&row), "{row:?} is a sheet row");
        assert!(!basic.contains(&row), "{row:?} is not among the basic rows");
    }
}

/// Clicks the switch of `label`, scrolled into view.
fn flip(harness: &mut Harness, label: &str) {
    let toggle = format!("[aria-label=\"{label}\"] .ds-toggle");
    scroll_into_view(harness, &toggle);
    click(harness, &toggle);
}

#[test]
fn a_write_lands_in_the_file_of_the_program_that_owns_the_row() {
    let sandbox = Sandbox::new("programs-writes");
    let mut harness = sandbox.harness(intelligence());
    for file in [
        "docket/settings.toml",
        "almanac/settings.toml",
        "cua/settings.toml",
    ] {
        assert_eq!(sandbox.read(file), None, "{file}: looking writes nothing");
    }

    // Top to bottom, because the page is only ever scrolled down.
    let tidy = ".ds-segmented[aria-label=\"Tidy memory\"] .ds-segmented-segment:nth-of-type(2)";
    scroll_into_view(&mut harness, tidy);
    click(&mut harness, tidy);
    let almanac = sandbox
        .read("almanac/settings.toml")
        .expect("almanac's file");
    assert!(almanac.contains("when = \"manual\""), "{almanac}");
    assert_eq!(sandbox.read("cua/settings.toml"), None);
    assert_eq!(sandbox.read("docket/settings.toml"), None);

    flip(&mut harness, "Pause when I touch the window");
    let cua = sandbox.read("cua/settings.toml").expect("cua's file");
    assert!(cua.contains("autopause = \"off\""), "{cua}");
    assert!(!cua.contains("when"), "{cua}");
    assert_eq!(sandbox.read("docket/settings.toml"), None);

    flip(&mut harness, "Other apps");
    let docket = sandbox.read("docket/settings.toml").expect("docket's file");
    assert!(docket.contains("expose = \"on\""), "{docket}");
    assert!(
        !docket.contains("autopause") && !docket.contains("when"),
        "{docket}"
    );
    assert_eq!(
        sandbox.read("quire/inferd.toml"),
        None,
        "inferd's file is not touched"
    );
}

#[test]
fn lending_this_computers_models_is_off_until_the_switch_is_flipped() {
    const LABEL: &str = "Let my other computers use this computer's models";
    let sandbox = Sandbox::new("programs-tailnet-serve");
    let mut harness = sandbox.harness(intelligence());
    let toggle = format!("[aria-label=\"{LABEL}\"] .ds-toggle");
    scroll_into_view(&mut harness, &toggle);
    assert_eq!(
        harness.attr(&toggle, "aria-checked").as_deref(),
        Some("false"),
        "off by default"
    );
    assert_eq!(
        sandbox.read("quire/inferd.toml"),
        None,
        "looking writes nothing"
    );
    // Turning it on is asked about first (tests/ui/lending.rs drives the answer).
    click(&mut harness, &toggle);
    assert_eq!(
        sandbox.read("quire/inferd.toml"),
        None,
        "the press alone writes nothing"
    );
}

#[test]
fn an_advanced_row_writes_its_own_file_too() {
    let sandbox = Sandbox::new("programs-advanced-write");
    let mut harness = sandbox.harness(intelligence());
    let opener = ".advanced [aria-label=\"Open Run limits\"]";
    scroll_into_view(&mut harness, opener);
    click(&mut harness, opener);
    assert_eq!(harness.count(".ds-sheet"), 1, "the section's sheet is up");
    let slider = "[aria-label=\"Steps per run\"] .ds-slider";
    scroll_into_view(&mut harness, slider);
    let at = harness.centre(slider).expect("the slider");
    harness.send(Input::click(Point { x: at.x, y: at.y }));
    harness.advance(Duration::from_millis(400));
    let cua = sandbox.read("cua/settings.toml").expect("cua's file");
    assert!(cua.contains("steps = "), "{cua}");
    assert_eq!(sandbox.read("docket/settings.toml"), None);
}

const TWO_APPS: &str = "[cua]\napps = [\
    { app = \"org.mozilla.firefox\", allow = \"ask\" }, \
    { app = \"org.gnome.Calculator\", allow = \"session\" }]\n";

/// The apps cua's file lists, as `app allow` lines.
fn apps_of(sandbox: &Sandbox) -> Vec<String> {
    let text = sandbox.read("cua/settings.toml").expect("cua's file");
    let table: toml::Table = text.parse().expect("valid TOML");
    table["cua"]["apps"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|row| {
            format!(
                "{} {}",
                row["app"].as_str().unwrap(),
                row["allow"].as_str().unwrap()
            )
        })
        .collect()
}

fn type_into(harness: &mut Harness, selector: &str, text: &str) {
    click(harness, selector);
    for letter in text.chars() {
        harness.send(Input::key(ShortcutKey::Char(letter)));
    }
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(Duration::from_millis(400));
}

#[test]
fn cuas_apps_are_a_row_each_and_can_be_removed() {
    let sandbox = Sandbox::new("programs-rows-remove");
    sandbox.write("cua/settings.toml", TWO_APPS);
    let mut harness = sandbox.harness(intelligence());
    assert_eq!(harness.count(".key-rows-row"), 2, "a row per app");
    let html = harness.html();
    assert!(html.contains("org.mozilla.firefox") && html.contains("org.gnome.Calculator"));
    assert!(
        !html.contains("Edit in file"),
        "the rows are edited here, not in the file"
    );

    let remove = ".key-rows-row:nth-child(1) [data-variant=inline]";
    scroll_into_view(&mut harness, remove);
    click(&mut harness, remove);
    assert_eq!(apps_of(&sandbox), ["org.gnome.Calculator session"]);
    assert_eq!(harness.count(".key-rows-row"), 1);
}

#[test]
fn cuas_apps_take_a_new_row_only_when_its_app_is_a_desktop_entry_id() {
    let sandbox = Sandbox::new("programs-rows-add");
    sandbox.write("cua/settings.toml", TWO_APPS);
    let mut harness = sandbox.harness(intelligence());
    let field = ".key-rows-new input";
    let add = ".key-rows-new .ds-button:last-child";
    scroll_into_view(&mut harness, field);
    assert_eq!(
        harness.attr(add, "aria-disabled").as_deref(),
        Some("true"),
        "an empty app is not a row"
    );

    type_into(&mut harness, field, "nope");
    assert_eq!(harness.attr(add, "aria-disabled").as_deref(), Some("true"));
    click(&mut harness, add);
    assert_eq!(apps_of(&sandbox).len(), 2, "a bad id is not written");

    let field = ".key-rows-new input";
    type_into(&mut harness, field, "org.example.Notes");
    assert_eq!(harness.attr(add, "aria-disabled"), None, "now it is a row");
    click(&mut harness, add);
    assert_eq!(
        apps_of(&sandbox),
        [
            "org.mozilla.firefox ask",
            "org.gnome.Calculator session",
            "org.example.Notes ask"
        ]
    );
    assert_eq!(harness.count(".key-rows-row"), 3);
}

#[test]
fn a_rows_choice_is_changed_in_place_and_keeps_the_other_rows() {
    let sandbox = Sandbox::new("programs-rows-choice");
    sandbox.write("cua/settings.toml", TWO_APPS);
    let mut harness = sandbox.harness(intelligence());
    let button = ".key-rows-row:nth-child(1) .ds-popup .ds-button";
    scroll_into_view(&mut harness, button);
    click(&mut harness, button);
    let item = harness
        .centre(".ds-menu-item:nth-child(3) .ds-menu-label")
        .expect("the menu is open");
    harness.send(Input::pointer_move(item));
    click(&mut harness, ".ds-menu-item:nth-child(3) .ds-menu-label");
    assert_eq!(
        apps_of(&sandbox),
        ["org.mozilla.firefox always", "org.gnome.Calculator session"]
    );
}

#[test]
fn rows_that_do_not_fit_are_shown_as_written_and_never_rewritten() {
    let sandbox = Sandbox::new("programs-rows-foreign");
    let written = "[cua]\napps = [{ app = \"org.a.B\", allow = \"ask\", note = \"mine\" }]\n";
    sandbox.write("cua/settings.toml", written);
    let harness = sandbox.harness(intelligence());
    assert_eq!(harness.count(".key-rows"), 0);
    assert!(harness.html().contains("Edit in file"));
    assert_eq!(sandbox.read("cua/settings.toml").as_deref(), Some(written));
}
