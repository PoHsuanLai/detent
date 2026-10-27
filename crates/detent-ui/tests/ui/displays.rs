//! The Displays page over a compositor that answers from memory: each display's resolution
//! ("Default for display" and the scaled sizes), refresh rate, rotation and mode size, and where displays sit
//! beside each other; each change goes to the compositor as one change, and a refusal is told.

use crate::support::Sandbox;
use crate::support::displays::Displays;
use crate::support::drive::{click, page_of, settle, showing, sidebar_page, title_of};
use detent_model::{
    Absence, DisplayChange, DisplayEdit, DisplayMode, DisplayPosition, DisplayRotation,
    DisplaysStatus, Millimetres, ModePreference, Monitor, OutputId, PixelSize, RefreshMilliHz,
    ScaleMilli,
};
use detent_ui::Opening;
use ds_harness::{Harness, Input};

const PAGE: &str = "[data-system=\"displays\"]";

fn displays() -> Opening {
    Opening {
        page: Some(detent_model::PageSlug("displays".to_owned())),
        query: String::new(),
    }
}

fn mode(width: u32, height: u32, hertz: u32, preferred: ModePreference) -> DisplayMode {
    DisplayMode {
        size: PixelSize { width, height },
        refresh: RefreshMilliHz(hertz * 1000),
        preferred,
    }
}

fn id(name: &str) -> OutputId {
    OutputId(name.to_owned())
}

/// A 14 inch panel: 3024 by 1964 at 120 or 60 Hertz, looking like 1512 by 982.
fn laptop() -> Monitor {
    let native = mode(3024, 1964, 120, ModePreference::Yes);
    Monitor {
        id: id("eDP-1"),
        name: "Studio Panel".to_owned(),
        modes: vec![native, mode(3024, 1964, 60, ModePreference::No)],
        current: Some(native),
        scale: ScaleMilli(2000),
        rotation: DisplayRotation::Standard,
        position: DisplayPosition { x: 0, y: 0 },
        size_mm: Some(Millimetres {
            width: 302,
            height: 189,
        }),
    }
}

/// A 27 inch 4K monitor at 60 Hertz, looking like 2560 by 1440, to the right of the panel.
fn monitor() -> Monitor {
    let native = mode(3840, 2160, 60, ModePreference::Yes);
    Monitor {
        id: id("DP-1"),
        name: "Desk Monitor".to_owned(),
        modes: vec![native],
        current: Some(native),
        scale: ScaleMilli(1500),
        rotation: DisplayRotation::Standard,
        position: DisplayPosition { x: 1512, y: 0 },
        size_mm: Some(Millimetres {
            width: 597,
            height: 336,
        }),
    }
}

fn world(name: &str, service: &Displays) -> (Sandbox, Harness) {
    let sandbox = Sandbox::new(name).with_displays(service.clone());
    let harness = sandbox.harness(displays());
    (sandbox, harness)
}

fn row(display: &str, nth: usize) -> String {
    format!("[data-display=\"{display}\"] .ds-field-row:nth-child({nth})")
}

fn button(display: &str, nth: usize) -> String {
    format!("{} .ds-popup .ds-button", row(display, nth))
}

/// Opens the pop-up of a row and picks its `item`th entry.
fn pick(harness: &mut Harness, popup: &str, item: usize) {
    click(harness, popup);
    let entry = format!(".ds-menu-item:nth-child({item}) .ds-menu-label");
    let at = harness.centre(&entry).expect("the menu is open");
    harness.send(Input::pointer_move(at));
    click(harness, &entry);
}

fn one() -> DisplaysStatus {
    DisplaysStatus {
        displays: vec![laptop()],
    }
}

fn two() -> DisplaysStatus {
    DisplaysStatus {
        displays: vec![laptop(), monitor()],
    }
}

#[test]
fn the_page_sits_after_desktop_and_dock_and_says_plainly_when_there_is_no_compositor() {
    let sandbox = Sandbox::new("displays-absent");
    let mut harness = sandbox.harness(displays());
    showing(
        &mut harness,
        "the unavailable row",
        "[data-system=\"unavailable\"]",
    );
    assert_eq!(page_of(&harness).as_deref(), Some("displays"));
    assert_eq!(title_of(&harness).as_deref(), Some("Displays"));
    assert_eq!(
        harness.text_of(&sidebar_page("dock")).as_deref(),
        Some("Desktop & Dock")
    );
    assert_eq!(
        harness.text_of(&sidebar_page("displays")).as_deref(),
        Some("Displays")
    );
    let row = harness.text_of(".ds-field-row").unwrap_or_default();
    assert!(
        row.contains("Displays aren\u{2019}t available right now."),
        "{row}"
    );
}

