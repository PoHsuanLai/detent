//! The actions of installed apps a corner may run: read from the apps' intents manifests, only
//! those that need nothing from the person, grouped by app for a searchable list.

use docket_core::{AgentReach, Manifest, ParamNeed, TargetKind, action_prefix, validate};

/// One action of an installed app that a corner can run as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotActionChoice {
    /// The app's name (`org.quire.Mail`).
    pub app: String,
    // SCHEMA: `intent` in a binding is the short name (`compose`), not `mail.compose`.
    /// The action without the app's prefix (`compose`), as the command line takes it.
    pub action: String,
    /// What the person reads ("Compose").
    pub label: String,
}

/// An installed app with a desktop entry, by the entry's id and its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotApp {
    /// The entry's id, without `.desktop`.
    pub id: String,
    /// `Name`.
    pub name: String,
}

/// What the machine knows of installed apps, to put names to the ids a binding stores.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HotKnown {
    /// The apps with desktop entries.
    pub apps: Vec<HotApp>,
    /// The actions their manifests offer.
    pub actions: Vec<HotActionChoice>,
}

/// The last element of an app's name ("Mail" for `org.quire.Mail`).
fn last_element(app: &str) -> &str {
    app.rsplit('.')
        .next()
        .filter(|last| !last.is_empty())
        .unwrap_or(app)
}

impl HotKnown {
    /// What the app `id` is called: its desktop entry's name, else the last element of its name.
    pub fn app_title(&self, id: &str) -> String {
        self.apps
            .iter()
            .find(|app| app.id == id && !app.name.is_empty())
            .map_or_else(|| last_element(id).to_owned(), |app| app.name.clone())
    }

    /// "Mail › Compose": the app and the action of a binding, in the person's words; the
    /// action's own name when no manifest offers it.
    pub fn action_line(&self, app: &str, action: &str) -> String {
        let label = self
            .actions
            .iter()
            .find(|choice| choice.app == app && choice.action == action)
            .map_or(action, |choice| choice.label.as_str());
        format!("{} \u{203a} {label}", self.app_title(app))
    }
}

/// The actions of the manifest `text` that a corner can run: for everyone, not kept for the person
/// alone, acting on nothing in particular and with no parameter that must be given. A text that
/// is not a valid manifest offers none.
pub fn actions_of(text: &str) -> Vec<HotActionChoice> {
    let Ok(parsed) = toml::from_str::<Manifest>(text) else {
        return Vec::new();
    };
    let Ok(valid) = validate(parsed) else {
        return Vec::new();
    };
    let manifest = valid.manifest();
    if !manifest.visibility.is_everyone() {
        return Vec::new();
    }
    let prefix = format!("{}.", action_prefix(&manifest.app));
    manifest
        .actions
        .iter()
        .filter(|decl| decl.reach != AgentReach::Hidden)
        .filter(|decl| decl.on == TargetKind::Nothing)
        .filter(|decl| decl.params.iter().all(|p| p.need != ParamNeed::Required))
        .map(|decl| HotActionChoice {
            app: manifest.app.as_str().to_owned(),
            action: decl
                .name
                .as_str()
                .strip_prefix(&prefix)
                .unwrap_or(decl.name.as_str())
                .to_owned(),
            label: decl.label.as_str().to_owned(),
        })
        .collect()
}

/// The actions of every manifest in `texts`. An app named by two manifests (the folders are read
/// in order of trust) is offered once, by the first.
pub fn actions_of_all<'a>(texts: impl IntoIterator<Item = &'a str>) -> Vec<HotActionChoice> {
    texts.into_iter().fold(Vec::new(), |mut all, text| {
        let more = actions_of(text);
        let taken = more
            .first()
            .is_some_and(|first| all.iter().any(|have| have.app == first.app));
        if !taken {
            all.extend(more);
        }
        all
    })
}

/// One app's actions in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotActionGroup {
    /// The app's name, as a binding stores it.
    pub app: String,
    /// What the list calls the app.
    pub title: String,
    /// Its actions, by label.
    pub actions: Vec<HotActionChoice>,
}

/// `known`'s actions in groups by app, apps by title and actions by label, narrowed to those
/// whose "App › Action" line holds every word of `query` (any case). No query lists all.
pub fn action_groups(known: &HotKnown, query: &str) -> Vec<HotActionGroup> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut groups: Vec<HotActionGroup> = Vec::new();
    for choice in &known.actions {
        let title = known.app_title(&choice.app);
        let line = format!("{title} {}", choice.label).to_lowercase();
        if !words.iter().all(|word| line.contains(word.as_str())) {
            continue;
        }
        match groups.iter_mut().find(|group| group.app == choice.app) {
            Some(group) => group.actions.push(choice.clone()),
            None => groups.push(HotActionGroup {
                app: choice.app.clone(),
                title,
                actions: vec![choice.clone()],
            }),
        }
    }
    for group in &mut groups {
        group.actions.sort_by(|a, b| a.label.cmp(&b.label));
    }
    groups.sort_by(|a, b| a.title.cmp(&b.title));
    groups
}

/// The apps whose name holds every word of `query` (any case), by name. No query lists all; an
/// app with no name is left out.
pub fn apps_matching(apps: &[HotApp], query: &str) -> Vec<HotApp> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut found: Vec<HotApp> = apps
        .iter()
        .filter(|app| !app.name.is_empty())
        .filter(|app| {
            let name = app.name.to_lowercase();
            words.iter().all(|word| name.contains(word.as_str()))
        })
        .cloned()
        .collect();
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}
