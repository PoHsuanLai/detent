//! What the person reads about the computers: a guest's line (a refusal comes with its own sentence).

use porter_core::lending::{GuestRow, RowState};

/// The line under a guest's name: how it stands.
pub fn guest_line(guest: &GuestRow) -> &'static str {
    match guest.state {
        RowState::Approved => "Can use this computer's models",
        RowState::Denied => "Not allowed to use this computer's models",
        RowState::Asking => "Asking to use this computer's models",
    }
}
