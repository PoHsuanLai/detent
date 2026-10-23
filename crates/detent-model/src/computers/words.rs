//! What the person reads about the computers: a guest's line (a refusal comes with its own sentence)
//! and the ways the Add a server sheet can be wrong.

use porter_core::lending::{GuestRow, RowState};

/// The line under a guest's name: how it stands.
pub fn guest_line(guest: &GuestRow) -> &'static str {
    match guest.state {
        RowState::Approved => "Can use this computer's models",
        RowState::Denied => "Not allowed to use this computer's models",
        RowState::Asking => "Asking to use this computer's models",
    }
}

/// What is wrong with the Add a server sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerFault {
    /// The name is empty.
    NoName,
    /// The name is over 64 characters.
    LongName,
    /// No model was picked.
    NoModel,
    /// Neither a number nor a file was given.
    NoReach,
    /// Both were.
    BothReach,
    /// The number is not 1 to 65535.
    BadPort,
    /// The key file could not be read.
    KeyFileUnreadable,
    /// The key file has nothing in it.
    KeyFileEmpty,
}

impl ServerFault {
    /// The sentence that says it.
    pub fn words(self) -> &'static str {
        match self {
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
}
