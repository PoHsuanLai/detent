//! What a row's menu lists: the module's choices, with the models this computer runs gathered
//! into the one Local choice.

use super::choice::Gate;
use super::local::LOCAL_ITEM;
use super::model::{AUTO, ModelRow};
use crate::coding_agents::GROUP_KEY;

/// One line of a row's menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuLine {
    /// A group's name, as the module gave it, over the choices that follow it.
    Header(String),
    /// The coding agents' group over the choices that follow it.
    Agents,
    /// A choice.
    Pick {
        /// What picking it sends.
        word: String,
        /// What it is called.
        label: PickLabel,
        /// Whether it can be picked.
        gate: Gate,
    },
}

/// What a menu choice is called: a fact, worded by the window, or the module's own name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickLabel {
    /// The empty word: no model chosen.
    Off,
    /// The models this computer runs.
    Local,
    /// A model or choice, by the name the module gave it.
    Named(String),
}

/// Whether a menu offers the way to clear the choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clearing {
    /// The menu lists the empty word, which the window calls Off.
    Offered,
    /// It does not: the first-run pages have a Skip of their own.
    Withheld,
}

impl ModelRow {
    fn pick(&self, word: &str) -> MenuLine {
        MenuLine::Pick {
            word: word.to_owned(),
            label: if word.is_empty() {
                PickLabel::Off
            } else {
                PickLabel::Named(self.label_of(word))
            },
            gate: self.gate(word),
        }
    }

    fn local_pick() -> MenuLine {
        MenuLine::Pick {
            word: LOCAL_ITEM.to_owned(),
            label: PickLabel::Local,
            gate: Gate::Open,
        }
    }

    /// The row's menu: the ungrouped choices first with Local right after Automatic, then each
    /// group under its name. The models this computer runs are not listed; Local asks for them,
    /// and a group left with none of its own choices has no header.
    pub fn menu(&self, clearing: Clearing) -> Vec<MenuLine> {
        let mut lines = Vec::new();
        let mut local_listed = false;
        for section in &self.sections {
            let words: Vec<&String> = section
                .words
                .iter()
                .filter(|word| !self.is_local(word))
                .filter(|word| !(word.is_empty() && clearing == Clearing::Withheld))
                .collect();
            if words.is_empty() {
                continue;
            }
            if let Some(group) = &section.group {
                if !local_listed {
                    lines.push(Self::local_pick());
                    local_listed = true;
                }
                lines.push(if group == GROUP_KEY {
                    MenuLine::Agents
                } else {
                    MenuLine::Header(group.clone())
                });
            }
            for word in words {
                lines.push(self.pick(word));
                if word == AUTO && !local_listed {
                    lines.push(Self::local_pick());
                    local_listed = true;
                }
            }
        }
        if !local_listed {
            lines.push(Self::local_pick());
        }
        lines
    }
}
