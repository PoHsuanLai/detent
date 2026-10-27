//! The service's properties read into a typed status.

use super::super::{Permille, ProblemKind, UnixSeconds, UpdatePhase, UpdateStatus, UpdateWire};
use super::offer;
use crate::Absence;

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
        problem_kind: String::new(),
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
            problem_kind: ProblemKind::Other,
        }
    );
    let bare = UpdateStatus::of_wire(UpdateWire {
        available: String::new(),
        last_checked: 0,
        progress: 4000,
        problem: "No network.".to_owned(),
        problem_kind: "version-changed".to_owned(),
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
    assert_eq!(bare.problem_kind, ProblemKind::VersionChanged);
    assert!(matches!(
        UpdateStatus::of_wire(wire("rebooting")),
        Err(Absence::Failed)
    ));
}

#[test]
fn every_word_the_service_uses_names_a_phase() {
    // The phases the service is working in, and those it is not.
    const WORKING: &[(&str, UpdatePhase)] = &[
        ("checking", UpdatePhase::Checking),
        ("downloading", UpdatePhase::Downloading),
        ("ready", UpdatePhase::Ready),
        ("installing", UpdatePhase::Installing),
    ];
    const AT_REST: &[(&str, UpdatePhase)] = &[
        ("idle", UpdatePhase::Idle),
        ("available", UpdatePhase::Available),
        ("installed", UpdatePhase::Installed),
        ("failed", UpdatePhase::Failed),
    ];
    for (word, phase) in WORKING.iter().chain(AT_REST) {
        assert_eq!(UpdatePhase::of_word(word), Some(*phase), "{word}");
    }
    assert!(WORKING.iter().all(|(_, phase)| phase.is_busy()));
    assert!(AT_REST.iter().all(|(_, phase)| !phase.is_busy()));
    assert_eq!(UpdatePhase::of_word("rebooting"), None);
}
