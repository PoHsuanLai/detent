//! What the llama.cpp note says: how to get it, the button that starts using it, and the restart
//! that follows.

/// The quiet line over the install command.
pub(crate) fn setup_line() -> &'static str {
    "To run models on this computer, install llama.cpp."
}

/// What the person reads when there is no package to install.
pub(crate) fn releases_line() -> &'static str {
    "Get llama.cpp's server from its releases page"
}

/// The button that makes the assistant look again.
pub(crate) fn start_using() -> &'static str {
    "Start using it"
}

/// The button of the question that comes before.
pub(crate) fn restart() -> &'static str {
    "Restart"
}

/// What the person reads when the restart did not go through.
pub(crate) fn restart_failed() -> &'static str {
    "The assistant could not be restarted. Try again in a moment."
}

/// The question before the restart: a title and a message. It does not know whether a
/// conversation is open, so it always says what happens to one.
pub(crate) fn restart_question() -> (&'static str, &'static str) {
    (
        "Restart the assistant to use llama.cpp?",
        "The assistant restarts so it can find llama.cpp. A conversation that is open stops.",
    )
}

/// What an empty program field says: the assistant looks for the program itself.
pub(crate) fn found_automatically() -> &'static str {
    "Found automatically"
}

/// The words "Programs for local models".
pub(crate) const PROGRAMS_FOR_LOCAL_MODELS: &str = "Programs for local models";

/// The words "Open {TITLE}".
pub(crate) fn open(title: impl std::fmt::Display) -> String {
    format!("Open {title}")
}

/// The words "Choose a file for {}".
pub(crate) fn choose_a_file_for(a: impl std::fmt::Display) -> String {
    format!("Choose a file for {a}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_note_says_what_each_fact_means() {
        assert_eq!(
            setup_line(),
            "To run models on this computer, install llama.cpp."
        );
        assert_eq!(
            releases_line(),
            "Get llama.cpp's server from its releases page"
        );
        assert_eq!(start_using(), "Start using it");
        assert_eq!(restart(), "Restart");
        assert_eq!(
            restart_failed(),
            "The assistant could not be restarted. Try again in a moment."
        );
        assert_eq!(found_automatically(), "Found automatically");

        let (title, message) = restart_question();
        assert_eq!(title, "Restart the assistant to use llama.cpp?");
        assert!(
            message.contains("conversation that is open stops"),
            "{message}"
        );
        for word in ["systemd", "D-Bus", "inferd"] {
            assert!(!title.contains(word) && !message.contains(word), "{word}");
        }
    }
}
