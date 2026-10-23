//! Adding and removing a coding assistant: the installed ones that are not offered yet, the form
//! the Add sheet shows, the question before one is removed, and the sentences for every refusal.
//! Pure; the files are read and written by `detent-files`.

use crate::outside::Assistant;
use docket_settings::{AddFault, AgentRow, ChoiceFault, NewAgent, Source, edit_agent_add};

/// Said on the Add sheet before Add: what the assistant reaches.
pub const INTERNET_LINE: &str = "It signs in to its own service and uses the internet.";

/// Said on the Add sheet when no installed assistant is waiting to be offered.
pub const NOTHING_NEW: &str = "Nothing new is installed yet.";

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

/// The Add sheet's form: the assistants to choose from, the one chosen, and its name as typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddForm {
    choices: Vec<FoundAgent>,
    picked: usize,
    label: String,
    /// Why the last Add was refused, in words.
    pub refusal: Option<&'static str>,
}

impl AddForm {
    /// The form over `choices`, the first picked and named after itself; none for no choices.
    pub fn open(choices: Vec<FoundAgent>) -> Option<Self> {
        let label = choices.first()?.name();
        Some(Self {
            choices,
            picked: 0,
            label,
            refusal: None,
        })
    }

    /// What to choose from: each id with the name it reads as.
    pub fn entries(&self) -> Vec<(String, String)> {
        self.choices
            .iter()
            .map(|one| (one.id.clone(), one.name()))
            .collect()
    }

    /// The id picked.
    pub fn picked_id(&self) -> &str {
        self.choices.get(self.picked).map_or("", |one| &one.id)
    }

    /// The name as typed.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Picks `id`, and names the assistant after itself; an id not offered changes nothing.
    pub fn pick(mut self, id: &str) -> Self {
        if let Some(at) = self.choices.iter().position(|one| one.id == id) {
            self.label = self.choices[at].name();
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

    /// What Add writes, checked the way docket checks it; the sentence for what it refuses.
    pub fn request(&self) -> Result<NewAgent, &'static str> {
        let found = self.choices.get(self.picked).ok_or(NOTHING_NEW)?;
        let new = NewAgent {
            program: found.id.clone(),
            registry: found.id.clone(),
            version: found.version.clone(),
            label: self.label.trim().to_owned(),
        };
        edit_agent_add("", &new).map(|_| new).map_err(add_refusal)
    }

    /// The form with the refusal of the write shown.
    pub fn refused(mut self, fault: AddFault) -> Self {
        self.refusal = Some(add_refusal(fault));
        self
    }
}

/// The sentence for why an assistant was not added.
pub fn add_refusal(fault: AddFault) -> &'static str {
    match fault {
        AddFault::Unreadable => {
            "The list of coding assistants can\u{2019}t be read, so nothing was changed."
        }
        AddFault::Listed => "That coding assistant is already offered.",
        AddFault::BadName => {
            "That coding assistant\u{2019}s name has characters that can\u{2019}t be kept."
        }
        AddFault::BadLabel => "Give it a name of up to 64 letters.",
        AddFault::Io => "The list of coding assistants can\u{2019}t be written.",
    }
}

/// The sentence for why an assistant was not removed.
pub fn remove_refusal(fault: ChoiceFault) -> &'static str {
    match fault {
        ChoiceFault::NoSuchAgent => "That coding assistant isn\u{2019}t on the list any more.",
        ChoiceFault::Unreadable => {
            "The list of coding assistants can\u{2019}t be read, so nothing was changed."
        }
        ChoiceFault::BadModel | ChoiceFault::BadWay | ChoiceFault::Io => {
            "The list of coding assistants can\u{2019}t be written."
        }
    }
}

/// The question before an assistant is removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveQuestion {
    /// The entry to take out.
    pub program: String,
    /// What the person calls it.
    pub label: String,
}

impl RemoveQuestion {
    /// The question's title.
    pub fn title(&self) -> String {
        format!("Remove {}?", self.label)
    }

    /// What removing does and does not do: only the list changes; the sign-in and the installed
    /// files stay where they are.
    pub fn message(&self) -> &'static str {
        "It won\u{2019}t be offered any more. Its sign-in stays."
    }
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
