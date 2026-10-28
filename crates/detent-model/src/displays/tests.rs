use super::*;
use crate::Absence;

fn mode(width: u32, height: u32, hz: u32, preferred: bool) -> DisplayMode {
    DisplayMode {
        size: PixelSize { width, height },
        refresh: RefreshMilliHz(hz * 1000),
        preferred,
    }
}

fn display(name: &str, x: i32, y: i32, size_mm: Option<(u32, u32)>) -> Monitor {
    let native = mode(3024, 1964, 120, true);
    Monitor {
        id: OutputId(name.to_owned()),
        name: format!("{name} panel"),
        modes: vec![native, mode(3024, 1964, 60, false)],
        current: Some(native),
        scale: ScaleMilli(2000),
        rotation: DisplayRotation::Standard,
        position: DisplayPosition { x, y },
        size_mm: size_mm.map(|(width, height)| Millimetres { width, height }),
    }
}

fn laptop() -> Monitor {
    display("eDP-1", 0, 0, Some((302, 189)))
}

fn status(displays: Vec<Monitor>) -> DisplaysStatus {
    DisplaysStatus { displays }
}

fn ready(status: DisplaysStatus) -> DisplaysState {
    DisplaysState::Ready(DisplaysReady {
        status,
        applying: None,
    })
}

fn id(name: &str) -> OutputId {
    OutputId(name.to_owned())
}

#[test]
fn the_default_scale_follows_the_density_the_display_reports() {
    let cases: [(&str, Monitor, u32); 3] = [
        ("a dense laptop panel", laptop(), 2000),
        (
            "a 27 inch 4K monitor",
            Monitor {
                current: Some(mode(3840, 2160, 60, true)),
                ..display("DP-1", 0, 0, Some((597, 336)))
            },
            1500,
        ),
        (
            "a display that does not say",
            display("DP-2", 0, 0, None),
            1000,
        ),
    ];
    for (case, display, want) in cases {
        assert_eq!(default_scale(&display), ScaleMilli(want), "{case}");
    }
}

#[test]
fn resolutions_are_the_mode_over_each_scale_widest_first_and_include_the_default() {
    let looks = looks_like(&laptop());
    let sizes: Vec<(u32, u32)> = looks
        .iter()
        .map(|l| (l.size.width, l.size.height))
        .collect();
    assert_eq!(
        sizes,
        [
            (3024, 1964),
            (2419, 1571),
            (2016, 1309),
            (1728, 1122),
            (1512, 982),
            (1209, 785),
            (1008, 654)
        ]
    );
    assert!(looks.iter().any(|l| l.scale == default_scale(&laptop())));
}

#[test]
fn a_narrow_display_is_not_offered_what_cannot_fit() {
    let small = Monitor {
        current: Some(mode(1280, 800, 60, true)),
        ..display("DSI-1", 0, 0, Some((200, 125)))
    };
    let widths: Vec<u32> = looks_like(&small).iter().map(|l| l.size.width).collect();
    assert!(
        widths
            .iter()
            .all(|w| *w >= 640 || *w == 1280 * 1000 / default_scale(&small).0),
        "{widths:?}"
    );
}

#[test]
fn the_logical_size_is_the_mode_over_the_scale_and_turns_with_the_display() {
    let mut turned = laptop();
    assert_eq!(
        turned.logical_size(),
        Some(PixelSize {
            width: 1512,
            height: 982
        })
    );
    turned.rotation = DisplayRotation::Quarter;
    assert_eq!(
        turned.logical_size(),
        Some(PixelSize {
            width: 982,
            height: 1512
        })
    );
}

#[test]
fn a_display_goes_beside_another_edge_to_edge_and_the_side_reads_back() {
    let main = laptop();
    let other = display("DP-1", 0, 0, Some((597, 336)));
    let cases = [
        (ArrangeSide::Right, DisplayPosition { x: 1512, y: 0 }),
        (ArrangeSide::Left, DisplayPosition { x: -1512, y: 0 }),
        (ArrangeSide::Above, DisplayPosition { x: 0, y: -982 }),
        (ArrangeSide::Below, DisplayPosition { x: 0, y: 982 }),
    ];
    for (side, want) in cases {
        assert_eq!(place_beside(&other, &main, side), Some(want), "{side:?}");
        let placed = Monitor {
            position: want,
            ..other.clone()
        };
        assert_eq!(side_of(&placed, &main), Some(side), "{side:?}");
    }
    let apart = Monitor {
        position: DisplayPosition { x: 40, y: 7 },
        ..other
    };
    assert_eq!(side_of(&apart, &main), None);
}

