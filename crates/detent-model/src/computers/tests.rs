//! The computers' transitions and words.

use super::{
    AddList, ComputerRow, ComputersEffect as Effect, ComputersFound, ComputersInput as Input,
    ComputersState as State, Standing, computers_step, guest_line,
};
use crate::picker::Absence;
use porter_core::lending::{
    Approval, CandidateModel, ComputerCandidate, GuestAnswer, GuestRow, RowState,
};
use porter_core::{NodeId, UnixSeconds};
use porter_infer::{ComputerName, PlaceId, PlaceKind, PlaceModel, PlaceRow, PlaceState};

fn node(text: &str) -> NodeId {
    NodeId::parse(text).expect("a node id")
}

fn lab() -> ComputerRow {
    ComputerRow {
        computer: ComputerName::parse("lab").expect("a name"),
        name: "lab".to_owned(),
        models: vec!["Qwen".to_owned()],
        standing: Standing::Answering,
    }
}

fn candidate() -> ComputerCandidate {
    ComputerCandidate::new(
        node("n1"),
        "studio".to_owned(),
        vec![CandidateModel::new("llama".to_owned(), "Llama".to_owned())],
        Approval::Needed,
    )
}

fn ready() -> State {
    let found = ComputersFound {
        computers: vec![lab()],
        guests: vec![],
    };
    computers_step(State::Looking, Input::Found(Ok(found))).0
}

fn choosing() -> State {
    let opened = computers_step(ready(), Input::OpenAdd).0;
    computers_step(opened, Input::Candidates(Ok(vec![candidate()]))).0
}

fn adding_of(state: &State) -> &AddList {
    match state {
        State::Ready(ready) => &ready.adding,
        other => panic!("not ready: {other:?}"),
    }
}

#[test]
fn only_the_persons_own_computers_make_rows() {
    let row = |kind, state| {
        PlaceRow::new(
            PlaceId::parse("computer:lab").expect("an id"),
            kind,
            "lab".to_owned(),
            state,
        )
        .with_models(vec![PlaceModel {
            id: porter_core::ModelId::parse("qwen").expect("a model id"),
            name: "Qwen".to_owned(),
        }])
    };
    assert_eq!(
        ComputerRow::of(&row(PlaceKind::OwnComputer, PlaceState::NotReady)),
        Some(ComputerRow {
            standing: Standing::Silent,
            ..lab()
        })
    );
    assert_eq!(
        ComputerRow::of(&row(PlaceKind::ThisComputer, PlaceState::Ready)),
        None
    );
    assert_eq!(
        ComputerRow::of(&row(PlaceKind::CloudAccount, PlaceState::Ready)),
        None
    );
}

#[test]
fn add_asks_who_is_there_and_lists_them() {
    let (opened, effects) = computers_step(ready(), Input::OpenAdd);
    assert_eq!(adding_of(&opened), &AddList::Looking);
    assert_eq!(effects, vec![Effect::LoadCandidates]);
    let (listed, _) = computers_step(opened, Input::Candidates(Ok(vec![candidate()])));
    assert_eq!(adding_of(&listed), &AddList::Choosing(vec![candidate()]));
}

#[test]
fn a_failed_look_closes_the_list_and_says_why() {
    let opened = computers_step(ready(), Input::OpenAdd).0;
    let (state, effects) = computers_step(opened, Input::Candidates(Err("no".to_owned())));
    assert_eq!(adding_of(&state), &AddList::Closed);
    assert_eq!(effects, vec![Effect::Tell("no".to_owned())]);
}

#[test]
fn candidates_that_arrive_after_done_are_dropped() {
    let opened = computers_step(ready(), Input::OpenAdd).0;
    let closed = computers_step(opened, Input::CloseAdd).0;
    let (state, _) = computers_step(closed, Input::Candidates(Ok(vec![candidate()])));
    assert_eq!(adding_of(&state), &AddList::Closed);
}

/// `(case, input, the effects it asks for)` from the list with Studio offered.
fn asks() -> Vec<(&'static str, Input, Vec<Effect>)> {
    vec![
        (
            "add asks for the node",
            Input::Add(node("n1")),
            vec![Effect::Add(node("n1"))],
        ),
        (
            "remove asks by computer name",
            Input::Remove(lab().computer),
            vec![Effect::Remove(lab().computer)],
        ),
        (
            "allow is passed on",
            Input::Answer(node("n2"), GuestAnswer::Allow),
            vec![Effect::Answer(node("n2"), GuestAnswer::Allow)],
        ),
        (
            "deny is passed on",
            Input::Answer(node("n2"), GuestAnswer::Deny),
            vec![Effect::Answer(node("n2"), GuestAnswer::Deny)],
        ),
        (
            "forget is passed on",
            Input::Forget(node("n2")),
            vec![Effect::Forget(node("n2"))],
        ),
    ]
}

#[test]
fn each_press_asks_the_service_once() {
    for (case, input, want) in asks() {
        assert_eq!(computers_step(choosing(), input).1, want, "{case}");
    }
}

#[test]
fn a_change_that_worked_reads_the_lists_and_the_candidates_again() {
    let (state, effects) = computers_step(choosing(), Input::Changed(Ok(())));
    assert_eq!(adding_of(&state), &AddList::Looking);
    assert_eq!(effects, vec![Effect::LoadCandidates, Effect::Reload]);
    let (_, effects) = computers_step(ready(), Input::Changed(Ok(())));
    assert_eq!(
        effects,
        vec![Effect::Reload],
        "no list is up, so none is read"
    );
}

#[test]
fn a_refused_change_is_told_and_the_lists_are_read_again() {
    let (_, effects) = computers_step(ready(), Input::Changed(Err("Nope.".to_owned())));
    assert_eq!(
        effects,
        vec![Effect::Tell("Nope.".to_owned()), Effect::Reload]
    );
}

#[test]
fn reading_again_keeps_the_add_list_and_a_missing_service_replaces_everything() {
    let found = ComputersFound {
        computers: vec![],
        guests: vec![],
    };
    let (kept, _) = computers_step(choosing(), Input::Found(Ok(found)));
    assert_eq!(adding_of(&kept), &AddList::Choosing(vec![candidate()]));
    let (gone, _) = computers_step(kept, Input::Found(Err(Absence::NotRunning)));
    assert_eq!(gone, State::Unavailable(Absence::NotRunning));
}

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
    for (state, line) in CASES {
        let guest = GuestRow::new(node("n2"), "laptop".to_owned(), *state, UnixSeconds(0));
        assert_eq!(guest_line(&guest), *line);
    }
}
