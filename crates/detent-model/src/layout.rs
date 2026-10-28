//! What a page draws, top to bottom, before its Advanced disclosure (design/22 section 5).

use crate::catalogue::{PageModel, SectionModel};
use crate::control::Control;
use crate::entry::KeyEntry;
use crate::memory_consent::MEMORY_SECTION;
use crate::page_traits::{Body, LivePane};

/// One thing a page draws.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// A titled card of file rows.
    Section(SectionModel),
    /// A pane that reads a service, for rows that are in no file.
    Pane(LivePane),
    /// The button that opens the Hot Corners sheet, over the corner keys the page's schema marks
    /// for it.
    HotCorners(Vec<KeyEntry>),
    /// The line under the Appearance page's "Use my styles" switch that counts the notes on the
    /// person's styles and opens the sheet that lists them. Not in any file; drawn only when there
    /// are notes.
    StyleNotes,
}

/// The keys of `page` that the Hot Corners sheet draws, in the order the schemas give them.
fn corner_keys(page: &PageModel) -> Vec<KeyEntry> {
    page.sections
        .iter()
        .chain(&page.advanced)
        .flat_map(|section| &section.keys)
        .filter(|entry| entry.control == Control::HotCorners)
        .cloned()
        .collect()
}

/// `sections` without the corner keys, which the Hot Corners sheet draws instead of rows. A
/// section those keys leave empty is left out.
fn without_corner_rows(sections: &[SectionModel]) -> Vec<SectionModel> {
    let hidden = |entry: &KeyEntry| entry.control == Control::HotCorners;
    sections
        .iter()
        .filter_map(|section| {
            let keys: Vec<KeyEntry> = section
                .keys
                .iter()
                .filter(|entry| !hidden(entry))
                .cloned()
                .collect();
            let emptied = keys.is_empty() && !section.keys.is_empty();
            (!emptied).then(|| SectionModel {
                title: section.title.clone(),
                footnote: section.footnote.clone(),
                keys,
            })
        })
        .collect()
}

/// The sections of `page` that are drawn as rows.
pub fn basic_sections(page: &PageModel) -> Vec<SectionModel> {
    without_corner_rows(&page.sections)
}

/// The advanced sections of `page` that are drawn as rows.
pub fn advanced_sections(page: &PageModel) -> Vec<SectionModel> {
    without_corner_rows(&page.advanced)
}

/// The Intelligence page's sections, in the order design/22 section 5 gives them. A section a
/// program names that is not listed follows these, in the order the schemas first mention it.
const INTELLIGENCE_SECTIONS: &[&str] = &["Assistant", "Models", MEMORY_SECTION, "Computer use"];

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
    let corners = corner_keys(page);
    let drawn = drawn(page);
    let sheet = (!corners.is_empty()).then_some(Block::HotCorners(corners));
    drawn.into_iter().chain(sheet).collect()
}

/// The Appearance page's section that holds the "Use my styles" switch, which the notes follow.
const STYLES_SECTION: &str = "Appearance";

/// `block`, followed by the notes line when it is the section that holds the styles switch.
fn with_style_notes(block: Block) -> Vec<Block> {
    match &block {
        Block::Section(section) if section.title == STYLES_SECTION => {
            vec![block, Block::StyleNotes]
        }
        _ => vec![block],
    }
}

/// What `page` draws before its Advanced disclosure and the Hot Corners button.
fn drawn(page: &PageModel) -> Vec<Block> {
    let sections = basic_sections(page).into_iter().map(Block::Section);
    let panes = |panes: &[LivePane]| panes.iter().copied().map(Block::Pane).collect::<Vec<_>>();
    match page.kind.traits().body {
        Body::Sections => sections.collect(),
        Body::Before(live) => panes(live).into_iter().chain(sections).collect(),
        Body::After(live) => sections.chain(panes(live)).collect(),
        Body::Only(pane) => vec![Block::Pane(pane)],
        Body::Intelligence => intelligence_blocks(page),
        Body::Styles => sections.flat_map(with_style_notes).collect(),
    }
}

/// The Intelligence page: its sections in design/22's order, with the live blocks slotted in.
fn intelligence_blocks(page: &PageModel) -> Vec<Block> {
    // Two slots per rank: the section at the even one, the live blocks at the odd one after
    // Models (the sort is stable, so they keep this order).
    let mut slotted: Vec<(usize, Block)> = basic_sections(page)
        .into_iter()
        .map(|section| (rank(&section.title) * 2, Block::Section(section)))
        .collect();
    let live = rank(PICKER_AFTER) * 2 + 1;
    // Right after Assistant: the first section is about the assistant, and so is this.
    slotted.push((rank("Assistant") * 2 + 1, Block::Pane(LivePane::Assistants)));
    slotted.push((live, Block::Pane(LivePane::LocalRuntimes)));
    slotted.push((live, Block::Pane(LivePane::ModelPicker)));
    slotted.sort_by_key(|(slot, _)| *slot);
    slotted.into_iter().map(|(_, block)| block).collect()
}

