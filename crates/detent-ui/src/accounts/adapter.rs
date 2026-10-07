//! The pieces of the Accounts pane that quire's R2a release changes (button roles, a controlled
//! plain text field, initial focus on an alert). Every use of them in the pane goes through here,
//! so the switch to R2a is these few lines and nothing else.

use detent_model::{Ask, RemovalQuestion};
use dioxus::prelude::*;
use ds::components::controls::button::Button;
use ds::components::controls::button_model::{Bezel, ButtonRole};
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::ActionWeight;

/// What an action's weight does to its button.
fn role_of(weight: ActionWeight) -> ButtonRole {
    match weight {
        ActionWeight::Plain => ButtonRole::Normal,
        ActionWeight::Destructive => ButtonRole::Destructive,
    }
}

/// A button for an action key. A destructive one is drawn as such; it never acts on its own
/// press (the pane puts a question between the press and the action).
#[component]
pub(super) fn ActionButton(
    label: String,
    weight: ActionWeight,
    availability: Availability,
    onpress: EventHandler<()>,
) -> Element {
    rsx! {
        Button {
            label,
            role: role_of(weight),
            size: ControlSize::Small,
            availability,
            onclick: move |_| onpress.call(()),
        }
    }
}

/// The quiet button on an account's row that opens the account.
#[component]
pub(crate) fn OpenButton(label: String, onpress: EventHandler<()>) -> Element {
    rsx! {
        Button {
            label,
            bezel: Bezel::Inline,
            size: ControlSize::Small,
            onclick: move |_| onpress.call(()),
        }
    }
}

/// The button that goes back to the list.
#[component]
pub(super) fn BackButton(label: String, onpress: EventHandler<()>) -> Element {
    rsx! {
        Button {
            label,
            icon: Some(Icon::ChevronLeft),
            bezel: Bezel::Inline,
            size: ControlSize::Small,
            onclick: move |_| onpress.call(()),
        }
    }
}

/// The question before an account is removed or a coding agent signed out: a critical alert whose
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
            "Its stored credentials and every permission given for it are deleted. \
             It cannot be undone.",
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

/// A text field for a client id: it holds what is typed until the caret leaves or Enter is
/// pressed, and is made anew when the stored value changes (the caller keys it on the value).
#[component]
pub(super) fn ClientField(label: String, value: String, onchange: EventHandler<String>) -> Element {
    let mut draft = use_signal(|| value.clone());
    rsx! {
        TextField {
            label,
            value: draft(),
            size: ControlSize::Small,
            oninput: move |next| draft.set(next),
            onchange: move |next: String| onchange.call(next),
        }
    }
}
