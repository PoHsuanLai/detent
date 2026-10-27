//! What the offer to get an engine says: Mac wording, and no path, program name or protocol.

use detent_model::{EngineAction, EngineMatter, InstallScope, Replacing};

/// What the label above the choice of who it is for says.
pub(crate) fn install_for() -> &'static str {
    "Install for"
}

/// What the pop-up button says for a scope.
pub(crate) fn scope(scope: InstallScope) -> &'static str {
    match scope {
        InstallScope::AllUsers => "All Users",
        InstallScope::JustMe => "Just for Me",
    }
}

/// The words on an action's button.
pub(crate) fn action(action: EngineAction) -> &'static str {
    match action {
        EngineAction::Download => "Download",
        EngineAction::ChooseFolder => "Choose Folder\u{2026}",
        EngineAction::TryAgain => "Try Again",
        EngineAction::NotNow => "Not Now",
        EngineAction::Acknowledge => "OK",
        EngineAction::Cancel => "Cancel",
        EngineAction::Continue => "Continue",
    }
}

/// The heading about the engine `name`.
pub(crate) fn title(name: &str, matter: &EngineMatter) -> String {
    match matter {
        EngineMatter::Download {
            replacing: Replacing::No,
            ..
        } => format!("Download {name}?"),
        EngineMatter::Download {
            replacing: Replacing::Yes,
            ..
        } => format!("Download {name} Again?"),
        EngineMatter::Folder => format!("{name} Isn't Available to Download"),
        EngineMatter::Unavailable => format!("{name} Isn't Available"),
        EngineMatter::Checking => format!("Checking {name}\u{2026}"),
        EngineMatter::Downloading => format!("Downloading {name}\u{2026}"),
        EngineMatter::Ready => format!("{name} Is Ready"),
        EngineMatter::Failed { .. } => format!("Couldn't Download {name}"),
    }
}

/// The sentence under the heading; empty when there is none.
pub(crate) fn message(name: &str, matter: &EngineMatter) -> String {
    match matter {
        EngineMatter::Download { replacing, size } => {
            let lead = match replacing {
                Replacing::No => {
                    format!("Your assistant needs {name} to run models on this computer.")
                }
                Replacing::Yes => format!(
                    "{name} was changed since it was installed, so your assistant won't use it. \
                     Download it again to run models on this computer."
                ),
            };
            match size {
                Some(bytes) => format!("{lead} It's about {}.", size_words(*bytes)),
                None => lead,
            }
        }
        EngineMatter::Folder => format!(
            "There is no ready-made download of {name} for this computer. If you already \
             have it, choose its folder."
        ),
        EngineMatter::Unavailable => format!("There is no download of {name} for this computer."),
        EngineMatter::Checking | EngineMatter::Downloading => String::new(),
        EngineMatter::Ready => "Your assistant restarts to use it.".to_owned(),
        EngineMatter::Failed { reason } => reason.clone(),
    }
}

/// A size as a person says it: `30 MB`, `4.4 GB` (decimal units, as the Mac does).
pub(crate) fn size_words(bytes: u64) -> String {
    const MB: u64 = 1_000_000;
    const GB: u64 = 1_000_000_000;
    if bytes >= GB - 5 * MB {
        // One decimal place, by whole tenths so no float is needed.
        let tenths = (bytes + GB / 20) / (GB / 10);
        format!("{}.{} GB", tenths / 10, tenths % 10)
    } else {
        format!("{} MB", ((bytes + MB / 2) / MB).max(1))
    }
}

/// The words "Checking".
pub(crate) const CHECKING: &str = "Checking";

#[cfg(test)]
mod tests {
    use super::*;

    fn download(replacing: Replacing, size: Option<u64>) -> EngineMatter {
        EngineMatter::Download { replacing, size }
    }

    #[test]
    fn the_offer_says_what_each_fact_means() {
        // `(bytes, said)`.
        const SIZES: &[(u64, &str)] = &[
            (17_851_611, "18 MB"),
            (30_497_889, "30 MB"),
            (440_236_547, "440 MB"),
            (996_000_000, "1.0 GB"),
            (4_411_066_545, "4.4 GB"),
            (1, "1 MB"),
        ];
        for (bytes, said) in SIZES {
            assert_eq!(size_words(*bytes), *said, "{bytes}");
        }

        // `(name, the matter, the title, the message)`.
        let cases = [
            (
                "llama.cpp",
                download(Replacing::No, Some(30_497_889)),
                "Download llama.cpp?",
                "Your assistant needs llama.cpp to run models on this computer. It's about 30 MB.",
            ),
            (
                "vLLM",
                download(Replacing::No, Some(4_411_066_545)),
                "Download vLLM?",
                "Your assistant needs vLLM to run models on this computer. It's about 4.4 GB.",
            ),
            (
                "llama.cpp",
                download(Replacing::Yes, None),
                "Download llama.cpp Again?",
                "llama.cpp was changed since it was installed, so your assistant won't use it. \
                 Download it again to run models on this computer.",
            ),
            (
                "Strata",
                EngineMatter::Folder,
                "Strata Isn't Available to Download",
                "There is no ready-made download of Strata for this computer. If you already \
                 have it, choose its folder.",
            ),
            (
                "Kokoro",
                EngineMatter::Unavailable,
                "Kokoro Isn't Available",
                "There is no download of Kokoro for this computer.",
            ),
            (
                "Strata",
                EngineMatter::Checking,
                "Checking Strata\u{2026}",
                "",
            ),
            (
                "llama.cpp",
                EngineMatter::Downloading,
                "Downloading llama.cpp\u{2026}",
                "",
            ),
            (
                "llama.cpp",
                EngineMatter::Ready,
                "llama.cpp Is Ready",
                "Your assistant restarts to use it.",
            ),
            (
                "llama.cpp",
                EngineMatter::Failed {
                    reason: "The download didn't finish.".to_owned(),
                },
                "Couldn't Download llama.cpp",
                "The download didn't finish.",
            ),
        ];
        for (name, matter, want_title, want_message) in cases {
            assert_eq!(title(name, &matter), want_title);
            assert_eq!(message(name, &matter), want_message);
            // No word is a developer word or a path.
            let said = format!("{want_title} {want_message}").to_lowercase();
            for banned in [
                "pkexec", "polkit", "sha", "http", "tar", "/", "cli", "api", "mcp",
            ] {
                assert!(
                    !said.split_whitespace().any(|word| word == banned),
                    "{said}"
                );
            }
            assert!(!said.contains('/'), "{said}");
        }

        // `(action, the label)`.
        const ACTIONS: &[(EngineAction, &str)] = &[
            (EngineAction::Download, "Download"),
            (EngineAction::ChooseFolder, "Choose Folder\u{2026}"),
            (EngineAction::TryAgain, "Try Again"),
            (EngineAction::NotNow, "Not Now"),
            (EngineAction::Acknowledge, "OK"),
            (EngineAction::Cancel, "Cancel"),
            (EngineAction::Continue, "Continue"),
        ];
        for (fact, want) in ACTIONS {
            assert_eq!(action(*fact), *want);
        }
        assert_eq!(scope(InstallScope::AllUsers), "All Users");
        assert_eq!(scope(InstallScope::JustMe), "Just for Me");
        assert_eq!(install_for(), "Install for");
    }
}
