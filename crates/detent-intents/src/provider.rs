//! The `IntentProvider1` of Settings: three actions over the catalogue and the settings files.

use crate::accounts::AccountsSource;
use crate::answer::{key_ref, not_found, outcome, own, own_label, page_ref, refusal};
use crate::ask::{Ask, ask};
use crate::journal::{Change, Journal};
use crate::limits::{ACCOUNTS_CHARS, KEY_VALUE_CHARS, many_lines, one_line};
use crate::manifest::{ManifestFault, manifest};
use detent_files::{ConfigHome, load, save};
use detent_model::{
    AgentSetting, Arrival, Catalogue, Control, KeyEntry, Notice, Outbox, PageSlug, TextFault,
    arrival, show_value,
};
use docket_client::IntentProvider;
use docket_core::{
    AppRefusal, EntityRef, FactLine, Hit, Invocation, LabelText, Outcome, Preview, SuggestAsk,
    UndoFault, UndoToken, Undoable, ValidManifest, Value,
};
use ds_settings::schema::KeyPath;
use ds_settings::target::SettingsTarget;
use porter_core::{AppName, DataClass};
use prov::{Actor, EntityId, Label, Labelled, Source};
use std::sync::{Arc, Mutex, MutexGuard};

/// The most things a suggestion or a preview lists.
const LISTED: usize = 20;

/// Settings, as the desktop's router sees it.
pub struct SettingsProvider {
    manifest: ValidManifest,
    app: AppName,
    catalogue: Arc<Catalogue>,
    home: ConfigHome,
    window: Outbox,
    accounts: Arc<dyn AccountsSource>,
    journal: Mutex<Journal>,
}

impl std::fmt::Debug for SettingsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsProvider")
            .field("app", &self.app)
            .finish_non_exhaustive()
    }
}

impl SettingsProvider {
    /// The provider of `catalogue`, whose files are under `home`; `window` is told what it must
    /// show or read again, and `accounts` says which accounts there are.
    pub fn new(
        catalogue: Arc<Catalogue>,
        home: ConfigHome,
        window: Outbox,
        accounts: Arc<dyn AccountsSource>,
    ) -> Result<SettingsProvider, ManifestFault> {
        let manifest = manifest()?;
        Ok(SettingsProvider {
            app: manifest.manifest().app.clone(),
            manifest,
            catalogue,
            home,
            window,
            accounts,
            journal: Mutex::new(Journal::default()),
        })
    }

    /// The app's name, `org.quire.Settings`.
    pub fn app(&self) -> &AppName {
        &self.app
    }

    /// The window the provider tells, for the app's own `org.freedesktop.Application` object.
    pub(crate) fn window(&self) -> Outbox {
        self.window.clone()
    }

    fn journal(&self) -> MutexGuard<'_, Journal> {
        self.journal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn key(&self, path: &KeyPath) -> Result<&KeyEntry, AppRefusal> {
        self.catalogue
            .entry(path)
            .ok_or_else(|| not_found(&self.app, "settings.key", &path.0))
    }

    /// What the file holds at `entry` now: the value as written when no control can hold it,
    /// else the value, else the key's default.
    fn current(&self, entry: &KeyEntry) -> Result<toml::Value, AppRefusal> {
        let spec = self
            .catalogue
            .file(&entry.file)
            .ok_or_else(|| not_found(&self.app, "settings.key", &entry.spec.path.0))?;
        let loaded = load(&self.home, spec);
        if let Some(whole) = loaded.invalid.iter().find(|bad| bad.path.is_empty()) {
            return Err(refusal(format!(
                "{} cannot be read: {}",
                entry.file.relative().display(),
                whole.reason
            )));
        }
        let path = &entry.spec.path;
        Ok(loaded
            .value
            .as_written(path)
            .or_else(|| loaded.value.get(path))
            .cloned()
            .unwrap_or_else(|| entry.spec.default.clone()))
    }

    /// The settable key at `path` and the value `text` makes for it, or why not.
    fn settable(&self, path: &KeyPath, text: &str) -> Result<(&KeyEntry, toml::Value), AppRefusal> {
        let entry = self.key(path)?;
        if entry.agent() != AgentSetting::Settable {
            return Err(refusal(format!(
                "{} is not a setting an assistant may change: only the person changes it, in Settings",
                path.0
            )));
        }
        let value = entry
            .control
            .from_text(text)
            .map_err(|fault| refusal(why(path, &fault)))?;
        Ok((entry, value))
    }

