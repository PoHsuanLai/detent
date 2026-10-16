//! What a row's menu lists: the module's choices, with the models this computer runs gathered
//! into the one Local choice.

use super::choice::Gate;
use super::local::{LOCAL_ITEM, LOCAL_LABEL};
use super::model::{AUTO, ModelRow};

/// One line of a row's menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuLine {
    /// A group's name over the choices that follow it.
    Header(String),
    /// A choice.
    Pick {
        /// What picking it sends.
        word: String,
        /// What a person reads for it.
        label: String,
        /// Whether it can be picked.
        gate: Gate,
    },
}

/// Whether a menu offers the way to clear the choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clearing {
    /// The menu lists "Off", the empty word.
    Offered,
    /// It does not: the first-run pages have a Skip of their own.
    Withheld,
}

impl ModelRow {
    fn pick(&self, word: &str) -> MenuLine {
        MenuLine::Pick {
            word: word.to_owned(),
            label: self.label_of(word),
            gate: self.gate(word),
        }
    }

    fn local_pick() -> MenuLine {
        MenuLine::Pick {
            word: LOCAL_ITEM.to_owned(),
            label: LOCAL_LABEL.to_owned(),
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
                lines.push(MenuLine::Header(group.clone()));
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
