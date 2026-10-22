//! A page whose schema marks the four corner keys draws one Hot Corners button and no rows for
//! them or for the commands they held before they held lists.

use detent_model::{
    Block, Catalogue, ConfigFile, Control, HOT_CORNERS_CONTROL, ProgramSchema, advanced_sections,
    basic_sections, blocks,
};
use ds_settings::schema::{
    AppId, ControlName, Exposure, FilePath, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section,
};

fn key(path: &str, kind: KeyKind, marked: bool, exposure: Exposure) -> KeySpec {
    KeySpec {
        control: marked.then(|| ControlName(HOT_CORNERS_CONTROL.to_owned())),
        path: KeyPath(path.to_owned()),
        kind,
        default: toml::Value::Array(Vec::new()),
        label: Label(path.to_owned()),
        help: Help(String::new()),
        page: Page::Dock,
        section: Section("Hot Corners".to_owned()),
        exposure,
        labels: Default::default(),
        agent: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
    }
}

fn catalogue(keys: Vec<KeySpec>) -> Catalogue {
    let file = ConfigFile::parse(&FilePath("sill/settings.toml".to_owned())).expect("a plain path");
    Catalogue::build(&[ProgramSchema {
        app: AppId("sill".to_owned()),
        file,
        keys,
        foreign: Default::default(),
    }])
}

fn paths(sections: &[detent_model::SectionModel]) -> Vec<String> {
    sections
        .iter()
        .flat_map(|section| &section.keys)
        .map(|entry| entry.spec.path.0.clone())
        .collect()
}

#[test]
fn marked_corner_keys_become_one_button_and_lose_their_rows() {
    let list = || KeyKind::List(Box::new(KeyKind::Text));
    let catalogue = catalogue(vec![
        key("hot_corners.top_left", list(), true, Exposure::Advanced),
        key("hot_corners.top_right", list(), true, Exposure::Advanced),
        key(
            "hot_corners.dwell_ms",
            KeyKind::Text,
            false,
            Exposure::Advanced,
        ),
    ]);
    let page = catalogue
        .pages
        .iter()
        .find(|page| page.page == Page::Dock)
        .expect("the dock page");
    let drawn = blocks(page);
    let Some(Block::HotCorners(corners)) = drawn.last() else {
        panic!("the button is the page's last block: {drawn:?}");
    };
    assert_eq!(
        corners
            .iter()
            .map(|entry| entry.spec.path.0.as_str())
            .collect::<Vec<_>>(),
        ["hot_corners.top_left", "hot_corners.top_right"]
    );
    assert!(
        corners
            .iter()
            .all(|entry| entry.control == Control::HotCorners)
    );
    assert_eq!(
        paths(&advanced_sections(page)),
        ["hot_corners.dwell_ms"],
        "the timing stays a row; the corners are in the sheet"
    );
    assert!(basic_sections(page).is_empty());
}

#[test]
fn a_page_without_marked_corner_keys_keeps_every_row_and_has_no_button() {
    let catalogue = catalogue(vec![
        key(
            "hot_corners.top_left",
            KeyKind::Text,
            false,
            Exposure::Advanced,
        ),
        key(
            "hot_corners.top_left_command",
            KeyKind::Text,
            false,
            Exposure::Advanced,
        ),
    ]);
    let page = catalogue
        .pages
        .iter()
        .find(|page| page.page == Page::Dock)
        .expect("the dock page");
    assert!(
        blocks(page)
            .iter()
            .all(|block| !matches!(block, Block::HotCorners(_)))
    );
    assert_eq!(
        paths(&advanced_sections(page)),
        ["hot_corners.top_left", "hot_corners.top_left_command"]
    );
}