#[test]
fn a_change_is_sent_once_and_the_next_waits_for_it() {
    let one = DisplayChange {
        output: id("eDP-1"),
        edit: DisplayEdit::Scale(ScaleMilli(1500)),
    };
    let (busy, effects) = displays_step(
        ready(status(vec![laptop()])),
        DisplaysInput::Change(one.clone()),
    );
    assert_eq!(effects, [DisplaysEffect::Apply(one)]);
    let (_, again) = displays_step(
        busy.clone(),
        DisplaysInput::Change(DisplayChange {
            output: id("eDP-1"),
            edit: DisplayEdit::Rotate(DisplayRotation::Half),
        }),
    );
    assert_eq!(again, []);
    let (done, _) = displays_step(busy, DisplaysInput::Applied(id("eDP-1"), Ok(())));
    assert_eq!(done, ready(status(vec![laptop()])));
}

#[test]
fn a_change_to_a_display_that_is_not_there_is_not_sent() {
    let (_, effects) = displays_step(
        ready(status(vec![laptop()])),
        DisplaysInput::Change(DisplayChange {
            output: id("DP-9"),
            edit: DisplayEdit::Rotate(DisplayRotation::Half),
        }),
    );
    assert_eq!(effects, []);
}

#[test]
fn arranging_sends_the_place_next_to_the_other_display() {
    let world = status(vec![laptop(), display("DP-1", 5000, 0, Some((597, 336)))]);
    let (_, effects) = displays_step(
        ready(world),
        DisplaysInput::Arrange(id("DP-1"), id("eDP-1"), ArrangeSide::Left),
    );
    assert_eq!(
        effects,
        [DisplaysEffect::Apply(DisplayChange {
            output: id("DP-1"),
            edit: DisplayEdit::Move(DisplayPosition { x: -1512, y: 0 }),
        })]
    );
}

#[test]
fn a_refused_change_is_said_and_frees_the_page() {
    let (busy, _) = displays_step(
        ready(status(vec![laptop()])),
        DisplaysInput::Change(DisplayChange {
            output: id("eDP-1"),
            edit: DisplayEdit::Rotate(DisplayRotation::Half),
        }),
    );
    let (free, effects) = displays_step(
        busy,
        DisplaysInput::Applied(
            id("eDP-1"),
            Err("That setting isn\u{2019}t supported.".to_owned()),
        ),
    );
    assert_eq!(
        effects,
        [DisplaysEffect::Tell(
            "That setting isn\u{2019}t supported.".to_owned()
        )]
    );
    assert!(matches!(
        free,
        DisplaysState::Ready(DisplaysReady { applying: None, .. })
    ));
}

#[test]
fn a_compositor_that_goes_leaves_nothing() {
    let (gone, _) = displays_step(
        ready(status(vec![laptop()])),
        DisplaysInput::Heard(Err(Absence::NotRunning)),
    );
    assert_eq!(gone, DisplaysState::Unavailable(Absence::NotRunning));
}

#[test]
fn the_sizes_a_display_lists_are_distinct_and_widest_first_with_its_native_one_known() {
    let native = mode(3840, 2160, 60, true);
    let modes = [
        mode(1920, 1080, 60, false),
        native,
        mode(1920, 1080, 120, false),
        mode(2560, 1440, 60, false),
    ];
    let sizes: Vec<(u32, u32)> = mode_sizes(&modes)
        .iter()
        .map(|size| (size.width, size.height))
        .collect();
    assert_eq!(sizes, [(3840, 2160), (2560, 1440), (1920, 1080)]);
    assert_eq!(native_size(&modes), Some(native.size));
    assert_eq!(native_size(&modes[..1]), None);
}

#[test]
fn a_size_runs_at_the_refresh_rate_the_display_runs_now_else_at_its_fastest() {
    let modes = [
        mode(3840, 2160, 60, true),
        mode(1920, 1080, 60, false),
        mode(1920, 1080, 120, false),
        mode(1280, 720, 30, false),
        mode(1280, 720, 50, false),
    ];
    let size = |width, height| PixelSize { width, height };
    let now = Some(modes[0]);
    assert_eq!(mode_at(&modes, now, size(1920, 1080)), Some(modes[1]));
    assert_eq!(mode_at(&modes, now, size(1280, 720)), Some(modes[4]));
    assert_eq!(mode_at(&modes, None, size(1920, 1080)), Some(modes[2]));
    assert_eq!(mode_at(&modes, now, size(800, 600)), None);
}
