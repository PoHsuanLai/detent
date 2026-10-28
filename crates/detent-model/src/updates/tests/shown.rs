//! What each standing shows, as facts, and the page's own place among the pages.

use super::super::{
    ByteCount, Checked, Downloaded, Permille, ReleaseNotes, UpdateAction, UpdateMeter, UpdateOffer,
    UpdatePhase, UpdateStanding, UpdateStatus, UpdatesInput, UpdatesNotice, UpdatesReady,
    UpdatesShown, UpdatesState, software_update_page, updates_shown,
};
use super::{NOW, offer, offered, page_of, ready, status};
use crate::Absence;
use crate::catalogue::{Catalogue, PageModel};
use crate::layout::{Block, blocks};
use crate::page::PageSlug;
use crate::page_kind::PageKind;
use crate::page_traits::LivePane;
use ds_settings::schema::Page;

#[test]
fn each_standing_shows_its_facts_actions_and_bar() {
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
    let resting = vec![UpdateAction::Check { running: false }];
    let five_minutes = Some(Checked::Ago(300));
    // `(case, the status, version, standing, bar, actions, last checked)`.
    type Case<'a> = (
        &'a str,
        UpdateStatus,
        &'a str,
        UpdateStanding,
        Option<UpdateMeter>,
        Vec<UpdateAction>,
        Option<Checked>,
    );
    let cases: Vec<Case> = vec![
        (
            "up to date",
            status(UpdatePhase::Idle),
            "0.3.1",
            UpdateStanding::UpToDate,
            None,
            resting.clone(),
            five_minutes,
        ),
        (
            "never checked",
            never,
            "0.3.1",
            UpdateStanding::UpToDate,
            None,
            resting,
            Some(Checked::Never),
        ),
        (
            "checking",
            status(UpdatePhase::Checking),
            "0.3.1",
            UpdateStanding::Checking,
            None,
            vec![UpdateAction::Check { running: true }],
            None,
        ),
        (
            "an update waits",
            offered(UpdatePhase::Available),
            "0.4.0",
            UpdateStanding::Available(ByteCount(45_600_000)),
            None,
            vec![UpdateAction::MoreInfo, UpdateAction::UpdateNow],
            five_minutes,
        ),
        (
            "downloading",
            download,
            "0.4.0",
            UpdateStanding::Downloading,
            Some(UpdateMeter::Known {
                share: Permille(250),
                done: Some(Downloaded {
                    done: ByteCount(11_400_000),
                    total: ByteCount(45_600_000),
                }),
            }),
            vec![],
            None,
        ),
        (
            "ready to install",
            offered(UpdatePhase::Ready),
            "0.4.0",
            UpdateStanding::Preparing,
            Some(UpdateMeter::Pending),
            vec![],
            None,
        ),
        (
            "installing",
            offered(UpdatePhase::Installing),
            "0.4.0",
            UpdateStanding::Installing,
            Some(UpdateMeter::Pending),
            vec![],
            None,
        ),
        (
            "installed",
            status(UpdatePhase::Installed),
            "0.3.1",
            UpdateStanding::Installed,
            None,
            vec![UpdateAction::LogOut],
            None,
        ),
        (
            "failed with the service's words",
            failed,
            "0.4.0",
            UpdateStanding::Failed(Some("There is no network.".to_owned())),
            None,
            vec![UpdateAction::TryAgain],
            five_minutes,
        ),
        (
            "failed without words",
            silent_failure,
            "0.3.1",
            UpdateStanding::Failed(None),
            None,
            vec![UpdateAction::TryAgain],
            five_minutes,
        ),
    ];
    for (case, status, version, standing, meter, actions, checked) in cases {
        let page = page_of(&ready(status));
        assert_eq!(page.version.as_deref(), Some(version), "{case}");
        assert_eq!(page.standing, standing, "{case}");
        assert_eq!(page.meter, meter, "{case}");
        assert_eq!(page.actions, actions, "{case}");
        assert_eq!(page.checked, checked, "{case}");
        assert_eq!(page.notice, UpdatesNotice::Nothing, "{case}");
        assert_eq!(page.notes, None, "{case}: the notes are closed");
    }
}

