//! The "Your computers" group (its rows are the assistant's places, drawn by the Assistants block),
//! the Add list, and the group of computers asking to use this one's models.

use super::server_sheet::ServerSheetView;
use crate::assistants::{PlaceEntry, PlaceRow};
use crate::copy::{self, Subject, absence as absence_words, guest_standing};
use crate::env::Env;
use detent_model::{
    AddList, ComputersEffect, ComputersInput, ComputersReady, ComputersState, computers_step,
};
use detent_system::bus::{BusSource, Daemon, next_owner, owner_changes};
use detent_system::computers::{Change, ChangeFailure, ComputersLink};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_help::HelpShown;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::stack::toast_hub::ToastHub;
use ds::style::tokens::control_size::ControlSize;
use porter_core::lending::{Approval, ComputerCandidate, GuestAnswer, GuestRow, RowState};
use std::sync::Arc;

/// The group's title.
const TITLE: &str = "Your computers";

/// The title of the group of computers that asked to borrow this computer's models.
const GUESTS_TITLE: &str = "Computers asking to use this computer's models";

/// What the list says before a computer is added.
const NONE_YET: &str = "Computers you add appear here. Each starts off.";

/// What the Add a server row says it is for.
const ADD_SERVER_HELP: &str =
    "A model server you already run on this computer, such as one started by hand.";

/// What the Add list says when Tailscale knows no computer of the person's that lends.
const NOBODY_LENDING: &str = "None of your other computers is lending its models. On the other \
    computer, turn on \u{201c}Let my other computers use this computer\u{2019}s models\u{201d}.";

/// What the Add list says while Tailscale is asked.
const LOOKING: &str = "Looking for your computers on Tailscale\u{2026}";

fn change_of(effect: ComputersEffect) -> Option<Change> {
    match effect {
        ComputersEffect::Add(node) => Some(Change::Add(node)),
        ComputersEffect::Remove(id) => Some(Change::Remove(id)),
        ComputersEffect::Answer(node, answer) => Some(Change::Answer(node, answer)),
        ComputersEffect::Forget(node) => Some(Change::Forget(node)),
        ComputersEffect::AddServer(draft) => Some(Change::Server(draft)),
        ComputersEffect::Reload
        | ComputersEffect::LoadCandidates
        | ComputersEffect::LoadServerModels
        | ComputersEffect::Tell(_) => None,
    }
}

/// The sentence for a change that was not made: the form's own, or the service's.
fn sentence_of_failure(failure: ChangeFailure) -> String {
    match failure {
        ChangeFailure::Fault(fault) => copy::server_fault(fault).to_owned(),
        ChangeFailure::Told(words) => words,
    }
}

/// Steps the state on `input` and does what that asks; what each ask ends with is fed back in.
fn drive(
    mut state: Signal<ComputersState>,
    toasts: ToastHub,
    link: Arc<dyn ComputersLink>,
    input: ComputersInput,
) {
    let (next, effects) = computers_step(state.peek().clone(), input);
    state.set(next);
    for effect in effects {
        let link = link.clone();
        match effect {
            ComputersEffect::Tell(text) => toasts.push(text, None),
            ComputersEffect::Reload => {
                spawn(async move {
                    let found = link.read().await;
                    drive(state, toasts, link, ComputersInput::Found(found));
                });
            }
            ComputersEffect::LoadCandidates => {
                spawn(async move {
                    let found = link.candidates().await;
                    drive(state, toasts, link, ComputersInput::Candidates(found));
                });
            }
            ComputersEffect::LoadServerModels => {
                spawn(async move {
                    let found = link.models().await;
                    drive(state, toasts, link, ComputersInput::ServerModels(found));
                });
            }
            other => {
                if let Some(asked) = change_of(other) {
                    let server = matches!(asked, Change::Server(_));
                    spawn(async move {
                        let ended = link.change(asked).await.map_err(sentence_of_failure);
                        let input = if server {
                            ComputersInput::ServerAdded(ended)
                        } else {
                            ComputersInput::Changed(ended)
                        };
                        drive(state, toasts, link, input);
                    });
                }
            }
        }
    }
}

