//! The live module: the rows, and what the fake does with a set.

use super::data::{POOL_KEY, PROGRAMS, ROWS, SLOT_ROWS};
use super::*;

impl LiveModule for Inferd {
    async fn permit(&self, _caller: &Caller, _access: Access) -> Verdict {
        self.policy
    }

    async fn describe(&self) -> LiveSchema {
        LiveSchema {
            version: 1,
            key: match self.layout {
                Layout::Kinds => ROWS,
                Layout::Slots => SLOT_ROWS,
            }
            .iter()
            .map(|(kind, label, tier)| self.spec(kind, label, tier))
            .chain(matches!(self.layout, Layout::Slots).then(|| self.pool_spec()))
            .chain(
                PROGRAMS
                    .iter()
                    .filter(|(path, _)| self.programs.lock().expect("programs").contains_key(*path))
                    .map(|(path, label)| Inferd::program_spec(path, label)),
            )
            .collect(),
        }
    }

    async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        if key.0 == POOL_KEY {
            let pool = self.pool.lock().expect("pool").clone();
            return Ok(toml::Value::Array(
                pool.into_iter().map(toml::Value::String).collect(),
            ));
        }
        if let Some(program) = self.programs.lock().expect("programs").get(&key.0) {
            return Ok(toml::Value::String(program.clone()));
        }
        let values = self.values.lock().expect("values");
        Ok(toml::Value::String(
            values.get(&key.0).cloned().unwrap_or_default(),
        ))
    }

    async fn set(&self, key: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        if key.0 == POOL_KEY {
            let toml::Value::Array(items) = value else {
                return Err(LiveError::BadValue("a pool is a list of models".to_owned()));
            };
            let words: Vec<String> = items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect();
            self.pool_sets
                .lock()
                .expect("pool sets")
                .push(words.clone());
            *self.pool.lock().expect("pool") = words;
            return Ok(());
        }
        let toml::Value::String(word) = value else {
            return Err(LiveError::BadValue("a model is text".to_owned()));
        };
        let is_program = self.programs.lock().expect("programs").contains_key(&key.0);
        if is_program {
            if !word.is_empty() && !word.starts_with('/') {
                return Err(LiveError::BadValue(NOT_WHOLE.to_owned()));
            }
            self.sets
                .lock()
                .expect("sets")
                .push((key.0.clone(), word.clone()));
            self.programs
                .lock()
                .expect("programs")
                .insert(key.0.clone(), word);
            return Ok(());
        }
        self.sets
            .lock()
            .expect("sets")
            .push((key.0.clone(), word.clone()));
        let held = *self.held.lock().expect("held");
        if held {
            self.release.notified().await;
        }
        self.values
            .lock()
            .expect("values")
            .insert(key.0.clone(), word);
        Ok(())
    }
}
