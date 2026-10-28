//! The sharing switch's words: the question put to the person before it is turned on.

use detent_model::{Existing, LendFailure};

/// The button that agrees.
pub(crate) const ALLOW: &str = "Allow";

/// The button that backs out.
pub(crate) const CANCEL: &str = "Cancel";

/// The question to put to the person: `(title, message)`.
pub(crate) fn sharing_question(existing: Existing) -> (String, String) {
    let mut message = "Your other computers on your own Tailscale network will be able to reach \
                       this computer and use the models that run on it. You can turn this off \
                       at any time."
        .to_owned();
    if existing == Existing::Different {
        message.push_str(" Sharing was set up here in another way before; allowing replaces it.");
    }
    (
        "Let your other computers use this computer's models?".to_owned(),
        message,
    )
}

/// Why the sharing could not be set up or taken away.
pub(crate) fn lend_failure(why: LendFailure) -> &'static str {
    match why {
        LendFailure::NoHome => {
            "There is no home folder to keep this setting in, so nothing was changed."
        }
        LendFailure::NoServices => {
            "This computer's services could not be reached, so nothing was changed."
        }
        LendFailure::NotInstalled => {
            "This computer is missing a file it needs to share its models, so nothing was \
             changed. Installing the software again puts it back."
        }
        LendFailure::CouldNotSave => {
            "Your setting for sharing this computer's models could not be saved. Check that \
             your home folder can be written to."
        }
        LendFailure::CouldNotReload => {
            "Your setting was saved, but the computer did not take it up yet. Signing out \
             and back in will apply it."
        }
        LendFailure::CouldNotRestart => {
            "Your setting was saved, but the models service did not restart. Try again, or \
             sign out and back in."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_question_is_plain_and_mentions_a_replaced_setup_only_when_there_is_one() {
        let (title, message) = sharing_question(Existing::Absent);
        assert_eq!(
            title,
            "Let your other computers use this computer's models?"
        );
        assert!(message.contains("your own Tailscale network"), "{message}");
        assert!(!message.contains("replaces"), "{message}");
        let (_, message) = sharing_question(Existing::Different);
        assert!(message.contains("replaces"), "{message}");
        for word in ["tailnet", "drop-in", "systemd", "daemon", "socket", "node"] {
            assert!(!message.to_lowercase().contains(word), "{word}");
        }
    }
}
