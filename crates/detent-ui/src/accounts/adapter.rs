//! The pieces of the Accounts pane that quire's R2a release changes (button roles, a controlled
//! plain text field, initial focus on an alert). Every use of them in the pane goes through here,
//! so the switch to R2a is these few lines and nothing else.

use detent_model::{
    Adding, Ask, ClientValue, HoldCounts, KEEP_HELP, LocalFiles, RemovalQuestion, SecretStanding,
    SyncHold,
};
use dioxus::prelude::*;
use ds::components::controls::button::Button;
use ds::components::controls::button_model::{Answers, Bezel, ButtonRole, ImagePosition};
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::components::overlays::inline_banner::InlineBanner;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::ActionWeight;

/// What an action's weight does to its button.
fn role_of(weight: ActionWeight) -> ButtonRole {
    match weight {
        ActionWeight::Plain => ButtonRole::Normal,
        ActionWeight::Destructive => ButtonRole::Destructive,
    }
}

/// The name a screen reader says for a button that acts on `subject`: "Revoke Photos allowed:
/// storage". Several rows carry the same visible word; this says which one it is.
fn spoken(label: &str, subject: &str) -> Common {
    Common {
        aria_label: Some(if subject.is_empty() {
            label.to_owned()
        } else {
            format!("{label} {subject}")
        }),
        ..Common::default()
    }
}

/// A button for an action key, named by the thing it acts on. A destructive one is drawn as such;
/// it never acts on its own press (the pane puts a question between the press and the action).
#[component]
pub(super) fn ActionButton(
    label: String,
    subject: String,
    weight: ActionWeight,
    availability: Availability,
    onpress: EventHandler<()>,
) -> Element {
    rsx! {
        Button {
            common: spoken(&label, &subject),
            label,
            role: role_of(weight),
            size: ControlSize::Small,
            availability,
            onclick: move |_| onpress.call(()),
        }
    }
}

/// The quiet button on an account's row that opens the account, named by it ("Manage Home
/// cloud").
#[component]
pub(crate) fn OpenButton(label: String, subject: String, onpress: EventHandler<()>) -> Element {
    rsx! {
        Button {
            common: spoken(&label, &subject),
            label,
            bezel: Bezel::Inline,
            size: ControlSize::Small,
            onclick: move |_| onpress.call(()),
        }
    }
}

/// The question before an account is removed or an assistant signed out: a critical alert whose
/// destructive button is never the default, so Return keeps the account signed in.
#[component]
pub(super) fn RemoveAlert(
    question: RemovalQuestion,
    remove_name: String,
    onanswer: EventHandler<bool>,
) -> Element {
    let (title, message, keep, action) = match question.ask {
        Ask::Remove => (
            format!("{remove_name} {}?", question.account),
            match question.files {
                LocalFiles::Kept => {
                    "Its stored credentials and every permission given for it are deleted. \
                     Files kept on this computer for the account are deleted too. \
                     It cannot be undone."
                }
                LocalFiles::None => {
                    "Its stored credentials and every permission given for it are deleted. \
                     It cannot be undone."
                }
            },
            "Keep",
            remove_name.clone(),
        ),
        Ask::SignOut => (
            format!("Sign out of {}?", question.account),
            "You'll need to sign in again to use it.",
            "Cancel",
            "Sign Out".to_owned(),
        ),
    };
    rsx! {
        Alert {
            title,
            message: Some(TextLine::from(message)),
            style: AlertStyle::Critical,
            buttons: vec![
                AlertButton::new(keep, AlertRole::Cancel, EventHandler::new(move |()| onanswer.call(false))),
                AlertButton::new(action, AlertRole::Destructive, EventHandler::new(move |()| onanswer.call(true))),
            ],
        }
    }
}

/// A text field for a sign-in key or secret. A key holds what is typed until the caret leaves or
/// Enter is pressed, and is made anew when the stored value changes (the caller keys it on the
/// value). A secret is write-only: it shows "Set" or "Not set", never what is stored, and what is
/// typed replaces it. Either waits (greyed) until its key is set, when it is a secret.
#[component]
pub(super) fn ClientField(
    label: String,
    value: ClientValue,
    availability: Availability,
    onchange: EventHandler<String>,
) -> Element {
    match value {
        ClientValue::Text(text) => rsx! {
            KeyField { label, value: text, availability, onchange }
        },
        ClientValue::Secret(standing) => rsx! {
            SecretField { label, standing, availability, onchange }
        },
    }
}

