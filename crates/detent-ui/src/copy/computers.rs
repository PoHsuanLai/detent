//! The computers pane's words: how a guest stands and what is wrong with the Add a server sheet.

use detent_model::external::RowState;
use detent_model::{ServerFault, ServerRefusal};

/// The line under a guest's name: how it stands.
pub(crate) fn guest_standing(state: RowState) -> &'static str {
    match state {
        RowState::Approved => "Can use this computer's models",
        RowState::Denied => "Not allowed to use this computer's models",
        RowState::Asking => "Asking to use this computer's models",
    }
}

/// The sentence that says what is wrong with the sheet.
pub(crate) fn server_fault(fault: ServerFault) -> &'static str {
    match fault {
        ServerFault::NoName => "Give the server a name.",
        ServerFault::LongName => "Use a shorter name, up to 64 characters.",
        ServerFault::NoModel => "Choose which model it runs.",
        ServerFault::NoReach => "Say where it answers: a number, or a connection file.",
        ServerFault::BothReach => "Use a number or a connection file, not both.",
        ServerFault::BadPort => "Where it answers must be a number from 1 to 65535.",
        ServerFault::KeyFileUnreadable => {
            "The key file could not be read. Choose it again, or type the key."
        }
        ServerFault::KeyFileEmpty => "The key file is empty.",
    }
}

/// The sentence under the sheet's fields when the last Add did not go through.
pub(crate) fn server_refusal(refusal: &ServerRefusal) -> String {
    match refusal {
        ServerRefusal::Form(fault) => server_fault(*fault).to_owned(),
        ServerRefusal::Service(sentence) => sentence.clone(),
    }
}

/// The words "Add a server".
pub(crate) const ADD_A_SERVER: &str = "Add a server";

/// The words "Add a model server you already run, so the assistant can use its models.".
pub(crate) const ADD_A_MODEL_SERVER_YOU_ALREADY: &str =
    "Add a model server you already run, so the assistant can use its models.";

/// The words "The server must answer on this computer.".
pub(crate) const THE_SERVER_MUST_ANSWER_ON_THIS: &str = "The server must answer on this computer.";

/// The words "Looking for models…".
pub(crate) const LOOKING_FOR_MODELS_ELLIPSIS: &str = "Looking for models\u{2026}";

/// The words "There are no models to choose from yet.".
pub(crate) const THERE_ARE_NO_MODELS_TO_CHOOSE: &str = "There are no models to choose from yet.";

/// The words "Where it answers".
pub(crate) const WHERE_IT_ANSWERS: &str = "Where it answers";

/// The words "A number, such as 8080".
pub(crate) const A_NUMBER_SUCH_AS_8080: &str = "A number, such as 8080";

/// The words "Don’t use it".
pub(crate) const DONT_USE_IT: &str = "Don\u{2019}t use it";

/// The words "Don’t use".
pub(crate) const DONT_USE: &str = "Don\u{2019}t use";

/// The words "Choose a file…".
pub(crate) const CHOOSE_A_FILE_ELLIPSIS: &str = "Choose a file\u{2026}";

/// The words "Choose a file for".
pub(crate) const CHOOSE_A_FILE_FOR: &str = "Choose a file for";

/// The words "Key (optional)".
pub(crate) const KEY_OPTIONAL: &str = "Key (optional)";

/// The words "Use a key file…".
pub(crate) const USE_A_KEY_FILE_ELLIPSIS: &str = "Use a key file\u{2026}";

/// The words "Use a key file for".
pub(crate) const USE_A_KEY_FILE_FOR: &str = "Use a key file for";

/// The words "Adding…".
pub(crate) const ADDING_ELLIPSIS: &str = "Adding\u{2026}";

/// The words "Your computers".
pub(crate) const YOUR_COMPUTERS: &str = "Your computers";

/// The words "Computers asking to use this computer's models".
pub(crate) const COMPUTERS_ASKING_TO_USE_THIS_COMPUTER: &str =
    "Computers asking to use this computer's models";

