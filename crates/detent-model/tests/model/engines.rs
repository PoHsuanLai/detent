//! llama.cpp on this computer: the install line for each distro and release, where it stands in
//! what the assistant lists, when the note shows, and the restart that follows installing it.

use detent_model::{
    EnginesEffect, EnginesInput, EnginesState, InstallAdvice, LLAMA_CPP_ID, ListedEngine, LlamaCpp,
    Program, Restart, Restarted, StartStanding, engines_step, install_advice, llama_cpp_of,
    setup_note,
};

fn command(text: &str) -> InstallAdvice {
    InstallAdvice::Command(text.to_owned())
}

#[test]
fn the_install_line_follows_the_distro_and_its_release() {
    // `(case, the os-release file, the advice)`.
    let cases: &[(&str, &str, InstallAdvice)] = &[
        (
            "fedora",
            "ID=fedora\nVERSION_ID=44\n",
            command("sudo dnf install llama-cpp"),
        ),
        (
            "a fedora relative",
            "ID=nobara\nID_LIKE=\"rhel fedora\"\nVERSION_ID=43\n",
            command("sudo dnf install llama-cpp"),
        ),
        (
            "arch without a release number",
            "ID=arch\n",
            command("sudo pacman -S llama-cpp"),
        ),
        (
            "debian 14",
            "ID=debian\nVERSION_ID=\"14\"\n",
            command("sudo apt install llama.cpp-tools"),
        ),
        (
            "debian 15",
            "ID=debian\nVERSION_ID=\"15\"\n",
            command("sudo apt install llama.cpp-tools"),
        ),
        (
            "debian testing has no number",
            "ID=debian\n",
            command("sudo apt install llama.cpp-tools"),
        ),
        (
            "debian 13",
            "ID=debian\nVERSION_ID=\"13\"\n",
            InstallAdvice::ReleasesPage,
        ),
        (
            "ubuntu 26.04",
            "ID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"26.04\"\n",
            command("sudo apt install llama.cpp-tools"),
        ),
        (
            "ubuntu 26.10",
            "ID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"26.10\"\n",
            command("sudo apt install llama.cpp-tools"),
        ),
        (
            "ubuntu 27.04",
            "ID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"27.04\"\n",
            command("sudo apt install llama.cpp-tools"),
        ),
        (
            "ubuntu 25.10",
            "ID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"25.10\"\n",
            InstallAdvice::ReleasesPage,
        ),
        (
            "ubuntu 24.04",
            "ID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"24.04\"\n",
            InstallAdvice::ReleasesPage,
        ),
        (
            "ubuntu with no number",
            "ID=ubuntu\nID_LIKE=debian\n",
            InstallAdvice::ReleasesPage,
        ),
        (
            "an ubuntu relative",
            "ID=linuxmint\nID_LIKE=\"ubuntu debian\"\nVERSION_ID=\"23\"\n",
            InstallAdvice::ReleasesPage,
        ),
        (
            "a number that is not one",
            "ID=ubuntu\nVERSION_ID=\"soon\"\n",
            InstallAdvice::ReleasesPage,
        ),
        (
            "opensuse",
            "ID=\"opensuse-tumbleweed\"\nID_LIKE=\"opensuse suse\"\nVERSION_ID=\"20261001\"\n",
            InstallAdvice::ReleasesPage,
        ),
        (
            "a distro nobody knows",
            "ID=nixos\n",
            InstallAdvice::ReleasesPage,
        ),
        ("no file", "", InstallAdvice::ReleasesPage),
    ];
    for (name, file, want) in cases {
        assert_eq!(&install_advice(file), want, "{name}");
    }
}

fn listed(id: &str, program: Program) -> ListedEngine {
    ListedEngine {
        id: id.to_owned(),
        program,
    }
}

#[test]
fn llama_cpp_stands_where_the_assistant_lists_it() {
    // `(case, what was listed, where llama.cpp stands)`.
    let cases: &[(&str, Vec<ListedEngine>, LlamaCpp)] = &[
        (
            "found",
            vec![listed(LLAMA_CPP_ID, Program::Here)],
            LlamaCpp::Found,
        ),
        (
            "missing among the others",
            vec![
                listed("vllm", Program::Here),
                listed(LLAMA_CPP_ID, Program::Missing),
                listed("kokoro", Program::Missing),
            ],
            LlamaCpp::Missing,
        ),
        (
            "not listed",
            vec![listed("vllm", Program::Missing)],
            LlamaCpp::Unknown,
        ),
        ("nothing listed", vec![], LlamaCpp::Unknown),
    ];
    for (name, rows, want) in cases {
        assert_eq!(llama_cpp_of(rows), *want, "{name}");
    }
}

