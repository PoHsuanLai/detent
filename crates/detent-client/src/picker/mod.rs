//! inferd's model picker, as the Intelligence page meets it: its live rows with the coding agents
//! docket lists laid over them, the picks the person makes, and the reasons it gives for a pick.
//! The steps are `detent-model`'s `step`.

mod models;
mod reasons;
#[cfg(test)]
mod reasons_tests;

pub use models::Models;
pub use reasons::{ReasonFeed, Reasons};

use crate::follow::{Followed, Slot, follow_with};
use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_files::{ConfigHome, DataHome, agents_switch, load_agents, programs_on, turn_on_agents};
use detent_model::{
    AgentWord, Described, PickerEffect, PickerInput, PickerState, agent_offers,
    picker_row_is_choice as is_choice, step, with_agents,
};
use detent_system::accounts::AccountsLink;
use detent_system::bus::{Daemon, words_of};
use detent_system::modules::{ModulesLink, reread};
use detent_system::picker::PickerLink;
use std::sync::Arc;

/// Where the coding agents are looked for: docket's files and the programs on `PATH`.
#[derive(Debug, Clone)]
pub(crate) struct AgentSearch {
    pub config_home: ConfigHome,
    pub data_home: DataHome,
    pub programs_path: String,
}

/// The picker's handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Picker {
    modules: Arc<dyn ModulesLink>,
    link: Arc<dyn PickerLink>,
    accounts: Arc<dyn AccountsLink>,
    search: AgentSearch,
    module: Slot,
    live: Live<PickerState>,
}

pane!(Picker, PickerState, follow);

impl Picker {
    pub(crate) fn over(
        modules: Arc<dyn ModulesLink>,
        link: Arc<dyn PickerLink>,
        accounts: Arc<dyn AccountsLink>,
        search: AgentSearch,
    ) -> Picker {
        Picker {
            modules,
            link,
            accounts,
            search,
            module: Slot::default(),
            live: Live::default(),
        }
    }

    /// Gives the pane what the person asked of a row: a pick, an engine for Local, a model in or
    /// out of Automatic's list. The returned work carries out what it asks for.
    pub fn send(&self, input: PickerInput) -> Pending {
        self.dispatch(input)
    }

    /// inferd's rows with the coding agents docket lists laid over the text slot's: the module
    /// knows no agent, and docket's files say which exist and whether they can be chosen now.
    fn with_coding_agents(&self, described: Described) -> Described {
        let agents = load_agents(&self.search.config_home, &self.search.data_home);
        let found = programs_on(&self.search.programs_path);
        let offers = agent_offers(&agents, &found, agents_switch(&self.search.config_home));
        with_agents(described, &offers)
    }

    /// Looks for inferd's picker and follows it; never starts inferd.
    async fn follow(&self) {
        follow_with(
            &*self.modules,
            &Daemon::inferd(),
            is_choice,
            async |followed| match followed {
                Followed::Opened(opened, described) => {
                    let described = self.with_coding_agents(described);
                    self.dispatch(PickerInput::Found(Ok(described))).await;
                    self.module.put(Some(opened));
                }
                Followed::Gone(absence) => {
                    self.dispatch(PickerInput::Found(Err(absence))).await;
                    self.module.put(None);
                }
                Followed::Changed(path, toml::Value::String(word)) => {
                    self.dispatch(PickerInput::Changed { path, word }).await;
                }
                Followed::Changed(path, toml::Value::Array(items)) => {
                    let words = words_of(&items);
                    self.dispatch(PickerInput::PoolChanged { path, words })
                        .await;
                }
                Followed::Changed(..) => {}
            },
        )
        .await;
    }
}

impl Machine for Picker {
    type Input = PickerInput;
    type Effect = PickerEffect;

    fn step(state: PickerState, input: PickerInput) -> (PickerState, Vec<PickerEffect>) {
        step(state, input)
    }

    async fn run(&self, effect: PickerEffect) -> Vec<PickerInput> {
        match effect {
            PickerEffect::Tell(text) => {
                self.live.notice(match AgentWord::of_key(&text) {
                    Some(word) => Notice::AgentBlocked(word),
                    None => Notice::Words(text),
                });
                Vec::new()
            }
            PickerEffect::AccountNotAdded(end) => {
                self.live.notice(Notice::AddEnded(end));
                Vec::new()
            }
            PickerEffect::PoolRefused => {
                self.live.notice(Notice::PoolRefused);
                Vec::new()
            }
            PickerEffect::Set { path, word } => {
                let Some(opened) = self.module.get() else {
                    return Vec::new();
                };
                vec![match self.link.set(&opened, &path, &word).await {
                    Ok(()) => PickerInput::Applied(path),
                    Err(why) => PickerInput::Refused { path, why },
                }]
            }
            PickerEffect::SetPool { path, words } => {
                let Some(opened) = self.module.get() else {
                    return Vec::new();
                };
                vec![match self.link.set_pool(&opened, &path, &words).await {
                    Ok(()) => PickerInput::PoolApplied(path),
                    Err(_) => PickerInput::PoolRefused(path),
                }]
            }
            PickerEffect::AddAccount => {
                vec![PickerInput::AccountSheetEnded(
                    self.accounts.add_account().await,
                )]
            }
            PickerEffect::TurnOnAgents => {
                let done = turn_on_agents(&self.search.config_home)
                    .map_err(|why| format!("Could not turn on coding agents. {why}"));
                vec![PickerInput::AgentsTurnedOn(done)]
            }
            PickerEffect::ReadAgain => {
                let Some(opened) = self.module.get() else {
                    return Vec::new();
                };
                vec![match reread(&opened, is_choice).await {
                    Ok(described) => PickerInput::Reread(self.with_coding_agents(described)),
                    Err(absence) => PickerInput::Found(Err(absence)),
                }]
            }
        }
    }
}
