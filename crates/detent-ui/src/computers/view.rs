//! The "Your computers" group (its rows are the assistant's places, drawn by the Assistants block),
//! the Add list, and the group of computers asking to use this one's models.

use super::server_sheet::ServerSheetView;
use crate::assistants::{PlaceEntry, PlaceRow};
use crate::copy::absence::{Subject, absence as absence_words};
use crate::copy::computers::guest_standing;
use crate::live::use_live;
use detent_model::external::{Approval, ComputerCandidate, GuestAnswer, GuestRow, RowState};
use detent_model::{AddList, ComputersInput, ComputersReady, ComputersState};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_help::HelpShown;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The group's title.
const TITLE: &str = crate::copy::computers::YOUR_COMPUTERS;

/// The title of the group of computers that asked to borrow this computer's models.
const GUESTS_TITLE: &str = crate::copy::computers::COMPUTERS_ASKING_TO_USE_THIS_COMPUTER;

/// What the list says before a computer is added.
const NONE_YET: &str = crate::copy::computers::COMPUTERS_YOU_ADD_APPEAR_HERE_EACH;

/// What the Add a server row says it is for.
const ADD_SERVER_HELP: &str = crate::copy::computers::A_MODEL_SERVER_YOU_ALREADY_RUN;

/// What the Add list says when Tailscale knows no computer of the person's that lends.
const NOBODY_LENDING: &str = crate::copy::computers::NONE_OF_YOUR_OTHER_COMPUTERS_IS;

/// What the Add list says while Tailscale is asked.
const LOOKING: &str = crate::copy::computers::LOOKING_FOR_YOUR_COMPUTERS_ON_TAILSCALE;

/// Follows the computers and hands back their state and the way to feed it an input.
pub(crate) fn use_computers() -> (Signal<ComputersState>, EventHandler<ComputersInput>) {
    let computers = use_live(|client| client.computers());
    let pane = computers.pane.clone();
    let feed = EventHandler::new(move |input: ComputersInput| {
        spawn(pane.send(input));
    });
    (computers.state, feed)
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
                FieldRow { label: TextLine::from(crate::copy::computers::ADD_A_SERVER),
                    help: Some(TextLine::from(ADD_SERVER_HELP)),
                    help_shown: HelpShown::OnRequest,
                    Button {
                        common: Common { aria_label: Some(crate::copy::computers::ADD_A_SERVER.to_owned()), id: Some("add-server".to_owned()), ..Common::default() },
                        label: crate::copy::computers::ADD_A_SERVER_ELLIPSIS,
                        size: ControlSize::Small,
                        onclick: move |_| feed.call(ComputersInput::OpenServer),
                    }
                }
                FieldRow { label: TextLine::from(crate::copy::computers::ADD_A_COMPUTER),
                    Button {
                        common: Common { aria_label: Some(crate::copy::computers::ADD_A_COMPUTER.to_owned()), ..Common::default() },
                        label: crate::copy::common::ADD_ELLIPSIS,
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
                    FieldGroup { title: Some(crate::copy::computers::ADD_A_COMPUTER.to_owned()),
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
        FieldGroup { title: Some(crate::copy::computers::ADD_A_COMPUTER.to_owned()),
            if found.is_empty() {
                FieldRow { label: TextLine::from(NOBODY_LENDING), availability: Availability::Disabled }
            }
            for candidate in found {
                CandidateLine { key: "{candidate.node}", candidate, feed }
            }
            FieldRow { label: TextLine::from(crate::copy::common::DONE),
                Button {
                    label: crate::copy::common::DONE,
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
        Approval::Needed => {
            crate::copy::computers::say_yes_on_when_it_asks(&models, &candidate.name)
        }
    };
    let node = candidate.node.clone();
    rsx! {
        FieldRow { label: TextLine::from(candidate.name.clone()), help: Some(TextLine::from(help)), help_shown: HelpShown::OnRequest,
            Button {
                common: Common { aria_label: Some(crate::copy::computers::add(&candidate.name)), ..Common::default() },
                label: crate::copy::common::ADD,
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
                GuestButton { label: crate::copy::computers::ALLOW, subject: guest.name.clone(),
                    onpress: move |()| feed.call(ComputersInput::Answer(a.clone(), GuestAnswer::Allow)) }
            }
            if deny {
                GuestButton { label: crate::copy::computers::DONT_ALLOW, subject: guest.name.clone(),
                    onpress: move |()| feed.call(ComputersInput::Answer(b.clone(), GuestAnswer::Deny)) }
            }
            GuestButton { label: crate::copy::common::FORGET, subject: guest.name.clone(),
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
