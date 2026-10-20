//! The sheets that forget a network, and join one that does not announce its name.

use detent_model::{
    HiddenForm, HiddenSecurity, OTHER_HELP, OTHER_TITLE, WifiInput, forget_network_body,
    forget_network_title,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::{Answers, ButtonRole};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// Asks before a network is forgotten.
#[component]
pub(super) fn ForgetSheet(name: String, press: EventHandler<WifiInput>) -> Element {
    let close = EventHandler::new(move |()| press.call(WifiInput::CloseSheet));
    let title = forget_network_title(&name);
    rsx! {
        Sheet {
            label: title.clone(),
            width: SheetWidth::Narrow,
            onclose: close,
            div { class: "system-sheet", "data-system": "forget",
                Label { text: title, role: LabelRole::Primary, style: LabelStyle::Headline }
                Label { text: forget_network_body(), role: LabelRole::Secondary }
                div { class: "system-sheet-buttons",
                    Button {
                        label: "Cancel",
                        common: Common { aria_label: Some("Cancel".to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        onclick: move |_| close.call(()),
                    }
                    Button {
                        label: "Forget",
                        common: Common { aria_label: Some("Forget".to_owned()), ..Common::default() },
                        role: ButtonRole::Destructive,
                        onclick: move |_| press.call(WifiInput::ConfirmForget),
                    }
                }
            }
        }
    }
}

/// Changes the form the sheet holds.
fn edit(mut form: Signal<HiddenForm>, change: impl FnOnce(HiddenForm) -> HiddenForm) {
    let next = change(form.peek().clone());
    form.set(next);
}

/// The name, security and password of a network that does not announce itself: Join waits until
/// the form could be a network.
#[component]
pub(super) fn OtherSheet(press: EventHandler<WifiInput>) -> Element {
    let form = use_signal(HiddenForm::default);
    let close = EventHandler::new(move |()| press.call(WifiInput::CloseSheet));
    let join = EventHandler::new(move |()| {
        let network = form.peek().network();
        if let Some(network) = network {
            press.call(WifiInput::JoinOther(network));
        }
    });
    let shown = form();
    let security = shown.security;
    let ready = shown.network().is_some();
    rsx! {
        Sheet {
            label: OTHER_TITLE.to_owned(),
            width: SheetWidth::Narrow,
            onclose: close,
            on_return: join,
            div { class: "system-sheet", "data-system": "other",
                Label { text: OTHER_TITLE, role: LabelRole::Primary, style: LabelStyle::Headline }
                Label { text: OTHER_HELP, role: LabelRole::Secondary }
                TextField {
                    label: "Network Name",
                    value: shown.name.clone(),
                    kind: FieldKind::Plain,
                    size: ControlSize::Small,
                    common: Common { id: Some("wifi-name".to_owned()), ..Common::default() },
                    oninput: move |name: String| edit(form, |form| HiddenForm { name, ..form }),
                    onsubmit: move |_: String| join.call(()),
                }
                PopUpButton::<String> {
                    items: HiddenSecurity::ALL
                        .iter()
                        .map(|choice| MenuItem::new(choice.words().to_owned(), choice.words()))
                        .collect::<Vec<_>>(),
                    value: Some(security.words().to_owned()),
                    size: ControlSize::Small,
                    common: Common { id: Some("wifi-security".to_owned()), ..Common::default() },
                    onpick: move |word: String| {
                        if let Some(security) = HiddenSecurity::ALL
                            .into_iter()
                            .find(|choice| choice.words() == word)
                        {
                            edit(form, |form| HiddenForm { security, ..form });
                        }
                    },
                }
                if security.takes_username() {
                    TextField {
                        label: "Username",
                        value: shown.username.clone(),
                        kind: FieldKind::Plain,
                        size: ControlSize::Small,
                        common: Common { id: Some("wifi-username".to_owned()), ..Common::default() },
                        oninput: move |username: String| edit(form, |form| HiddenForm { username, ..form }),
                        onsubmit: move |_: String| join.call(()),
                    }
                }
                if security.takes_password() {
                    TextField {
                        label: "Password",
                        value: String::new(),
                        kind: FieldKind::Secure,
                        size: ControlSize::Small,
                        common: Common { id: Some("wifi-password".to_owned()), ..Common::default() },
                        oninput: move |password: String| edit(form, |form| HiddenForm { password, ..form }),
                        onsubmit: move |_: String| join.call(()),
                    }
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
