use super::tests::{CHOOSE_DIRECTLY, agents, described, path};
use super::{AgentOffer, offers, with_agents};
use crate::picker::{
    Clearing, Gate, MenuLine, PickerEffect, PickerInput, PickerState, TurnOnAnswer, step,
};
use docket_settings::AcpAgents;

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
        .position(|line| *line == MenuLine::Agents)
        .expect("the group's header");
    let words: Vec<(&str, Gate)> = menu[at + 1..]
        .iter()
        .filter_map(|line| match line {
            MenuLine::Pick { word, gate, .. } => Some((word.as_str(), gate.clone())),
            MenuLine::Header(_) | MenuLine::Agents => None,
        })
        .collect();
    assert_eq!(
        words,
        [
            ("agent/claude-code", Gate::Open),
            (
                "agent/gemini",
                Gate::Blocked("agents:not-installed".to_owned())
            ),
            (
                "agent/codex",
                Gate::Blocked("agents:sign-in:Codex".to_owned())
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
    assert_eq!(
        effects,
        [PickerEffect::Tell("agents:not-installed".to_owned())]
    );
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
        Gate::Blocked(CHOOSE_DIRECTLY.to_owned())
    );
    let pool_path = pool.path.clone().expect("the pool's key");
    let (next, effects) = step(
        std::mem::take(&mut state),
        PickerInput::PoolToggled {
            path: pool_path,
            word: "agent/claude-code".to_owned(),
        },
    );
    assert_eq!(effects, [PickerEffect::Tell(CHOOSE_DIRECTLY.to_owned())]);
    assert!(matches!(next, PickerState::Ready(_)));
}
