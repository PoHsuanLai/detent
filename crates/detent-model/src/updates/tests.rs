//! The Software Update page: the service's words parsed, what a press may ask, and what each
//! standing shows.

use super::words::{age_words, size_words};
use super::{
    ButtonActivity, ByteCount, NotesSheet, NotesState, Permille, UnixSeconds, UpdateButton,
    UpdateMeter, UpdateOffer, UpdatePhase, UpdateStatus, UpdateWire, UpdatesEffect, UpdatesInput,
    UpdatesReady, UpdatesShown, UpdatesState, is_software_update, software_update_page,
    updates_shown, updates_step,
};
use crate::catalogue::{Catalogue, PageModel};
use crate::layout::{Block, blocks};
use crate::page::PageSlug;
use crate::picker::Absence;
use ds_settings::schema::Page;

const NOW: UnixSeconds = UnixSeconds(1_000_000);

fn offer() -> UpdateOffer {
    UpdateOffer {
        version: "0.4.0".to_owned(),
        notes: "Faster start.".to_owned(),
        size: ByteCount(45_600_000),
    }
}

fn status(phase: UpdatePhase) -> UpdateStatus {
    UpdateStatus {
        phase,
        current: Some("0.3.1".to_owned()),
        offer: None,
        progress: Permille(0),
        last_checked: Some(UnixSeconds(NOW.0 - 300)),
        problem: None,
    }
}

fn offered(phase: UpdatePhase) -> UpdateStatus {
    UpdateStatus {
        offer: Some(offer()),
        ..status(phase)
    }
}

fn ready(status: UpdateStatus) -> UpdatesState {
    UpdatesState::Ready(UpdatesReady {
        status,
        info: NotesState::Closed,
    })
}

fn page_of(state: &UpdatesState) -> super::UpdatesPage {
    match updates_shown(state, NOW) {
        UpdatesShown::Page(page) => page,
        other => panic!("not a page: {other:?}"),
    }
}

#[test]
fn sizes_are_told_in_decimal_units_with_one_decimal_until_three_digits() {
    // `(bytes, the words)`.
    const CASES: &[(u64, &str)] = &[
        (0, "0 bytes"),
        (1, "1 byte"),
        (812, "812 bytes"),
        (1_000, "1.0 KB"),
        (45_600_000, "45.6 MB"),
        (345_000_000, "345 MB"),
        (1_240_000_000, "1.2 GB"),
    ];
    for (bytes, words) in CASES {
        assert_eq!(size_words(ByteCount(*bytes)), *words, "{bytes}");
    }
}

#[test]
fn ages_count_back_from_now_in_the_largest_whole_unit() {
    // `(seconds ago, the words)`; a negative age is a clock a little ahead.
    const CASES: &[(i64, &str)] = &[
        (-30, "just now"),
        (0, "just now"),
        (59, "just now"),
        (60, "1 minute ago"),
        (300, "5 minutes ago"),
        (3_600, "1 hour ago"),
        (7_200, "2 hours ago"),
        (86_400, "yesterday"),
        (172_799, "yesterday"),
        (172_800, "2 days ago"),
        (864_000, "10 days ago"),
    ];
    for (ago, words) in CASES {
        let words_now = age_words(NOW, UnixSeconds(NOW.0 - ago));
        assert_eq!(words_now, *words, "{ago} seconds");
    }
}

fn wire(state: &str) -> UpdateWire {
    UpdateWire {
        state: state.to_owned(),
        current: "0.3.1".to_owned(),
        available: "0.4.0".to_owned(),
        notes: "Faster start.".to_owned(),
        size: 45_600_000,
        progress: 500,
        last_checked: 700_000,
        problem: String::new(),
    }
}

#[test]
fn the_services_properties_become_a_typed_status() {
    let parsed = UpdateStatus::of_wire(wire("downloading")).expect("a known state");
    assert_eq!(
        parsed,
        UpdateStatus {
            phase: UpdatePhase::Downloading,
            current: Some("0.3.1".to_owned()),
            offer: Some(offer()),
            progress: Permille(500),
            last_checked: Some(UnixSeconds(700_000)),
            problem: None,
        }
    );
    let bare = UpdateStatus::of_wire(UpdateWire {
        available: String::new(),
        last_checked: 0,
        progress: 4000,
        problem: "No network.".to_owned(),
        ..wire("failed")
    })
    .expect("a known state");
    assert_eq!(bare.offer, None, "no version on offer");
    assert_eq!(bare.last_checked, None, "zero is never");
    assert_eq!(
        bare.progress,
        Permille(1000),
        "a share is at most all of it"
    );
    assert_eq!(bare.problem.as_deref(), Some("No network."));
    assert!(matches!(
        UpdateStatus::of_wire(wire("rebooting")),
        Err(Absence::Failed(_))
    ));
}