#[cfg(test)]
mod tests {
    use super::{Block, blocks};
    use crate::catalogue::{PageModel, SectionModel};
    use crate::page::PageSlug;
    use crate::page_kind::PageKind;
    use ds_settings::schema::Page;

    fn page(kind: Page, titles: &[&str]) -> PageModel {
        PageModel {
            kind: PageKind::of(&kind),
            slug: PageSlug::of(&kind),
            title: kind.label().to_owned(),
            page: kind,
            sections: titles
                .iter()
                .map(|title| SectionModel {
                    title: (*title).to_owned(),
                    footnote: None,
                    keys: Vec::new(),
                })
                .collect(),
            advanced: Vec::new(),
        }
    }

    fn names(blocks: &[Block]) -> Vec<String> {
        blocks
            .iter()
            .map(|block| match block {
                Block::Section(section) => section.title.clone(),
                Block::Pane(pane) => format!("<{pane:?}>"),
                Block::HotCorners(_) => "<hot corners>".to_owned(),
                Block::StyleNotes => "<style notes>".to_owned(),
            })
            .collect()
    }

    /// `(case, page, sections the schemas mention in order, the blocks drawn)`.
    const CASES: &[(&str, Page, &[&str], &[&str])] = &[
        (
            "intelligence follows design/22 whatever order the schemas were read in",
            Page::Intelligence,
            &["Models", "Assistant", "Memory", "Computer use"],
            &[
                "Assistant",
                "<Assistants>",
                "Models",
                "<LocalRuntimes>",
                "<ModelPicker>",
                "Memory",
                "Computer use",
            ],
        ),
        (
            "the picker takes the place of Models when no schema has it",
            Page::Intelligence,
            &["Assistant"],
            &[
                "Assistant",
                "<Assistants>",
                "<LocalRuntimes>",
                "<ModelPicker>",
            ],
        ),
        (
            "a section nobody listed follows the listed ones in schema order",
            Page::Intelligence,
            &["Zed", "Assistant", "Alpha", "Models"],
            &[
                "Assistant",
                "<Assistants>",
                "Models",
                "<LocalRuntimes>",
                "<ModelPicker>",
                "Zed",
                "Alpha",
            ],
        ),
        (
            "the accounts pane comes before any section of the page",
            Page::Accounts,
            &["Sign-in clients"],
            &["<Accounts>", "Sign-in clients"],
        ),
        (
            "the privacy pane and the screen-sharing apps come before any section of the page",
            Page::Privacy,
            &["Screen"],
            &["<Privacy>", "<ScreenSharing>", "Screen"],
        ),
        (
            "the spaces pane comes before any section of the page",
            Page::Spaces,
            &["Look"],
            &["<Spaces>", "Look"],
        ),
        (
            "the shortcuts follow the sections of the keyboard page",
            Page::KeyboardAndShortcuts,
            &["Typing"],
            &["Typing", "<Shortcuts>"],
        ),
        (
            "the style notes follow the appearance section that holds the styles switch",
            Page::Appearance,
            &["Icons", "Appearance", "Time"],
            &["Icons", "Appearance", "<style notes>", "Time"],
        ),
        (
            "the dock preview comes before the sections of the dock page",
            Page::Dock,
            &["Size", "Behaviour"],
            &["<DockPreview>", "Size", "Behaviour"],
        ),
        (
            "another page keeps the schema order and has no pane",
            Page::Power,
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
    fn the_computers_own_pages_draw_only_their_live_block() {
        use crate::{bluetooth_page, displays_page, wifi_page};
        let cases = [
            (wifi_page(), "<Wifi>"),
            (bluetooth_page(), "<Bluetooth>"),
            (displays_page(), "<Displays>"),
        ];
        for (kind, want) in cases {
            let drawn = blocks(&page(kind.clone(), &[]));
            assert_eq!(names(&drawn), [want], "{kind:?}");
        }
    }

    #[test]
    fn the_extensions_page_is_its_list_and_nothing_else() {
        let drawn = blocks(&page(crate::extensions_page(), &["Turned off"]));
        assert_eq!(names(&drawn), ["<Extensions>"]);
    }

    #[test]
    fn the_assistants_model_page_draws_only_its_row_of_the_picker() {
        let drawn = blocks(&page(crate::assistant_model::assistant_model_page(), &[]));
        assert_eq!(names(&drawn), ["<AssistantModel>"]);
    }
}
