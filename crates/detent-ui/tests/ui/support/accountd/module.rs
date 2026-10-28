//! The live module: what the fake says of each row, and what it does with a set or an action.

use super::rows::{action, rows_of, spec};
use super::*;

impl LiveModule for Accountd {
    async fn permit(&self, _caller: &Caller, _access: Access) -> Verdict {
        *self.calls.lock().expect("calls") += 1;
        self.policy
    }

    async fn describe(&self) -> LiveSchema {
        let mut key: Vec<KeySpec> = self
            .accounts
            .lock()
            .expect("accounts")
            .iter()
            .flat_map(rows_of)
            .filter(|row| row.path.0.contains(".spend.") == (self.module == ModuleId::Inferd))
            .collect();
        if self.continue_as {
            key.push(spec(
                "accounts.sync.add".to_owned(),
                "Sync",
                "Continue as",
                action("Continue as\u{2026}", ActionWeight::Plain),
                toml::Value::Boolean(false),
            ));
        }
        let mut client = spec(
            "accounts.clients.microsoft".to_owned(),
            "Sign-in clients",
            "microsoft client id",
            KeyKind::Text,
            toml::Value::String(String::new()),
        );
        client.exposure = Exposure::Advanced;
        let mut secret = spec(
            "accounts.clients.microsoft.secret".to_owned(),
            "Sign-in clients",
            "microsoft client secret",
            KeyKind::Text,
            toml::Value::String(String::new()),
        );
        secret.exposure = Exposure::Advanced;
        if matches!(self.module, ModuleId::Accountd) {
            key.push(client);
            key.push(secret);
        }
        LiveSchema { version: 1, key }
    }

    async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        let path = key.0.as_str();
        if path == "accounts.clients.microsoft" {
            return Ok(toml::Value::String(
                self.client.lock().expect("client").clone(),
            ));
        }
        if path == "accounts.clients.microsoft.secret" {
            return Ok(toml::Value::String(
                self.secret.lock().expect("secret").clone(),
            ));
        }
        let accounts = self.accounts.lock().expect("accounts");
        let on = accounts.iter().find_map(|held| {
            let services = held
                .services
                .iter()
                .map(|(k, on)| (format!("service.{k}"), on));
            let sync = held.sync.iter().map(|(k, on)| (format!("sync.{k}"), on));
            let files = held
                .show_in_files
                .iter()
                .map(|(on, _)| ("storage.files".to_owned(), on));
            services.chain(sync).chain(files).find_map(|(row, on)| {
                (path == format!("accounts.{}.{row}", held.id)).then_some(*on)
            })
        });
        match on {
            Some(on) => Ok(toml::Value::String(
                if on { "on" } else { "off" }.to_owned(),
            )),
            None => Ok(toml::Value::Boolean(false)),
        }
    }

    async fn set(&self, key: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        self.sets
            .lock()
            .expect("sets")
            .push((key.0.clone(), value.clone()));
        let path = key.0.as_str();
        if path == "accounts.clients.microsoft" {
            *self.client.lock().expect("client") = value.as_str().unwrap_or_default().to_owned();
            return Ok(());
        }
        if path == "accounts.clients.microsoft.secret" {
            *self.secret.lock().expect("secret") = value.as_str().unwrap_or_default().to_owned();
            return Ok(());
        }
        let mut accounts = self.accounts.lock().expect("accounts");
        for held in accounts.iter_mut() {
            for (kind, on) in &mut held.services {
                if path == format!("accounts.{}.service.{kind}", held.id) {
                    *on = value.as_str() == Some("on");
                }
            }
            for (leaf, on) in &mut held.sync {
                if path == format!("accounts.{}.sync.{leaf}", held.id) {
                    *on = value.as_str() == Some("on");
                }
            }
            if let Some((on, _)) = &mut held.show_in_files
                && path == format!("accounts.{}.storage.files", held.id)
            {
                *on = value.as_str() == Some("on");
            }
            held.grants
                .retain(|(grant, _)| path != format!("accounts.{}.grant.{grant}", held.id));
        }
        accounts.retain(|held| path != format!("accounts.{}.remove", held.id));
        Ok(())
    }
}
