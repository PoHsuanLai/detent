//! The outside assistants docket may start (Claude Code and the like), one row each: what it is,
//! how it stands, the model it runs and, when it asked for a sign-in, the way to sign in. What the
//! agent offered is what docket recorded when it last started; opening this page starts nothing.

use crate::env::Env;
use detent_files::{load_agents, save_agent_choice};
use detent_model::{Entry, model_choice, model_menu, model_word, way_choice, way_menu};
use dioxus::prelude::*;
use docket_settings::{AgentChoice, AgentRow};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The group's title.
const TITLE: &str = "Coding assistants";

fn items_of(menu: &[Entry]) -> Vec<MenuItem<String>> {
    menu.iter()
        .map(|entry| MenuItem::new(entry.word.clone(), entry.name.clone()))
        .collect()
}

/// The rows of the outside assistants `agents.toml` lists, nothing when it lists none.
#[component]
pub(super) fn OutsideAssistants() -> Element {
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
    let rows = agents().rows;
    if rows.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "assistants", "data-assistants": "coding",
            FieldGroup { title: Some(TITLE.to_owned()),
                for row in rows {
                    OutsideRow { key: "{row.program}", row, choose }
                }
            }
        }
    }
}

#[component]
fn OutsideRow(row: AgentRow, choose: EventHandler<(String, AgentChoice)>) -> Element {
    let models = model_menu(&row);
    let ways = way_menu(&row);
    let help = format!("{}. {}", row.install_line(), row.sign_in_line());
    let current = model_word(&row).unwrap_or_default();
    let way = row.way.clone();
    let (for_model, for_way) = (row.program.clone(), row.program.clone());
    rsx! {
        FieldRow { label: TextLine::from(row.label.clone()), help: Some(TextLine::from(help)),
            PopUpButton::<String> {
                items: items_of(&models),
                value: Some(current),
                size: ControlSize::Small,
                onpick: move |word: String| choose.call((for_model.clone(), model_choice(&word))),
            }
        }
        if !ways.is_empty() {
            FieldRow { label: TextLine::from("Sign in with"),
                PopUpButton::<String> {
                    items: items_of(&ways),
                    value: way,
                    size: ControlSize::Small,
                    onpick: move |word: String| choose.call((for_way.clone(), way_choice(&word))),
                }
            }
        }
    }
}
