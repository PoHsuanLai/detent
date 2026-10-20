use super::*;
use crate::picker::{
    Clearing, Gate, MenuLine, PickerEffect, PickerInput, PickerState, TurnOnAnswer, step,
};
use docket_settings::{
    AcpAgents, AgentRow, Agents, Found, Install, KNOWN, ModelState, Offers, SignInState, Source,
};
use ds_settings::schema::{
    AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section, WordLabels,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn path(text: &str) -> KeyPath {
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

fn agents() -> Agents {
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

fn described() -> Described {
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

fn reasons(spec: &KeySpec) -> Vec<(&str, &str)> {
    spec.unavailable
        .iter()
        .map(|(word, reason)| (word.0.as_str(), reason.0.as_str()))
        .collect()
}

#[test]
fn each_agent_is_ready_or_greyed_with_the_first_reason_a_person_can_act_on() {
    let offered = offers(&agents(), &[], AcpAgents::Off);
    let got: Vec<(&str, &str, String)> = offered
        .iter()
        .map(|offer| {
            let standing = match &offer.standing {
                AgentStanding::Ready => "ready".to_owned(),
                AgentStanding::Blocked(reason) => reason.words(),
            };
            (offer.word.as_str(), offer.label.as_str(), standing)
        })
        .collect();
    assert_eq!(
        got,
        [
            ("agent/claude-code", "Claude Code", "Turned off".to_owned()),
            ("agent/gemini", "Gemini", "Not installed".to_owned()),
            ("agent/codex", "Codex", "Sign in to Codex first".to_owned()),
        ],
        "not installed beats the switch; a name that is no id is no word"
    );
    let on = offers(&agents(), &[], AcpAgents::On);
    assert_eq!(on[0].standing, AgentStanding::Ready);
    assert_eq!(
        on[1].standing,
        AgentStanding::Blocked(AgentReason::NotInstalled)
    );
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
            .all(|offer| offer.standing == AgentStanding::Blocked(AgentReason::NotSetUp))
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
    assert!(
        row.groups
            .iter()
            .all(|(_, group)| group.0 == CODING_AGENTS_GROUP_FOR_TEST)
    );
    assert_eq!(row.groups.len(), 3);
    assert_eq!(
        reasons(row),
        [
            ("agent/codex", "Sign in to Codex first"),
            ("agent/gemini", "Not installed")
        ],
        "a ready agent carries no reason"
    );

    let pool = spec(&laid, "ai.auto.pool.text");
    assert!(
        reasons(pool)
            .iter()
            .all(|(_, reason)| *reason == "Choose it directly")
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

const CODING_AGENTS_GROUP_FOR_TEST: &str = "Coding Agents";

fn ready_with(offered: &[AgentOffer]) -> PickerState {
    let laid = with_agents(described(), offered);
    step(PickerState::Looking, PickerInput::Found(Ok(laid))).0
}

fn row_of(state: &PickerState) -> &crate::picker::ModelRow {
    let PickerState::Ready(ready) = state else {
        panic!("ready");
    };
    ready
        .row(&path("ai.model.text.balanced"))
        .expect("the assistant's row")
}

#[test]
fn the_menu_lists_the_agents_under_their_own_header_and_a_blocked_one_cannot_be_picked() {
    let state = ready_with(&offers(&agents(), &[], AcpAgents::On));
    let menu = row_of(&state).menu(Clearing::Offered);
    let at = menu
        .iter()
        .position(|line| *line == MenuLine::Header("Coding Agents".to_owned()))
        .expect("the group's header");
    let words: Vec<(&str, Gate)> = menu[at + 1..]
        .iter()
        .filter_map(|line| match line {
            MenuLine::Pick { word, gate, .. } => Some((word.as_str(), gate.clone())),
            MenuLine::Header(_) => None,
        })
        .collect();
    assert_eq!(
        words,
        [
            ("agent/claude-code", Gate::Open),
            ("agent/gemini", Gate::Blocked("Not installed".to_owned())),
            (
                "agent/codex",
                Gate::Blocked("Sign in to Codex first".to_owned())
            ),
        ]
    );

    // Picking the blocked one says why and sets nothing.
    let (state, effects) = step(
        state,
        PickerInput::Chose {
            path: path("ai.model.text.balanced"),
            word: "agent/gemini".to_owned(),
        },
    );
    assert_eq!(effects, [PickerEffect::Tell("Not installed".to_owned())]);
    assert_eq!(row_of(&state).value, "");
}

#[test]
fn picking_a_ready_agent_sets_it_like_any_choice() {
    let state = ready_with(&offers(&agents(), &[], AcpAgents::On));
    let (_, effects) = step(
        state,
        PickerInput::Chose {
            path: path("ai.model.text.balanced"),
            word: "agent/claude-code".to_owned(),
        },
    );
    assert_eq!(
        effects,
        [PickerEffect::Set {
            path: path("ai.model.text.balanced"),
            word: "agent/claude-code".to_owned()
        }]
    );
}

#[test]
fn picking_a_turned_off_agent_asks_and_a_yes_turns_them_on_and_sets_it() {
    let key = path("ai.model.text.balanced");
    let state = ready_with(&offers(&agents(), &[], AcpAgents::Off));
    assert_eq!(row_of(&state).gate("agent/claude-code"), Gate::TurnOn);

    // The pick asks; nothing is set, and no other pick is taken while the question stands.
    let (state, effects) = step(
        state,
        PickerInput::Chose {
            path: key.clone(),
            word: "agent/claude-code".to_owned(),
        },
    );
    assert!(effects.is_empty());
    let PickerState::Ready(asked) = &state else {
        panic!("ready");
    };
    assert_eq!(
        asked.turning_on().map(|t| t.word.as_str()),
        Some("agent/claude-code")
    );
    let (state, effects) = step(
        state,
        PickerInput::Chose {
            path: key.clone(),
            word: "".to_owned(),
        },
    );
    assert!(effects.is_empty());

    // Yes asks the app to turn them on; once it has, the agent is set.
    let (state, effects) = step(state, PickerInput::TurnOn(TurnOnAnswer::Yes));
    assert_eq!(effects, [PickerEffect::TurnOnAgents]);
    let (state, effects) = step(state, PickerInput::AgentsTurnedOn(Ok(())));
    assert_eq!(
        effects,
        [PickerEffect::Set {
            path: key,
            word: "agent/claude-code".to_owned()
        }]
    );
    assert_eq!(row_of(&state).gate("agent/claude-code"), Gate::Open);
}

#[test]
fn a_no_leaves_them_off_and_a_failed_turn_on_says_so() {
    let key = path("ai.model.text.balanced");
    let asked = |state| {
        step(
            state,
            PickerInput::Chose {
                path: key.clone(),
                word: "agent/claude-code".to_owned(),
            },
        )
        .0
    };
    let off = ready_with(&offers(&agents(), &[], AcpAgents::Off));

    let (state, effects) = step(asked(off.clone()), PickerInput::TurnOn(TurnOnAnswer::No));
    assert!(effects.is_empty());
    assert_eq!(row_of(&state).gate("agent/claude-code"), Gate::TurnOn);
    assert_eq!(row_of(&state).value, "");

    let why = "Could not save that choice.".to_owned();
    let (state, effects) = step(asked(off), PickerInput::AgentsTurnedOn(Err(why.clone())));
    assert_eq!(effects, [PickerEffect::Tell(why)]);
    assert_eq!(row_of(&state).gate("agent/claude-code"), Gate::TurnOn);
}

#[test]
fn an_agent_in_automatics_list_is_greyed_and_cannot_be_added() {
    let mut state = ready_with(&offers(&agents(), &[], AcpAgents::On));
    let pool = {
        let PickerState::Ready(ready) = &state else {
            panic!("ready");
        };
        ready
            .row(&path("ai.model.text.balanced"))
            .expect("row")
            .pool
            .clone()
    };
    assert_eq!(
        pool.gate("agent/claude-code"),
        Gate::Blocked("Choose it directly".to_owned())
    );
    let pool_path = pool.path.clone().expect("the pool's key");
    let (next, effects) = step(
        std::mem::take(&mut state),
        PickerInput::PoolToggled {
            path: pool_path,
            word: "agent/claude-code".to_owned(),
        },
    );
    assert_eq!(
        effects,
        [PickerEffect::Tell("Choose it directly".to_owned())]
    );
    assert!(matches!(next, PickerState::Ready(_)));
}
