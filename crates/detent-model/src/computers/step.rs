//! The computers' one transition: `(state, input) -> (state, effects)`. The list, the candidates
//! of the Add sheet and the guests are read from the assistant's service; every change is asked
//! of it and the lists are read again after.

use crate::Absence;
use crate::Live;
use porter_core::NodeId;
use porter_core::lending::{ComputerCandidate, GuestAnswer, GuestRow};
use porter_infer::{ComputerName, PlaceKind, PlaceRow, PlaceState, PlaceTarget};

use super::server::{NewServer, ServerDraft, ServerEdit, ServerModel, ServerPhase, ServerRefusal};

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
    /// The computer's name in the service: what Remove names it by.
    pub computer: ComputerName,
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
        match (place.kind, place.id.target()) {
            (PlaceKind::OwnComputer, PlaceTarget::OwnComputer(computer)) => Some(ComputerRow {
                computer,
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

/// What the service lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputersFound {
    /// The person's computers.
    pub computers: Vec<ComputerRow>,
    /// The computers that asked to borrow models.
    pub guests: Vec<GuestRow>,
}

/// Whether the Add list is up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddList {
    /// It is not.
    Closed,
    /// It is, and Tailscale is being asked who is there.
    Looking,
    /// It is, listing these.
    Choosing(Vec<ComputerCandidate>),
}

/// A list that is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputersReady {
    /// The person's computers.
    pub computers: Vec<ComputerRow>,
    /// The computers that asked to borrow models.
    pub guests: Vec<GuestRow>,
    /// The Add list.
    pub adding: AddList,
    /// The Add a server sheet, while it is open.
    pub server: Option<NewServer>,
}

/// Where the computers stand.
pub type ComputersState = Live<ComputersReady>;

/// What happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputersInput {
    /// The lists were read.
    Found(Result<ComputersFound, Absence>),
    /// The person pressed Add.
    OpenAdd,
    /// The candidates were read, or why not.
    Candidates(Result<Vec<ComputerCandidate>, String>),
    /// The person pressed Done on the Add list.
    CloseAdd,
    /// The person chose a candidate.
    Add(NodeId),
    /// The person pressed Remove on a computer.
    Remove(ComputerName),
    /// The person answered a guest.
    Answer(NodeId, GuestAnswer),
    /// The person asked to forget a guest.
    Forget(NodeId),
    /// A change ended: done, or why not.
    Changed(Result<(), ServerRefusal>),
    /// The person pressed Add a server.
    OpenServer,
    /// The models a server may run were read.
    ServerModels(Vec<ServerModel>),
    /// The person changed the Add a server sheet.
    ServerEdit(ServerEdit),
    /// The person pressed Add on the sheet.
    AddServer,
    /// The person pressed Cancel on the sheet.
    CloseServer,
    /// Adding the server ended: done, or why not.
    ServerAdded(Result<(), ServerRefusal>),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputersEffect {
    /// Read the lists again.
    Reload,
    /// Ask who can be added.
    LoadCandidates,
    /// Add the computer.
    Add(NodeId),
    /// Remove the computer.
    Remove(ComputerName),
    /// Pass the person's answer on.
    Answer(NodeId, GuestAnswer),
    /// Forget the guest.
    Forget(NodeId),
    /// Ask which models a server may run.
    LoadServerModels,
    /// Add the server.
    AddServer(ServerDraft),
    /// Tell the person something, in the service's words.
    Tell(String),
    /// Tell the person why a change was not made.
    Refused(ServerRefusal),
}

/// The state after `input`, and what to do about it.
pub fn computers_step(
    state: ComputersState,
    input: ComputersInput,
) -> (ComputersState, Vec<ComputersEffect>) {
    match input {
        ComputersInput::Found(read) => (
            state.found(
                read,
                |found| ComputersReady {
                    computers: found.computers,
                    guests: found.guests,
                    adding: AddList::Closed,
                    server: None,
                },
                |ready, found| ComputersReady {
                    computers: found.computers,
                    guests: found.guests,
                    ..ready
                },
            ),
            vec![],
        ),
        other => state.inner(|ready| ready_step(ready, other)),
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
        ComputersInput::Remove(computer) => effects.push(ComputersEffect::Remove(computer)),
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
        ComputersInput::Changed(Err(refusal)) => {
            effects.push(ComputersEffect::Refused(refusal));
            effects.push(ComputersEffect::Reload);
        }
        ComputersInput::Found(_) => {}
        server => server_step(&mut ready, &mut effects, server),
    }
    (ready, effects)
}

/// The Add a server sheet's part of the transition.
fn server_step(
    ready: &mut ComputersReady,
    effects: &mut Vec<ComputersEffect>,
    input: ComputersInput,
) {
    match (input, ready.server.take()) {
        (ComputersInput::OpenServer, _) => {
            ready.server = Some(NewServer::opened());
            effects.push(ComputersEffect::LoadServerModels);
        }
        (ComputersInput::ServerModels(models), Some(sheet)) => {
            ready.server = Some(sheet.offering(models));
        }
        (ComputersInput::ServerEdit(edit), Some(mut sheet))
            if sheet.phase == ServerPhase::Editing =>
        {
            sheet.form = sheet.form.edited(edit);
            sheet.fault = None;
            ready.server = Some(sheet);
        }
        (ComputersInput::AddServer, Some(mut sheet)) if sheet.phase == ServerPhase::Editing => {
            match sheet.form.check() {
                Ok(draft) => {
                    sheet.phase = ServerPhase::Adding;
                    sheet.fault = None;
                    effects.push(ComputersEffect::AddServer(draft));
                }
                Err(fault) => sheet.fault = Some(ServerRefusal::Form(fault)),
            }
            ready.server = Some(sheet);
        }
        (ComputersInput::CloseServer, Some(sheet)) if sheet.phase == ServerPhase::Adding => {
            ready.server = Some(sheet);
        }
        (ComputersInput::CloseServer, _) => {}
        (ComputersInput::ServerAdded(Ok(())), Some(_)) => effects.push(ComputersEffect::Reload),
        (ComputersInput::ServerAdded(Err(refusal)), Some(sheet)) => {
            ready.server = Some(sheet.refused(refusal));
        }
        (_, sheet) => ready.server = sheet,
    }
}
