//! Whether llama.cpp's program is here, the note that offers it when it is not, and the restart
//! that follows installing it.

use super::advice::InstallAdvice;

/// The engine's id in inferd's list.
pub const LLAMA_CPP_ID: &str = "llama_cpp";

/// The line over the install command.
pub const SETUP_LINE: &str = "To run models on this computer, install llama.cpp.";

/// The button that makes the assistant look again.
pub const START_USING: &str = "Start using it";

/// The button of the question that comes before.
pub const RESTART: &str = "Restart";

/// What the person reads when the restart did not go through.
const COULD_NOT: &str = "The assistant could not be restarted. Try again in a moment.";

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
    /// "Start using it" was pressed.
    StartUsing,
    /// The question was answered Restart.
    Allowed,
    /// The question was answered Cancel.
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
    /// Tell the person this.
    Tell(&'static str),
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
            vec![EnginesEffect::Tell(COULD_NOT), EnginesEffect::Look],
        ),
        _ => (state, vec![]),
    }
}

/// How the "Start using it" button stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartButton {
    /// It takes a press.
    Ready,
    /// The assistant is being restarted.
    Working,
}

/// The note under "Models on this computer" while llama.cpp's program is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupNote {
    /// The quiet line.
    pub line: &'static str,
    /// How to get it: a command to copy, or the releases page's words.
    pub install: InstallAdvice,
    /// The button beside it.
    pub button: StartButton,
}

/// The note, when llama.cpp's program is missing; nothing in any other state.
pub fn setup_note(state: &EnginesState, advice: &InstallAdvice) -> Option<SetupNote> {
    (state.llama_cpp == LlamaCpp::Missing).then(|| SetupNote {
        line: SETUP_LINE,
        install: advice.clone(),
        button: match state.restart {
            Restart::Working => StartButton::Working,
            Restart::Idle | Restart::Asking => StartButton::Ready,
        },
    })
}

/// The question before the restart, while it is asked: a title and a message. It does not know
/// whether a conversation is open, so it always says what happens to one.
pub fn restart_question(state: &EnginesState) -> Option<(String, String)> {
    (state.restart == Restart::Asking).then(|| {
        (
            "Restart the assistant to use llama.cpp?".to_owned(),
            "The assistant restarts so it can find llama.cpp. A conversation that is open \
             stops."
                .to_owned(),
        )
    })
}
