//! What a page draws, top to bottom, before its Advanced disclosure (design/22 section 5).

use crate::assistant_model::is_assistant_model;
use crate::catalogue::{PageModel, SectionModel};
use crate::updates::is_software_update;
use ds_settings::schema::Page;

/// One thing a page draws.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// A titled card of file rows.
    Section(SectionModel),
    /// inferd's live model picker (design/22 section 9.4): the Intelligence page's rows that are
    /// not in any file.
    ModelPicker,
    /// "On this computer": the local runtimes accountd lists (Ollama, llama.cpp, LM Studio), each
    /// linked to its account. Not in any file.
    LocalRuntimes,
    /// accountd's live Accounts pane (PLAN section 2.5): the Accounts page's rows that are not in
    /// any file.
    AccountsPane,
    /// Where the built-in assistant may run (this computer, "Your computers" with the Add list
    /// and the computers asking to borrow this one's models, cloud accounts), and the assistants
    /// that sign themselves in: the Intelligence page's rows that are in no schema.
    Assistants,
    /// The desktop-wide Spaces accountd keeps, each with Remove: the Spaces page's rows that are
    /// in no file.
    SpacesPane,
    /// The desktop's shortcuts, grouped, each with its chord and who set it: the Keyboard
    /// page's rows that are in no schema.
    Shortcuts,
    /// The update service's page: where the flow stands, and the buttons that move it. Not in
    /// any file.
    SoftwareUpdate,
    /// The assistant's model: inferd's picker, reduced to the one row the assistant is routed by.
    /// Not in any file.
    AssistantModel,
}

/// The Intelligence page's sections, in the order design/22 section 5 gives them. A section a
/// program names that is not listed follows these, in the order the schemas first mention it.
const INTELLIGENCE_SECTIONS: &[&str] =
    &["Companion", "Models", "Memory", "Computer use", "Assistant"];

/// The section the local runtimes and the model picker follow.
const PICKER_AFTER: &str = "Models";

fn rank(title: &str) -> usize {
    INTELLIGENCE_SECTIONS
        .iter()
        .position(|known| *known == title)
        .unwrap_or(INTELLIGENCE_SECTIONS.len())
}

/// The blocks of `page`. Every page lists its sections as the schemas gave them, except
/// Intelligence: its sections follow design/22's order and the model picker sits right after
/// Models (the runtimes first, then the picker), or where Models would be when no schema has that
/// section; and Accounts, whose live pane comes first.
pub fn blocks(page: &PageModel) -> Vec<Block> {
    let sections = page.sections.iter().cloned().map(Block::Section);
    match page.page {
        Page::Accounts => {
            return std::iter::once(Block::AccountsPane)
                .chain(sections)
                .collect();
        }
        Page::Spaces => {
            return std::iter::once(Block::SpacesPane).chain(sections).collect();
        }
        Page::KeyboardAndShortcuts => {
            return sections.chain(std::iter::once(Block::Shortcuts)).collect();
        }
        Page::App(_) if is_software_update(&page.page) => {
            return std::iter::once(Block::SoftwareUpdate).collect();
        }
        Page::App(_) if is_assistant_model(&page.page) => {
            return std::iter::once(Block::AssistantModel).collect();
        }
        Page::Intelligence => {}
        _ => return sections.collect(),
    }
    // Two slots per rank: the section at the even one, the live blocks at the odd one after
    // Models (the sort is stable, so they keep this order).
    let mut slotted: Vec<(usize, Block)> = page
        .sections
        .iter()
        .map(|section| (rank(&section.title) * 2, Block::Section(section.clone())))
        .collect();
    let live = rank(PICKER_AFTER) * 2 + 1;
    // Right after Companion: the first section is about the assistant, and so is this.
    slotted.push((rank("Companion") * 2 + 1, Block::Assistants));
    slotted.push((live, Block::LocalRuntimes));
    slotted.push((live, Block::ModelPicker));
    slotted.sort_by_key(|(slot, _)| *slot);
    slotted.into_iter().map(|(_, block)| block).collect()
}

#[cfg(test)]
mod tests {
    use super::{Block, blocks};
    use crate::catalogue::{PageModel, SectionModel};
    use crate::page::PageSlug;
    use ds_settings::schema::Page;

    fn page(kind: Page, titles: &[&str]) -> PageModel {
        PageModel {
            slug: PageSlug::of(&kind),
            page: kind,
            sections: titles
                .iter()
                .map(|title| SectionModel {
                    title: (*title).to_owned(),
                    keys: Vec::new(),
                })
                .collect(),
            advanced: Vec::new(),
        }
    }

    fn names(blocks: &[Block]) -> Vec<&str> {
        blocks
            .iter()
            .map(|block| match block {
                Block::Section(section) => section.title.as_str(),
                Block::LocalRuntimes => "<runtimes>",
                Block::ModelPicker => "<picker>",
                Block::AccountsPane => "<accounts>",
                Block::Assistants => "<assistants>",
                Block::SpacesPane => "<spaces>",
                Block::Shortcuts => "<shortcuts>",
                Block::SoftwareUpdate => "<updates>",
                Block::AssistantModel => "<assistant>",
            })
            .collect()
    }

    /// `(case, page, sections the schemas mention in order, the blocks drawn)`.
    const CASES: &[(&str, Page, &[&str], &[&str])] = &[
        (
            "intelligence follows design/22 whatever order the schemas were read in",
            Page::Intelligence,
            &["Assistant", "Models", "Companion", "Memory", "Computer use"],
            &[
                "Companion",
                "<assistants>",
                "Models",
                "<runtimes>",
                "<picker>",
                "Memory",
                "Computer use",
                "Assistant",
            ],
        ),
        (
            "the picker takes the place of Models when no schema has it",
            Page::Intelligence,
            &["Assistant", "Companion"],
            &[
                "Companion",
                "<assistants>",
                "<runtimes>",
                "<picker>",
                "Assistant",
            ],
        ),
        (
            "a section nobody listed follows the listed ones in schema order",
            Page::Intelligence,
            &["Zed", "Assistant", "Alpha", "Models"],
            &[
                "<assistants>",
                "Models",
                "<runtimes>",
                "<picker>",
                "Assistant",
                "Zed",
                "Alpha",
            ],
        ),
        (
            "the accounts pane comes before any section of the page",
            Page::Accounts,
            &["Sign-in clients"],
            &["<accounts>", "Sign-in clients"],
        ),
        (
            "the spaces pane comes before any section of the page",
            Page::Spaces,
            &["Look"],
            &["<spaces>", "Look"],
        ),
        (
            "the shortcuts follow the sections of the keyboard page",
            Page::KeyboardAndShortcuts,
            &["Typing"],
            &["Typing", "<shortcuts>"],
        ),
        (
            "another page keeps the schema order and has no picker",
            Page::Dock,
            &["Size", "Behaviour"],
            &["Size", "Behaviour"],
        ),
    ];

    #[test]
    fn a_page_draws_its_blocks_in_the_documented_order() {
        for (case, kind, titles, want) in CASES {
            let drawn = blocks(&page(kind.clone(), titles));
            assert_eq!(names(&drawn), *want, "{case}");
        }
    }

    #[test]
    fn the_assistants_model_page_draws_only_its_row_of_the_picker() {
        let drawn = blocks(&page(crate::assistant_model::assistant_model_page(), &[]));
        assert_eq!(names(&drawn), ["<assistant>"]);
    }
}
