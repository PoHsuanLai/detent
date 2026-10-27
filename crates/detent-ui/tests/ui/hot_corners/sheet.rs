//! The Hot Corners button, the corner cells and what each pick writes.

use super::*;

#[test]
fn the_dock_page_ends_in_a_button_and_the_corners_have_no_rows_of_their_own() {
    let sandbox = world("hot-corners-button", &Fake::default());
    let harness = sandbox.harness(opening());
    assert_eq!(harness.count("#hot-corners-open"), 1);
    assert_eq!(
        harness.count(".hot-corners-screen"),
        0,
        "the sheet is closed"
    );
    let html = harness.html();
    assert!(html.contains("Hot Corners\u{2026}"), "{html}");
    assert!(
        !html.contains("hot_corners.top_left"),
        "neither the corners nor their old command keys have a row:\n{html}"
    );
}

#[test]
fn each_pick_writes_its_corner_in_the_new_form() {
    let sandbox = world("hot-corners-picks", &Fake::default());
    let mut harness = opened(&sandbox);
    assert_eq!(harness.count(".hot-corners-cell"), 4);
    assert_eq!(sandbox.read(FILE), None, "looking writes nothing");

    // (corner, the line picked, what the corner stores, what its pop-up shows after)
    let cases: &[(&str, &str, &str, &str)] = &[
        (
            "top_left",
            "Mission Control",
            "v = [{ action = \"workspaces\" }]",
            "Mission Control",
        ),
        (
            "top_right",
            "Lock Screen",
            "v = [{ action = \"lock\" }]",
            "Lock Screen",
        ),
        (
            "bottom_left",
            "Notification Centre",
            "v = [{ action = \"notification_center\" }]",
            "Notification Centre",
        ),
        (
            "bottom_right",
            "Assistant",
            "v = [{ action = \"assistant\" }]",
            "Assistant",
        ),
        ("top_left", "\u{2013}", "v = []", "\u{2013}"),
    ];
    for (corner, line, stores, shown) in cases {
        pick(&mut harness, corner, 1, line);
        assert_eq!(
            stored(&sandbox, corner),
            Some(value(stores)),
            "{corner}: {line}"
        );
        assert_eq!(
            shows(&harness, corner, 1).as_deref(),
            Some(*shown),
            "{corner}"
        );
    }
    let text = sandbox.read(FILE).expect("the file was written");
    assert!(
        !text.contains("_command"),
        "the old command keys are not written back:\n{text}"
    );
}

#[test]
fn a_file_written_before_the_lists_shows_as_it_was_and_is_rewritten_only_where_changed() {
    let sandbox = world("hot-corners-legacy", &Fake::default());
    let before = "[hot_corners]\ntop_left = \"lock\"\ntop_right = \"command\"\ntop_right_command = \"notify-send hi\"\nbottom_left = \"none\"\n";
    sandbox.write(FILE, before);
    let mut harness = opened(&sandbox);
    // (corner, what its pop-up shows)
    let cases = [
        ("top_left", "Lock Screen"),
        ("top_right", "Command: notify-send hi"),
        ("bottom_left", "\u{2013}"),
        ("bottom_right", "\u{2013}"),
    ];
    for (corner, shown) in cases {
        assert_eq!(
            shows(&harness, corner, 1).as_deref(),
            Some(shown),
            "{corner}"
        );
    }
    assert_eq!(
        sandbox.read(FILE).as_deref(),
        Some(before),
        "showing writes nothing"
    );

    pick(&mut harness, "bottom_left", 1, "Launcher");
    let text = sandbox.read(FILE).expect("the file");
    assert_eq!(
        stored(&sandbox, "bottom_left"),
        Some(value("v = [{ action = \"launcher\" }]")),
        "{text}"
    );
    assert!(
        text.contains("top_left = \"lock\"") && text.contains("top_right = \"command\""),
        "the corners that were not changed keep what they held:\n{text}"
    );
}
