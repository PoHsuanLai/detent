//! The outside assistants (Claude Code and the like), one row each, whichever of the two sources
//! know it: docket's (what it is, the model it runs, the way it signs in) and accountd's (how its
//! sign-in stands, and the way to its account). What the agent offered is what docket recorded
//! when it last started; opening this page starts nothing.

use crate::accounts::{AccountLink, OpenButton};
use crate::env::Env;
use detent_files::{load_agents, save_agent_choice};
use detent_model::{
    AccountsPane, Assistant, Entry, assistants_of, model_choice, model_menu, model_word,
    way_choice, way_menu,
};
use dioxus::prelude::*;
use docket_settings::{AgentChoice, AgentRow};
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use porter_core::sheet::ProviderGroup;

/// The group's title.
const TITLE: &str = "Coding assistants";

fn items_of(menu: &[Entry]) -> Vec<MenuItem<String>> {
    menu.iter()
        .map(|entry| MenuItem::new(entry.word.clone(), entry.name.clone()))
        .collect()
}

/// The rows of the outside assistants `agents.toml` lists and the sign-ins accountd lists,
/// nothing when there are none.
#[component]
pub(super) fn OutsideAssistants(pane: AccountsPane) -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let mut agents = use_signal({
        let env = env.clone();
        move || load_agents(&env.config_home, &env.data_home)
    });
    let choose = EventHandler::new(move |(program, choice): (String, AgentChoice)| {
        if let Err(why) = save_agent_choice(&env.config_home, &program, &choice) {
            toasts.push(format!("Could not save that choice. {why}"), None);
        }
        agents.set(load_agents(&env.config_home, &env.data_home));
    });
    let accounts: Vec<_> = pane.in_group(ProviderGroup::Agent).cloned().collect();
    let rows = assistants_of(&agents().rows, &accounts);
    if rows.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "assistants", "data-assistants": "coding",
            FieldGroup { title: Some(TITLE.to_owned()),
                for assistant in rows {
                    OutsideRow { key: "{assistant.name}", assistant, choose }
                }
            }
        }
    }
}

#[component]
fn OutsideRow(assistant: Assistant, choose: EventHandler<(String, AgentChoice)>) -> Element {
    let link = use_context::<AccountLink>();
    let Assistant {
        name,
        agent,
        account,
    } = assistant;
    // Docket's sign-in line is what the agent last said; once accountd knows the account, its
    // status says it live.
    let known = account.is_some();
    let help = agent.as_ref().map(|row| {
        if known {
            row.install_line()
        } else {
            format!("{}. {}", row.install_line(), row.sign_in_line())
        }
    });
    rsx! {
        FieldRow { label: TextLine::from(name.clone()), help: help.map(TextLine::from),
            if let Some(account) = account.clone() {
                Label { text: account.status.label(), role: LabelRole::Secondary, style: LabelStyle::Footnote }
                OpenButton {
                    label: if account.reauth.is_some() { "Sign in" } else { "Manage" },
                    subject: account.label.clone(),
                    onpress: {
                        let key = account.key.clone();
                        move |()| link.open(key.clone())
                    },
                }
            }
            if let Some(row) = agent.clone() {
                ModelMenu { row, choose }
            }
        }
        if let Some(row) = agent {
            WayRow { row, choose }
        }
    }
}

#[component]
fn ModelMenu(row: AgentRow, choose: EventHandler<(String, AgentChoice)>) -> Element {
    let models = model_menu(&row);
    let current = model_word(&row).unwrap_or_default();
    let program = row.program.clone();
    rsx! {
        PopUpButton::<String> {
            items: items_of(&models),
            value: Some(current),
            size: ControlSize::Small,
            onpick: move |word: String| choose.call((program.clone(), model_choice(&word))),
        }
    }
}

#[component]
fn WayRow(row: AgentRow, choose: EventHandler<(String, AgentChoice)>) -> Element {
    let ways = way_menu(&row);
    let program = row.program.clone();
    if ways.is_empty() {
        return rsx! {};
    }
    rsx! {
        FieldRow { label: TextLine::from("Sign in with"),
            PopUpButton::<String> {
                items: items_of(&ways),
                value: row.way.clone(),
                size: ControlSize::Small,
                onpick: move |word: String| choose.call((program.clone(), way_choice(&word))),
            }
        }
    }
}