    fn open_page(&self, slug: PageSlug) -> Result<Outcome, AppRefusal> {
        let page = self
            .catalogue
            .page(&slug)
            .ok_or_else(|| not_found(&self.app, "settings.page", &slug.0))?;
        let said = format!("Opened {}", page.title);
        self.window
            .send(Notice::OpenPage(slug))
            .map_err(|gone| refusal(gone.to_string()))?;
        Ok(outcome(&said, Undoable::No))
    }

    /// Shows the row of a key or the account a target names; a target no page holds is not found.
    fn open_target(&self, target: SettingsTarget) -> Result<Outcome, AppRefusal> {
        let said = match arrival(&self.catalogue, &target) {
            Some(Arrival::Page { slug, .. }) => self
                .catalogue
                .page(&slug)
                .map(|page| format!("Opened {}", page.title)),
            Some(Arrival::Account(_)) => Some("Opened Accounts".to_owned()),
            Some(Arrival::Extension(_)) => Some("Opened Extensions".to_owned()),
            None => None,
        }
        .ok_or_else(|| not_found(&self.app, "settings.key", &target.to_string()))?;
        self.window
            .send(Notice::OpenTarget(target))
            .map_err(|gone| refusal(gone.to_string()))?;
        Ok(outcome(&said, Undoable::No))
    }

    /// The accounts, a line each: labels, kinds and state, nothing that names or reaches one.
    async fn list_accounts(&self, space: prov::SpaceId) -> Result<Outcome, AppRefusal> {
        let pane = self.accounts.read().await.map_err(|absence| {
            refusal(format!(
                "the accounts cannot be listed: {}",
                absence_line(absence)
            ))
        })?;
        let lines: Vec<String> = pane
            .accounts
            .iter()
            .map(crate::accounts::account_line)
            .collect();
        let mut out = outcome("", Undoable::No);
        out.value = Some(Labelled::new(
            Value::Text(many_lines(&lines.join("\n"), ACCOUNTS_CHARS)),
            Label::untrusted(Source::Web, DataClass::AppOwn, space),
        ));
        Ok(out)
    }

    fn read_key(&self, path: &KeyPath, space: prov::SpaceId) -> Result<Outcome, AppRefusal> {
        let entry = self.key(path)?;
        let text = one_line(&show_value(&self.current(entry)?), KEY_VALUE_CHARS);
        let mut out = outcome("", Undoable::No);
        out.value = Some(Labelled::new(
            Value::Text(text),
            Label::untrusted(Source::File, DataClass::AppOwn, space),
        ));
        Ok(out)
    }

    fn set_key(&self, path: &KeyPath, text: &str) -> Result<Outcome, AppRefusal> {
        let (entry, after) = self.settable(path, text)?;
        let before = self.current(entry)?;
        save(&self.home, &entry.file, path, &after).map_err(|error| refusal(error.to_string()))?;
        let token = self
            .journal()
            .record(Change {
                file: entry.file.clone(),
                key: path.clone(),
                before,
                after: after.clone(),
            })
            .ok_or(AppRefusal::Busy)?;
        // The window may have gone; the file is written either way.
        let _ = self.window.send(Notice::Reload);
        let said = format!("Set {} to {}", entry.spec.label.0, show_value(&after));
        Ok(outcome(&said, Undoable::Yes(token)))
    }
}

fn why(path: &KeyPath, fault: &TextFault) -> String {
    match fault {
        TextFault::FileOnly => format!("{} can only be edited in its file", path.0),
        TextFault::NotAWord(words) => format!("{} takes one of: {}", path.0, words.join(", ")),
        TextFault::NotANumber => format!("{} takes a number", path.0),
        TextFault::OutOfRange(min, max) => {
            format!("{} takes a number from {min} to {max}", path.0)
        }
    }
}

impl IntentProvider for SettingsProvider {
    fn manifest(&self) -> &ValidManifest {
        &self.manifest
    }

    async fn perform(&self, inv: Invocation) -> Result<Outcome, AppRefusal> {
        match ask(&inv, &self.app)? {
            Ask::OpenPage(slug) => self.open_page(slug),
            Ask::OpenTarget(target) => self.open_target(target),
            Ask::ReadKey(path) => self.read_key(&path, inv.space),
            Ask::SetKey { key, text } => self.set_key(&key, &text),
            Ask::ListAccounts => self.list_accounts(inv.space).await,
            Ask::OpenAccounts => self.open_page(PageSlug("accounts".to_owned())),
            Ask::OpenIntelligence => self.open_page(PageSlug("intelligence".to_owned())),
        }
    }