#[test]
fn a_computer_with_no_display_connected_says_so() {
    let service = Displays::missing(Absence::NoDevice);
    let (_sandbox, mut harness) = world("displays-none", &service);
    showing(
        &mut harness,
        "the unavailable row",
        "[data-system=\"unavailable\"]",
    );
    let row = harness.text_of(".ds-field-row").unwrap_or_default();
    assert!(row.contains("No display is connected."), "{row}");
}

#[test]
fn a_display_shows_its_name_and_its_resolution_refresh_rate_and_rotation() {
    let service = Displays::answering(one());
    let (_sandbox, mut harness) = world("displays-one", &service);
    showing(&mut harness, "the page", PAGE);
    showing(&mut harness, "the display", "[data-display=\"eDP-1\"]");
    assert!(harness.html().contains("Studio Panel"));
    let resolution = harness.text_of(&row("eDP-1", 1)).unwrap_or_default();
    assert!(resolution.contains("Resolution"), "{resolution}");
    assert!(
        resolution.contains("Default for display (1512 \u{d7} 982)"),
        "the default is the scale that suits the panel's density: {resolution}"
    );
    assert!(
        resolution.contains("Looks like 1512 \u{d7} 982"),
        "{resolution}"
    );
    let refresh = harness.text_of(&row("eDP-1", 2)).unwrap_or_default();
    assert!(
        refresh.contains("Refresh Rate") && refresh.contains("120 Hertz"),
        "{refresh}"
    );
    let rotation = harness.text_of(&row("eDP-1", 3)).unwrap_or_default();
    assert!(
        rotation.contains("Rotation") && rotation.contains("Standard"),
        "{rotation}"
    );
    assert_eq!(
        harness.count("[data-display=\"eDP-1\"] .ds-field-row"),
        4,
        "resolution, refresh rate, rotation and mode size; no Position row with one display"
    );
}

#[test]
fn choosing_a_scaled_resolution_sends_that_scale_and_the_row_follows() {
    let service = Displays::answering(one());
    let (_sandbox, mut harness) = world("displays-resolution", &service);
    showing(&mut harness, "the display", "[data-display=\"eDP-1\"]");
    // Widest first: native, 2419, 2016, 1728, 1512 (the default), 1209, 1008.
    pick(&mut harness, &button("eDP-1", 1), 3);
    assert_eq!(
        service.asked(),
        [DisplayChange {
            output: id("eDP-1"),
            edit: DisplayEdit::Scale(ScaleMilli(1500)),
        }]
    );
    settle(&mut harness, "the new resolution", |h| {
        h.text_of(&row("eDP-1", 1))
            .unwrap_or_default()
            .contains("Looks like 2016 \u{d7} 1309")
    });
}

#[test]
fn choosing_a_refresh_rate_sends_it_for_the_size_the_display_runs_at() {
    let service = Displays::answering(one());
    let (_sandbox, mut harness) = world("displays-refresh", &service);
    showing(&mut harness, "the display", "[data-display=\"eDP-1\"]");
    pick(&mut harness, &button("eDP-1", 2), 2);
    assert_eq!(
        service.asked(),
        [DisplayChange {
            output: id("eDP-1"),
            edit: DisplayEdit::Refresh(RefreshMilliHz(60_000)),
        }]
    );
    settle(&mut harness, "the new rate", |h| {
        h.text_of(&row("eDP-1", 2))
            .unwrap_or_default()
            .contains("60 Hertz")
    });
}

/// A monitor that also lists a smaller size, with nothing beside it so its rows keep their places.
fn flexible() -> DisplaysStatus {
    let native = mode(3840, 2160, 60, ModePreference::Yes);
    DisplaysStatus {
        displays: vec![Monitor {
            modes: vec![native, mode(1920, 1080, 60, ModePreference::No)],
            ..monitor()
        }],
    }
}

