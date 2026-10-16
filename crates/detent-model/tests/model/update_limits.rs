//! What the update service may make the page hold: the page keeps a short version, a few screens
//! of notes and one sentence of problem, whatever the service sends (DETENT-06).

use detent_model::{Absence, ProblemKind, UpdateStatus, UpdateWire};

fn wire() -> UpdateWire {
    UpdateWire {
        state: "available".to_owned(),
        current: "0.3.1".to_owned(),
        available: "0.4.0-rc.1+build.5".to_owned(),
        notes: "Faster start.\nFewer crashes.".to_owned(),
        size: 1000,
        progress: 0,
        last_checked: 5,
        problem: String::new(),
        problem_kind: String::new(),
    }
}

#[test]
fn a_real_looking_version_and_short_notes_pass_unchanged() {
    let status = UpdateStatus::of_wire(wire()).expect("a status");
    let offer = status.offer.expect("an offer");
    assert_eq!(offer.version.as_str(), "0.4.0-rc.1+build.5");
    assert_eq!(offer.notes, "Faster start.\nFewer crashes.");
}

#[test]
fn a_version_that_is_long_or_not_a_version_is_a_service_that_makes_no_sense() {
    for odd in [
        "Quire 0.3.1",
        "0.4.0\nEnter your password to continue",
        &"9".repeat(65),
        "<b>0.4</b>",
    ] {
        let offered = UpdateWire {
            available: odd.to_owned(),
            ..wire()
        };
        assert!(
            matches!(UpdateStatus::of_wire(offered), Err(Absence::Failed(_))),
            "{odd:?}"
        );
        let installed = UpdateWire {
            current: odd.to_owned(),
            ..wire()
        };
        assert!(
            matches!(UpdateStatus::of_wire(installed), Err(Absence::Failed(_))),
            "{odd:?}"
        );
    }
}

#[test]
fn five_megabytes_of_notes_are_a_few_screens() {
    let huge = "A line of the notes that goes on.\n".repeat(150_000);
    assert!(huge.len() > 5_000_000);
    let status = UpdateStatus::of_wire(UpdateWire {
        notes: huge,
        ..wire()
    })
    .expect("a status");
    let notes = status.offer.expect("an offer").notes;
    assert!(
        notes.lines().count() <= 200,
        "{} lines",
        notes.lines().count()
    );
    assert!(notes.len() <= 16 * 1024, "{} bytes", notes.len());
    assert!(notes.starts_with("A line of the notes"));

    // A cut falls on a character boundary, never inside one.
    let wide = UpdateStatus::of_wire(UpdateWire {
        notes: "\u{3042}".repeat(10_000),
        ..wire()
    })
    .expect("a status");
    assert!(wide.offer.expect("an offer").notes.len() <= 16 * 1024);
}

#[test]
fn a_problem_is_one_short_line() {
    let long = format!("No network.\n{}", "x".repeat(5000));
    let status = UpdateStatus::of_wire(UpdateWire {
        state: "failed".to_owned(),
        problem: long,
        ..wire()
    })
    .expect("a status");
    let problem = status.problem.expect("a problem");
    assert!(!problem.contains('\n'));
    assert_eq!(
        problem.chars().count(),
        301,
        "300 characters and an ellipsis"
    );
}

#[test]
fn a_failure_kind_the_page_does_not_know_is_a_plain_failure() {
    let long = format!("version-changed{}", "x".repeat(60));
    let padded = format!("{}version-changed", "x".repeat(50));
    // `(what the service sent, the kind it is)`.
    let table = [
        ("version-changed", ProblemKind::VersionChanged),
        ("", ProblemKind::Other),
        ("offline", ProblemKind::Other),
        ("a-kind-from-a-newer-service", ProblemKind::Other),
        ("Version-Changed", ProblemKind::Other),
        (long.as_str(), ProblemKind::Other),
        (padded.as_str(), ProblemKind::Other),
    ];
    for (sent, expected) in table {
        let status = UpdateStatus::of_wire(UpdateWire {
            state: "failed".to_owned(),
            problem: "It did not work.".to_owned(),
            problem_kind: sent.to_owned(),
            ..wire()
        })
        .expect("a status");
        assert_eq!(status.problem_kind, expected, "{sent:?}");
        assert_eq!(status.problem.as_deref(), Some("It did not work."));
    }
}
