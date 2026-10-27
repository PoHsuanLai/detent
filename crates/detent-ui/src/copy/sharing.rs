//! The sharing switch's words: the question put to the person before it is turned on.

use detent_model::Existing;

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