#[test]
fn choosing_another_mode_size_sends_it_and_a_display_with_one_size_has_nothing_to_choose() {
    let service = Displays::answering(flexible());
    let (_sandbox, mut harness) = world("displays-mode-size", &service);
    showing(&mut harness, "the display", "[data-display=\"DP-1\"]");
    let words = harness.text_of(&row("DP-1", 4)).unwrap_or_default();
    assert!(
        words.contains("Display Mode") && words.contains("3840 \u{d7} 2160 (Native)"),
        "{words}"
    );
    pick(&mut harness, &button("DP-1", 4), 2);
    assert_eq!(
        service.asked(),
        [DisplayChange {
            output: id("DP-1"),
            edit: DisplayEdit::Size(PixelSize {
                width: 1920,
                height: 1080
            }),
        }]
    );
    settle(&mut harness, "the new size", |h| {
        h.text_of(&row("DP-1", 4))
            .unwrap_or_default()
            .contains("1920 \u{d7} 1080")
    });
    let single = Displays::answering(one());
    let (_other, mut alone) = world("displays-one-size", &single);
    showing(&mut alone, "the display", "[data-display=\"eDP-1\"]");
    assert_eq!(
        alone.attr(&row("eDP-1", 4), "aria-disabled").as_deref(),
        Some("true")
    );
}

#[test]
fn a_display_with_one_refresh_rate_has_nothing_to_choose() {
    let service = Displays::answering(two());
    let (_sandbox, mut harness) = world("displays-one-rate", &service);
    showing(&mut harness, "the display", "[data-display=\"DP-1\"]");
    assert_eq!(
        harness.attr(&row("DP-1", 2), "aria-disabled").as_deref(),
        Some("true")
    );
}

#[test]
fn rotating_a_display_sends_the_rotation() {
    let service = Displays::answering(one());
    let (_sandbox, mut harness) = world("displays-rotate", &service);
    showing(&mut harness, "the display", "[data-display=\"eDP-1\"]");
    pick(&mut harness, &button("eDP-1", 3), 2);
    assert_eq!(
        service.asked(),
        [DisplayChange {
            output: id("eDP-1"),
            edit: DisplayEdit::Rotate(DisplayRotation::Quarter),
        }]
    );
    settle(&mut harness, "the new rotation", |h| {
        h.text_of(&row("eDP-1", 3))
            .unwrap_or_default()
            .contains("90\u{b0}")
    });
}

#[test]
fn with_two_displays_each_says_where_it_sits_and_can_be_moved_to_another_side() {
    let service = Displays::answering(two());
    let (_sandbox, mut harness) = world("displays-arrange", &service);
    showing(
        &mut harness,
        "the second display",
        "[data-display=\"DP-1\"]",
    );
    let place = harness.text_of(&row("DP-1", 4)).unwrap_or_default();
    assert!(
        place.contains("Position") && place.contains("Right of Studio Panel"),
        "{place}"
    );

    // Right, Left, Above, Below: the second entry puts it on the panel's left, edge to edge.
    pick(&mut harness, &button("DP-1", 4), 2);
    assert_eq!(
        service.asked(),
        [DisplayChange {
            output: id("DP-1"),
            edit: DisplayEdit::Move(DisplayPosition { x: -2560, y: 0 }),
        }]
    );
    settle(&mut harness, "the new place", |h| {
        h.text_of(&row("DP-1", 4))
            .unwrap_or_default()
            .contains("Left of Studio Panel")
    });
}

#[test]
fn a_change_the_compositor_refuses_is_told_in_a_toast() {
    let service = Displays::answering(one());
    service.refuses("This display can\u{2019}t be set that way.");
    let (_sandbox, mut harness) = world("displays-refused", &service);
    showing(&mut harness, "the display", "[data-display=\"eDP-1\"]");
    pick(&mut harness, &button("eDP-1", 3), 3);
    settle(&mut harness, "the toast", |h| h.count(".ds-toast") == 1);
    let toast = harness.text_of(".ds-toast").unwrap_or_default();
    assert!(
        toast.contains("This display can\u{2019}t be set that way."),
        "{toast}"
    );
    assert!(
        harness
            .text_of(&row("eDP-1", 3))
            .unwrap_or_default()
            .contains("Standard"),
        "nothing changed"
    );
}
