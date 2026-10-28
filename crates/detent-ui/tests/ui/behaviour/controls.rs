//! What the controls write: switches, segments, swatches, sliders and failed writes.

use super::*;

#[test]
fn clicking_a_page_shows_it() {
    let sandbox = Sandbox::new("click-a-page");
    let mut harness = sandbox.harness(Opening::default());
    assert_eq!(page_of(&harness).as_deref(), Some("appearance"));
    assert_eq!(title_of(&harness).as_deref(), Some("Appearance"));

    click(&mut harness, &sidebar_page("dock"));
    assert_eq!(page_of(&harness).as_deref(), Some("dock"));
    assert_eq!(title_of(&harness).as_deref(), Some("Desktop & Dock"));
    assert_eq!(
        harness
            .attr(&sidebar_page("dock"), "aria-selected")
            .as_deref(),
        Some("true"),
        "the clicked row is the selected one"
    );
    assert_eq!(
        harness
            .attr(&sidebar_page("appearance"), "aria-selected")
            .as_deref(),
        Some("false")
    );
}

#[test]
fn toggling_a_switch_writes_the_file_and_the_value_round_trips() {
    let sandbox = Sandbox::new("toggle-writes");
    let mut harness = sandbox.harness(opening("dock"));
    assert_eq!(
        sandbox.read("sill/settings.toml"),
        None,
        "nothing is written by looking"
    );
    assert_eq!(
        harness.attr(".ds-toggle", "aria-checked").as_deref(),
        Some("true"),
        "magnification starts on, its default"
    );

    click(&mut harness, ".ds-toggle");
    assert_eq!(
        harness.attr(".ds-toggle", "aria-checked").as_deref(),
        Some("false"),
        "the switch turned off"
    );
    let text = sandbox
        .read("sill/settings.toml")
        .expect("the file was written");
    assert!(text.contains("magnification = \"off\""), "{text}");

    // Read back the way the owning program reads it, and by a second run of the app.
    let spec = harness_spec(&sandbox);
    let loaded = load(&sandbox.config_home(), &spec);
    let key = detent_model_key("dock.magnification");
    assert_eq!(
        loaded.value.get(&key),
        Some(&toml::Value::String("off".to_owned()))
    );
    let again = sandbox.harness(opening("dock"));
    assert_eq!(
        again.attr(".ds-toggle", "aria-checked").as_deref(),
        Some("false")
    );
}

#[test]
fn the_theme_is_three_picture_choices_and_a_press_writes_its_word() {
    const GROUP: &str = "[*|data-key=\"appearance.theme\"]";
    let sandbox = Sandbox::new("theme-pictures");
    let mut harness = sandbox.harness(opening("appearance"));
    assert_eq!(sandbox.read("quire/appearance.toml"), None);
    assert_eq!(harness.count(&format!("{GROUP} .ds-radio-group-item")), 3);
    assert_eq!(
        harness.count(&format!("{GROUP} .ds-radio-group-image .ds-ext-icon")),
        3,
        "each choice has its picture"
    );
    assert_eq!(
        harness.attr(GROUP, "data-arrangement").as_deref(),
        Some("row")
    );
    assert_eq!(
        harness.attr(&theme_choice(3), "aria-checked").as_deref(),
        Some("true"),
        "the default, the word that follows the desktop, starts checked"
    );

    click(&mut harness, &theme_choice(2));
    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains("theme = \"dark\""), "{text}");
    assert_eq!(
        harness.attr(&theme_choice(2), "aria-checked").as_deref(),
        Some("true"),
        "Dark is selected"
    );
    click(&mut harness, &theme_choice(1));
    let text = sandbox.read("quire/appearance.toml").expect("written");
    assert!(text.contains("theme = \"light\""), "{text}");
}

#[test]
fn the_dock_page_opens_with_a_preview_at_the_stored_tile_size() {
    const PREVIEW: &str = "[*|data-dock-preview]";
    let sandbox = Sandbox::new("dock-preview");
    sandbox.write("sill/settings.toml", "[dock]\ntile_size_px = 64\n");
    let harness = sandbox.harness(opening("dock"));
    assert_eq!(harness.count(PREVIEW), 1);
    assert_eq!(
        harness.attr(PREVIEW, "data-dock-preview").as_deref(),
        Some("64")
    );
    assert_eq!(harness.count("[*|data-dock-tile]"), 5);
    assert!(
        harness
            .attr(".dock-preview-pill", "style")
            .is_some_and(|style| style.contains("--row-icon-tile:64px")),
        "the tiles are drawn at the stored size"
    );

    let sandbox = Sandbox::new("dock-preview-default");
    let harness = sandbox.harness(opening("dock"));
    assert_eq!(
        harness.attr(PREVIEW, "data-dock-preview").as_deref(),
        Some("48"),
        "with nothing stored, the key's default"
    );
}

