//! The eight services Settings talks to, and the role the owner of each one's bus name must hold.

use peephole_bus::peephole_core::{Proof, RoleName, roles};

/// accountd: the accounts and their live settings module.
pub const ACCOUNTS_BUS: &str = "org.quire.Accounts1";
/// syncd: the datasets and the deletions it holds for the person's word.
pub const SYNC_BUS: &str = "org.quire.Sync1";
/// inferd: the model picker, the person's computers, the AI keys.
pub const INFERENCE_BUS: &str = "org.quire.Inference1";
/// slipway: the software-update service.
pub const UPDATES_BUS: &str = "org.quire.Updates1";
/// memoryd: the controls Settings reaches.
pub const MEMORY_BUS: &str = "org.quire.Memory1";
/// The compositor: the Privacy list of which program may use which power, served as a live
/// settings module.
pub const COMPOSITOR_BUS: &str = "org.quire.Compositor1";
/// The screen-sharing portal (transom): the apps it lets share the screen without asking again,
/// and the call that takes one back. The name is the portal backend's own; the interface Settings
/// calls on it is `org.quire.Portal1`.
pub const PORTAL_BUS: &str = "org.freedesktop.impl.portal.desktop.quire";

/// The shell (sill): each display's brightness, which Settings lists and sets.
pub const DISPLAYS_BUS: &str = "org.quire.Displays1";

/// The proof a service's owner must show. All eight run as units, so the unit manager confirms the
/// unit: a process that merely sits in the right cgroup, or runs the right image from a unit the
/// person can edit, does not pass.
pub const SERVICE_PROOF: Proof = Proof::Full;

/// The role the owner of `name` must hold; none for a name that is not one of the eight (such a
/// name is never vouched for).
pub fn required_role(name: &str) -> Option<RoleName> {
    let role = match name {
        ACCOUNTS_BUS => roles::ACCOUNTS,
        SYNC_BUS => roles::SYNC,
        INFERENCE_BUS => roles::INFERENCE,
        UPDATES_BUS => roles::UPDATER,
        MEMORY_BUS => roles::MEMORY,
        COMPOSITOR_BUS => roles::COMPOSITOR,
        PORTAL_BUS => roles::PORTAL,
        DISPLAYS_BUS => roles::SHELL,
        _ => return None,
    };
    Some(role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_service_names_the_role_of_its_program() {
        let wanted = [
            (ACCOUNTS_BUS, "accounts"),
            (SYNC_BUS, "sync"),
            (INFERENCE_BUS, "inference"),
            (UPDATES_BUS, "updater"),
            (MEMORY_BUS, "memory"),
            (COMPOSITOR_BUS, "compositor"),
            (PORTAL_BUS, "portal"),
            (DISPLAYS_BUS, "shell"),
        ];
        for (name, role) in wanted {
            let held = required_role(name).map(|role| role.to_string());
            assert_eq!(held.as_deref(), Some(role), "{name}");
        }
    }

    #[test]
    fn a_name_that_is_not_a_service_has_no_role() {
        assert_eq!(required_role("org.quire.Settings"), None);
        assert_eq!(required_role("org.example.Accounts1"), None);
        assert_eq!(required_role(""), None);
    }
}
