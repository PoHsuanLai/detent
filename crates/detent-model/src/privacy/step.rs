//! The page's one transition: `(state, input) -> (state, effects)`.

use super::pane::{
    ALLOWED_WORD, CellKind, DENIED_WORD, PowerStanding, PrivacyPane, ProgramSubject,
};
use super::power::PrivacyPower;
use crate::Absence;
use crate::Live;
use crate::picker::Described;
use ds_settings::schema::KeyPath;

/// What the page knows.
pub type PrivacyState = Live<PrivacyReady>;

/// The list and the change in flight.
#[derive(Debug, Clone, PartialEq)]
pub struct PrivacyReady {
    /// The programs.
    pub pane: PrivacyPane,
    /// The cell a change was sent for and has not been answered.
    pub pending: Option<(ProgramSubject, PrivacyPower)>,
}

/// What the page hears.
#[derive(Debug, Clone, PartialEq)]
pub enum PrivacyInput {
    /// The compositor's rows, read, or why they could not be.
    Found(Result<Described, Absence>),
    /// The person picked a standing for a power of a program.
    Pick {
        /// The program.
        subject: ProgramSubject,
        /// The power.
        power: PrivacyPower,
        /// The standing picked.
        standing: PowerStanding,
    },
    /// The change was taken.
    Done,
    /// The change was refused, with the compositor's own words.
    Failed(String),
}

/// The two positions of a row that allows or denies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacySwitch {
    /// Allow.
    On,
    /// Deny.
    Off,
}

impl PrivacySwitch {
    /// The word the row takes.
    pub fn word(self) -> &'static str {
        match self {
            PrivacySwitch::On => ALLOWED_WORD,
            PrivacySwitch::Off => DENIED_WORD,
        }
    }
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivacyEffect {
    /// Set the row to the switch's word.
    Set {
        /// The row.
        path: KeyPath,
        /// Allow or deny.
        to: PrivacySwitch,
    },
    /// Run the action that forgets the answer, so the program asks again.
    Forget(KeyPath),
    /// Read the rows again.
    Reload,
    /// Tell the person something went wrong.
    Tell(String),
}

/// The state after `input`, and what to do about it.
pub fn privacy_step(
    state: PrivacyState,
    input: PrivacyInput,
) -> (PrivacyState, Vec<PrivacyEffect>) {
    match input {
        PrivacyInput::Found(read) => (
            state.found(
                read,
                |described| PrivacyReady {
                    pane: PrivacyPane::of(&described),
                    pending: None,
                },
                |ready, described| PrivacyReady {
                    pane: PrivacyPane::of(&described),
                    ..ready
                },
            ),
            vec![],
        ),
        other => state.inner(|ready| ready_step(ready, other)),
    }
}

fn ready_step(mut ready: PrivacyReady, input: PrivacyInput) -> (PrivacyReady, Vec<PrivacyEffect>) {
    let mut effects = Vec::new();
    match input {
        PrivacyInput::Pick {
            subject,
            power,
            standing,
        } if ready.pending.is_none() => {
            if let Some(effect) = change(&ready.pane, &subject, power, standing) {
                effects.push(effect);
                ready.pending = Some((subject, power));
            }
        }
        PrivacyInput::Done => {
            ready.pending = None;
            effects.push(PrivacyEffect::Reload);
        }
        PrivacyInput::Failed(why) => {
            ready.pending = None;
            effects.push(PrivacyEffect::Tell(why));
            effects.push(PrivacyEffect::Reload);
        }
        PrivacyInput::Pick { .. } | PrivacyInput::Found(_) => {}
    }
    (ready, effects)
}

/// What picking `standing` for `power` of `subject` asks of the compositor: nothing when the
/// cell is an administrator's, has no answer yet, or already stands there.
fn change(
    pane: &PrivacyPane,
    subject: &ProgramSubject,
    power: PrivacyPower,
    standing: PowerStanding,
) -> Option<PrivacyEffect> {
    let cell = pane
        .programs
        .iter()
        .find(|program| program.subject == *subject)?
        .cell(power)?;
    let CellKind::Remembered { switch, forget } = &cell.kind else {
        return None;
    };
    if cell.standing == standing {
        return None;
    }
    match standing {
        PowerStanding::Allowed => Some(PrivacyEffect::Set {
            path: switch.clone(),
            to: PrivacySwitch::On,
        }),
        PowerStanding::Denied => Some(PrivacyEffect::Set {
            path: switch.clone(),
            to: PrivacySwitch::Off,
        }),
        PowerStanding::Ask => forget.clone().map(PrivacyEffect::Forget),
    }
}
