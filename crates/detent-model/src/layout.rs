//! What a page draws, top to bottom, before its Advanced disclosure (design/22 section 5).

use crate::catalogue::{PageModel, SectionModel};
use ds_settings::schema::Page;

/// One thing a page draws.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// A titled card of file rows.
    Section(SectionModel),
    /// inferd's live model picker (design/22 section 9.4): the Intelligence page's rows that are
    /// not in any file.
    ModelPicker,
}

/// The Intelligence page's sections, in the order design/22 section 5 gives them. A section a
/// program names that is not listed follows these, in the order the schemas first mention it.
const INTELLIGENCE_SECTIONS: &[&str] =
    &["Companion", "Models", "Memory", "Computer use", "Privacy"];

/// The section the model picker follows.
const PICKER_AFTER: &str = "Models";

fn rank(title: &str) -> usize {
    INTELLIGENCE_SECTIONS
        .iter()
        .position(|known| *known == title)
        .unwrap_or(INTELLIGENCE_SECTIONS.len())
}

/// The blocks of `page`. Every page lists its sections as the schemas gave them, except
/// Intelligence: its sections follow design/22's order and the model picker sits right after
/// Models, or where Models would be when no schema has that section.
pub fn blocks(page: &PageModel) -> Vec<Block> {
    let sections = page.sections.iter().cloned().map(Block::Section);
    if page.page != Page::Intelligence {
        return sections.collect();
    }
    // Two slots per rank: the section at the even one, the picker at the odd one after Models.
    let mut slotted: Vec<(usize, Block)> = page
        .sections
        .iter()
        .map(|section| (rank(&section.title) * 2, Block::Section(section.clone())))
        .collect();
    slotted.push((rank(PICKER_AFTER) * 2 + 1, Block::ModelPicker));
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
                Block::ModelPicker => "<picker>",
            })
            .collect()
    }

    /// `(case, page, sections the schemas mention in order, the blocks drawn)`.
    const CASES: &[(&str, Page, &[&str], &[&str])] = &[
        (
            "intelligence follows design/22 whatever order the schemas were read in",
            Page::Intelligence,
            &["Privacy", "Models", "Companion", "Memory", "Computer use"],
            &[
                "Companion",
                "Models",
                "<picker>",
                "Memory",
                "Computer use",
                "Privacy",
            ],
        ),
        (
            "the picker takes the place of Models when no schema has it",
            Page::Intelligence,
            &["Privacy", "Companion"],
            &["Companion", "<picker>", "Privacy"],
        ),
        (
            "a section nobody listed follows the listed ones in schema order",
            Page::Intelligence,
            &["Zed", "Privacy", "Alpha", "Models"],
            &["Models", "<picker>", "Privacy", "Zed", "Alpha"],
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
}
