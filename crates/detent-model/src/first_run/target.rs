//! Which row of inferd's picker a runtime chosen at first run is written to, and with which word.

use crate::accounts::AccountKey;
use crate::assistant_model::ASSISTANT_ROWS;
use crate::picker::Described;
use ds_settings::schema::{KeyKind, KeyPath};

/// The row to set and the model word to set it to: the first model of `runtime` the module offers
/// for the text slot that can be picked now. `None` when the module has no such row, or lists no
/// model of that runtime (an `account/model` word names the account before its slash).
pub fn runtime_choice(described: &Described, runtime: &AccountKey) -> Option<(KeyPath, String)> {
    let prefix = format!("{}/", runtime.0);
    ASSISTANT_ROWS.iter().find_map(|row| {
        let spec = described.keys.iter().find(|spec| spec.path.0 == *row)?;
        let KeyKind::Menu { variants } = &spec.kind else {
            return None;
        };
        let word = variants.iter().find(|word| {
            word.starts_with(&prefix)
                && !spec
                    .unavailable
                    .keys()
                    .any(|gated| gated.0.as_str() == word.as_str())
        })?;
        Some((spec.path.clone(), word.clone()))
    })
}

#[cfg(test)]
mod tests {
    use super::runtime_choice;
    use crate::accounts::AccountKey;
    use crate::picker::Described;
    use ds_settings::schema::{
        AgentSetting, ChoiceWord, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section,
        UnavailableReason,
    };
    use std::collections::BTreeMap;

    fn menu(path: &str, words: &[&str], gated: &[&str]) -> KeySpec {
        KeySpec {
            path: KeyPath(path.to_owned()),
            kind: KeyKind::Menu {
                variants: words.iter().map(|w| (*w).to_owned()).collect(),
            },
            default: toml::Value::String(String::new()),
            label: Label("Balanced".to_owned()),
            help: Help(String::new()),
            page: Page::Intelligence,
            section: Section("Models / Text".to_owned()),
            exposure: Exposure::Basic,
            labels: Default::default(),
            unavailable: gated
                .iter()
                .map(|w| {
                    (
                        ChoiceWord((*w).to_owned()),
                        UnavailableReason("no".to_owned()),
                    )
                })
                .collect(),
            groups: Default::default(),
            agent: AgentSetting::HandsOff,
        }
    }

    fn described(keys: Vec<KeySpec>) -> Described {
        Described {
            keys,
            values: BTreeMap::new(),
            ..Default::default()
        }
    }

    fn key(text: &str) -> AccountKey {
        AccountKey(text.to_owned())
    }

    #[test]
    fn the_first_model_of_the_runtime_in_the_text_slots_balanced_row() {
        let rows = described(vec![
            menu("ai.model.text.fast", &["", "auto", "ollama/tiny"], &[]),
            menu(
                "ai.model.text.balanced",
                &["", "auto", "lm-studio/qwen", "ollama/llama3", "ollama/phi"],
                &[],
            ),
        ]);
        assert_eq!(
            runtime_choice(&rows, &key("ollama")),
            Some((
                KeyPath("ai.model.text.balanced".to_owned()),
                "ollama/llama3".to_owned()
            ))
        );
        assert_eq!(
            runtime_choice(&rows, &key("llama-cpp")),
            None,
            "no model of it"
        );
        assert_eq!(
            runtime_choice(&rows, &key("ollama-x")),
            None,
            "a prefix is not a runtime"
        );
    }

    #[test]
    fn a_gated_model_is_skipped_and_an_older_module_is_read() {
        let rows = described(vec![menu(
            "ai.model.llm.balanced",
            &["", "auto", "ollama/a", "ollama/b"],
            &["ollama/a"],
        )]);
        assert_eq!(
            runtime_choice(&rows, &key("ollama")),
            Some((
                KeyPath("ai.model.llm.balanced".to_owned()),
                "ollama/b".to_owned()
            ))
        );
        assert_eq!(runtime_choice(&described(vec![]), &key("ollama")), None);
    }
}