#[test]
fn every_word_the_service_uses_names_a_phase() {
    // `(the word, the phase, whether the service is working)`.
    const CASES: &[(&str, UpdatePhase, bool)] = &[
        ("idle", UpdatePhase::Idle, false),
        ("checking", UpdatePhase::Checking, true),
        ("available", UpdatePhase::Available, false),
        ("downloading", UpdatePhase::Downloading, true),
        ("ready", UpdatePhase::Ready, true),
        ("installing", UpdatePhase::Installing, true),
        ("installed", UpdatePhase::Installed, false),
        ("failed", UpdatePhase::Failed, false),
    ];
    for (word, phase, busy) in CASES {
        assert_eq!(UpdatePhase::of_word(word), Some(*phase), "{word}");
        assert_eq!(phase.is_busy(), *busy, "{word}");
    }
}

#[test]
fn a_press_asks_the_service_only_when_the_standing_allows_it() {
    // `(case, the status, the press, what is asked)`.
    let cases: Vec<(&str, UpdateStatus, UpdatesInput, Vec<UpdatesEffect>)> = vec![
        (
            "check when idle",
            status(UpdatePhase::Idle),
            UpdatesInput::Check,
            vec![UpdatesEffect::Check],
        ),
        (
            "check again when an update waits",
            offered(UpdatePhase::Available),
            UpdatesInput::Check,
            vec![UpdatesEffect::Check],
        ),
        (
            "no check while one is running",
            status(UpdatePhase::Checking),
            UpdatesInput::Check,
            vec![],
        ),
        (
            "no check while installing",
            offered(UpdatePhase::Installing),
            UpdatesInput::Check,
            vec![],
        ),
        (
            "update now installs",
            offered(UpdatePhase::Available),
            UpdatesInput::UpdateNow,
            vec![UpdatesEffect::Install],
        ),
        (
            "update now with nothing on offer asks nothing",
            status(UpdatePhase::Available),
            UpdatesInput::UpdateNow,
            vec![],
        ),
        (
            "update now while downloading asks nothing",
            offered(UpdatePhase::Downloading),
            UpdatesInput::UpdateNow,
            vec![],
        ),
        (
            "try again installs what was on offer",
            offered(UpdatePhase::Failed),
            UpdatesInput::TryAgain,
            vec![UpdatesEffect::Install],
        ),
        (
            "try again looks again when nothing was on offer",
            status(UpdatePhase::Failed),
            UpdatesInput::TryAgain,
            vec![UpdatesEffect::Check],
        ),
        (
            "try again when nothing failed asks nothing",
            status(UpdatePhase::Idle),
            UpdatesInput::TryAgain,
            vec![],
        ),
        (
            "log out once installed",
            status(UpdatePhase::Installed),
            UpdatesInput::LogOut,
            vec![UpdatesEffect::LogOut],
        ),
        (
            "no log out before it is installed",
            offered(UpdatePhase::Available),
            UpdatesInput::LogOut,
            vec![],
        ),
        (
            "a refusal is told in the service's words",
            status(UpdatePhase::Idle),
            UpdatesInput::Refused("Another update is in progress.".to_owned()),
            vec![UpdatesEffect::Tell(
                "Another update is in progress.".to_owned(),
            )],
        ),
    ];
    for (case, status, press, want) in cases {
        let (_, effects) = updates_step(ready(status), press);
        assert_eq!(effects, want, "{case}");
    }
}

#[test]
fn a_page_without_a_service_asks_nothing() {
    for state in [
        UpdatesState::Looking,
        UpdatesState::Unavailable(Absence::NotRunning),
    ] {
        for press in [
            UpdatesInput::Check,
            UpdatesInput::UpdateNow,
            UpdatesInput::TryAgain,
            UpdatesInput::LogOut,
        ] {
            let (after, effects) = updates_step(state.clone(), press);
            assert_eq!((after, effects), (state.clone(), vec![]));
        }
    }
}

#[test]
fn what_the_service_says_is_taken_and_a_silent_service_is_unavailable() {
    let (state, _) = updates_step(
        UpdatesState::Looking,
        UpdatesInput::Heard(Ok(status(UpdatePhase::Idle))),
    );
    assert_eq!(state, ready(status(UpdatePhase::Idle)));
    let (state, _) = updates_step(state, UpdatesInput::Heard(Err(Absence::NotRunning)));
    assert_eq!(state, UpdatesState::Unavailable(Absence::NotRunning));
    let (state, _) = updates_step(state, UpdatesInput::Heard(Ok(status(UpdatePhase::Idle))));
    assert_eq!(state, ready(status(UpdatePhase::Idle)), "it came back");
}