#[test]
fn the_note_shows_only_while_llama_cpp_is_missing_and_starting_follows_the_restart() {
    let advice = command("sudo dnf install llama-cpp");
    // `(case, llama.cpp, the restart, whether starting can be asked for when the note shows)`.
    let cases = [
        ("looking", LlamaCpp::Looking, Restart::Idle, None),
        ("unknown", LlamaCpp::Unknown, Restart::Idle, None),
        ("found", LlamaCpp::Found, Restart::Idle, None),
        (
            "found after a restart",
            LlamaCpp::Found,
            Restart::Working,
            None,
        ),
        (
            "missing",
            LlamaCpp::Missing,
            Restart::Idle,
            Some(StartStanding::Ready),
        ),
        (
            "missing and asked",
            LlamaCpp::Missing,
            Restart::Asking,
            Some(StartStanding::Ready),
        ),
        (
            "missing and restarting",
            LlamaCpp::Missing,
            Restart::Working,
            Some(StartStanding::Working),
        ),
    ];
    for (name, llama_cpp, restart, start) in cases {
        let note = setup_note(&EnginesState { llama_cpp, restart }, &advice);
        assert_eq!(note.as_ref().map(|note| note.start), start, "{name}");
        if let Some(note) = note {
            assert_eq!(note.install, advice, "{name}");
        }
    }
}

#[test]
fn the_restart_asks_first_and_looks_again_when_it_ends() {
    let missing = EnginesState {
        llama_cpp: LlamaCpp::Missing,
        restart: Restart::Idle,
    };
    let at = |restart| EnginesState { restart, ..missing };
    // `(case, state, input, next state, effects)`.
    let cases: &[(
        &str,
        EnginesState,
        EnginesInput,
        EnginesState,
        Vec<EnginesEffect>,
    )] = &[
        (
            "pressed",
            missing,
            EnginesInput::StartUsing,
            at(Restart::Asking),
            vec![],
        ),
        (
            "allowed",
            at(Restart::Asking),
            EnginesInput::Allowed,
            at(Restart::Working),
            vec![EnginesEffect::Restart],
        ),
        (
            "cancelled",
            at(Restart::Asking),
            EnginesInput::Cancelled,
            missing,
            vec![],
        ),
        (
            "done",
            at(Restart::Working),
            EnginesInput::Restarted(Restarted::Done),
            missing,
            vec![EnginesEffect::Look],
        ),
        (
            "failed",
            at(Restart::Working),
            EnginesInput::Restarted(Restarted::Failed),
            missing,
            vec![EnginesEffect::RestartFailed, EnginesEffect::Look],
        ),
        (
            "listed found",
            missing,
            EnginesInput::Listed(LlamaCpp::Found),
            EnginesState {
                llama_cpp: LlamaCpp::Found,
                ..missing
            },
            vec![],
        ),
        (
            "listing keeps the restart",
            at(Restart::Working),
            EnginesInput::Listed(LlamaCpp::Missing),
            at(Restart::Working),
            vec![],
        ),
        (
            "the assistant away while it restarts",
            at(Restart::Working),
            EnginesInput::Listed(LlamaCpp::Unknown),
            at(Restart::Working),
            vec![],
        ),
        (
            "the assistant away when nothing was asked",
            missing,
            EnginesInput::Listed(LlamaCpp::Unknown),
            EnginesState {
                llama_cpp: LlamaCpp::Unknown,
                ..missing
            },
            vec![],
        ),
        (
            "pressed twice asks once",
            at(Restart::Asking),
            EnginesInput::StartUsing,
            at(Restart::Asking),
            vec![],
        ),
        (
            "pressed while working",
            at(Restart::Working),
            EnginesInput::StartUsing,
            at(Restart::Working),
            vec![],
        ),
        (
            "pressed with nothing to start",
            EnginesState {
                llama_cpp: LlamaCpp::Found,
                restart: Restart::Idle,
            },
            EnginesInput::StartUsing,
            EnginesState {
                llama_cpp: LlamaCpp::Found,
                restart: Restart::Idle,
            },
            vec![],
        ),
        (
            "allowed with no question",
            missing,
            EnginesInput::Allowed,
            missing,
            vec![],
        ),
    ];
    for (name, state, input, next, effects) in cases {
        assert_eq!(
            engines_step(*state, *input),
            (*next, effects.clone()),
            "{name}"
        );
    }
}
