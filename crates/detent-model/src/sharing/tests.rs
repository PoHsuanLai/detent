//! The sharing switch's transitions and the words of its question.

use super::{Existing, Sharing, SharingEffect, SharingInput, Way, sharing_question, sharing_step};

fn failed() -> Result<(), String> {
    Err("It did not work.".to_owned())
}

type Case = (Sharing, SharingInput, Sharing, Vec<SharingEffect>);

#[test]
fn the_transitions() {
    let cases: Vec<Case> = vec![
        (
            Sharing::Idle,
            SharingInput::Switched(Way::On, Existing::Absent),
            Sharing::Asking(Existing::Absent),
            vec![],
        ),
        (
            Sharing::Idle,
            SharingInput::Switched(Way::Off, Existing::Same),
            Sharing::Working(Way::Off),
            vec![SharingEffect::Disable],
        ),
        (
            Sharing::Asking(Existing::Same),
            SharingInput::Allowed,
            Sharing::Working(Way::On),
            vec![SharingEffect::Enable],
        ),
        (
            Sharing::Asking(Existing::Same),
            SharingInput::Cancelled,
            Sharing::Idle,
            vec![],
        ),
        (
            Sharing::Working(Way::On),
            SharingInput::Finished(Ok(())),
            Sharing::Idle,
            vec![SharingEffect::Record(Way::On)],
        ),
        (
            Sharing::Working(Way::On),
            SharingInput::Finished(failed()),
            Sharing::Idle,
            vec![SharingEffect::Tell("It did not work.".to_owned())],
        ),
        (
            Sharing::Working(Way::Off),
            SharingInput::Finished(Ok(())),
            Sharing::Idle,
            vec![SharingEffect::Record(Way::Off)],
        ),
        (
            Sharing::Working(Way::Off),
            SharingInput::Finished(failed()),
            Sharing::Idle,
            vec![
                SharingEffect::Record(Way::Off),
                SharingEffect::Tell("It did not work.".to_owned()),
            ],
        ),
        // Presses that mean nothing now change nothing.
        (
            Sharing::Working(Way::On),
            SharingInput::Switched(Way::Off, Existing::Same),
            Sharing::Working(Way::On),
            vec![],
        ),
        (Sharing::Idle, SharingInput::Allowed, Sharing::Idle, vec![]),
    ];
    for (state, input, next, effects) in cases {
        let shown = format!("{state:?} + {input:?}");
        assert_eq!(sharing_step(state, input), (next, effects), "{shown}");
    }
}

#[test]
fn the_question_is_plain_and_mentions_a_replaced_setup_only_when_there_is_one() {
    assert_eq!(sharing_question(Sharing::Idle), None);
    let (title, message) = sharing_question(Sharing::Asking(Existing::Absent)).expect("asked");
    assert_eq!(
        title,
        "Let your other computers use this computer's models?"
    );
    assert!(message.contains("your own Tailscale network"), "{message}");
    assert!(!message.contains("replaces"), "{message}");
    let (_, message) = sharing_question(Sharing::Asking(Existing::Different)).expect("asked");
    assert!(message.contains("replaces"), "{message}");
    for word in ["tailnet", "drop-in", "systemd", "daemon", "socket", "node"] {
        assert!(!message.to_lowercase().contains(word), "{word}");
    }
}