#[test]
fn the_notes_open_while_an_update_waits_and_close_when_it_does_not() {
    let (state, effects) = updates_step(
        ready(offered(UpdatePhase::Available)),
        UpdatesInput::MoreInfo,
    );
    assert_eq!(effects, vec![]);
    let UpdatesState::Ready(open) = &state else {
        panic!("not ready: {state:?}");
    };
    assert_eq!(open.info, NotesState::Open);

    // The same update, heard again: the sheet stays.
    let (state, _) = updates_step(
        state,
        UpdatesInput::Heard(Ok(offered(UpdatePhase::Available))),
    );
    assert!(matches!(&state, UpdatesState::Ready(r) if r.info == NotesState::Open));

    // The download starts behind the sheet: it goes.
    let (after, _) = updates_step(
        state.clone(),
        UpdatesInput::Heard(Ok(offered(UpdatePhase::Downloading))),
    );
    assert!(matches!(&after, UpdatesState::Ready(r) if r.info == NotesState::Closed));

    let (closed, _) = updates_step(state, UpdatesInput::CloseInfo);
    assert!(matches!(&closed, UpdatesState::Ready(r) if r.info == NotesState::Closed));

    let (none, _) = updates_step(ready(status(UpdatePhase::Idle)), UpdatesInput::MoreInfo);
    assert!(
        matches!(&none, UpdatesState::Ready(r) if r.info == NotesState::Closed),
        "nothing to read about"
    );
}

#[test]
fn each_standing_shows_its_words_buttons_and_bar() {
    let download = UpdateStatus {
        progress: Permille(250),
        ..offered(UpdatePhase::Downloading)
    };
    let failed = UpdateStatus {
        problem: Some("There is no network.".to_owned()),
        ..offered(UpdatePhase::Failed)
    };
    let never = UpdateStatus {
        last_checked: None,
        ..status(UpdatePhase::Idle)
    };
    let silent_failure = status(UpdatePhase::Failed);
    // `(case, the status, title, detail, bar, buttons, last checked)`.
    type Case<'a> = (
        &'a str,
        UpdateStatus,
        &'a str,
        &'a str,
        UpdateMeter,
        Vec<UpdateButton>,
        Option<&'a str>,
    );
    let cases: Vec<Case> = vec![
        (
            "up to date",
            status(UpdatePhase::Idle),
            "Quire 0.3.1",
            "Your computer is up to date",
            UpdateMeter::None,
            vec![UpdateButton::Check(ButtonActivity::Still)],
            Some("Last checked 5 minutes ago"),
        ),
        (
            "never checked",
            never,
            "Quire 0.3.1",
            "Your computer is up to date",
            UpdateMeter::None,
            vec![UpdateButton::Check(ButtonActivity::Still)],
            Some("Not checked yet"),
        ),
        (
            "checking",
            status(UpdatePhase::Checking),
            "Quire 0.3.1",
            "Checking for updates\u{2026}",
            UpdateMeter::None,
            vec![UpdateButton::Check(ButtonActivity::Busy)],
            None,
        ),
        (
            "an update waits",
            offered(UpdatePhase::Available),
            "Quire 0.4.0",
            "Update available \u{b7} 45.6 MB",
            UpdateMeter::None,
            vec![UpdateButton::MoreInfo, UpdateButton::UpdateNow],
            Some("Last checked 5 minutes ago"),
        ),
        (
            "downloading",
            download,
            "Quire 0.4.0",
            "Downloading\u{2026}",
            UpdateMeter::Known {
                share: Permille(250),
                words: Some("11.4 MB of 45.6 MB".to_owned()),
            },
            vec![],
            None,
        ),
        (
            "ready to install",
            offered(UpdatePhase::Ready),
            "Quire 0.4.0",
            "Preparing to install\u{2026}",
            UpdateMeter::Pending,
            vec![],
            None,
        ),
        (
            "installing",
            offered(UpdatePhase::Installing),
            "Quire 0.4.0",
            "Installing\u{2026}",
            UpdateMeter::Pending,
            vec![],
            None,
        ),
        (
            "installed",
            status(UpdatePhase::Installed),
            "Quire 0.3.1",
            "Log out to finish the update",
            UpdateMeter::None,
            vec![UpdateButton::LogOut],
            None,
        ),
        (
            "failed with the service's words",
            failed,
            "Quire 0.4.0",
            "There is no network.",
            UpdateMeter::None,
            vec![UpdateButton::TryAgain],
            Some("Last checked 5 minutes ago"),
        ),
        (
            "failed without words",
            silent_failure,
            "Quire 0.3.1",
            "The update did not finish. Try again in a moment.",
            UpdateMeter::None,
            vec![UpdateButton::TryAgain],
            Some("Last checked 5 minutes ago"),
        ),
    ];
    for (case, status, title, detail, meter, buttons, checked) in cases {
        let page = page_of(&ready(status));
        assert_eq!(page.title, title, "{case}");
        assert_eq!(page.detail, detail, "{case}");
        assert_eq!(page.meter, meter, "{case}");
        assert_eq!(page.buttons, buttons, "{case}");
        assert_eq!(page.checked.as_deref(), checked, "{case}");
        assert_eq!(page.notes, None, "{case}: the sheet is closed");
    }
}