#[test]
fn a_page_with_no_version_known_names_none() {
    let nameless = UpdateStatus {
        current: None,
        ..status(UpdatePhase::Idle)
    };
    assert_eq!(page_of(&ready(nameless)).version, None);
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
        Some(UpdateMeter::Known {
            share: Permille(400),
            done: None
        })
    );
}

#[test]
fn the_notes_carry_the_notes_or_none_when_the_release_has_none() {
    let open = |status| {
        UpdatesState::Ready(UpdatesReady {
            status,
            notes_open: true,
            notice: UpdatesNotice::Nothing,
        })
    };
    assert_eq!(
        page_of(&open(offered(UpdatePhase::Available))).notes,
        Some(ReleaseNotes {
            version: "0.4.0".to_owned(),
            text: Some("Faster start.".to_owned()),
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
        page_of(&open(bare)).notes.map(|notes| notes.text),
        Some(None)
    );
}

#[test]
fn a_page_without_the_service_says_why() {
    // `(case, the state, what is shown)`.
    let cases = [
        (
            "no bus",
            UpdatesState::Unavailable(Absence::NoBus),
            UpdatesShown::Unavailable(Absence::NoBus),
        ),
        (
            "not running",
            UpdatesState::Unavailable(Absence::NotRunning),
            UpdatesShown::Unavailable(Absence::NotRunning),
        ),
        (
            "not answering",
            UpdatesState::Unavailable(Absence::Failed),
            UpdatesShown::Unavailable(Absence::Failed),
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
fn every_action_makes_the_input_it_names() {
    // `(action, input)`.
    const CASES: &[(UpdateAction, UpdatesInput)] = &[
        (UpdateAction::Check { running: false }, UpdatesInput::Check),
        (UpdateAction::Check { running: true }, UpdatesInput::Check),
        (UpdateAction::MoreInfo, UpdatesInput::MoreInfo),
        (UpdateAction::UpdateNow, UpdatesInput::UpdateNow),
        (UpdateAction::TryAgain, UpdatesInput::TryAgain),
        (UpdateAction::LogOut, UpdatesInput::LogOut),
    ];
    for (action, input) in CASES {
        assert_eq!(action.input(), *input, "{action:?}");
    }
}

#[test]
fn the_page_has_its_own_slug_label_and_place_and_draws_only_its_pane() {
    let page = software_update_page();
    assert_eq!(PageKind::of(&page), PageKind::SoftwareUpdate);
    assert_eq!(
        PageKind::of(&Page::App("anyview".to_owned())),
        PageKind::Program
    );
    assert_eq!(PageSlug::of(&page).0, "software-update");
    assert_eq!(page.label(), "Software Update");

    let catalogue = Catalogue::build(&[]);
    let slugs: Vec<&str> = catalogue.pages.iter().map(|p| p.slug.0.as_str()).collect();
    assert_eq!(
        slugs,
        [
            "wifi",
            "bluetooth",
            "extensions",
            "assistant-model",
            "privacy",
            "displays",
            "spaces",
            "software-update",
            "accounts"
        ]
    );
    let model: &PageModel = catalogue
        .page(&PageSlug("software-update".to_owned()))
        .expect("always listed");
    assert_eq!(blocks(model), [Block::Pane(LivePane::SoftwareUpdate)]);

    let mut order = [
        Page::App("zed".to_owned()),
        software_update_page(),
        Page::Viewer,
        Page::Power,
    ];
    let table = ds_settings::schema::PageTable::built_in();
    order.sort_by(|a, b| crate::page::sidebar_order(&table, a, b));
    let slugs: Vec<String> = order.iter().map(|p| PageSlug::of(p).0).collect();
    assert_eq!(slugs, ["power", "software-update", "viewer", "app:zed"]);
}
