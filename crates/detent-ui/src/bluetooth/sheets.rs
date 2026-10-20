//! The two sheets over the Bluetooth page: forgetting a device, and a device asking while it
//! pairs.

use detent_model::{
    BtInput, PairingAnswer, PairingQuestion, forget_body, forget_title, pairing_body, pairing_title,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::{Answers, ButtonRole};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;

/// Asks before a device is forgotten.
#[component]
pub(super) fn ForgetSheet(name: String, press: EventHandler<BtInput>) -> Element {
    let close = EventHandler::new(move |()| press.call(BtInput::Cancel));
    let title = forget_title(&name);
    rsx! {
        Sheet {
            label: title.clone(),
            width: SheetWidth::Narrow,
            onclose: close,
            div { class: "system-sheet", "data-system": "forget",
                Label { text: title, role: LabelRole::Primary, style: LabelStyle::Headline }
                Label { text: forget_body(), role: LabelRole::Secondary }
                div { class: "system-sheet-buttons",
                    Button {
                        label: "Cancel",
                        common: Common { aria_label: Some("Cancel".to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        onclick: move |_| close.call(()),
                    }
                    Button {
                        label: "Forget Device",
                        common: Common { aria_label: Some("Forget Device".to_owned()), ..Common::default() },
                        role: ButtonRole::Destructive,
                        onclick: move |_| press.call(BtInput::ConfirmForget),
                    }
                }
            }
        }
    }
}

/// What a device asks while pairing: Connect says yes, Cancel says no. A number to type on the
/// device has nothing to answer, so it has Cancel only.
#[component]
pub(super) fn PairingSheet(question: PairingQuestion, press: EventHandler<BtInput>) -> Element {
    let close = EventHandler::new(move |()| press.call(BtInput::Cancel));
    let title = pairing_title(&question);
    let yes = EventHandler::new(move |()| press.call(BtInput::Answer(PairingAnswer::Yes)));
    rsx! {
        Sheet {
            label: title.clone(),
            width: SheetWidth::Narrow,
            onclose: close,
            div { class: "system-sheet", "data-system": "pairing",
                Label { text: title, role: LabelRole::Primary, style: LabelStyle::Headline }
                Label { text: pairing_body(&question), role: LabelRole::Secondary }
                div { class: "system-sheet-buttons",
                    Button {
                        label: "Cancel",
                        common: Common { aria_label: Some("Cancel".to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        onclick: move |_| close.call(()),
                    }
                    if question.wants_answer() {
                        Button {
                            label: "Connect",
                        common: Common { aria_label: Some("Connect".to_owned()), ..Common::default() },
                            answers: Answers::Return,
                            onclick: move |_| yes.call(()),
                        }
                    }
                }
            }
        }
    }
}
