//! Adding a model server the person already runs: what the sheet holds, what it checks, and the
//! draft that leaves it. The sheet asks for a name, the model it runs, where it answers on this
//! computer (a number, or a connection file: exactly one) and, if it wants one, a key.

use crate::Described;
use crate::picker::Ready;
use ds_settings::schema::ChoicePlace;
use porter_core::{ModelId, SecretText};
use std::io;
use std::path::{Path, PathBuf};

/// What is wrong with the Add a server sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerFault {
    /// The name is empty.
    NoName,
    /// The name is over 64 characters.
    LongName,
    /// No model was picked.
    NoModel,
    /// Neither a number nor a file was given.
    NoReach,
    /// Both were.
    BothReach,
    /// The number is not 1 to 65535.
    BadPort,
    /// The key file could not be read.
    KeyFileUnreadable,
    /// The key file has nothing in it.
    KeyFileEmpty,
}

/// Why the last Add did not go through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerRefusal {
    /// The sheet itself is wrong.
    Form(ServerFault),
    /// The service refused, in its own sentence.
    Service(String),
}

/// The longest name a server may be given, in characters.
pub const NAME_MOST: usize = 64;

/// A model the person can say a server runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerModel {
    /// The assistant's id for it.
    pub id: ModelId,
    /// What a person reads for it.
    pub label: String,
}

/// The models of `described` that run on this computer or a computer of the person's, once each,
/// in the order the module lists them.
pub fn server_models(described: &Described) -> Vec<ServerModel> {
    let mut found: Vec<ServerModel> = Vec::new();
    for kind in Ready::of(described).kinds {
        for row in kind.rows {
            for word in row.sections.iter().flat_map(|section| section.words.iter()) {
                let runs_here = matches!(
                    row.places.get(word),
                    Some(ChoicePlace::ThisComputer { .. } | ChoicePlace::YourComputer { .. })
                );
                let Some((_, model)) = word.split_once('/').filter(|_| runs_here) else {
                    continue;
                };
                let Ok(id) = ModelId::parse(model) else {
                    continue;
                };
                if found.iter().all(|one| one.id != id) {
                    found.push(ServerModel {
                        id,
                        label: row.label_of(word),
                    });
                }
            }
        }
    }
    found
}

/// Where the server answers, on this computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerReach {
    /// A number.
    Port(u16),
    /// A connection file, by its full path.
    Socket(PathBuf),
}

/// The key the server wants, as the person gave it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum KeyField {
    /// None was given.
    #[default]
    Nothing,
    /// Typed into the field.
    Typed(SecretText),
    /// Kept in a file, read only when Add is pressed.
    File(PathBuf),
}

/// What the person changed in the sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerEdit {
    /// The name was typed.
    Name(String),
    /// A model was picked.
    Model(ModelId),
    /// The number was typed.
    Port(String),
    /// A connection file was chosen, or taken away.
    Socket(Option<PathBuf>),
    /// A key was typed.
    KeyText(SecretText),
    /// A key file was chosen, or taken away.
    KeyFile(Option<PathBuf>),
}

/// What the sheet holds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerForm {
    /// The name as typed.
    pub name: String,
    /// The model picked.
    pub model: Option<ModelId>,
    /// The number as typed.
    pub port: String,
    /// The connection file chosen.
    pub socket: Option<PathBuf>,
    /// The key given.
    pub key: KeyField,
}

/// What a checked sheet asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerDraft {
    /// The name, trimmed.
    pub name: String,
    /// The model it runs.
    pub model: ModelId,
    /// Where it answers.
    pub reach: ServerReach,
    /// The key, if any.
    pub key: KeyField,
}

impl ServerForm {
    /// The form after `edit`.
    pub fn edited(mut self, edit: ServerEdit) -> ServerForm {
        match edit {
            ServerEdit::Name(name) => self.name = name,
            ServerEdit::Model(model) => self.model = Some(model),
            ServerEdit::Port(port) => self.port = port,
            ServerEdit::Socket(socket) => self.socket = socket,
            ServerEdit::KeyText(text) => {
                self.key = if text.expose().trim().is_empty() {
                    KeyField::Nothing
                } else {
                    KeyField::Typed(text)
                };
            }
            ServerEdit::KeyFile(file) => {
                self.key = file.map_or(KeyField::Nothing, KeyField::File);
            }
        }
        self
    }

    /// What the form asks for, or the first thing wrong with it.
    pub fn check(&self) -> Result<ServerDraft, ServerFault> {
        let name = self.name.trim();
        let letters = name.chars().count();
        if letters == 0 {
            return Err(ServerFault::NoName);
        }
        if letters > NAME_MOST {
            return Err(ServerFault::LongName);
        }
        let model = self.model.clone().ok_or(ServerFault::NoModel)?;
        let reach = match (self.port.trim(), &self.socket) {
            ("", None) => return Err(ServerFault::NoReach),
            ("", Some(file)) => ServerReach::Socket(file.clone()),
            (_, Some(_)) => return Err(ServerFault::BothReach),
            (number, None) => match number.parse::<u16>() {
                Ok(port) if port > 0 => ServerReach::Port(port),
                _ => return Err(ServerFault::BadPort),
            },
        };
        Ok(ServerDraft {
            name: name.to_owned(),
            model,
            reach,
            key: self.key.clone(),
        })
    }
}

impl ServerDraft {
    /// The key to send: typed text as it is, a file's text read now by `read` and trimmed; `None`
    /// when no key was given. The key is never logged or shown.
    pub fn secret(
        &self,
        read: impl FnOnce(&Path) -> io::Result<String>,
    ) -> Result<Option<SecretText>, ServerFault> {
        match &self.key {
            KeyField::Nothing => Ok(None),
            KeyField::Typed(text) => Ok(Some(SecretText::new(text.expose().trim()))),
            KeyField::File(path) => {
                let text = SecretText::new(
                    read(path)
                        .map_err(|_| ServerFault::KeyFileUnreadable)?
                        .trim(),
                );
                if text.expose().is_empty() {
                    Err(ServerFault::KeyFileEmpty)
                } else {
                    Ok(Some(text))
                }
            }
        }
    }
}

/// Whether the sheet's models are still being looked up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerModels {
    /// They are.
    Looking,
    /// These are on offer.
    Offered(Vec<ServerModel>),
}

/// Whether the sheet takes edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerPhase {
    /// It does.
    Editing,
    /// Add was pressed and the service has not answered.
    Adding,
}

/// The server being added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewServer {
    /// What it holds.
    pub form: ServerForm,
    /// The models to pick from.
    pub models: ServerModels,
    /// Whether it takes edits.
    pub phase: ServerPhase,
    /// Why the last Add did not go through, when it did not.
    pub fault: Option<ServerRefusal>,
}

impl NewServer {
    /// A sheet just opened, with the models still being looked up.
    pub fn opened() -> NewServer {
        NewServer {
            form: ServerForm::default(),
            models: ServerModels::Looking,
            phase: ServerPhase::Editing,
            fault: None,
        }
    }

    /// The sheet with `models` on offer, the first picked unless one is.
    pub fn offering(mut self, models: Vec<ServerModel>) -> NewServer {
        if self.form.model.is_none() {
            self.form.model = models.first().map(|model| model.id.clone());
        }
        self.models = ServerModels::Offered(models);
        self
    }

    /// The sheet, back to editing with `refusal` under the fields.
    pub fn refused(mut self, refusal: ServerRefusal) -> NewServer {
        self.phase = ServerPhase::Editing;
        self.fault = Some(refusal);
        self
    }
}
