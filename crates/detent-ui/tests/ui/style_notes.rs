//! The Appearance page's "Use my styles" switch and the notes under it, over scratch style files.

use crate::support::Sandbox;
use crate::support::drive::click;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::Harness;
use std::time::Duration;

const SWITCH: &str = ".section .ds-toggle[*|aria-label=\"Use my styles\"]";
const LINE: &str = ".style-notes";
const SHEET_ROWS: &str = ".style-notes-sheet .ds-field-row";

fn appearance() -> Opening {
    Opening {
        page: Some(PageSlug("appearance".to_owned())),
        query: String::new(),
    }
}

/// Lets the file reads finish: they run on their own threads.
fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    for _ in 0..750 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

#[test]
fn use_my_styles_is_a_switch_that_writes_off_and_use() {
    let sandbox = Sandbox::new("use-my-styles");
    let mut harness = sandbox.harness(appearance());
    assert_eq!(harness.count(SWITCH), 1, "a use/off key is one switch");
    assert_eq!(
        harness.attr(SWITCH, "aria-checked").as_deref(),
        Some("true"),
        "Use is the default, and on"
    );
    click(&mut harness, SWITCH);
    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains("custom_styles = \"off\""), "{text}");
    assert_eq!(
        harness.attr(SWITCH, "aria-checked").as_deref(),
        Some("false")
    );
    click(&mut harness, SWITCH);
    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains("custom_styles = \"use\""), "{text}");
}

#[test]
fn styles_with_nothing_to_say_have_no_line() {
    let sandbox = Sandbox::new("style-notes-none");
    sandbox.write("quire/style.css", "body { color: red; }\n");
    let mut harness = sandbox.harness(appearance());
    for _ in 0..40 {
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(harness.count(LINE), 0, "{}", harness.html());
}

#[test]
fn the_notes_of_every_style_file_are_counted_and_listed_with_their_file_and_line() {
    let sandbox = Sandbox::new("style-notes-listed");
    sandbox.write(
        "quire/style.css",
        "/* mine */\nbody { transition: all 1s; }\n",
    );
    sandbox.write(
        "sill/style.css",
        "\nwindow { box-shadow: 0 0 200px black; }\n",
    );
    let mut harness = sandbox.harness(appearance());
    settle(&mut harness, "the notes line", |h| h.count(LINE) == 1);
    assert_eq!(
        harness
            .text_of(&format!("{LINE} .ds-field-row-title"))
            .as_deref(),
        Some("2 notes")
    );
    click(&mut harness, &format!("{LINE} .ds-button"));
    assert_eq!(harness.count(SHEET_ROWS), 2, "{}", harness.html());
    let html = harness.html();
    for want in [
        "Animates everything",
        "quire/style.css, line 2",
        "Large shadows may slow the desktop",
        "sill/style.css, line 2",
    ] {
        assert!(html.contains(want), "{want} is in the sheet:\n{html}");
    }
}

#[test]
fn a_class_an_installed_app_publishes_is_not_an_internal_selector() {
    let style = ".sill-bar { color: red; }\n";
    let published = "app = \"sill\"\nclasses = [\"sill-bar\"]\n";

    let without = Sandbox::new("style-notes-unpublished");
    without.write("quire/style.css", style);
    let mut harness = without.harness(appearance());
    settle(&mut harness, "the notes line", |h| h.count(LINE) == 1);
    assert_eq!(
        harness
            .text_of(&format!("{LINE} .ds-field-row-title"))
            .as_deref(),
        Some("1 note"),
        "a part nobody published may change in an update"
    );

    let with = Sandbox::new("style-notes-published");
    with.write("quire/style.css", style);
    with.write_data("quire/selectors/sill.toml", published);
    let mut harness = with.harness(appearance());
    for _ in 0..40 {
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(harness.count(LINE), 0, "{}", harness.html());
}
