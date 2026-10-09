//! The computers' one transition: `(state, input) -> (state, effects)`. The list, the candidates
//! of the Add sheet and the guests are read from the assistant's service; every change is asked
//! of it and the lists are read again after.

use crate::picker::Absence;
use porter_infer::{PlaceKind, PlaceRow, PlaceState};

/// A Tailscale node's id, as Tailscale names the computer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeKey(pub String);

/// Whether a computer answers now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// It does.
    Answering,
    /// It does not (off, or not lending).
    Silent,
}

/// One of the person's computers the assistant may borrow models from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerRow {
    /// The place's id, which names it to Remove.
    pub id: String,
    /// What the person calls it.
    pub name: String,
    /// The names of the models it lends now.
    pub models: Vec<String>,
    /// Whether it answers.
    pub standing: Standing,
}

impl ComputerRow {
    /// The row of `place`, when it is one of the person's own computers.
    pub fn of(place: &PlaceRow) -> Option<ComputerRow> {
        match place.kind {
            PlaceKind::OwnComputer => Some(ComputerRow {
                id: place.id.as_str().to_owned(),
                name: place.name.clone(),
                models: place.models.iter().map(|m| m.name.clone()).collect(),
                standing: match place.state {
                    PlaceState::Ready => Standing::Answering,
                    PlaceState::NotReady => Standing::Silent,
                },
            }),
            _ => None,
        }
    }
}

/// Whether the person still has to say yes on the other computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Approval {
    /// Nothing is waiting for them.
    Given,
    /// The other computer will ask them first.
    Needed,
}

/// A Tailscale computer that can be added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Its node id.
    pub node: NodeKey,
    /// What Tailscale calls it.
    pub name: String,
    /// The names of the models it lends.
    pub models: Vec<String>,
    /// Whether the person still has to say yes on it.
    pub approval: Approval,
}

/// How a computer that asked to borrow this one's models stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestState {
    /// It may.
    Approved,
    /// It may not.
    Denied,
    /// It asked and waits for an answer.
    Asking,
}

/// A computer that asked to borrow this computer's models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guest {
    /// Its node id.
    pub node: NodeKey,
    /// What Tailscale calls it.
    pub name: String,
    /// How it stands.
    pub state: GuestState,
    /// When it began asking, in seconds since 1970.
    pub since: i64,
}

/// The person's answer to a guest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestAnswer {
    /// Let it use the models.
    Allow,
    /// Keep it out.
    Deny,
}

/// What the service lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputersFound {
    /// The person's computers.
    pub computers: Vec<ComputerRow>,
    /// The computers that asked to borrow models.
    pub guests: Vec<Guest>,
}

/// Whether the Add list is up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddList {
    /// It is not.
    Closed,
    /// It is, and Tailscale is being asked who is there.
    Looking,
    /// It is, listing these.
    Choosing(Vec<Candidate>),
}

/// A list that is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputersReady {
    /// The person's computers.
    pub computers: Vec<ComputerRow>,
    /// The computers that asked to borrow models.
    pub guests: Vec<Guest>,
    /// The Add list.
    pub adding: AddList,
}

/// Where the computers stand.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ComputersState {
    /// Looking for the service.
    #[default]
    Looking,
    /// The service cannot be used, and why.
    Unavailable(Absence),
    /// It answered.
    Ready(ComputersReady),
}

/// What happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputersInput {
    /// The lists were read.
    Found(Result<ComputersFound, Absence>),
    /// The person pressed Add.
    OpenAdd,
    /// The candidates were read, or why not.
    Candidates(Result<Vec<Candidate>, String>),
    /// The person pressed Done on the Add list.
    CloseAdd,
    /// The person chose a candidate.
    Add(NodeKey),
    /// The person pressed Remove on a computer, by place id.
    Remove(String),
    /// The person answered a guest.
    Answer(NodeKey, GuestAnswer),
    /// The person asked to forget a guest.
    Forget(NodeKey),
    /// A change ended: done, or the sentence that says why not.
    Changed(Result<(), String>),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputersEffect {
    /// Read the lists again.
    Reload,
    /// Ask who can be added.
    LoadCandidates,
    /// Add the computer.
    Add(NodeKey),
    /// Remove the computer, by place id.
    Remove(String),
    /// Pass the person's answer on.
    Answer(NodeKey, GuestAnswer),
    /// Forget the guest.
    Forget(NodeKey),
    /// Tell the person something.
    Tell(String),
}

/// The state after `input`, and what to do about it.
pub fn computers_step(
    state: ComputersState,
    input: ComputersInput,
) -> (ComputersState, Vec<ComputersEffect>) {
    match (state, input) {
        (ComputersState::Ready(ready), ComputersInput::Found(Ok(found))) => (
            ComputersState::Ready(ComputersReady {
                computers: found.computers,
                guests: found.guests,
                ..ready
            }),
            vec![],
        ),
        (_, ComputersInput::Found(Ok(found))) => (
            ComputersState::Ready(ComputersReady {
                computers: found.computers,
                guests: found.guests,
                adding: AddList::Closed,
            }),
            vec![],
        ),
        (_, ComputersInput::Found(Err(absence))) => (ComputersState::Unavailable(absence), vec![]),
        (ComputersState::Ready(ready), input) => {
            let (ready, effects) = ready_step(ready, input);
            (ComputersState::Ready(ready), effects)
        }
        (state, _) => (state, vec![]),
    }
}

fn ready_step(
    mut ready: ComputersReady,
    input: ComputersInput,
) -> (ComputersReady, Vec<ComputersEffect>) {
    let mut effects = Vec::new();
    match input {
        ComputersInput::OpenAdd => {
            ready.adding = AddList::Looking;
            effects.push(ComputersEffect::LoadCandidates);
        }
        ComputersInput::Candidates(Ok(found)) => {
            if ready.adding == AddList::Looking {
                ready.adding = AddList::Choosing(found);
            }
        }
        ComputersInput::Candidates(Err(why)) => {
            ready.adding = AddList::Closed;
            effects.push(ComputersEffect::Tell(why));
        }
        ComputersInput::CloseAdd => ready.adding = AddList::Closed,
        ComputersInput::Add(node) => effects.push(ComputersEffect::Add(node)),
        ComputersInput::Remove(id) => effects.push(ComputersEffect::Remove(id)),
        ComputersInput::Answer(node, answer) => effects.push(ComputersEffect::Answer(node, answer)),
        ComputersInput::Forget(node) => effects.push(ComputersEffect::Forget(node)),
        ComputersInput::Changed(Ok(())) => {
            // An added computer leaves the candidates; the list is read again with the rest.
            if matches!(ready.adding, AddList::Choosing(_)) {
                effects.push(ComputersEffect::LoadCandidates);
                ready.adding = AddList::Looking;
            }
            effects.push(ComputersEffect::Reload);
        }
        ComputersInput::Changed(Err(sentence)) => {
            effects.push(ComputersEffect::Tell(sentence));
            effects.push(ComputersEffect::Reload);
        }
        ComputersInput::Found(_) => {}
    }
    (ready, effects)
}
