//! The Extensions page and the Look menu, over look Extensions installed in a scratch data home.

use crate::support::Sandbox;
use crate::support::drive::{click, page_of, scroll_into_view, sidebar_page, title_of};
use detent_model::{Notice, PageSlug};
use detent_testkit::{
    Changes, install_file_icons, install_look, install_torn_look, install_your_changes,
};
use detent_ui::Opening;
use ds_harness::Harness;
use ds_settings::target::SettingsTarget;
use std::time::Duration;

const PAPER: &str = "org.example.Paper";
const LOOK_MENU: &str = "#key-appearance-look .ds-popup .ds-button";
const ICONS_MENU: &str = "#key-appearance-file-icons .ds-popup .ds-button";
const YOUR_CHANGES: &str = "#extension-user-yourchanges";
const ROW: &str = "#extension-org-example-paper";

fn opening(page: &str) -> Opening {
    Opening {
        page: Some(PageSlug(page.to_owned())),
        query: String::new(),
    }
}

fn install(sandbox: &Sandbox) {
    install_look(&sandbox.data_home().0, PAPER, "Paper", "Warm paper");
}

/// Opens the Look menu and returns the names of its items, in order.
fn look_menu(harness: &mut Harness) -> Vec<String> {
    menu_items(harness, LOOK_MENU)
}

/// Opens the menu whose button is `button` and returns the names of its items, in order.
fn menu_items(harness: &mut Harness, button: &str) -> Vec<String> {
    scroll_into_view(harness, button);
    click(harness, button);
    assert_eq!(
        harness.attr(button, "aria-expanded").as_deref(),
        Some("true"),
        "the click opened the menu"
    );
    let words = (1..=harness.count(".ds-menu-item"))
        .filter_map(|n| harness.text_of(&format!(".ds-menu-item:nth-child({n}) .ds-menu-label")))
        .collect();
    // A click outside an open menu only closes it (as on a Mac), so the menu is put away first.
    click(harness, button);
    assert_eq!(
        harness.attr(button, "aria-expanded").as_deref(),
        Some("false"),
        "a second click closed the menu"
    );
    words
}

#[test]
fn an_installed_look_is_in_the_look_menu_after_standard_and_on_the_extensions_page() {
    let sandbox = Sandbox::new("extensions-listed");
    install(&sandbox);
    install_torn_look(&sandbox.data_home().0, "org.example.Torn", "Torn");

    let mut harness = sandbox.harness(opening("appearance"));
    assert_eq!(look_menu(&mut harness), ["Standard", "Paper"]);
    drop(harness);

    let harness = sandbox.harness(opening("extensions"));
    assert_eq!(page_of(&harness).as_deref(), Some("extensions"));
    assert_eq!(title_of(&harness).as_deref(), Some("Extensions"));
    let titles = harness.text_of(".extensions").unwrap_or_default();
    for want in [
        "Looks",
        "Paper",
        "Warm paper",
        "Can\u{2019}t Be Used",
        "Torn",
        "Some files in this Extension could not be read",
    ] {
        assert!(titles.contains(want), "{want} is on the page:\n{titles}");
    }
    assert_eq!(
        harness.count(".extensions .ds-toggle"),
        1,
        "the Extension that cannot be used has no switch"
    );
    assert_eq!(
        harness
            .attr(&format!("{ROW} .ds-toggle"), "aria-checked")
            .as_deref(),
        Some("true")
    );
}

#[test]
fn switching_a_look_off_writes_the_file_and_drops_it_from_the_menu_until_it_is_back_on() {
    let sandbox = Sandbox::new("extensions-switch");
    install(&sandbox);
    let mut harness = sandbox.harness(opening("extensions"));
    let switch = format!("{ROW} .ds-toggle");
    assert_eq!(sandbox.read("quire/extensions.toml"), None);

    click(&mut harness, &switch);
    let text = sandbox
        .read("quire/extensions.toml")
        .expect("the file was written");
    assert!(text.contains(PAPER), "{text}");
    assert_eq!(
        harness.attr(&switch, "aria-checked").as_deref(),
        Some("false")
    );
    assert_eq!(
        harness.count(ROW),
        1,
        "off is still listed, to turn on again"
    );

    // Appearance is the fourth page; the menu is read afresh when the page is shown.
    click(&mut harness, &sidebar_page("appearance"));
    assert_eq!(page_of(&harness).as_deref(), Some("appearance"));
    assert_eq!(look_menu(&mut harness), ["Standard"]);

    click(&mut harness, &sidebar_page("extensions"));
    assert_eq!(page_of(&harness).as_deref(), Some("extensions"));
    click(&mut harness, &switch);
    let text = sandbox
        .read("quire/extensions.toml")
        .expect("the file was written");
    assert!(!text.contains(PAPER), "{text}");
    click(&mut harness, &sidebar_page("appearance"));
    assert_eq!(look_menu(&mut harness), ["Standard", "Paper"]);
}