/// Reads the lists, then reads them again each time inferd says they changed or comes back.
async fn follow_lists(
    state: Signal<ComputersState>,
    toasts: ToastHub,
    bus: BusSource,
    link: Arc<dyn ComputersLink>,
) {
    let connection = match detent_system::bus::connect(&bus).await {
        Ok(connection) => connection,
        Err(absence) => return drive(state, toasts, link, ComputersInput::Found(Err(absence))),
    };
    let Ok(mut owners) = owner_changes(&connection, &Daemon::inferd()).await else {
        return;
    };
    loop {
        // Subscribed before the first read, so a change in between is not lost.
        let watch = link.changes().await;
        reread(state, toasts, &link).await;
        if let Ok(mut watch) = watch {
            while watch.recv().await.is_some() {
                reread(state, toasts, &link).await;
            }
        }
        if next_owner(&mut owners).await.is_none() {
            return;
        }
    }
}

/// Reads the lists and steps the state on what was found.
async fn reread(state: Signal<ComputersState>, toasts: ToastHub, link: &Arc<dyn ComputersLink>) {
    let found = link.read().await;
    drive(state, toasts, link.clone(), ComputersInput::Found(found));
}

/// Follows the computers and hands back their state and the way to feed it an input.
pub(crate) fn use_computers() -> (Signal<ComputersState>, EventHandler<ComputersInput>) {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(ComputersState::default);
    use_future({
        let (bus, link) = (env.bus.clone(), env.computers.clone());
        move || follow_lists(state, toasts, bus.clone(), link.clone())
    });
    let link = env.computers.clone();
    let feed = EventHandler::new(move |input: ComputersInput| {
        drive(state, toasts, link.clone(), input);
    });
    (state, feed)
}

