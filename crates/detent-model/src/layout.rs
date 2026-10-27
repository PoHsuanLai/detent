//! What a page draws, top to bottom, before its Advanced disclosure (design/22 section 5).

use crate::catalogue::{PageModel, SectionModel};
use crate::control::Control;
use crate::entry::KeyEntry;
use crate::memory_consent::MEMORY_SECTION;
use crate::page_kind::PageKind;

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
    /// The programs and the powers the compositor lists for them, each with Allowed, Denied or
    /// Ask: the Privacy page's rows that are in no file.
    PrivacyPane,
    /// The apps the screen-sharing portal lets share the screen without asking again, each with
    /// Remove: the Privacy page's rows that are in no file.
    ScreenSharing,
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
    /// The Wi-Fi switch, the network this computer is on and the ones in range. Not in any file.
    Wifi,
    /// The Bluetooth switch and the devices. Not in any file.
    Bluetooth,
    /// Each display's resolution, refresh rate, rotation and place. Not in any file.
    Displays,
    /// The installed Extensions, grouped by what they provide, each with a switch. Not in any
    /// schema's keys.
    Extensions,
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
    let drawn = page.kind.blocks(page);
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

impl PageKind {
    /// What the page draws before its Advanced disclosure and the Hot Corners button.
    fn blocks(self, page: &PageModel) -> Vec<Block> {
        let sections = basic_sections(page).into_iter().map(Block::Section);
        let live = |block: Block| std::iter::once(block);
        match self {
            PageKind::Accounts => live(Block::AccountsPane).chain(sections).collect(),
            PageKind::Privacy => [Block::PrivacyPane, Block::ScreenSharing]
                .into_iter()
                .chain(sections)
                .collect(),
            PageKind::Spaces => live(Block::SpacesPane).chain(sections).collect(),
            PageKind::KeyboardAndShortcuts => sections.chain(live(Block::Shortcuts)).collect(),
            PageKind::SoftwareUpdate => live(Block::SoftwareUpdate).collect(),
            PageKind::AssistantModel => live(Block::AssistantModel).collect(),
            PageKind::Wifi => live(Block::Wifi).collect(),
            PageKind::Bluetooth => live(Block::Bluetooth).collect(),
            PageKind::Displays => live(Block::Displays).collect(),
            PageKind::Extensions => live(Block::Extensions).collect(),
            PageKind::Intelligence => intelligence_blocks(page),
            PageKind::Appearance => sections.flat_map(with_style_notes).collect(),
            PageKind::Dock
            | PageKind::MouseAndGestures
            | PageKind::Notifications
            | PageKind::Power
            | PageKind::Apps
            | PageKind::Viewer
            | PageKind::Wallpaper
            | PageKind::Program => sections.collect(),
        }
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
    slotted.push((rank("Assistant") * 2 + 1, Block::Assistants));
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
                Block::PrivacyPane => "<privacy>",
                Block::ScreenSharing => "<screen sharing>",
                Block::SpacesPane => "<spaces>",
                Block::Shortcuts => "<shortcuts>",
                Block::SoftwareUpdate => "<updates>",
                Block::AssistantModel => "<assistant>",
                Block::Wifi => "<wifi>",
                Block::Bluetooth => "<bluetooth>",
                Block::Displays => "<displays>",
                Block::Extensions => "<extensions>",
                Block::HotCorners(_) => "<hot corners>",
                Block::StyleNotes => "<style notes>",
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
                "<assistants>",
                "Models",
                "<runtimes>",
                "<picker>",
                "Memory",
                "Computer use",
            ],
        ),
        (
            "the picker takes the place of Models when no schema has it",
            Page::Intelligence,
            &["Assistant"],
            &["Assistant", "<assistants>", "<runtimes>", "<picker>"],
        ),
        (
            "a section nobody listed follows the listed ones in schema order",
            Page::Intelligence,
            &["Zed", "Assistant", "Alpha", "Models"],
            &[
                "Assistant",
                "<assistants>",
                "Models",
                "<runtimes>",
                "<picker>",
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
            "the privacy pane and the screen-sharing apps come before any section of the page",
            Page::Privacy,
            &["Screen"],
            &["<privacy>", "<screen sharing>", "Screen"],
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
            "the style notes follow the appearance section that holds the styles switch",
            Page::Appearance,
            &["Icons", "Appearance", "Time"],
            &["Icons", "Appearance", "<style notes>", "Time"],
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
    fn the_computers_own_pages_draw_only_their_live_block() {
        use crate::{bluetooth_page, displays_page, wifi_page};
        let cases = [
            (wifi_page(), "<wifi>"),
            (bluetooth_page(), "<bluetooth>"),
            (displays_page(), "<displays>"),
        ];
        for (kind, want) in cases {
            let drawn = blocks(&page(kind.clone(), &[]));
            assert_eq!(names(&drawn), [want], "{kind:?}");
        }
    }

    #[test]
    fn the_extensions_page_is_its_list_and_nothing_else() {
        let drawn = blocks(&page(crate::extensions_page(), &["Turned off"]));
        assert_eq!(names(&drawn), ["<extensions>"]);
    }

    #[test]
    fn the_assistants_model_page_draws_only_its_row_of_the_picker() {
        let drawn = blocks(&page(crate::assistant_model::assistant_model_page(), &[]));
        assert_eq!(names(&drawn), ["<assistant>"]);
    }
}
