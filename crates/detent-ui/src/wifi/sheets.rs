//! Two sheets over the Wi-Fi page: the password (or account) of a network, and the details of the
//! one this computer is on.

use detent_model::{
    Account, AskKind, Passphrase, PasswordAsk, WifiInput, WifiNetwork, details_rows, password_help,
    password_title,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// Asks for the password of a network, or the username and password of one that signs in: Join
/// waits until what was typed could be one.
#[component]
pub(super) fn PasswordSheet(ask: PasswordAsk, press: EventHandler<WifiInput>) -> Element {
    let mut typed = use_signal(String::new);
    let mut username = use_signal(String::new);
    let title = password_title(&ask.ssid.0, ask.kind);
    let close = EventHandler::new(move |()| press.call(WifiInput::CloseSheet));
    let (ssid, kind) = (ask.ssid.clone(), ask.kind);
    let join = EventHandler::new(move |()| match kind {
        AskKind::Password => {
            let passphrase = Passphrase::new(&typed.peek());
            if passphrase.is_long_enough() {
                press.call(WifiInput::Password(ssid.clone(), passphrase));
            }
        }
        AskKind::Account => {
            if let Some(account) = Account::new(&username.peek(), &typed.peek()) {
                press.call(WifiInput::SignIn(ssid.clone(), account));
            }
        }
    });
    let ready = match kind {
        AskKind::Password => Passphrase::new(&typed()).is_long_enough(),
        AskKind::Account => Account::new(&username(), &typed()).is_some(),
    };
    rsx! {
        Sheet {
            label: title.clone(),
            width: SheetWidth::Narrow,
            onclose: close,
            on_return: join,
            div { class: "system-sheet", "data-system": "password",
                Label { text: title, role: LabelRole::Primary, style: LabelStyle::Headline }
                Label { text: password_help(ask.standing, kind), role: LabelRole::Secondary }
                if kind == AskKind::Account {
                    TextField {
                        label: "Username",
                        value: username(),
                        kind: FieldKind::Plain,
                        size: ControlSize::Small,
                        common: Common { id: Some("wifi-username".to_owned()), ..Common::default() },
                        oninput: move |text: String| username.set(text),
                        onsubmit: move |_: String| join.call(()),
                    }
                }
                TextField {
                    label: "Password",
                    value: String::new(),
                    kind: FieldKind::Secure,
                    size: ControlSize::Small,
                    common: Common { id: Some("wifi-password".to_owned()), ..Common::default() },
                    oninput: move |text: String| typed.set(text),
                    onsubmit: move |_: String| join.call(()),
                }
                div { class: "system-sheet-buttons",
                    Button {
                        label: "Cancel",
                        common: Common { aria_label: Some("Cancel".to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        onclick: move |_| close.call(()),
                    }
                    Button {
                        label: "Join",
                        common: Common { aria_label: Some("Join".to_owned()), ..Common::default() },
                        answers: Answers::Return,
                        availability: if ready { Availability::Enabled } else { Availability::Disabled },
                        onclick: move |_| join.call(()),
                    }
                }
            }
        }
    }
}

/// The name, security and signal of the network this computer is on, with Done.
#[component]
pub(super) fn DetailsSheet(network: WifiNetwork, press: EventHandler<WifiInput>) -> Element {
    let close = EventHandler::new(move |()| press.call(WifiInput::CloseSheet));
    let title = network.ssid.0.clone();
    rsx! {
        Sheet {
            label: title.clone(),
            width: SheetWidth::Narrow,
            onclose: close,
            on_return: close,
            div { class: "system-sheet", "data-system": "details",
                Label { text: title, role: LabelRole::Primary, style: LabelStyle::Headline }
                for (label, value) in details_rows(&network) {
                    div { key: "{label}", class: "system-detail",
                        Label { text: label, role: LabelRole::Secondary }
                        Label { text: value, role: LabelRole::Primary }
                    }
                }
                div { class: "system-sheet-buttons",
                    Button {
                        label: "Done",
                        common: Common { aria_label: Some("Done".to_owned()), ..Common::default() },
                        answers: Answers::Return,
                        onclick: move |_| close.call(()),
                    }
                }
            }
        }
    }
}
