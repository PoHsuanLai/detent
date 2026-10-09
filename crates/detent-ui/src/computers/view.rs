//! The "Your computers" group and the group of computers asking to use this one's models.

use super::io::{Change, Watch, candidates, change, read};
use crate::env::Env;
use crate::live::{BusSource, Daemon, next_owner, owner_changes};
use detent_model::{
    Absence, AddList, ComputerRow, ComputersEffect, ComputersInput, ComputersReady, ComputersState,
    Standing, computers_step, guest_line,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::stack::toast_hub::ToastHub;
use ds::style::tokens::control_size::ControlSize;
use porter_core::lending::{Approval, ComputerCandidate, GuestAnswer, GuestRow, RowState};

/// The group's title.
const TITLE: &str = "Your computers";

/// The title of the group of computers that asked to borrow this computer's models.
const GUESTS_TITLE: &str = "Computers asking to use this computer's models";

/// What the list says before a computer is added.
const NONE_YET: &str = "Computers you add appear here. The assistant can use their models.";

/// What the Add list says when Tailscale knows no computer of the person's that lends.
const NOBODY_LENDING: &str = "None of your other computers is lending its models. On the other \
    computer, turn on \u{201c}Let my other computers use this computer\u{2019}s models\u{201d}.";

/// What the Add list says while Tailscale is asked.
const LOOKING: &str = "Looking for your computers on Tailscale\u{2026}";

/// What the absence of the assistant says here.
fn absence_help(absence: &Absence) -> &'static str {
    match absence {
        Absence::NoBus => {
            "Settings is not connected to the desktop, so your computers cannot be listed."
        }
        Absence::NotRunning => "The assistant is not running, so your computers cannot be listed.",
        Absence::NotPermitted => {
            "Your computers can only be managed when Settings is opened from the desktop."
        }
        Absence::Failed(_) => "The assistant did not answer. Try again in a moment.",
    }
}

fn change_of(effect: ComputersEffect) -> Option<Change> {
    match effect {
        ComputersEffect::Add(node) => Some(Change::Add(node)),
        ComputersEffect::Remove(id) => Some(Change::Remove(id)),
        ComputersEffect::Answer(node, answer) => Some(Change::Answer(node, answer)),
        ComputersEffect::Forget(node) => Some(Change::Forget(node)),
        ComputersEffect::Reload | ComputersEffect::LoadCandidates | ComputersEffect::Tell(_) => {
            None
        }
    }
}

/// Steps the state on `input` and does what that asks; what each ask ends with is fed back in.
fn drive(
    mut state: Signal<ComputersState>,
    toasts: ToastHub,
    bus: BusSource,
    input: ComputersInput,
) {
    let (next, effects) = computers_step(state.peek().clone(), input);
    state.set(next);
    for effect in effects {
        let bus = bus.clone();
        match effect {
            ComputersEffect::Tell(text) => toasts.push(text, None),
            ComputersEffect::Reload => {
                spawn(async move {
                    let found = read(&bus).await;
                    drive(state, toasts, bus, ComputersInput::Found(found));
                });
            }
            ComputersEffect::LoadCandidates => {
                spawn(async move {
                    let found = candidates(&bus).await;
                    drive(state, toasts, bus, ComputersInput::Candidates(found));
                });
            }
            other => {
                if let Some(asked) = change_of(other) {
                    spawn(async move {
                        let ended = change(&bus, asked).await;
                        drive(state, toasts, bus, ComputersInput::Changed(ended));
                    });
                }
            }
        }
    }
}

/// Reads the lists, then reads them again each time inferd says they changed or comes back.
async fn follow_lists(state: Signal<ComputersState>, toasts: ToastHub, bus: BusSource) {
    let connection = match crate::live::connect(&bus).await {
        Ok(connection) => connection,
        Err(absence) => return drive(state, toasts, bus, ComputersInput::Found(Err(absence))),
    };
    let Ok(mut owners) = owner_changes(&connection, &Daemon::inferd()).await else {
        return;
    };
    loop {
        // Subscribed before the first read, so a change in between is not lost.
        let watch = Watch::start(&bus).await;
        reread(state, toasts, &bus).await;
        if let Ok(mut watch) = watch {
            while watch.changed().await.is_some() {
                reread(state, toasts, &bus).await;
            }
        }
        if next_owner(&mut owners).await.is_none() {
            return;
        }
    }
}

/// Reads the lists and steps the state on what was found.
async fn reread(state: Signal<ComputersState>, toasts: ToastHub, bus: &BusSource) {
    let found = read(bus).await;
    drive(state, toasts, bus.clone(), ComputersInput::Found(found));
}

/// The person's computers, the Add list, and the guests.
#[component]
pub(crate) fn Computers() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(ComputersState::default);
    use_future({
        let bus = env.bus.clone();
        move || follow_lists(state, toasts, bus.clone())
    });
    let bus = env.bus.clone();
    let feed = EventHandler::new(move |input: ComputersInput| {
        drive(state, toasts, bus.clone(), input);
    });
    match state() {
        ComputersState::Looking => rsx! {},
        ComputersState::Unavailable(absence) => rsx! {
            div { class: "computers", "data-computers": "unavailable",
                FieldGroup { title: Some(TITLE.to_owned()),
                    FieldRow {
                        label: TextLine::from(TITLE),
                        help: Some(TextLine::from(absence_help(&absence))),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        ComputersState::Ready(ready) => rsx! { Listed { ready, feed } },
    }
}

#[component]
fn Listed(ready: ComputersReady, feed: EventHandler<ComputersInput>) -> Element {
    let add_list = match ready.adding {
        AddList::Closed => Availability::Enabled,
        AddList::Looking | AddList::Choosing(_) => Availability::Disabled,
    };
    rsx! {
        div { class: "computers", "data-computers": "list",
            FieldGroup { title: Some(TITLE.to_owned()),
                if ready.computers.is_empty() {
                    FieldRow { label: TextLine::from(NONE_YET), availability: Availability::Disabled,
                        Label { text: "None", role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                    }
                }
                for computer in ready.computers.iter().cloned() {
                    ComputerLine { key: "{computer.computer}", computer, feed }
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
fn ComputerLine(computer: ComputerRow, feed: EventHandler<ComputersInput>) -> Element {
    let help = if computer.models.is_empty() {
        "No models right now".to_owned()
    } else {
        computer.models.join(", ")
    };
    let standing = match computer.standing {
        Standing::Answering => "Ready",
        Standing::Silent => "Not answering",
    };
    let computer_id = computer.computer.clone();
    rsx! {
        FieldRow { label: TextLine::from(computer.name.clone()), help: Some(TextLine::from(help)),
            Label { text: standing, role: LabelRole::Secondary, style: LabelStyle::Footnote }
            Button {
                common: Common { aria_label: Some(format!("Remove {}", computer.name)), ..Common::default() },
                label: "Remove",
                size: ControlSize::Small,
                onclick: move |_| feed.call(ComputersInput::Remove(computer_id.clone())),
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
        FieldRow { label: TextLine::from(candidate.name.clone()), help: Some(TextLine::from(help)),
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
        FieldRow { label: TextLine::from(guest.name.clone()), help: Some(TextLine::from(guest_line(&guest))),
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
