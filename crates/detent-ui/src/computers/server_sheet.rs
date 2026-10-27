//! The Add a server sheet: a model server the person already runs, so the assistant can use its
//! models. It asks for a name, the model, where the server answers on this computer and, if it
//! wants one, a key.

use crate::copy::computers::server_refusal;
use crate::live::use_client;
use detent_client::Client;
use detent_model::external::{ModelId, SecretText};
use detent_model::{
    ComputersInput, HotOpenKind, KeyField, NewServer, ServerEdit, ServerModels, ServerPhase,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use std::path::PathBuf;

/// The sheet's title.
const TITLE: &str = crate::copy::computers::ADD_A_SERVER;

/// What the sheet says it is for.
const ABOUT: &str = crate::copy::computers::ADD_A_MODEL_SERVER_YOU_ALREADY;

/// Under the place the server answers.
const ON_THIS_COMPUTER: &str = crate::copy::computers::THE_SERVER_MUST_ANSWER_ON_THIS;

/// While the models are looked up.
const LOOKING: &str = crate::copy::computers::LOOKING_FOR_MODELS_ELLIPSIS;

/// When there is no model to pick.
const NO_MODELS: &str = crate::copy::computers::THERE_ARE_NO_MODELS_TO_CHOOSE;

/// Which file was chosen.
fn file_line(path: &std::path::Path) -> String {
    path.display().to_string()
}

fn button(label: &str, subject: &str, id: &str) -> Common {
    Common {
        id: Some(id.to_owned()),
        aria_label: Some(format!("{label} {subject}").trim().to_owned()),
        ..Common::default()
    }
}

/// Asks the person for a file and feeds the choice in.
fn choose_file(
    client: &Client,
    then: impl FnOnce(PathBuf) -> ComputersInput + 'static,
    feed: EventHandler<ComputersInput>,
) {
    let corners = client.corners();
    spawn(async move {
        if let Some(path) = corners.choose(HotOpenKind::File).await {
            feed.call(then(PathBuf::from(path)));
        }
    });
}

/// The sheet, with the sentence of the last refusal under its fields.
#[component]
pub(super) fn ServerSheetView(sheet: NewServer, feed: EventHandler<ComputersInput>) -> Element {
    let client = use_client();
    let editing = sheet.phase == ServerPhase::Editing;
    let close = EventHandler::new(move |()| feed.call(ComputersInput::CloseServer));
    let add = EventHandler::new(move |()| feed.call(ComputersInput::AddServer));
    let edit = move |edit: ServerEdit| feed.call(ComputersInput::ServerEdit(edit));
    let form = sheet.form.clone();
    let chosen_model = form.model.as_ref().map(ToString::to_string);
    let socket_client = client.clone();
    let key_client = client;
    rsx! {
        Sheet {
            label: TITLE.to_owned(),
            width: SheetWidth::Narrow,
            onclose: close,
            on_return: add,
            div { class: "system-sheet", "data-system": "add-server",
                Label { text: TITLE, role: LabelRole::Primary, style: LabelStyle::Headline }
                Label { text: ABOUT, role: LabelRole::Secondary }
                TextField {
                    label: crate::copy::common::NAME,
                    value: form.name.clone(),
                    kind: FieldKind::Plain,
                    size: ControlSize::Small,
                    common: Common { id: Some("server-name".to_owned()), ..Common::default() },
                    oninput: move |name: String| edit(ServerEdit::Name(name)),
                    onsubmit: move |_: String| add.call(()),
                }
                match sheet.models.clone() {
                    ServerModels::Looking => rsx! { Label { text: LOOKING, role: LabelRole::Tertiary } },
                    ServerModels::Offered(models) if models.is_empty() => rsx! {
                        Label { text: NO_MODELS, role: LabelRole::Tertiary }
                    },
                    ServerModels::Offered(models) => rsx! {
                        Label { text: "Model it runs", role: LabelRole::Secondary }
                        PopUpButton::<String> {
                            items: models
                                .iter()
                                .map(|model| MenuItem::new(model.id.to_string(), model.label.clone()))
                                .collect::<Vec<_>>(),
                            value: chosen_model,
                            size: ControlSize::Small,
                            common: Common { id: Some("server-model".to_owned()), ..Common::default() },
                            onpick: move |word: String| {
                                if let Ok(id) = ModelId::parse(&word) {
                                    edit(ServerEdit::Model(id));
                                }
                            },
                        }
                    },
                }
                TextField {
                    label: crate::copy::computers::WHERE_IT_ANSWERS,
                    value: form.port.clone(),
                    placeholder: crate::copy::computers::A_NUMBER_SUCH_AS_8080,
                    kind: FieldKind::Plain,
                    size: ControlSize::Small,
                    common: Common { id: Some("server-port".to_owned()), ..Common::default() },
                    oninput: move |port: String| edit(ServerEdit::Port(port)),
                    onsubmit: move |_: String| add.call(()),
                }
                div { class: "system-sheet-buttons",
                    if let Some(path) = form.socket.clone() {
                        Label { text: file_line(&path), role: LabelRole::Secondary }
                        Button {
                            label: crate::copy::computers::DONT_USE_IT,
                            common: button(crate::copy::computers::DONT_USE, crate::copy::computers::THE_CONNECTION_FILE, "server-socket-clear"),
                            onclick: move |_| edit(ServerEdit::Socket(None)),
                        }
                    }
                    Button {
                        label: crate::copy::computers::CHOOSE_A_FILE_ELLIPSIS,
                        common: button(crate::copy::computers::CHOOSE_A_FILE_FOR, crate::copy::computers::WHERE_IT_ANSWERS_SUBJECT, "server-socket"),
                        onclick: move |_| choose_file(&socket_client, |path| ComputersInput::ServerEdit(ServerEdit::Socket(Some(path))), feed),
                    }
                }
                Label { text: ON_THIS_COMPUTER, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                TextField {
                    label: crate::copy::computers::KEY_OPTIONAL,
                    value: String::new(),
                    kind: FieldKind::Secure,
                    size: ControlSize::Small,
                    common: Common { id: Some("server-key".to_owned()), ..Common::default() },
                    oninput: move |key: String| edit(ServerEdit::KeyText(SecretText::new(key))),
                    onsubmit: move |_: String| add.call(()),
                }
                div { class: "system-sheet-buttons",
                    if let KeyField::File(path) = form.key.clone() {
                        Label { text: file_line(&path), role: LabelRole::Secondary }
                        Button {
                            label: crate::copy::computers::DONT_USE_IT,
                            common: button(crate::copy::computers::DONT_USE, crate::copy::computers::THE_KEY_FILE, "server-key-clear"),
                            onclick: move |_| edit(ServerEdit::KeyFile(None)),
                        }
                    }
                    Button {
                        label: crate::copy::computers::USE_A_KEY_FILE_ELLIPSIS,
                        common: button(crate::copy::computers::USE_A_KEY_FILE_FOR, crate::copy::computers::THE_SERVER, "server-key-file"),
                        onclick: move |_| choose_file(&key_client, |path| ComputersInput::ServerEdit(ServerEdit::KeyFile(Some(path))), feed),
                    }
                }
                if let Some(fault) = sheet.fault.clone() {
                    div { "data-system": "add-server-fault",
                        Label { text: server_refusal(&fault), role: LabelRole::Primary, severity: Some(Severity::Warn) }
                    }
                }
                div { class: "system-sheet-buttons",
                    Button {
                        label: crate::copy::common::CANCEL,
                        common: Common { aria_label: Some(crate::copy::common::CANCEL.to_owned()), id: Some("server-cancel".to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        availability: if editing { Availability::Enabled } else { Availability::Disabled },
                        onclick: move |_| close.call(()),
                    }
                    Button {
                        label: if editing { crate::copy::common::ADD } else { crate::copy::computers::ADDING_ELLIPSIS },
                        common: Common { aria_label: Some(crate::copy::common::ADD.to_owned()), id: Some("server-add".to_owned()), ..Common::default() },
                        answers: Answers::Return,
                        availability: if editing { Availability::Enabled } else { Availability::Disabled },
                        onclick: move |_| add.call(()),
                    }
                }
            }
        }
    }
}
