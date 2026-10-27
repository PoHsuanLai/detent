//! The computers pane's words: how a guest stands and what is wrong with the Add a server sheet.

use detent_model::{ServerFault, ServerRefusal};
use porter_core::lending::RowState;

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
