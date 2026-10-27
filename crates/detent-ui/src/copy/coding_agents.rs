//! The coding assistants' Add sheet and the question before one is removed: every sentence for
//! what the sheet says and for each refusal.

use detent_model::external::{AddFault, ChoiceFault};
use detent_model::{AddRefusal, Presence, RemoveQuestion};

/// Said on the Add sheet before Add: what the assistant reaches.
pub(crate) const INTERNET_LINE: &str = "It signs in to its own service and uses the internet.";

/// Said on the Add sheet when no installed assistant is waiting to be offered.
pub(crate) const NOTHING_NEW: &str = "Nothing new is installed yet.";

/// Said on the Add sheet while the list of coding assistants is fetched.
pub(crate) const LOADING: &str = "Looking for coding assistants\u{2026}";

/// Said on the Add sheet when the list of coding assistants cannot be fetched.
const LIST_FAILED: &str = "The list of coding assistants can\u{2019}t be fetched right now.";

/// Said under the Add sheet when an assistant cannot be downloaded and nothing says why.
const DOWNLOAD_FAILED: &str = "That coding assistant couldn\u{2019}t be downloaded.";

/// The sentence for a list that could not be fetched, given the first line the downloader said.
pub(crate) fn list_failure(said: Option<&str>) -> String {
    match said {
        Some(reason) => format!("{LIST_FAILED} {reason}"),
        None => LIST_FAILED.to_owned(),
    }
}

/// The mark beside an assistant's name; none for one that is not installed.
pub(crate) fn presence_mark(presence: Presence) -> Option<&'static str> {
    match presence {
        Presence::Installed => Some("Installed"),
        Presence::Online => None,
    }
}

/// What the sheet says once the download of `name` has begun.
pub(crate) fn downloading(name: &str) -> String {
    format!("Downloading {name}\u{2026}")
}

/// The sentence for why an assistant was not added.
pub(crate) fn add_refusal(refusal: &AddRefusal) -> String {
    match refusal {
        AddRefusal::NothingNew => NOTHING_NEW.to_owned(),
        AddRefusal::Fault(fault) => add_fault(*fault).to_owned(),
        AddRefusal::DownloadSaid(said) => said.clone(),
        AddRefusal::DownloadFailed => DOWNLOAD_FAILED.to_owned(),
    }
}

fn add_fault(fault: AddFault) -> &'static str {
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
pub(crate) fn remove_refusal(fault: ChoiceFault) -> &'static str {
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

/// The question's title.
pub(crate) fn remove_title(question: &RemoveQuestion) -> String {
    format!("Remove {}?", question.label)
}

/// What removing does and does not do: only the list changes; the sign-in and the installed files
/// stay where they are.
pub(crate) fn remove_message() -> &'static str {
    "It won\u{2019}t be offered any more. Its sign-in stays."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_list_and_a_presence_have_their_words() {
        assert_eq!(
            list_failure(Some("no route to the host")),
            format!("{LIST_FAILED} no route to the host")
        );
        assert_eq!(list_failure(None), LIST_FAILED);
        assert_eq!(presence_mark(Presence::Installed), Some("Installed"));
        assert_eq!(presence_mark(Presence::Online), None);
        assert_eq!(downloading("Codex"), "Downloading Codex\u{2026}");
    }

    #[test]
    fn a_download_says_what_the_downloader_said_or_that_it_failed() {
        assert_eq!(
            add_refusal(&AddRefusal::DownloadSaid(
                "the archive did not match its checksum".to_owned()
            )),
            "the archive did not match its checksum"
        );
        assert_eq!(add_refusal(&AddRefusal::DownloadFailed), DOWNLOAD_FAILED);
        assert_eq!(add_refusal(&AddRefusal::NothingNew), NOTHING_NEW);
    }

    #[test]
    fn every_refusal_has_a_sentence_without_developer_words() {
        let adds = [
            AddFault::Unreadable,
            AddFault::Listed,
            AddFault::BadName,
            AddFault::BadLabel,
            AddFault::Io,
        ];
        let removes = [
            ChoiceFault::NoSuchAgent,
            ChoiceFault::Unreadable,
            ChoiceFault::BadModel,
            ChoiceFault::BadWay,
            ChoiceFault::Io,
        ];
        let sentences = adds
            .into_iter()
            .map(|fault| add_refusal(&AddRefusal::Fault(fault)))
            .chain(
                removes
                    .into_iter()
                    .map(|fault| remove_refusal(fault).to_owned()),
            );
        for words in sentences {
            assert!(words.ends_with('.'), "{words}");
            for bad in ["agents.toml", "ACP", "registry", "TOML"] {
                assert!(!words.contains(bad), "{bad} in {words}");
            }
        }
    }

    #[test]
    fn the_removal_question_says_what_stays() {
        let question = RemoveQuestion {
            program: "claude-code".to_owned(),
            label: "Claude Code".to_owned(),
        };
        assert_eq!(remove_title(&question), "Remove Claude Code?");
        assert_eq!(
            remove_message(),
            "It won\u{2019}t be offered any more. Its sign-in stays."
        );
    }
}