#[component]
fn KeyField(
    label: String,
    value: String,
    availability: Availability,
    onchange: EventHandler<String>,
) -> Element {
    let mut draft = use_signal(|| value.clone());
    rsx! {
        TextField {
            label,
            value: draft(),
            availability,
            size: ControlSize::Small,
            oninput: move |next| draft.set(next),
            onchange: move |next: String| onchange.call(next),
        }
    }
}

/// A secure field that starts empty with the secret's standing where the text would be. What is
/// typed is handed on when the caret leaves or Enter is pressed, and the field is made anew, so
/// the text is not drawn again.
#[component]
fn SecretField(
    label: String,
    standing: SecretStanding,
    availability: Availability,
    onchange: EventHandler<String>,
) -> Element {
    let mut made = use_signal(|| 0_u32);
    rsx! {
        Fragment { key: "{made}",
            TextField {
                label,
                value: String::new(),
                placeholder: standing.label().to_owned(),
                kind: FieldKind::Secure,
                availability,
                size: ControlSize::Small,
                oninput: move |_: String| {},
                onchange: move |next: String| {
                    made += 1;
                    onchange.call(next);
                },
            }
        }
    }
}

/// The plus under the list, which opens the add sheet; it waits while one is open or a change
/// is in flight.
#[component]
pub(super) fn AddAccountButton(
    adding: Adding,
    availability: Availability,
    onpress: EventHandler<()>,
) -> Element {
    let availability = match adding {
        Adding::Idle => availability,
        Adding::SheetOpen => Availability::Disabled,
    };
    rsx! {
        Button {
            label: "Add Account\u{2026}",
            title: Some("Add Account".to_owned()),
            icon: Some(Icon::Plus),
            image: ImagePosition::Only,
            size: ControlSize::Small,
            availability,
            onclick: move |_| onpress.call(()),
        }
    }
}

/// The button on the AI keys' row, which opens the key service's add sheet; it waits while a
/// sheet is open or a change is in flight.
#[component]
pub(super) fn AddKeyButton(
    adding: Adding,
    availability: Availability,
    onpress: EventHandler<()>,
) -> Element {
    let availability = match adding {
        Adding::Idle => availability,
        Adding::SheetOpen => Availability::Disabled,
    };
    rsx! {
        Button {
            common: spoken("Add AI Key\u{2026}", ""),
            label: "Add AI Key\u{2026}",
            size: ControlSize::Small,
            availability,
            onclick: move |_| onpress.call(()),
        }
    }
}

/// The minus under the list, which asks to remove the selected account.
#[component]
pub(super) fn RemoveAccountButton(
    availability: Availability,
    onpress: EventHandler<()>,
) -> Element {
    rsx! {
        Button {
            label: "Remove Account",
            title: Some("Remove Account".to_owned()),
            icon: Some(Icon::Minus),
            image: ImagePosition::Only,
            size: ControlSize::Small,
            availability,
            onclick: move |_| onpress.call(()),
        }
    }
}

/// The banner on a sync row whose deletion waits for the person: Keep (the default, asks nothing
/// of anyone) and Remove (destructive, never the default).
#[component]
pub(super) fn HoldBanner(
    hold: SyncHold,
    onkeep: EventHandler<()>,
    onremove: EventHandler<()>,
) -> Element {
    let HoldCounts { held, .. } = hold.counts;
    rsx! {
        InlineBanner {
            severity: Severity::Warn,
            text: TextLine::from(hold.banner()),
            detail: Some(TextLine::from(KEEP_HELP)),
            common: Common {
                aria_label: Some(format!("{held} files removed on the server")),
                ..Common::default()
            },
            actions: rsx! {
                Button {
                    label: "Keep",
                    size: ControlSize::Small,
                    answers: Answers::Return,
                    onclick: move |_| onkeep.call(()),
                }
                Button {
                    label: "Remove",
                    role: ButtonRole::Destructive,
                    size: ControlSize::Small,
                    onclick: move |_| onremove.call(()),
                }
            },
        }
    }
}
