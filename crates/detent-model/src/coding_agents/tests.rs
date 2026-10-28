use super::*;
use docket_settings::{
    AcpAgents, AgentRow, Agents, Found, Install, KNOWN, ModelState, Offers, SignInState, Source,
};
use ds_settings::schema::{
    AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section, WordLabels,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

fn row(program: &str, label: &str, source: Source, sign_in: SignInState) -> AgentRow {
    AgentRow {
        program: program.to_owned(),
        label: label.to_owned(),
        source,
        command: None,
        model: ModelState::AgentsOwn,
        offers: Offers::NotSeenYet,
        sign_in,
        way: None,
        rewind: docket_core::Rewind::Docket,
    }
}

fn installed() -> Source {
    Source::Own {
        installed: Install::Installed,
    }
}

pub(super) fn agents() -> Agents {
    Agents {
        rows: vec![
            row(
                "claude-code",
                "Claude Code",
                installed(),
                SignInState::SignedIn,
            ),
            row(
                "gemini",
                "Gemini",
                Source::Own {
                    installed: Install::NotInstalled,
                },
                SignInState::NotSeenYet,
            ),
            row(
                "codex",
                "Codex",
                installed(),
                SignInState::NeedsSignIn { ways: Vec::new() },
            ),
            row("Odd Name", "Odd", installed(), SignInState::SignedIn),
        ],
    }
}

fn menu(key: &str, kind: KeyKind) -> KeySpec {
    KeySpec {
        ends: None,
        control: None,
        path: path(key),
        kind,
        default: toml::Value::String(String::new()),
        label: Label("Balanced".to_owned()),
        help: Help(String::new()),
        page: Page::Intelligence,
        section: Section("Models / Text".to_owned()),
        exposure: Exposure::Basic,
        labels: WordLabels(BTreeMap::from([(
            "local/qwen".to_owned(),
            "qwen".to_owned(),
        )])),
        unavailable: BTreeMap::new(),
        groups: BTreeMap::new(),
        places: BTreeMap::new(),
        note: None,
        agent: AgentSetting::HandsOff,
    }
}

fn variants() -> Vec<String> {
    ["", "auto", "local/qwen"].map(str::to_owned).to_vec()
}

pub(super) fn described() -> Described {
    Described {
        keys: vec![
            menu(
                "ai.model.text.balanced",
                KeyKind::Menu {
                    variants: variants(),
                },
            ),
            menu(
                "ai.model.computer_use.balanced",
                KeyKind::Menu {
                    variants: variants(),
                },
            ),
            menu(
                "ai.auto.pool.text",
                KeyKind::List(Box::new(KeyKind::Menu {
                    variants: vec!["local/qwen".to_owned()],
                })),
            ),
        ],
        values: BTreeMap::new(),
        lists: BTreeMap::new(),
    }
}

fn spec<'a>(described: &'a Described, key: &str) -> &'a KeySpec {
    described
        .keys
        .iter()
        .find(|spec| spec.path.0 == key)
        .expect("the key")
}

/// The group's key in these tests.
pub(super) const GROUP: &str = super::GROUP_KEY;

/// The reason of a word of Automatic's list in these tests.
pub(super) const CHOOSE_DIRECTLY: &str = super::CHOOSE_DIRECTLY_KEY;

fn reasons(spec: &KeySpec) -> Vec<(&str, &str)> {
    spec.unavailable
        .iter()
        .map(|(word, reason)| (word.0.as_str(), reason.0.as_str()))
        .collect()
}

#[test]
fn each_agent_is_ready_or_greyed_with_the_first_reason_a_person_can_act_on() {
    let offered = offers(&agents(), &[], AcpAgents::Off);
    let got: Vec<(&str, &str, Option<AgentReason>)> = offered
        .iter()
        .map(|offer| {
            (
                offer.word.as_str(),
                offer.label.as_str(),
                offer.blocked.clone(),
            )
        })
        .collect();
    let blocked = Some;
    assert_eq!(
        got,
        [
            (
                "agent/claude-code",
                "Claude Code",
                blocked(AgentReason::TurnedOff)
            ),
            ("agent/gemini", "Gemini", blocked(AgentReason::NotInstalled)),
            (
                "agent/codex",
                "Codex",
                blocked(AgentReason::NeedsSignIn {
                    label: "Codex".to_owned()
                })
            ),
        ],
        "not installed beats the switch; a name that is no id is no word"
    );
    let on = offers(&agents(), &[], AcpAgents::On);
    assert_eq!(on[0].blocked, None);
    assert_eq!(on[1].blocked, Some(AgentReason::NotInstalled));
}

#[test]
fn a_known_program_no_entry_runs_is_listed_as_not_set_up_once_per_agent() {
    let at = |program: &str| PathBuf::from(format!("/usr/bin/{program}"));
    let found = vec![
        Found {
            known: KNOWN[0],
            at: at("claude-agent-acp"),
        },
        Found {
            known: KNOWN[1],
            at: at("claude"),
        },
        Found {
            known: KNOWN[3],
            at: at("codex"),
        },
    ];
    let none = Agents { rows: Vec::new() };
    let offered = offers(&none, &found, AcpAgents::On);
    let words: Vec<(&str, &str)> = offered
        .iter()
        .map(|offer| (offer.word.as_str(), offer.label.as_str()))
        .collect();
    assert_eq!(
        words,
        [
            ("agent/claude-agent-acp", "Claude Code"),
            ("agent/codex", "Codex")
        ]
    );
    assert!(
        offered
            .iter()
            .all(|offer| offer.blocked == Some(AgentReason::NotSetUp))
    );
}

#[test]
fn the_group_lands_on_the_text_rows_and_the_pool_and_on_nothing_else() {
    let offered = offers(&agents(), &[], AcpAgents::On);
    let laid = with_agents(described(), &offered);

    let row = spec(&laid, "ai.model.text.balanced");
    let KeyKind::Menu { variants } = &row.kind else {
        panic!("a menu");
    };
    assert_eq!(
        variants,
        &[
            "",
            "auto",
            "local/qwen",
            "agent/claude-code",
            "agent/gemini",
            "agent/codex"
        ]
    );
    assert_eq!(row.labels.of("agent/claude-code"), Some("Claude Code"));
    assert!(row.groups.iter().all(|(_, group)| group.0 == GROUP));
    assert_eq!(row.groups.len(), 3);
    assert_eq!(
        reasons(row),
        [
            ("agent/codex", "agents:sign-in:Codex"),
            ("agent/gemini", "agents:not-installed")
        ],
        "a ready agent carries no reason"
    );

    let pool = spec(&laid, "ai.auto.pool.text");
    assert!(
        reasons(pool)
            .iter()
            .all(|(_, reason)| *reason == CHOOSE_DIRECTLY)
    );
    assert_eq!(
        reasons(pool).len(),
        3,
        "Automatic never takes an agent, ready or not"
    );

    assert_eq!(
        spec(&laid, "ai.model.computer_use.balanced"),
        spec(&described(), "ai.model.computer_use.balanced"),
        "another slot is as the module described it"
    );
}
