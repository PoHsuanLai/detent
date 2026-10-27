//! The outside assistants (Claude Code and the like), one row each, whichever of the two sources
//! know it: docket's (what it is, the model it runs, the way it signs in) and accountd's (how its
//! sign-in stands, and the way to its account). What the agent offered is what docket recorded
//! when it last started; opening this page starts nothing.

use super::add_flow;
use super::add_view::{AddSheet, RemoveAlert};
use crate::accounts::{AccountLink, OpenButton};
use crate::copy::{assistants, coding_agents};
use crate::env::Env;
use detent_client::{installed_agents, load_agents, remove_agent, save_agent_choice};
use detent_model::external::ProviderGroup;
use detent_model::external::{AgentChoice, AgentRow, Agents};
use detent_model::{
    AccountsPane, AddForm, Assistant, Entry, FoundAgent, RemoveAnswer, RemoveQuestion,
    assistants_of, model_choice, model_menu, model_word, removal_of, unlisted_found, way_choice,
    way_menu,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The group's title.
const TITLE: &str = crate::copy::assistants::CODING_ASSISTANTS;

fn items_of(menu: &[Entry]) -> Vec<MenuItem<String>> {
    menu.iter()
        .map(|entry| {
            let name = entry.name.as_deref().unwrap_or(assistants::agents_own());
            MenuItem::new(entry.word.clone(), name.to_owned())
        })
        .collect()
}

/// What docket's files say now: the assistants listed, and the installed ones not yet listed.
#[derive(Clone)]
pub(super) struct Shown {
    pub(super) agents: Agents,
    pub(super) new: Vec<FoundAgent>,
}

pub(super) fn shown(env: &Env) -> Shown {
    let agents = load_agents(&env.config_home, &env.data_home);
    let new = unlisted_found(&installed_agents(&env.data_home), &agents.rows);
    Shown { agents, new }
}

/// The rows of the outside assistants `agents.toml` lists and the sign-ins accountd lists, with
/// Add where an installed assistant is not listed yet; nothing when there is neither.
#[component]
pub(super) fn OutsideAssistants(pane: AccountsPane) -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let mut now = use_signal({
        let env = env.clone();
        move || shown(&env)
    });
    let mut adding = use_signal(|| None::<AddForm>);
    let mut removing = use_signal(|| None::<RemoveQuestion>);
    let choose = EventHandler::new({
        let env = env.clone();
        move |(program, choice): (String, AgentChoice)| {
            if let Err(why) = save_agent_choice(&env.config_home, &program, &choice) {
                toasts.push(
                    crate::copy::assistants::could_not_save_that_choice(&why),
                    None,
                );
            }
            now.set(shown(&env));
        }
    });
    let submit = EventHandler::new({
        let env = env.clone();
        move |()| add_flow::submit(&env, adding, now)
    });
    let answer = EventHandler::new({
        let env = env.clone();
        move |answer: RemoveAnswer| {
            let Some(question) = removing.peek().clone() else {
                return;
            };
            removing.set(None);
            if answer == RemoveAnswer::Remove
                && let Err(fault) = remove_agent(&env.config_home, &question.program)
            {
                toasts.push(coding_agents::remove_refusal(fault).to_owned(), None);
            }
            now.set(shown(&env));
        }
    });
    let accounts: Vec<_> = pane.in_group(ProviderGroup::Agent).cloned().collect();
    let Shown { agents, new } = now();
    let rows = assistants_of(&agents.rows, &accounts);
    if rows.is_empty() && new.is_empty() && env.client.agents_program().is_none() {
        return rsx! {};
    }
    // With the program that lists what is on offer, Add is there even when nothing is installed.
    let can_add = !new.is_empty() || env.client.agents_program().is_some();
    rsx! {
        div { class: "assistants", "data-assistants": "coding",
            FieldGroup { title: Some(TITLE.to_owned()),
                for assistant in rows {
                    OutsideRow {
                        key: "{assistant.name}",
                        assistant,
                        choose,
                        remove: EventHandler::new(move |question: RemoveQuestion| removing.set(Some(question))),
                    }
                }
                if can_add {
                    FieldRow { label: TextLine::from(crate::copy::assistants::ADD_A_CODING_ASSISTANT),
                        Button {
                            common: Common { aria_label: Some(crate::copy::assistants::ADD_A_CODING_ASSISTANT.to_owned()), ..Common::default() },
                            label: crate::copy::common::ADD_ELLIPSIS,
                            size: ControlSize::Small,
                            onclick: {
                                let env = env.clone();
                                move |_| add_flow::open(&env, adding, &now.peek())
                            },
                        }
                    }
                }
            }
            if let Some(form) = adding() {
                AddSheet {
                    form,
                    picked: move |id: String| {
                        let next = adding.peek().clone().map(|form| form.pick(&id));
                        adding.set(next);
                    },
                    typed: move |text: String| {
                        let next = adding.peek().clone().map(|form| form.typed(text));
                        adding.set(next);
                    },
                    submit,
                    close: move |()| add_flow::close(adding),
                }
            }
            if let Some(question) = removing() {
                RemoveAlert { question, answer }
            }
        }
    }
}

#[component]
fn OutsideRow(
    assistant: Assistant,
    choose: EventHandler<(String, AgentChoice)>,
    remove: EventHandler<RemoveQuestion>,
) -> Element {
    let link = use_context::<AccountLink>();
    let removal = removal_of(&assistant);
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
                Label { text: crate::copy::accounts::status_label(&account.status), role: LabelRole::Secondary, style: LabelStyle::Footnote }
                OpenButton {
                    label: if account.reauth.is_some() { crate::copy::assistants::SIGN_IN } else { crate::copy::common::MANAGE },
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
            if let Some(question) = removal {
                Button {
                    common: Common { aria_label: Some(crate::copy::assistants::remove(&question.label)), ..Common::default() },
                    label: crate::copy::common::REMOVE,
                    size: ControlSize::Small,
                    onclick: move |_| remove.call(question.clone()),
                }
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
        FieldRow { label: TextLine::from(crate::copy::assistants::SIGN_IN_WITH),
            PopUpButton::<String> {
                items: items_of(&ways),
                value: row.way.clone(),
                size: ControlSize::Small,
                onpick: move |word: String| choose.call((program.clone(), way_choice(&word))),
            }
        }
    }
}
