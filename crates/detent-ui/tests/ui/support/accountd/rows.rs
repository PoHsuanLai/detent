//! The rows a held account describes, as the real modules do.

use super::*;

pub(super) fn spec(
    path: String,
    section: &str,
    label: &str,
    kind: KeyKind,
    default: toml::Value,
) -> KeySpec {
    KeySpec {
        ends: None,
        control: None,
        path: KeyPath(path),
        kind,
        default,
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Accounts,
        section: Section(section.to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
        agent: AgentSetting::HandsOff,
    }
}

pub(super) fn action(name: &str, weight: ActionWeight) -> KeyKind {
    KeyKind::Live {
        action: LiveAction {
            label: ActionLabel(name.to_owned()),
            weight,
        },
    }
}

pub(super) fn rows_of(held: &Held) -> Vec<KeySpec> {
    let id = &held.id;
    let section = held.label.as_str();
    let fixed = |leaf: &str, label: &str, word: &str| {
        spec(
            format!("accounts.{id}.{leaf}"),
            section,
            label,
            KeyKind::Fixed {
                variant: word.to_owned(),
            },
            toml::Value::String(word.to_owned()),
        )
    };
    let mut rows = vec![
        fixed("label", "Account", &held.label),
        fixed("state", "Status", &held.state),
    ];
    if let Some(words) = &held.expires {
        let mut row = fixed("expires", "Sign-in", words);
        row.help = Help(
            "Google ends the sign-ins of an app that is not verified yet after 7 days; signing in again starts another 7."
                .to_owned(),
        );
        rows.push(row);
    }
    if let Some(slug) = &held.sign_in {
        rows.push(fixed("sign_in", "How", slug));
    }
    if let Some((provider, name)) = &held.provider {
        let mut row = fixed("provider", "Account type", provider);
        row.labels = ds_settings::schema::WordLabels(
            [(provider.clone(), name.clone())].into_iter().collect(),
        );
        rows.push(row);
    }
    if let Some(slug) = &held.group {
        let mut row = fixed("group", "Group", slug);
        row.labels = ds_settings::schema::WordLabels(
            [
                ("internet", "Internet Accounts"),
                ("intelligence", "Intelligence"),
                ("agent", "Assistants"),
            ]
            .into_iter()
            .map(|(word, label)| (word.to_owned(), label.to_owned()))
            .collect(),
        );
        rows.push(row);
    }
    if held.on_this_computer {
        rows.push(fixed("place", "Where", "this_computer"));
    }
    for (kind, _) in &held.services {
        rows.push(spec(
            format!("accounts.{id}.service.{kind}"),
            section,
            kind,
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()],
            },
            toml::Value::String("on".to_owned()),
        ));
    }
    for (leaf, _) in &held.sync {
        let label = match leaf.as_str() {
            "files" => "Keep this account's files on this computer",
            _ => "Upload new photos from a folder on this computer to Google Photos",
        };
        let mut row = spec(
            format!("accounts.{id}.sync.{leaf}"),
            section,
            label,
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()],
            },
            toml::Value::String("off".to_owned()),
        );
        if leaf == "photos" {
            row.help = Help(
                "Sends each new photo you put in the upload folder to an album in Google Photos. \
                 It never reads or deletes anything else in your Google Photos."
                    .to_owned(),
            );
        }
        rows.push(row);
    }
    if let Some((_, help)) = &held.show_in_files {
        let mut row = spec(
            format!("accounts.{id}.storage.files"),
            section,
            "Show in Files",
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()],
            },
            toml::Value::String("off".to_owned()),
        );
        row.help = Help(help.clone());
        rows.push(row);
    }
    for (n, app) in held.used_by.iter().enumerate() {
        rows.push(fixed(&format!("used_by.{n}"), "Used by", app));
    }
    if let Some((month, cap)) = held.spend {
        rows.push(fixed("spend.month", "This month", &month.to_string()));
        rows.push(fixed("spend.cap", "Cap", &cap.to_string()));
    }
    for (grant, what) in &held.grants {
        rows.push(spec(
            format!("accounts.{id}.grant.{grant}"),
            section,
            what,
            action("Revoke", ActionWeight::Plain),
            toml::Value::Boolean(false),
        ));
        if let Some((_, since, scope)) = held.grant_facts.iter().find(|(g, ..)| g == grant) {
            rows.push(fixed(&format!("grant.{grant}.since"), "Since", since));
            rows.push(fixed(&format!("grant.{grant}.scope"), "Lasts", scope));
        }
    }
    rows.push(spec(
        format!("accounts.{id}.reauth"),
        section,
        "Sign in again",
        action("Sign in again", ActionWeight::Plain),
        toml::Value::Boolean(false),
    ));
    rows.push(spec(
        format!("accounts.{id}.remove"),
        section,
        "Remove account",
        action("Remove account", ActionWeight::Destructive),
        toml::Value::Boolean(false),
    ));
    rows
}
