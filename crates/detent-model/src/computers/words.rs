//! What the person reads about the computers: a refusal's sentence and a guest's line.

use super::step::{Guest, GuestState};

/// What a refusal the bus named says, in plain words. `name` is the part after
/// `org.quire.Inference1.Error.Computer.`; a name this does not know gets the general sentence.
pub fn refusal_sentence(name: &str) -> &'static str {
    match name {
        "NotOnTailscale" => "That is not one of your computers on Tailscale.",
        "NotAnswering" => {
            "That computer isn't lending its models right now. Check that it is on, and that it \
             is set to let your other computers use its models."
        }
        "NotAsking" => "That computer is not asking to use this computer's models.",
        "TooManyAsking" => "Too many computers are asking at once. Answer some of them first.",
        "NotThere" => "There is no computer with that name.",
        "AddedByHand" => {
            "That computer was added by hand in the settings file, so it can only be removed there."
        }
        "AlreadyThere" => "A computer with that name is already there.",
        _ => "Your computers could not be changed. Try again in a moment.",
    }
}

/// The line under a guest's name: how it stands.
pub fn guest_line(guest: &Guest) -> &'static str {
    match guest.state {
        GuestState::Approved => "Can use this computer's models",
        GuestState::Denied => "Not allowed to use this computer's models",
        GuestState::Asking => "Asking to use this computer's models",
    }
}
