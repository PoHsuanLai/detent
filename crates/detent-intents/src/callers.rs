//! Who may ask the running Settings to show itself, and whom a second `detent` may hand its
//! activation token to (audit DETENT-16).
//!
//! `org.freedesktop.Application` is a public door on the session bus: any program can call
//! `Activate` and `ActivateAction`. A call focuses the window, opens a page, and carries an
//! activation token, so the door admits only the callers the desktop has a reason to expect.

use detent_callers::{BusPeers, Proof, Refused, Requirement, access_denied, allowing, roles};
use std::sync::atomic::{AtomicUsize, Ordering};
use zbus::fdo;
use zbus::message::Header;

/// The roles that may call `Activate` and `ActivateAction`: the shell (the launcher, a notification
/// or a menu), another Settings (a second `detent` handing over), and the agent tier (an assistant
/// that opens a page). All three run as units, so the proof is the unit's (and, in a `dev-callers`
/// build, a stand-in named in the person's own rows is let in as well).
pub(crate) fn activators() -> Vec<Requirement> {
    [roles::SHELL, roles::SETTINGS, roles::AGENT_TIER]
        .into_iter()
        .flat_map(|role| allowing(role, Proof::Full))
        .collect()
}

/// The role the owner of `org.quire.Settings` must hold before a second `detent` sends it an
/// activation token: Settings itself.
pub(crate) fn settings_owner() -> Vec<Requirement> {
    allowing(roles::SETTINGS, Proof::Full)
}

/// How many refused calls the activation object has seen, so that a program that keeps calling
/// fills the log with a few lines and not a page of them.
#[derive(Debug, Default)]
pub(crate) struct Refusals(AtomicUsize);

impl Refusals {
    /// Counts one more refusal; the first, the second, the fourth, the eighth and so on are worth a
    /// line in the log.
    fn mention(&self) -> Mention {
        let seen = self.0.fetch_add(1, Ordering::Relaxed) + 1;
        if seen.is_power_of_two() {
            Mention::Yes(seen)
        } else {
            Mention::No
        }
    }
}

/// Whether a refusal gets a line in the log, and how many there have been.
enum Mention {
    Yes(usize),
    No,
}

/// `Ok` when the sender of the message in `header` holds one of the roles in [`activators`]. A
/// message that names no sender, a sender that cannot be identified and one in another role are
/// refused with the same `AccessDenied`, and a line in the log.
pub(crate) async fn admit(
    peers: &BusPeers,
    refusals: &Refusals,
    header: &Header<'_>,
) -> fdo::Result<()> {
    let sender = header.sender().map(|name| name.as_str());
    match peers.require(sender, &activators()).await {
        Ok(_) => Ok(()),
        Err(refused) => {
            note(refusals, refused, sender.unwrap_or("no sender"));
            Err(access_denied(refused))
        }
    }
}

fn note(refusals: &Refusals, refused: Refused, sender: &str) {
    if let Mention::Yes(seen) = refusals.mention() {
        eprintln!(
            "detent: a call to show Settings from {sender} was dropped ({refused}); {seen} so far"
        );
    }
}
