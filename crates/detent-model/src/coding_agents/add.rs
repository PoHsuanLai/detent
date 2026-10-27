//! Adding and removing a coding assistant: the installed ones that are not offered yet, the form
//! the Add sheet shows, the question before one is removed, and the facts behind every refusal
//! (the window words them). Pure; the files are read and written by `detent-files`.

use super::add_choice::{AddAction, Candidate, Listing, Presence, Step, said_line};
use crate::outside::Assistant;
use docket_settings::{AddFault, AgentRow, NewAgent, Source, edit_agent_add};

/// Why Add did not add the assistant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddRefusal {
    /// No installed assistant is waiting to be offered, so none is picked.
    NothingNew,
    /// docket refused the entry, or could not write it.
    Fault(AddFault),
    /// The download failed, and the downloader said this.
    DownloadSaid(String),
    /// The download failed and nothing says why.
    DownloadFailed,
}

/// An installed assistant, as found in the folder docket keeps them in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundAgent {
    /// The id it is installed under.
    pub id: String,
    /// The version to offer: the last one installed.
    pub version: String,
}

impl FoundAgent {
    /// What the person is first shown for it: `claude-code` reads `Claude Code`.
    pub fn name(&self) -> String {
        self.id
            .split(['-', '_', '.'])
            .filter(|word| !word.is_empty())
            .map(|word| {
                let mut letters = word.chars();
                letters
                    .next()
                    .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn is_listed(found: &FoundAgent, row: &AgentRow) -> bool {
    row.program == found.id || matches!(&row.source, Source::Registry { id, .. } if *id == found.id)
}

/// The installed assistants no row lists yet, in the order found.
pub fn unlisted_found(found: &[FoundAgent], rows: &[AgentRow]) -> Vec<FoundAgent> {
    found
        .iter()
        .filter(|one| rows.iter().all(|row| !is_listed(one, row)))
        .cloned()
        .collect()
}

/// What Add does for the assistant picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// It is installed: list it.
    Add(NewAgent),
    /// It is not: download `version` of `id`, then list it.
    Install {
        /// The id to download.
        id: String,
        /// The version to download.
        version: String,
        /// The entry to write once it is installed.
        then: NewAgent,
    },
}

/// The Add sheet's form: the assistants to choose from, the one chosen, and its name as typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddForm {
    choices: Vec<Candidate>,
    /// The programs and ids the list has already, which are not offered again.
    listed: Vec<String>,
    picked: usize,
    label: String,
    list: Listing,
    step: Step,
    /// Why the last Add was refused.
    pub refusal: Option<AddRefusal>,
}

fn listed_words(rows: &[AgentRow]) -> Vec<String> {
    rows.iter()
        .flat_map(|row| {
            let id = match &row.source {
                Source::Registry { id, .. } => Some(id.clone()),
                Source::Own { .. } => None,
            };
            std::iter::once(row.program.clone()).chain(id)
        })
        .collect()
}

impl AddForm {
    fn start(found: &[FoundAgent], rows: &[AgentRow], list: Listing) -> Self {
        let choices: Vec<Candidate> = unlisted_found(found, rows)
            .iter()
            .map(Candidate::installed)
            .collect();
        let label = choices.first().map(|one| one.name.clone());
        Self {
            choices,
            listed: listed_words(rows),
            picked: 0,
            label: label.unwrap_or_default(),
            list,
            step: Step::Idle,
            refusal: None,
        }
    }

    /// The form while the list of assistants is fetched; the installed ones not in `rows` wait in it.
    pub fn loading(found: &[FoundAgent], rows: &[AgentRow]) -> Self {
        Self::start(found, rows, Listing::Loading)
    }

    /// The form over the installed assistants only, when there is nothing to fetch a list with.
    pub fn offline(found: &[FoundAgent], rows: &[AgentRow]) -> Self {
        Self::start(found, rows, Listing::Offline)
    }

    /// The form with the fetched `offers` first, in their order, then the installed ones the list
    /// does not know; those `rows` have already are left out. Only a form still loading takes it.
    pub fn catalogue(mut self, offers: Vec<Candidate>) -> Self {
        if self.list != Listing::Loading {
            return self;
        }
        let rest: Vec<Candidate> = self
            .choices
            .iter()
            .filter(|one| offers.iter().all(|offer| offer.id != one.id))
            .cloned()
            .collect();
        self.choices = offers
            .into_iter()
            .chain(rest)
            .filter(|one| !self.listed.contains(&one.id))
            .collect();
        self.picked = 0;
        self.label = self
            .choices
            .first()
            .map_or_else(String::new, |one| one.name.clone());
        self.list = Listing::Listed;
        self
    }

    /// The form with the reason the list was not fetched, the installed ones still offered. Only a
    /// form still loading takes it.
    pub fn unreachable(mut self, said: &str) -> Self {
        if self.list == Listing::Loading {
            self.list = Listing::Failed(said_line(said));
        }
        self
    }

    /// How far the list has come.
    pub fn listing(&self) -> &Listing {
        &self.list
    }

    /// How far Add has come.
    pub fn step(&self) -> Step {
        self.step
    }

    /// What to choose from, in order.
    pub fn entries(&self) -> &[Candidate] {
        &self.choices
    }

    /// The assistant picked.
    pub fn picked(&self) -> Option<&Candidate> {
        self.choices.get(self.picked)
    }

    /// The id picked.
    pub fn picked_id(&self) -> &str {
        self.picked().map_or("", |one| &one.id)
    }

    /// The name as typed.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// What the Add button can do now.
    pub fn button(&self) -> AddAction {
        match (&self.list, self.step, self.choices.is_empty()) {
            (_, Step::Starting | Step::Downloading, _) => AddAction::Busy,
            (Listing::Loading, _, _) | (_, _, true) => AddAction::Off,
            _ => AddAction::Ready,
        }
    }

    /// Picks `id`, and names the assistant after itself; an id not offered changes nothing, and
    /// nothing is picked while a download runs.
    pub fn pick(mut self, id: &str) -> Self {
        if self.step != Step::Idle {
            return self;
        }
        if let Some(at) = self.choices.iter().position(|one| one.id == id) {
            self.label = self.choices[at].name.clone();
            self.picked = at;
            self.refusal = None;
        }
        self
    }

    /// The name typed.
    pub fn typed(mut self, label: String) -> Self {
        self.label = label;
        self.refusal = None;
        self
    }

    /// What Add does, checked the way docket checks the entry; the refusal when it cannot.
    pub fn request(&self) -> Result<Request, AddRefusal> {
        let found = self.picked().ok_or(AddRefusal::NothingNew)?;
        let new = NewAgent {
            program: found.id.clone(),
            registry: found.id.clone(),
            version: found.version.clone(),
            label: self.label.trim().to_owned(),
        };
        edit_agent_add("", &new).map_err(AddRefusal::Fault)?;
        Ok(match found.presence {
            Presence::Installed => Request::Add(new),
            Presence::Online => Request::Install {
                id: found.id.clone(),
                version: found.version.clone(),
                then: new,
            },
        })
    }

    /// The form with the download asked for.
    pub fn installing(mut self) -> Self {
        self.step = Step::Starting;
        self.refusal = None;
        self
    }

    /// The form with the download begun.
    pub fn downloading(mut self) -> Self {
        if self.step == Step::Starting {
            self.step = Step::Downloading;
        }
        self
    }

    /// The assistant being downloaded, once the download has begun; nothing before.
    pub fn download_of(&self) -> Option<&Candidate> {
        self.picked().filter(|_| self.step == Step::Downloading)
    }

    /// The form with the download's failure shown, ready for another Add.
    pub fn install_failed(mut self, said: &str) -> Self {
        self.step = Step::Idle;
        self.refusal = Some(match said_line(said) {
            Some(line) => AddRefusal::DownloadSaid(line),
            None => AddRefusal::DownloadFailed,
        });
        self
    }

    /// The form with the refusal of the write shown.
    pub fn refused(mut self, fault: AddFault) -> Self {
        self.step = Step::Idle;
        self.refusal = Some(AddRefusal::Fault(fault));
        self
    }
}

/// The person's answer to the question before an assistant is removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveAnswer {
    /// Take it off the list.
    Remove,
    /// Leave it where it is.
    Keep,
}

/// The question before an assistant is removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveQuestion {
    /// The entry to take out.
    pub program: String,
    /// What the person calls it.
    pub label: String,
}

/// The question for `assistant`, when it can be removed: an assistant added from the installed
/// ones. One whose program the person wrote by hand is theirs to edit, and has no Remove.
pub fn removal_of(assistant: &Assistant) -> Option<RemoveQuestion> {
    let row = assistant.agent.as_ref()?;
    match row.source {
        Source::Registry { .. } => Some(RemoveQuestion {
            program: row.program.clone(),
            label: row.label.clone(),
        }),
        Source::Own { .. } => None,
    }
}