#[test]
fn choosing_an_accent_swatch_writes_that_accent() {
    const SWATCH_THREE: &str = "[*|data-arrangement=swatches] .ds-radio-group-item:nth-of-type(3)";
    let sandbox = Sandbox::new("swatch-writes");
    let mut harness = sandbox.harness(opening("appearance"));
    click(&mut harness, SWATCH_THREE);
    let want = ds::prelude::Accent::ALL[2].slug();
    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains(&format!("accent = \"{want}\"")), "{text}");
    assert_eq!(
        harness.attr(SWATCH_THREE, "aria-checked").as_deref(),
        Some("true")
    );
}

#[test]
fn moving_a_slider_writes_a_number_in_its_range() {
    // The Size slider; the first slider in the document is inside a closed disclosure.
    const SIZE: &str = ".ds-slider[*|aria-label=Size]";
    let sandbox = Sandbox::new("slider-writes");
    let mut harness = sandbox.harness(opening("dock"));
    let track = harness.rect(SIZE).expect("a slider on the Dock page");
    let at = Point {
        x: Px(track.origin.x.0 + track.size.width.0 * 0.9),
        y: Px(track.origin.y.0 + track.size.height.0 / 2.0),
    };
    harness.send(Input::pointer_down(at));
    harness.advance(Duration::from_millis(200));
    harness.send(Input::pointer_up(at));
    harness.advance(Duration::from_millis(400));
    let text = sandbox
        .read("sill/settings.toml")
        .expect("the file was written");
    let table: toml::Table = text.parse().expect("valid toml");
    let size = table["dock"]["tile_size_px"]
        .as_integer()
        .expect("an integer");
    assert!(
        (74..=76).contains(&size),
        "a press 90% along 32..=80 stores about 75, not the default 48: {size}\n{text}"
    );
}

#[test]
fn a_write_that_fails_says_so_and_leaves_the_file_alone() {
    let sandbox = Sandbox::new("write-refused");
    sandbox.write("sill/settings.toml", "[dock\nmagnification = ");
    let mut harness = sandbox.harness(opening("dock"));
    click(&mut harness, ".ds-toggle");
    assert_eq!(harness.count(".ds-toast"), 1, "a toast reports the refusal");
    assert_eq!(
        sandbox.read("sill/settings.toml").as_deref(),
        Some("[dock\nmagnification = "),
        "what the person typed is kept"
    );
}

#[test]
fn reduce_motion_is_a_switch_that_is_on_for_reduced() {
    let sandbox = Sandbox::new("reduce-motion");
    let mut harness = sandbox.harness(opening("appearance"));
    let switch = ".section .ds-toggle[*|aria-label=\"Reduce motion\"]";
    assert_eq!(
        harness.count(switch),
        1,
        "a two-word key that is on and off is one switch"
    );
    assert_eq!(
        harness.attr(switch, "aria-checked").as_deref(),
        Some("false"),
        "Standard is off"
    );
    click(&mut harness, switch);
    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains("motion_level = \"reduced\""), "{text}");
    assert_eq!(
        harness.attr(switch, "aria-checked").as_deref(),
        Some("true")
    );
}

#[test]
fn a_two_word_choice_that_is_not_on_and_off_is_two_segments() {
    let sandbox = Sandbox::new("two-segments");
    let mut harness = sandbox.harness(opening("intelligence"));
    let start = ".ds-segmented[*|aria-label=\"New conversations start in\"]";
    assert_eq!(
        harness.count(&format!("{start} .ds-segmented-segment")),
        2,
        "Auto and Manual"
    );
    click(
        &mut harness,
        &format!("{start} .ds-segmented-segment:nth-of-type(2)"),
    );
    let text = sandbox
        .read("docket/settings.toml")
        .expect("the file was written");
    assert!(text.contains("start_mode = \"manual\""), "{text}");
}