/// The words "Computers you add appear here. Each starts off.".
pub(crate) const COMPUTERS_YOU_ADD_APPEAR_HERE_EACH: &str =
    "Computers you add appear here. Each starts off.";

/// The words "A model server you already run on this computer, such as one started by hand.".
pub(crate) const A_MODEL_SERVER_YOU_ALREADY_RUN: &str =
    "A model server you already run on this computer, such as one started by hand.";

/// The words "None of your other computers is lending its models. On the other computer, turn on \u{201c}Let my other computers use this computer’s models\u{201d}.".
pub(crate) const NONE_OF_YOUR_OTHER_COMPUTERS_IS: &str = "None of your other computers is lending its models. On the other \
    computer, turn on \u{201c}Let my other computers use this computer\u{2019}s models\u{201d}.";

/// The words "Looking for your computers on Tailscale…".
pub(crate) const LOOKING_FOR_YOUR_COMPUTERS_ON_TAILSCALE: &str =
    "Looking for your computers on Tailscale\u{2026}";

/// The words "Add a server…".
pub(crate) const ADD_A_SERVER_ELLIPSIS: &str = "Add a server\u{2026}";

/// The words "Add a computer".
pub(crate) const ADD_A_COMPUTER: &str = "Add a computer";

/// The words "{models}. Say yes on {} when it asks.".
pub(crate) fn say_yes_on_when_it_asks(
    models: impl std::fmt::Display,
    a: impl std::fmt::Display,
) -> String {
    format!("{models}. Say yes on {a} when it asks.")
}

/// The words "Add {}".
pub(crate) fn add(a: impl std::fmt::Display) -> String {
    format!("Add {a}")
}

/// The words "Allow".
pub(crate) const ALLOW: &str = "Allow";

/// The words "Don’t Allow".
pub(crate) const DONT_ALLOW: &str = "Don\u{2019}t Allow";

/// What a button that clears the connection file is about, as a screen reader says it.
pub(crate) const THE_CONNECTION_FILE: &str = "the connection file";

/// What a button that chooses where the server answers is about, as a screen reader says it.
pub(crate) const WHERE_IT_ANSWERS_SUBJECT: &str = "where it answers";

/// What a button that clears the key file is about, as a screen reader says it.
pub(crate) const THE_KEY_FILE: &str = "the key file";

/// What a button that picks the server's key file is about, as a screen reader says it.
pub(crate) const THE_SERVER: &str = "the server";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_guest_line_says_how_it_stands() {
        const CASES: &[(RowState, &str)] = &[
            (RowState::Approved, "Can use this computer's models"),
            (
                RowState::Denied,
                "Not allowed to use this computer's models",
            ),
            (RowState::Asking, "Asking to use this computer's models"),
        ];
        for (state, want) in CASES {
            assert_eq!(guest_standing(*state), *want);
        }
    }

    #[test]
    fn a_refusal_says_its_own_sentence() {
        let cases = [
            (
                ServerRefusal::Form(ServerFault::NoName),
                "Give the server a name.",
            ),
            (
                ServerRefusal::Form(ServerFault::LongName),
                "Use a shorter name, up to 64 characters.",
            ),
            (
                ServerRefusal::Form(ServerFault::NoModel),
                "Choose which model it runs.",
            ),
            (
                ServerRefusal::Form(ServerFault::NoReach),
                "Say where it answers: a number, or a connection file.",
            ),
            (
                ServerRefusal::Form(ServerFault::BothReach),
                "Use a number or a connection file, not both.",
            ),
            (
                ServerRefusal::Form(ServerFault::BadPort),
                "Where it answers must be a number from 1 to 65535.",
            ),
            (
                ServerRefusal::Form(ServerFault::KeyFileUnreadable),
                "The key file could not be read. Choose it again, or type the key.",
            ),
            (
                ServerRefusal::Form(ServerFault::KeyFileEmpty),
                "The key file is empty.",
            ),
            (ServerRefusal::Service("Taken.".to_owned()), "Taken."),
        ];
        for (refusal, want) in cases {
            assert_eq!(server_refusal(&refusal), want);
        }
    }
}
