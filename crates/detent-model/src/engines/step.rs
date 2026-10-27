//! Whether llama.cpp's program is here, the note that offers it when it is not, and the restart
//! that follows installing it.

use super::advice::InstallAdvice;

/// The engine's id in inferd's list.
pub const LLAMA_CPP_ID: &str = "llama_cpp";

/// Whether an engine's program is on this computer, as inferd lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Program {
    /// It is.
    Here,
    /// It is not.
    Missing,
}

/// One engine inferd lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedEngine {
    /// The engine's stable id.
    pub id: String,
    /// Whether its program is here.
    pub program: Program,
}

/// Where llama.cpp stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LlamaCpp {
    /// The assistant has not been asked yet.
    #[default]
    Looking,
    /// The assistant is not running, did not answer, or does not list it: nothing to say.
    Unknown,
    /// Its program is here.
    Found,
    /// Its program is not here.
    Missing,
}

/// llama.cpp's standing in what the assistant listed.
pub fn llama_cpp_of(listed: &[ListedEngine]) -> LlamaCpp {
    listed
        .iter()
        .find(|engine| engine.id == LLAMA_CPP_ID)
        .map_or(LlamaCpp::Unknown, |engine| match engine.program {
            Program::Here => LlamaCpp::Found,
            Program::Missing => LlamaCpp::Missing,
        })
}

/// Where the restart stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Restart {
    /// Nothing asked.
    #[default]
    Idle,
    /// The question is on screen.
    Asking,
    /// The assistant is being restarted.
    Working,
}

/// How a restart ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restarted {
    /// The assistant came back up.
    Done,
    /// It did not.
    Failed,
}

/// What the note knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EnginesState {
    /// Where llama.cpp stands.
    pub llama_cpp: LlamaCpp,
    /// Where the restart stands.
    pub restart: Restart,
}

/// What happens to the note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnginesInput {
    /// The assistant listed its engines.
    Listed(LlamaCpp),
    /// The person asked to start using it.
    StartUsing,
    /// The person allowed the restart.
    Allowed,
    /// The person called the restart off.
    Cancelled,
    /// The restart ended.
    Restarted(Restarted),
}

/// What the window does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnginesEffect {
    /// Restart the assistant.
    Restart,
    /// Ask the assistant for its engines again.
    Look,
    /// Tell the person the restart did not go through.
    RestartFailed,
}

/// The next state, and what to do about it.
pub fn engines_step(
    state: EnginesState,
    input: EnginesInput,
) -> (EnginesState, Vec<EnginesEffect>) {
    let at = |restart| EnginesState { restart, ..state };
    match (state.restart, input) {
        // The assistant is away while it restarts: that says nothing about llama.cpp.
        (Restart::Working, EnginesInput::Listed(LlamaCpp::Unknown)) => (state, vec![]),
        (_, EnginesInput::Listed(llama_cpp)) => (EnginesState { llama_cpp, ..state }, vec![]),
        (Restart::Idle, EnginesInput::StartUsing) if state.llama_cpp == LlamaCpp::Missing => {
            (at(Restart::Asking), vec![])
        }
        (Restart::Asking, EnginesInput::Allowed) => {
            (at(Restart::Working), vec![EnginesEffect::Restart])
        }
        (Restart::Asking, EnginesInput::Cancelled) => (at(Restart::Idle), vec![]),
        (Restart::Working, EnginesInput::Restarted(Restarted::Done)) => {
            (at(Restart::Idle), vec![EnginesEffect::Look])
        }
        (Restart::Working, EnginesInput::Restarted(Restarted::Failed)) => (
            at(Restart::Idle),
            vec![EnginesEffect::RestartFailed, EnginesEffect::Look],
        ),
        _ => (state, vec![]),
    }
}

/// Whether starting to use llama.cpp can be asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartStanding {
    /// It can be asked for.
    Ready,
    /// The assistant is being restarted.
    Working,
}

/// The note under "Models on this computer" while llama.cpp's program is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupNote {
    /// How to get it: a command to copy, or the releases page's words.
    pub install: InstallAdvice,
    /// Whether starting to use it can be asked for.
    pub start: StartStanding,
}

/// The note, when llama.cpp's program is missing; nothing in any other state.
pub fn setup_note(state: &EnginesState, advice: &InstallAdvice) -> Option<SetupNote> {
    (state.llama_cpp == LlamaCpp::Missing).then(|| SetupNote {
        install: advice.clone(),
        start: match state.restart {
            Restart::Working => StartStanding::Working,
            Restart::Idle | Restart::Asking => StartStanding::Ready,
        },
    })
}