#[test]
fn a_download_of_unknown_size_has_a_bar_and_no_count() {
    let unknown = UpdateStatus {
        offer: Some(UpdateOffer {
            size: ByteCount(0),
            ..offer()
        }),
        progress: Permille(400),
        ..status(UpdatePhase::Downloading)
    };
    assert_eq!(
        page_of(&ready(unknown)).meter,
        UpdateMeter::Known {
            share: Permille(400),
            words: None
        }
    );
}

#[test]
fn the_notes_sheet_carries_the_notes_or_says_there_are_none() {
    let open = |status| {
        UpdatesState::Ready(UpdatesReady {
            status,
            info: NotesState::Open,
        })
    };
    assert_eq!(
        page_of(&open(offered(UpdatePhase::Available))).notes,
        Some(NotesSheet {
            title: "Quire 0.4.0".to_owned(),
            body: "Faster start.".to_owned(),
        })
    );
    let bare = UpdateStatus {
        offer: Some(UpdateOffer {
            notes: "  \n".to_owned(),
            ..offer()
        }),
        ..status(UpdatePhase::Available)
    };
    assert_eq!(
        page_of(&open(bare)).notes.map(|sheet| sheet.body),
        Some("No release notes came with this update.".to_owned())
    );
}

#[test]
fn a_page_without_the_service_says_so_plainly() {
    // `(case, the state, the words)`.
    let cases = [
        (
            "no bus",
            UpdatesState::Unavailable(Absence::NoBus),
            UpdatesShown::Unavailable("Software updates are not available on this computer."),
        ),
        (
            "not running",
            UpdatesState::Unavailable(Absence::NotRunning),
            UpdatesShown::Unavailable("Software updates are not available on this computer."),
        ),
        (
            "not answering",
            UpdatesState::Unavailable(Absence::Failed("x".to_owned())),
            UpdatesShown::Unavailable("Software updates did not answer. Try again in a moment."),
        ),
        (
            "still looking",
            UpdatesState::Looking,
            UpdatesShown::Looking,
        ),
    ];
    for (case, state, want) in cases {
        assert_eq!(updates_shown(&state, NOW), want, "{case}");
    }
}

#[test]
fn every_button_has_words_and_the_press_it_makes() {
    // `(button, label, press)`.
    const CASES: &[(UpdateButton, &str, UpdatesInput)] = &[
        (
            UpdateButton::Check(ButtonActivity::Still),
            "Check for Updates",
            UpdatesInput::Check,
        ),
        (
            UpdateButton::Check(ButtonActivity::Busy),
            "Check for Updates",
            UpdatesInput::Check,
        ),
        (
            UpdateButton::MoreInfo,
            "More Info\u{2026}",
            UpdatesInput::MoreInfo,
        ),
        (
            UpdateButton::UpdateNow,
            "Update Now",
            UpdatesInput::UpdateNow,
        ),
        (UpdateButton::TryAgain, "Try Again", UpdatesInput::TryAgain),
        (
            UpdateButton::LogOut,
            "Log Out\u{2026}",
            UpdatesInput::LogOut,
        ),
    ];
    for (button, label, press) in CASES {
        assert_eq!(button.label(), *label);
        assert_eq!(button.press(), *press, "{label}");
    }
}

#[test]
fn the_page_has_its_own_slug_label_and_place_and_draws_only_its_pane() {
    let page = software_update_page();
    assert!(is_software_update(&page));
    assert!(!is_software_update(&Page::App("anyview".to_owned())));
    assert_eq!(PageSlug::of(&page).0, "software-update");
    assert_eq!(page.label(), "Software Update");

    let catalogue = Catalogue::build(&[]);
    let slugs: Vec<&str> = catalogue.pages.iter().map(|p| p.slug.0.as_str()).collect();
    assert_eq!(slugs, ["accounts", "software-update"]);
    let model: &PageModel = catalogue
        .page(&PageSlug("software-update".to_owned()))
        .expect("always listed");
    assert_eq!(blocks(model), [Block::SoftwareUpdate]);

    let mut order = [
        Page::App("zed".to_owned()),
        software_update_page(),
        Page::Viewer,
        Page::Power,
    ];
    order.sort_by(crate::page::sidebar_order);
    let slugs: Vec<String> = order.iter().map(|p| PageSlug::of(p).0).collect();
    assert_eq!(slugs, ["power", "viewer", "software-update", "app:zed"]);
}