    async fn dry_run(&self, inv: Invocation) -> Result<Preview, AppRefusal> {
        let Ask::SetKey { key, text } = ask(&inv, &self.app)? else {
            return Ok(Preview::None);
        };
        let (entry, after) = self.settable(&key, &text)?;
        let before = one_line(&show_value(&self.current(entry)?), KEY_VALUE_CHARS);
        let label_of_after = inv
            .args
            .iter()
            .find(|(name, _)| name.as_str() == "value")
            .map_or_else(|| own_label(&self.app), |(_, arg)| arg.label.clone());
        Ok(Preview::TextChange {
            before: Labelled::new(
                before,
                Label::untrusted(Source::File, DataClass::AppOwn, inv.space),
            ),
            after: Labelled::new(show_value(&after), label_of_after),
        })
    }

    async fn undo(&self, token: UndoToken, _actor: Actor) -> Result<(), UndoFault> {
        let change = self
            .journal()
            .find(&token)
            .cloned()
            .ok_or(UndoFault::Gone)?;
        let entry = self.catalogue.entry(&change.key).ok_or(UndoFault::Gone)?;
        if self.current(entry).map_err(|_| UndoFault::Conflict)? != change.after {
            self.journal().forget(&token);
            return Err(UndoFault::Conflict);
        }
        save(&self.home, &change.file, &change.key, &change.before)
            .map_err(|_| UndoFault::Conflict)?;
        self.journal().forget(&token);
        let _ = self.window.send(Notice::Reload);
        Ok(())
    }

    /// Never answers: the launcher's search reads intentd's own index (the pages and keys in
    /// `quire/search/org.quire.Settings.toml`, see `search_file`), so a query never reaches
    /// this app. Design: search-index/DESIGN.md, "Launcher search".
    async fn search(&self, _text: &str) -> Vec<Hit> {
        Vec::new()
    }

    async fn preview(&self, id: &EntityId) -> Preview {
        match id.kind.as_str() {
            "settings.key" => self.preview_key(id.key.as_str()),
            "settings.page" => self.preview_page(id.key.as_str()),
            _ => Preview::None,
        }
    }

    async fn suggest(&self, ask: SuggestAsk) -> Vec<EntityRef> {
        let typed = ask.typed.to_lowercase();
        let matches = |label: &str, path: &str| {
            label.to_lowercase().contains(&typed) || path.to_lowercase().contains(&typed)
        };
        match (ask.action.name.as_str(), ask.param.as_str()) {
            ("settings.page.show", "page") => self
                .catalogue
                .pages
                .iter()
                .filter(|page| matches(&page.title, &page.slug.0))
                .filter_map(|page| page_ref(&self.app, &page.slug.0, &page.title))
                .take(LISTED)
                .collect(),
            (action @ ("settings.key.read" | "settings.key.set"), "key") => self
                .catalogue
                .entries()
                .map(|(_, entry)| entry)
                .filter(|entry| {
                    action == "settings.key.read" || entry.agent() == AgentSetting::Settable
                })
                .filter(|entry| matches(&entry.spec.label.0, &entry.spec.path.0))
                .filter_map(|entry| key_ref(&self.app, entry))
                .take(LISTED)
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl SettingsProvider {
    /// The words of the key and, when its control offers a closed set (words or a range), the
    /// value it holds. A free-text value is somebody's words and is not previewed.
    fn preview_key(&self, path: &str) -> Preview {
        let Ok(entry) = self.key(&KeyPath(path.to_owned())) else {
            return Preview::None;
        };
        let held = match entry.control {
            Control::Text
            | Control::Colour
            | Control::Picture
            | Control::HotCorners
            | Control::Rows(_)
            | Control::FileOnly(_) => None,
            _ => self.current(entry).ok().map(|value| show_value(&value)),
        };
        let line = |label: &str, text: String| {
            LabelText::parse(label).ok().map(|label| FactLine {
                label,
                value: own(&self.app, text),
            })
        };
        Preview::Facts(
            [
                line("Setting", entry.spec.label.0.clone()),
                line("Help", entry.spec.help.0.clone()),
                held.and_then(|text| line("Value", text)),
            ]
            .into_iter()
            .flatten()
            .collect(),
        )
    }

    fn preview_page(&self, slug: &str) -> Preview {
        let Some(page) = self.catalogue.page(&PageSlug(slug.to_owned())) else {
            return Preview::None;
        };
        Preview::List(
            page.sections
                .iter()
                .flat_map(|section| &section.keys)
                .filter_map(|entry| key_ref(&self.app, entry))
                .take(LISTED)
                .collect(),
        )
    }
}

/// Why the accounts cannot be listed, in the words an agent is told.
fn absence_line(absence: detent_model::Absence) -> &'static str {
    use detent_model::Absence;
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop",
        Absence::NotRunning => "the service is not running",
        Absence::NotPermitted => "Settings was not opened from the desktop",
        Absence::Unavailable => "the service could not be confirmed",
        Absence::NoDevice | Absence::Failed => "the service did not answer",
    }
}