/// The person's computers (`rows`, one per computer the service lists), the Add list, and the
/// guests.
#[component]
pub(crate) fn Computers(
    state: ComputersState,
    feed: EventHandler<ComputersInput>,
    rows: Vec<PlaceEntry>,
) -> Element {
    match state {
        ComputersState::Looking => rsx! {},
        ComputersState::Unavailable(absence) => rsx! {
            div { class: "computers", "data-computers": "unavailable",
                FieldGroup { title: Some(TITLE.to_owned()),
                    FieldRow {
                        label: TextLine::from(TITLE),
                        help: Some(TextLine::from(absence_words(Subject::Computers, absence))),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        ComputersState::Ready(ready) => rsx! { Listed { ready, feed, rows } },
    }
}

#[component]
fn Listed(
    ready: ComputersReady,
    feed: EventHandler<ComputersInput>,
    rows: Vec<PlaceEntry>,
) -> Element {
    let add_list = match ready.adding {
        AddList::Closed => Availability::Enabled,
        AddList::Looking | AddList::Choosing(_) => Availability::Disabled,
    };
    rsx! {
        div { class: "computers", "data-computers": "list",
            FieldGroup { title: Some(TITLE.to_owned()),
                if rows.is_empty() {
                    FieldRow { label: TextLine::from(NONE_YET), availability: Availability::Disabled,
                        Label { text: "None", role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                    }
                }
                for row in rows {
                    PlaceRow { key: "{row.view.name}", view: row.view, on: row.on }
                }
                FieldRow { label: TextLine::from("Add a server"),
                    help: Some(TextLine::from(ADD_SERVER_HELP)),
                    help_shown: HelpShown::OnRequest,
                    Button {
                        common: Common { aria_label: Some("Add a server".to_owned()), id: Some("add-server".to_owned()), ..Common::default() },
                        label: "Add a server\u{2026}",
                        size: ControlSize::Small,
                        onclick: move |_| feed.call(ComputersInput::OpenServer),
                    }
                }
                FieldRow { label: TextLine::from("Add a computer"),
                    Button {
                        common: Common { aria_label: Some("Add a computer".to_owned()), ..Common::default() },
                        label: "Add\u{2026}",
                        size: ControlSize::Small,
                        availability: add_list,
                        onclick: move |_| feed.call(ComputersInput::OpenAdd),
                    }
                }
            }
            if let Some(sheet) = ready.server.clone() {
                ServerSheetView { sheet, feed }
            }
            match ready.adding.clone() {
                AddList::Closed => rsx! {},
                AddList::Looking => rsx! {
                    FieldGroup { title: Some("Add a computer".to_owned()),
                        FieldRow { label: TextLine::from(LOOKING), availability: Availability::Disabled }
                    }
                },
                AddList::Choosing(found) => rsx! { Candidates { found, feed } },
            }
            if !ready.guests.is_empty() {
                FieldGroup { title: Some(GUESTS_TITLE.to_owned()),
                    for guest in ready.guests.iter().cloned() {
                        GuestLine { key: "{guest.node}", guest, feed }
                    }
                }
            }
        }
    }
}

#[component]
fn Candidates(found: Vec<ComputerCandidate>, feed: EventHandler<ComputersInput>) -> Element {
    rsx! {
        FieldGroup { title: Some("Add a computer".to_owned()),
            if found.is_empty() {
                FieldRow { label: TextLine::from(NOBODY_LENDING), availability: Availability::Disabled }
            }
            for candidate in found {
                CandidateLine { key: "{candidate.node}", candidate, feed }
            }
            FieldRow { label: TextLine::from("Done"),
                Button {
                    label: "Done",
                    size: ControlSize::Small,
                    onclick: move |_| feed.call(ComputersInput::CloseAdd),
                }
            }
        }
    }
}

#[component]
fn CandidateLine(candidate: ComputerCandidate, feed: EventHandler<ComputersInput>) -> Element {
    let models = candidate
        .models
        .iter()
        .map(|model| model.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let help = match candidate.approval {
        Approval::Given => models,
        Approval::Needed => format!("{models}. Say yes on {} when it asks.", candidate.name),
    };
    let node = candidate.node.clone();
    rsx! {
        FieldRow { label: TextLine::from(candidate.name.clone()), help: Some(TextLine::from(help)), help_shown: HelpShown::OnRequest,
            Button {
                common: Common { aria_label: Some(format!("Add {}", candidate.name)), ..Common::default() },
                label: "Add",
                size: ControlSize::Small,
                onclick: move |_| feed.call(ComputersInput::Add(node.clone())),
            }
        }
    }
}

#[component]
fn GuestLine(guest: GuestRow, feed: EventHandler<ComputersInput>) -> Element {
    let allow = guest.state != RowState::Approved;
    let deny = guest.state != RowState::Denied;
    let (a, b, c) = (guest.node.clone(), guest.node.clone(), guest.node.clone());
    rsx! {
        FieldRow { label: TextLine::from(guest.name.clone()), help: Some(TextLine::from(guest_standing(guest.state))),
            if allow {
                GuestButton { label: "Allow", subject: guest.name.clone(),
                    onpress: move |()| feed.call(ComputersInput::Answer(a.clone(), GuestAnswer::Allow)) }
            }
            if deny {
                GuestButton { label: "Don\u{2019}t Allow", subject: guest.name.clone(),
                    onpress: move |()| feed.call(ComputersInput::Answer(b.clone(), GuestAnswer::Deny)) }
            }
            GuestButton { label: "Forget", subject: guest.name.clone(),
                onpress: move |()| feed.call(ComputersInput::Forget(c.clone())) }
        }
    }
}

#[component]
fn GuestButton(label: String, subject: String, onpress: EventHandler<()>) -> Element {
    rsx! {
        Button {
            common: Common { aria_label: Some(format!("{label} {subject}")), ..Common::default() },
            label,
            size: ControlSize::Small,
            onclick: move |_| onpress.call(()),
        }
    }
}
