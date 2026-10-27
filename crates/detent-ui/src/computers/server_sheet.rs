//! The Add a server sheet: a model server the person already runs, so the assistant can use its
//! models. It asks for a name, the model, where the server answers on this computer and, if it
//! wants one, a key.

use crate::copy::server_refusal;
use crate::env::Env;
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
use porter_core::{ModelId, SecretText};
use std::path::PathBuf;

/// The sheet's title.
const TITLE: &str = "Add a server";

/// What the sheet says it is for.
const ABOUT: &str = "Add a model server you already run, so the assistant can use its models.";

/// Under the place the server answers.
const ON_THIS_COMPUTER: &str = "The server must answer on this computer.";

/// While the models are looked up.
const LOOKING: &str = "Looking for models\u{2026}";

/// When there is no model to pick.
const NO_MODELS: &str = "There are no models to choose from yet.";

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
    env: &Env,
    then: impl FnOnce(PathBuf) -> ComputersInput + 'static,
    feed: EventHandler<ComputersInput>,
) {
    let corners = env.corners.clone();
    spawn(async move {
        if let Some(path) = corners.choose(HotOpenKind::File).await {
            feed.call(then(PathBuf::from(path)));
        }
    });
}

/// The sheet, with the sentence of the last refusal under its fields.
#[component]
pub(super) fn ServerSheetView(sheet: NewServer, feed: EventHandler<ComputersInput>) -> Element {
    let env = use_context::<Env>();
    let editing = sheet.phase == ServerPhase::Editing;
    let close = EventHandler::new(move |()| feed.call(ComputersInput::CloseServer));
    let add = EventHandler::new(move |()| feed.call(ComputersInput::AddServer));
    let edit = move |edit: ServerEdit| feed.call(ComputersInput::ServerEdit(edit));
    let form = sheet.form.clone();
    let chosen_model = form.model.as_ref().map(ToString::to_string);
    let socket_env = env.clone();
    let key_env = env;
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
                    label: "Name",
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
                    label: "Where it answers",
                    value: form.port.clone(),
                    placeholder: "A number, such as 8080",
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
                            label: "Don\u{2019}t use it",
                            common: button("Don\u{2019}t use", "the connection file", "server-socket-clear"),
                            onclick: move |_| edit(ServerEdit::Socket(None)),
                        }
                    }
                    Button {
                        label: "Choose a file\u{2026}",
                        common: button("Choose a file for", "where it answers", "server-socket"),
                        onclick: move |_| choose_file(&socket_env, |path| ComputersInput::ServerEdit(ServerEdit::Socket(Some(path))), feed),
                    }
                }
                Label { text: ON_THIS_COMPUTER, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
                TextField {
                    label: "Key (optional)",
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
                            label: "Don\u{2019}t use it",
                            common: button("Don\u{2019}t use", "the key file", "server-key-clear"),
                            onclick: move |_| edit(ServerEdit::KeyFile(None)),
                        }
                    }
                    Button {
                        label: "Use a key file\u{2026}",
                        common: button("Use a key file for", "the server", "server-key-file"),
                        onclick: move |_| choose_file(&key_env, |path| ComputersInput::ServerEdit(ServerEdit::KeyFile(Some(path))), feed),
                    }
                }
                if let Some(fault) = sheet.fault.clone() {
                    div { "data-system": "add-server-fault",
                        Label { text: server_refusal(&fault), role: LabelRole::Primary, severity: Some(Severity::Warn) }
                    }
                }
                div { class: "system-sheet-buttons",
                    Button {
                        label: "Cancel",
                        common: Common { aria_label: Some("Cancel".to_owned()), id: Some("server-cancel".to_owned()), ..Common::default() },
                        answers: Answers::Escape,
                        availability: if editing { Availability::Enabled } else { Availability::Disabled },
                        onclick: move |_| close.call(()),
                    }
                    Button {
                        label: if editing { "Add" } else { "Adding\u{2026}" },
                        common: Common { aria_label: Some("Add".to_owned()), id: Some("server-add".to_owned()), ..Common::default() },
                        answers: Answers::Return,
                        availability: if editing { Availability::Enabled } else { Availability::Disabled },
                        onclick: move |_| add.call(()),
                    }
                }
            }
        }
    }
}
