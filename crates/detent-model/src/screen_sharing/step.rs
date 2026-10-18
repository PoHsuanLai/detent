//! The list's one transition: `(state, input) -> (state, effects)`. Remove takes the app back at
//! the portal, then the list is read again.

use super::grant::ScreenGrant;
use crate::Absence;
use crate::Live;
use crate::privacy::ProgramSubject;

/// A list that is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenSharingReady {
    /// The apps, the one allowed last first.
    pub grants: Vec<ScreenGrant>,
    /// The app whose removal is under way: nothing else is asked meanwhile.
    pub removing: Option<ProgramSubject>,
}

/// Where the list stands.
pub type ScreenSharingState = Live<ScreenSharingReady>;

/// What happened to the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenSharingInput {
    /// The portal's list was read.
    Found(Result<Vec<ScreenGrant>, Absence>),
    /// The person pressed Remove on an app.
    Remove(ProgramSubject),
    /// The portal took the app back, or said in these words why not.
    Removed(Result<(), String>),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenSharingEffect {
    /// Ask the portal to take the app back, so it is asked next time.
    Revoke(ProgramSubject),
    /// Read the portal's list again.
    Reload,
    /// Tell the person something went wrong.
    Tell(String),
}

/// The grants newest first, and by app where two were allowed together.
fn newest_first(mut grants: Vec<ScreenGrant>) -> Vec<ScreenGrant> {
    grants.sort_by(|a, b| b.granted.cmp(&a.granted).then_with(|| a.app.cmp(&b.app)));
    grants
}

/// The state after `input`, and what to do about it.
pub fn screen_sharing_step(
    state: ScreenSharingState,
    input: ScreenSharingInput,
) -> (ScreenSharingState, Vec<ScreenSharingEffect>) {
    match input {
        ScreenSharingInput::Found(read) => (
            state.found(
                read,
                |grants| ScreenSharingReady {
                    grants: newest_first(grants),
                    removing: None,
                },
                |ready, grants| ScreenSharingReady {
                    grants: newest_first(grants),
                    ..ready
                },
            ),
            vec![],
        ),
        other => state.inner(|ready| ready_step(ready, other)),
    }
}

fn ready_step(
    mut ready: ScreenSharingReady,
    input: ScreenSharingInput,
) -> (ScreenSharingReady, Vec<ScreenSharingEffect>) {
    let mut effects = Vec::new();
    match input {
        ScreenSharingInput::Remove(app)
            if ready.removing.is_none() && ready.grants.iter().any(|grant| grant.app == app) =>
        {
            effects.push(ScreenSharingEffect::Revoke(app.clone()));
            ready.removing = Some(app);
        }
        ScreenSharingInput::Removed(Ok(())) => {
            ready.removing = None;
            effects.push(ScreenSharingEffect::Reload);
        }
        ScreenSharingInput::Removed(Err(why)) => {
            ready.removing = None;
            effects.push(ScreenSharingEffect::Tell(why));
            effects.push(ScreenSharingEffect::Reload);
        }
        ScreenSharingInput::Remove(_) | ScreenSharingInput::Found(_) => {}
    }
    (ready, effects)
}
