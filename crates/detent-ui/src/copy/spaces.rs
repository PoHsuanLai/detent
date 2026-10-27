//! The Spaces page's words: the question before a Space is removed.

use almanac_core::{HistoryFate, MemoryFate};

/// The line under the history choices.
pub(crate) const HISTORY_NOTE: &str =
    "History is what happened in this Space, like files you saved and messages you sent.";

/// The history choices, first the one that keeps it.
pub(crate) const HISTORY_CHOICES: [HistoryFate; 2] = [HistoryFate::Keep, HistoryFate::Delete];

/// What a history choice is called.
pub(crate) fn history_label(fate: HistoryFate) -> &'static str {
    match fate {
        HistoryFate::Keep => "Keep its history",
        HistoryFate::Delete => "Delete its history",
    }
}

/// The question's title.
pub(crate) fn removal_title(name: &str) -> String {
    format!("Remove \u{201c}{name}\u{201d}?")
}

/// The line under the title.
pub(crate) fn removal_body(memories: MemoryFate) -> &'static str {
    match memories {
        MemoryFate::MoveToApps => "Its memories move to the apps that saved them.",
        MemoryFate::Delete => "Its memories will be deleted.",
    }
}

/// The link button that flips what becomes of the memories.
pub(crate) fn flip_label(memories: MemoryFate) -> &'static str {
    match memories {
        MemoryFate::MoveToApps => "Delete its memories instead",
        MemoryFate::Delete => "Move its memories instead",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_question_is_worded_by_what_becomes_of_the_memories() {
        assert_eq!(removal_title("Trip"), "Remove \u{201c}Trip\u{201d}?");
        let cases = [
            (
                MemoryFate::MoveToApps,
                "Its memories move to the apps that saved them.",
                "Delete its memories instead",
            ),
            (
                MemoryFate::Delete,
                "Its memories will be deleted.",
                "Move its memories instead",
            ),
        ];
        for (memories, body, flip) in cases {
            assert_eq!(removal_body(memories), body);
            assert_eq!(flip_label(memories), flip);
        }
    }

    #[test]
    fn the_history_choices_keep_first() {
        assert_eq!(
            HISTORY_CHOICES[0],
            HistoryFate::Keep,
            "the first is selected"
        );
        assert_eq!(history_label(HistoryFate::Keep), "Keep its history");
        assert_eq!(history_label(HistoryFate::Delete), "Delete its history");
    }
}
