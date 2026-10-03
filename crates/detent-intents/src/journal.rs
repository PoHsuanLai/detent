//! The undo entries of settings changes: for each token, the key, the value it had and the
//! value the change gave it. In memory only: an undo after the app restarted is `Gone`.

use detent_model::ConfigFile;
use docket_core::UndoToken;
use ds_settings::schema::KeyPath;
use std::collections::BTreeMap;

/// How many changes can be taken back; the oldest is forgotten first.
const KEPT: usize = 256;

/// One change that can be undone.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Change {
    /// The file the key lives in.
    pub file: ConfigFile,
    /// The key.
    pub key: KeyPath,
    /// What the key held before: the old value, or its default when the file said nothing.
    pub before: toml::Value,
    /// What the change set.
    pub after: toml::Value,
}

/// The changes that can still be undone.
#[derive(Debug, Default)]
pub(crate) struct Journal {
    next: u64,
    changes: BTreeMap<u64, Change>,
}

impl Journal {
    /// Writes `change` down and returns the token that undoes it.
    pub(crate) fn record(&mut self, change: Change) -> Option<UndoToken> {
        self.next += 1;
        self.changes.insert(self.next, change);
        while self.changes.len() > KEPT {
            self.changes.pop_first();
        }
        UndoToken::parse(&format!("settings-undo-{}", self.next)).ok()
    }

    /// The change `token` names, without forgetting it.
    pub(crate) fn find(&self, token: &UndoToken) -> Option<&Change> {
        self.changes.get(&number(token)?)
    }

    /// Forgets the change `token` names.
    pub(crate) fn forget(&mut self, token: &UndoToken) {
        if let Some(n) = number(token) {
            self.changes.remove(&n);
        }
    }
}

fn number(token: &UndoToken) -> Option<u64> {
    token.as_str().strip_prefix("settings-undo-")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{Change, Journal, KEPT};
    use detent_model::ConfigFile;
    use ds_settings::schema::{FilePath, KeyPath};

    fn change(n: i64) -> Change {
        Change {
            file: ConfigFile::parse(&FilePath("sill/settings.toml".into())).expect("a file"),
            key: KeyPath("dock.tile_size_px".into()),
            before: toml::Value::Integer(n),
            after: toml::Value::Integer(n + 1),
        }
    }

    #[test]
    fn a_token_finds_its_change_until_it_is_forgotten() {
        let mut journal = Journal::default();
        let first = journal.record(change(1)).expect("a token");
        let second = journal.record(change(2)).expect("a token");
        assert_ne!(first, second);
        assert_eq!(journal.find(&first), Some(&change(1)));
        journal.forget(&first);
        assert_eq!(journal.find(&first), None);
        assert_eq!(journal.find(&second), Some(&change(2)));
    }

    #[test]
    fn the_oldest_change_is_forgotten_past_the_limit() {
        let mut journal = Journal::default();
        let first = journal.record(change(0)).expect("a token");
        let last = (1..=KEPT as i64)
            .filter_map(|n| journal.record(change(n)))
            .last()
            .expect("a token");
        assert_eq!(journal.find(&first), None);
        assert!(journal.find(&last).is_some());
    }

    #[test]
    fn a_token_the_journal_did_not_mint_finds_nothing() {
        let journal = Journal::default();
        let stranger = docket_core::UndoToken::parse("mail-undo-1").expect("a token");
        assert_eq!(journal.find(&stranger), None);
    }
}
