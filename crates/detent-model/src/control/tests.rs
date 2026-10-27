use super::{Control, FileOnly, TogglePair};
use ds_settings::schema::{
    ActionLabel, ActionWeight, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, LiveAction, Page,
    Section,
};
use toml::Value;

fn spec(kind: KeyKind, default: Value) -> KeySpec {
    KeySpec {
        control: None,
        path: KeyPath("x.y".to_owned()),
        kind,
        default,
        label: Label("Y".to_owned()),
        help: Help(String::new()),
        page: Page::Appearance,
        section: Section(String::new()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        agent: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
    }
}

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|w| (*w).to_owned()).collect()
}

fn pair(a: &str, b: &str) -> Option<TogglePair> {
    TogglePair::of(&[a.to_owned(), b.to_owned()])
}

#[test]
fn every_key_kind_has_its_one_control() {
    let cases: Vec<(&str, KeyKind, Value, Control)> = vec![
        (
            "fixed",
            KeyKind::Fixed {
                variant: "sheet".to_owned(),
            },
            Value::String("sheet".to_owned()),
            Control::Readout("sheet".to_owned()),
        ),
        (
            "toggle",
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()],
            },
            Value::String("on".to_owned()),
            Control::Toggle(pair("on", "off").unwrap()),
        ),
        (
            "segmented",
            KeyKind::Segmented {
                variants: words(&["a", "b", "c"]),
            },
            Value::String("a".to_owned()),
            Control::Segmented(words(&["a", "b", "c"])),
        ),
        (
            "menu",
            KeyKind::Menu {
                variants: words(&["a", "b", "c", "d", "e", "f"]),
            },
            Value::String("a".to_owned()),
            Control::Menu(words(&["a", "b", "c", "d", "e", "f"])),
        ),
        (
            "bounded",
            KeyKind::Bounded {
                min: 1,
                max: 9,
                unit: Some("px".to_owned()),
            },
            Value::Integer(3),
            Control::Slider(crate::range::Range {
                min: 1,
                max: 9,
                unit: Some("px".to_owned()),
                number: crate::range::Number::Whole,
            }),
        ),
        (
            "text",
            KeyKind::Text,
            Value::String(String::new()),
            Control::Text,
        ),
        (
            "colour",
            KeyKind::Colour,
            Value::String("#fff".to_owned()),
            Control::Colour,
        ),
        (
            "shortcut",
            KeyKind::Shortcut,
            Value::String("ctrl+a".to_owned()),
            Control::FileOnly(FileOnly::Shortcut),
        ),
        (
            "list",
            KeyKind::List(Box::new(KeyKind::Text)),
            Value::Array(Vec::new()),
            Control::FileOnly(FileOnly::List),
        ),
        (
            "live action",
            KeyKind::Live {
                action: LiveAction {
                    label: ActionLabel("Remove".to_owned()),
                    weight: ActionWeight::Destructive,
                },
            },
            Value::String(String::new()),
            Control::FileOnly(FileOnly::Action),
        ),
        (
            "provided",
            KeyKind::Provided {
                capability: "look".to_owned(),
            },
            Value::String("standard".to_owned()),
            Control::Provided(words(&["standard"])),
        ),
    ];
    for (name, kind, default, want) in cases {
        assert_eq!(Control::of(&spec(kind, default)), want, "{name}");
    }
}

#[test]
fn a_control_admits_only_its_own_values() {
    let toggle = Control::Toggle(pair("on", "off").unwrap());
    let text = |s: &str| Value::String(s.to_owned());
    assert_eq!(toggle.admit(&text("on")), Some(text("on")));
    assert_eq!(toggle.admit(&text("bounceyy")), None);
    assert_eq!(toggle.admit(&Value::Integer(1)), None);
    assert_eq!(Control::Text.admit(&text("hello")), Some(text("hello")));
    assert_eq!(Control::Text.admit(&Value::Integer(1)), None);
    // A provided key keeps the id of an Extension that is gone, whatever is on offer.
    let provided = Control::Provided(words(&["standard"]));
    assert_eq!(
        provided.admit(&text("org.example.Paper")),
        Some(text("org.example.Paper"))
    );
    assert_eq!(provided.admit(&Value::Integer(1)), None);
}

#[test]
fn a_text_key_that_asks_for_pictures_gets_the_grid_and_nothing_else_does() {
    use ds_settings::schema::ControlName;
    let asking = |kind: KeyKind, default: Value| {
        let mut asked = spec(kind, default);
        asked.control = Some(ControlName("picture".to_owned()));
        asked
    };
    let text = asking(KeyKind::Text, Value::String(String::new()));
    assert_eq!(Control::of(&text), Control::Picture);
    let number = asking(
        KeyKind::Bounded {
            min: 0,
            max: 9,
            unit: None,
        },
        Value::Integer(1),
    );
    assert!(matches!(Control::of(&number), Control::Slider(_)));
    let path = Value::String("/a/b.png".to_owned());
    assert_eq!(Control::Picture.admit(&path), Some(path.clone()));
    assert_eq!(Control::Picture.admit(&Value::Integer(1)), None);
}

#[test]
fn a_key_that_asks_for_the_corners_gets_them_whatever_its_kind_and_holds_words_or_a_list() {
    use ds_settings::schema::ControlName;
    for kind in [KeyKind::Text, KeyKind::List(Box::new(KeyKind::Text))] {
        let mut asked = spec(kind, Value::Array(Vec::new()));
        asked.control = Some(ControlName("corners".to_owned()));
        assert_eq!(Control::of(&asked), Control::HotCorners);
    }
    let held = |text: &str| -> Value {
        let table: toml::Table = text.parse().expect("test TOML");
        table["v"].clone()
    };
    for (value, kept) in [
        ("v = \"lock\"", true),
        ("v = []", true),
        ("v = [{ action = \"lock\" }]", true),
        ("v = \"teleport\"", false),
        ("v = 3", false),
    ] {
        let value = held(value);
        let want = kept.then(|| value.clone());
        assert_eq!(Control::HotCorners.admit(&value), want, "{value}");
    }
}

#[test]
fn a_two_word_key_is_drawn_as_the_rule_says() {
    let two = |a: &str, b: &str| {
        Control::of(&spec(
            KeyKind::Toggle {
                variants: [a.to_owned(), b.to_owned()],
            },
            Value::String(a.to_owned()),
        ))
    };
    assert_eq!(
        two("on", "off"),
        Control::Toggle(pair("on", "off").unwrap())
    );
    assert_eq!(
        two("ring", "bar"),
        Control::Segmented(words(&["ring", "bar"]))
    );
}