#[test]
fn a_stored_look_nothing_has_is_kept_and_shown_as_unavailable() {
    let sandbox = Sandbox::new("extensions-stale-look");
    let stored = "version = 1\n[appearance]\nlook = \"org.example.Gone\"\n";
    sandbox.write("quire/appearance.toml", stored);
    let mut harness = sandbox.harness(opening("appearance"));
    assert_eq!(
        harness
            .text_of(&format!("{LOOK_MENU} .ds-button-label"))
            .as_deref(),
        Some("org.example.Gone"),
        "the menu shows what is stored, not Standard"
    );
    let items = look_menu(&mut harness);
    assert_eq!(items.first().map(String::as_str), Some("Standard"));
    assert!(items.contains(&"org.example.Gone".to_owned()), "{items:?}");
    click(&mut harness, LOOK_MENU);
    assert!(harness.html().contains("Not installed, or turned off"));
    click(&mut harness, LOOK_MENU);
    assert_eq!(
        sandbox.read("quire/appearance.toml").as_deref(),
        Some(stored),
        "looking wrote nothing"
    );
}

#[test]
fn an_extension_target_opens_the_extensions_page_with_its_row_marked() {
    let sandbox = Sandbox::new("extensions-target");
    install(&sandbox);
    install_look(
        &sandbox.data_home().0,
        "org.example.Slate",
        "Slate",
        "Cool slate",
    );
    let (mut harness, outbox) = sandbox.harness_told(opening("appearance"));
    let target: SettingsTarget = format!("extension:{PAPER}").parse().expect("a target");
    outbox
        .send(Notice::OpenTarget(target))
        .expect("the window is listening");
    harness.advance(Duration::from_millis(400));
    assert_eq!(page_of(&harness).as_deref(), Some("extensions"));
    assert_eq!(harness.count(".key-hit"), 1, "{}", harness.html());
    assert_eq!(harness.count(&format!("{ROW}.key-hit")), 1);
}

#[test]
fn an_installed_file_icons_set_is_in_the_file_icons_menu_after_standard() {
    let sandbox = Sandbox::new("extensions-file-icons");
    install_file_icons(
        &sandbox.data_home().0,
        "org.example.Dots",
        "Dots",
        "Round dots",
    );
    let mut harness = sandbox.harness(opening("appearance"));
    assert_eq!(menu_items(&mut harness, ICONS_MENU), ["Standard", "Dots"]);
    drop(harness);

    let harness = sandbox.harness(opening("extensions"));
    let text = harness.text_of(".extensions").unwrap_or_default();
    assert!(
        text.contains("File Icons") && text.contains("Dots"),
        "{text}"
    );
}

#[test]
fn your_changes_is_not_in_the_menu_and_has_a_switch_only_while_it_holds_nothing() {
    for (changes, switches) in [(Changes::None, 1), (Changes::Held, 0)] {
        let sandbox = Sandbox::new("extensions-your-changes");
        install_your_changes(&sandbox.data_home().0, changes);
        let mut harness = sandbox.harness(opening("appearance"));
        assert_eq!(menu_items(&mut harness, ICONS_MENU), ["Standard"]);
        drop(harness);

        let harness = sandbox.harness(opening("extensions"));
        assert_eq!(harness.count(YOUR_CHANGES), 1, "the page lists it");
        assert_eq!(
            harness.count(&format!("{YOUR_CHANGES} .ds-toggle")),
            switches
        );
    }
}
