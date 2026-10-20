//! The page over a fake publication: the rows casement lists, as they are named and typed.

use super::*;
use crate::Absence;
use crate::origin::SchemaOrigin;
use crate::picker::Described;
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Note, Page, Section,
};
use std::collections::BTreeMap;

const GRIM: &str = "/usr/bin/grim";
const LOUPE: &str = "org.gnome.Loupe";

fn spec(path: &str, label: &str, kind: KeyKind) -> KeySpec {
    KeySpec {
        control: None,
        path: KeyPath(path.to_owned()),
        kind,
        default: toml::Value::Boolean(false),
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Privacy,
        section: Section("Screen".to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
        agent: AgentSetting::HandsOff,
    }
}

/// What the compositor lists, row by row.
#[derive(Default)]
struct Publication {
    keys: Vec<KeySpec>,
    values: BTreeMap<KeyPath, String>,
}

impl Publication {
    /// The person's answer for `subject`: a switch and its "ask again" action.
    fn answered(mut self, power: &str, id: &str, subject: &str, word: &str) -> Publication {
        let base = format!("privacy.{power}.{id}");
        self.keys.push(spec(
            &base,
            subject,
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()],
            },
        ));
        self.keys.push(spec(
            &format!("{base}.forget"),
            "Ask Again",
            KeyKind::Live {
                action: LiveAction {
                    label: ActionLabel("Ask Again".to_owned()),
                    weight: ActionWeight::Plain,
                },
            },
        ));
        self.values.insert(KeyPath(base), word.to_owned());
        self
    }

    /// A power nobody has answered for `subject`: a switch at `ask`, with no action beside it.
    fn unanswered(mut self, power: &str, id: &str, subject: &str) -> Publication {
        let base = format!("privacy.{power}.{id}");
        self.keys.push(spec(
            &base,
            subject,
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()],
            },
        ));
        self.values.insert(KeyPath(base), "ask".to_owned());
        self
    }

    /// An administrator's answer: a read-out that says who set it.
    fn fixed(mut self, power: &str, id: &str, subject: &str, word: &str) -> Publication {
        let base = format!("privacy.{power}.{id}");
        let mut row = spec(
            &base,
            subject,
            KeyKind::Fixed {
                variant: word.to_owned(),
            },
        );
        row.note = Some(Note(SET_BY_ADMINISTRATOR.to_owned()));
        self.keys.push(row);
        self.values.insert(KeyPath(base), word.to_owned());
        self
    }

    /// A section nobody has asked in.
    fn empty(mut self, power: &str) -> Publication {
        let base = format!("privacy.{power}.none");
        self.keys.push(spec(
            &base,
            "No program has asked yet.",
            KeyKind::Fixed {
                variant: String::new(),
            },
        ));
        self.values.insert(KeyPath(base), String::new());
        self
    }

    fn described(&self) -> Described {
        Described {
            keys: self.keys.clone(),
            values: self.values.clone(),
            ..Default::default()
        }
    }

    fn pane(&self) -> PrivacyPane {
        PrivacyPane::of(&self.described())
    }
}

fn subject(text: &str) -> ProgramSubject {
    ProgramSubject(text.to_owned())
}

fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

fn standing(pane: &PrivacyPane, who: &str, power: PrivacyPower) -> PowerStanding {
    pane.programs
        .iter()
        .find(|program| program.subject.0 == who)
        .and_then(|program| program.cell(power))
        .map(|cell| cell.standing)
        .unwrap_or_else(|| panic!("{who} has a {power:?} cell"))
}

fn ready(publication: &Publication) -> PrivacyState {
    privacy_step(
        PrivacyState::Looking,
        PrivacyInput::Found(Ok(publication.described())),
    )
    .0
}

fn pick(
    state: PrivacyState,
    who: &str,
    power: PrivacyPower,
    to: PowerStanding,
) -> (PrivacyState, Vec<PrivacyEffect>) {
    privacy_step(
        state,
        PrivacyInput::Pick {
            subject: subject(who),
            power,
            standing: to,
        },
    )
}

fn pending(state: &PrivacyState) -> Option<(ProgramSubject, PrivacyPower)> {
    match state {
        PrivacyState::Ready(ready) => ready.pending.clone(),
        other => panic!("not ready: {other:?}"),
    }
}

#[test]
fn the_powers_are_named_in_plain_words_in_the_pages_order() {
    let labels: Vec<&str> = PrivacyPower::ALL
        .iter()
        .map(|power| power.label())
        .collect();
    assert_eq!(
        labels,
        [
            "Screen recording",
            "Keyboard input",
            "Typing help",
            "Clipboard",
            "Screen lock",
            "Displays",
            "Windows"
        ]
    );
    for power in PrivacyPower::ALL {
        assert_eq!(PrivacyPower::of_word(power.word()), Some(power));
    }
    assert_eq!(PrivacyPower::of_word("teleport"), None);
}

#[test]
fn rows_become_programs_with_every_power_and_the_unanswered_ones_ask() {
    let shown = Publication::default()
        .answered("screen", "aaa", GRIM, "on")
        .answered("clipboard", "bbb", GRIM, "off")
        .answered("windows", "ccc", LOUPE, "on")
        .empty("keys")
        .empty("lock")
        .pane();
    let who: Vec<&str> = shown
        .programs
        .iter()
        .map(|p| p.subject.0.as_str())
        .collect();
    assert_eq!(who, [GRIM, LOUPE], "in the order the rows first name them");
    assert_eq!(shown.programs[0].cells.len(), 7);
    // `(program, power, where it stands)`.
    const CASES: &[(&str, PrivacyPower, PowerStanding)] = &[
        (GRIM, PrivacyPower::ScreenRecording, PowerStanding::Allowed),
        (GRIM, PrivacyPower::Clipboard, PowerStanding::Denied),
        (GRIM, PrivacyPower::KeyboardInput, PowerStanding::Ask),
        (LOUPE, PrivacyPower::Windows, PowerStanding::Allowed),
        (LOUPE, PrivacyPower::ScreenRecording, PowerStanding::Ask),
    ];
    for (who, power, want) in CASES {
        assert_eq!(standing(&shown, who, *power), *want, "{who} {power:?}");
    }
    assert_eq!(
        Publication::default().empty("screen").pane(),
        PrivacyPane::default(),
        "a section nobody asked in lists no program"
    );
}

#[test]
fn an_administrators_answer_is_locked_and_wins_over_the_persons() {
    let shown = Publication::default()
        .answered("screen", "aaa", GRIM, "on")
        .fixed("screen", "bbb", GRIM, "off")
        .fixed("keys", "ccc", GRIM, "on")
        .pane();
    let grim = &shown.programs[0];
    let screen = grim.cell(PrivacyPower::ScreenRecording).expect("a cell");
    assert!(screen.locked() && !screen.editable());
    assert_eq!(
        screen.standing,
        PowerStanding::Denied,
        "the floor, not the person"
    );
    assert!(
        grim.cell(PrivacyPower::KeyboardInput)
            .expect("a cell")
            .locked()
    );
    let clipboard = grim.cell(PrivacyPower::Clipboard).expect("a cell");
    assert!(
        !clipboard.locked() && !clipboard.editable(),
        "nobody has answered"
    );
}

#[test]
fn a_power_nobody_answered_is_a_switch_at_ask_with_nothing_to_forget() {
    // Grim has answered one power; Loupe none, so it is listed only because of its Ask rows.
    let shown = Publication::default()
        .answered("screen", "aaa", GRIM, "on")
        .unanswered("keys", "bbb", GRIM)
        .unanswered("screen", "ccc", LOUPE)
        .unanswered("keys", "ddd", LOUPE)
        .pane();
    let who: Vec<&str> = shown
        .programs
        .iter()
        .map(|p| p.subject.0.as_str())
        .collect();
    assert_eq!(who, [GRIM, LOUPE]);
    // `(program, power, the switch)`.
    const CASES: &[(&str, PrivacyPower, &str)] = &[
        (GRIM, PrivacyPower::KeyboardInput, "privacy.keys.bbb"),
        (LOUPE, PrivacyPower::ScreenRecording, "privacy.screen.ccc"),
        (LOUPE, PrivacyPower::KeyboardInput, "privacy.keys.ddd"),
    ];
    for (who, power, switch) in CASES {
        let cell = shown
            .programs
            .iter()
            .find(|program| program.subject.0 == *who)
            .and_then(|program| program.cell(*power))
            .expect("a cell");
        assert_eq!(cell.standing, PowerStanding::Ask, "{who} {power:?}");
        assert_eq!(
            cell.kind,
            CellKind::Remembered {
                switch: path(switch),
                forget: None,
            },
            "{who} {power:?}"
        );
        assert!(cell.editable() && !cell.locked(), "{who} {power:?}");
    }
    // The answered one beside it keeps its own action.
    assert_eq!(
        standing(&shown, GRIM, PrivacyPower::ScreenRecording),
        PowerStanding::Allowed
    );
}

#[test]
fn a_row_that_is_not_an_answer_is_left_out() {
    let mut shown = Publication::default().answered("screen", "aaa", GRIM, "on");
    // An administrator's read-out cannot say `ask`: the floor is Allowed or Denied.
    shown.keys.push(spec(
        "privacy.keys.ggg",
        GRIM,
        KeyKind::Fixed {
            variant: "ask".to_owned(),
        },
    ));
    shown
        .values
        .insert(path("privacy.keys.ggg"), "ask".to_owned());
    // A word that is neither on nor off, a power this page does not know, an id with a dot.
    shown
        .keys
        .push(spec("privacy.screen.ddd", GRIM, KeyKind::Text));
    shown
        .keys
        .push(spec("privacy.teleport.eee", "/bin/x", KeyKind::Text));
    shown
        .keys
        .push(spec("privacy.screen.f.g", "/bin/y", KeyKind::Text));
    shown
        .values
        .insert(path("privacy.screen.ddd"), "maybe".to_owned());
    let pane = shown.pane();
    assert_eq!(pane.programs.len(), 1);
    assert_eq!(
        standing(&pane, GRIM, PrivacyPower::ScreenRecording),
        PowerStanding::Allowed
    );
    let keys = pane.programs[0]
        .cell(PrivacyPower::KeyboardInput)
        .expect("a cell");
    assert_eq!(
        keys.kind,
        CellKind::Asking,
        "the read-out at ask is not an answer"
    );
}

#[test]
fn a_pick_sends_the_one_change_the_compositor_takes() {
    let shown = Publication::default()
        .answered("screen", "aaa", GRIM, "on")
        .answered("clipboard", "bbb", GRIM, "off")
        .unanswered("keys", "ccc", GRIM);
    let switch = path("privacy.screen.aaa");
    let forget = path("privacy.screen.aaa.forget");
    // `(case, the cell, what is picked, the effect)`.
    let cases: &[(&str, PrivacyPower, PowerStanding, Option<PrivacyEffect>)] = &[
        (
            "deny what was allowed",
            PrivacyPower::ScreenRecording,
            PowerStanding::Denied,
            Some(PrivacyEffect::Set {
                path: switch.clone(),
                to: PrivacySwitch::Off,
            }),
        ),
        (
            "ask again",
            PrivacyPower::ScreenRecording,
            PowerStanding::Ask,
            Some(PrivacyEffect::Forget(forget)),
        ),
        (
            "allow what was denied",
            PrivacyPower::Clipboard,
            PowerStanding::Allowed,
            Some(PrivacyEffect::Set {
                path: path("privacy.clipboard.bbb"),
                to: PrivacySwitch::On,
            }),
        ),
        (
            "pick what already stands",
            PrivacyPower::ScreenRecording,
            PowerStanding::Allowed,
            None,
        ),
        (
            "a power the compositor lists no row for",
            PrivacyPower::Windows,
            PowerStanding::Allowed,
            None,
        ),
        (
            "allow what nobody answered: the first answer",
            PrivacyPower::KeyboardInput,
            PowerStanding::Allowed,
            Some(PrivacyEffect::Set {
                path: path("privacy.keys.ccc"),
                to: PrivacySwitch::On,
            }),
        ),
        (
            "deny what nobody answered: the first answer",
            PrivacyPower::KeyboardInput,
            PowerStanding::Denied,
            Some(PrivacyEffect::Set {
                path: path("privacy.keys.ccc"),
                to: PrivacySwitch::Off,
            }),
        ),
        (
            "ask what already asks: nothing to forget",
            PrivacyPower::KeyboardInput,
            PowerStanding::Ask,
            None,
        ),
    ];
    for (case, power, to, want) in cases {
        let (after, effects) = pick(ready(&shown), GRIM, *power, *to);
        assert_eq!(effects, want.iter().cloned().collect::<Vec<_>>(), "{case}");
        assert_eq!(
            pending(&after).is_some(),
            want.is_some(),
            "{case}: a change in flight only when one was sent"
        );
    }
    assert_eq!(PrivacySwitch::On.word(), "on");
    assert_eq!(PrivacySwitch::Off.word(), "off");
}

#[test]
fn a_locked_cell_sends_nothing() {
    let shown = Publication::default().fixed("screen", "bbb", GRIM, "off");
    for to in PowerStanding::CHOICES {
        let (after, effects) = pick(ready(&shown), GRIM, PrivacyPower::ScreenRecording, to);
        assert!(effects.is_empty(), "{to:?}: {effects:?}");
        assert_eq!(pending(&after), None);
    }
}

#[test]
fn one_change_at_a_time_and_the_answer_reads_the_rows_again() {
    let shown = Publication::default().answered("screen", "aaa", GRIM, "on");
    let (busy, first) = pick(
        ready(&shown),
        GRIM,
        PrivacyPower::ScreenRecording,
        PowerStanding::Denied,
    );
    assert_eq!(first.len(), 1);
    let (busy, second) = pick(
        busy,
        GRIM,
        PrivacyPower::ScreenRecording,
        PowerStanding::Ask,
    );
    assert!(second.is_empty(), "while one is in flight: {second:?}");
    let (done, effects) = privacy_step(busy.clone(), PrivacyInput::Done);
    assert_eq!(effects, [PrivacyEffect::Reload]);
    assert_eq!(pending(&done), None);
    let (failed, effects) = privacy_step(busy, PrivacyInput::Failed("no".to_owned()));
    assert_eq!(
        effects,
        [PrivacyEffect::Tell("no".to_owned()), PrivacyEffect::Reload]
    );
    assert_eq!(pending(&failed), None);
}

#[test]
fn the_compositor_not_being_there_is_a_state_and_its_return_a_list() {
    let (gone, _) = privacy_step(
        PrivacyState::Looking,
        PrivacyInput::Found(Err(Absence::NotRunning)),
    );
    assert_eq!(gone, PrivacyState::Unavailable(Absence::NotRunning));
    let shown = Publication::default().answered("screen", "aaa", GRIM, "on");
    let (back, _) = privacy_step(gone, PrivacyInput::Found(Ok(shown.described())));
    assert_eq!(back, ready(&shown));
}

fn app(id: &str, name: &str, exec: Option<&str>, flatpak: Option<&str>) -> DesktopApp {
    DesktopApp {
        id: id.to_owned(),
        name: name.to_owned(),
        exec: exec.map(str::to_owned),
        try_exec: None,
        flatpak: flatpak.map(str::to_owned),
        icon: Some(std::path::PathBuf::from(format!("/icons/{id}.svg"))),
        origin: SchemaOrigin::System,
    }
}

fn planted(mut entry: DesktopApp) -> DesktopApp {
    entry.origin = SchemaOrigin::PersonWritable;
    entry
}

fn book_of(apps: Vec<DesktopApp>) -> ProgramBook {
    ProgramBook {
        apps,
        system_bins: vec![std::path::PathBuf::from("/usr/bin")],
    }
}

#[test]
fn a_program_is_named_by_the_system_entry_that_starts_it_else_by_its_file_and_folder() {
    let mut try_only = app("try", "Tried", None, None);
    try_only.try_exec = Some("/opt/try/bin/tried".to_owned());
    let book = book_of(vec![
        app("grim", "Grim", Some("/usr/bin/grim"), None),
        app("bare", "Bare", Some("bare"), None),
        try_only,
        app("loupe", "Image Viewer", Some("flatpak"), Some(LOUPE)),
        app("nameless", "", Some("/usr/bin/nameless"), None),
    ]);
    // `(case, subject, name, has the entry's icon, the folder shown)`.
    const CASES: &[(&str, &str, &str, bool, Option<&str>)] = &[
        ("the same path", "/usr/bin/grim", "Grim", true, None),
        (
            "a bare name is the image's file name in a system folder",
            "/usr/bin/bare",
            "Bare",
            true,
            None,
        ),
        (
            "a bare name does not name an image elsewhere",
            "/opt/x/bare",
            "bare",
            false,
            Some("/opt/x"),
        ),
        (
            "TryExec starts the image",
            "/opt/try/bin/tried",
            "Tried",
            true,
            None,
        ),
        (
            "a sandboxed app by its id",
            LOUPE,
            "Image Viewer",
            true,
            None,
        ),
        (
            "an entry of a sandboxed app starts no host program",
            "/usr/bin/flatpak",
            "flatpak",
            false,
            Some("/usr/bin"),
        ),
        (
            "no entry: the file name and its folder",
            "/usr/bin/unknown",
            "unknown",
            false,
            Some("/usr/bin"),
        ),
        (
            "no entry: the app's id",
            "org.example.Gone",
            "org.example.Gone",
            false,
            None,
        ),
        (
            "an entry without a name: the file name, the entry's icon",
            "/usr/bin/nameless",
            "nameless",
            true,
            Some("/usr/bin"),
        ),
        ("nothing to go by", "", "A program", false, None),
    ];
    for (case, who, name, icon, place) in CASES {
        let look = book.look_of(&subject(who));
        assert_eq!(look.name, *name, "{case}");
        assert_eq!(look.icon.is_some(), *icon, "{case}");
        assert_eq!(look.place.as_deref(), *place, "{case}");
    }
    let long = book_of(vec![app(
        "long",
        &"x".repeat(100),
        Some("/usr/bin/long"),
        None,
    )]);
    let name = long.look_of(&subject("/usr/bin/long")).name;
    assert_eq!(name.chars().count(), 48);
    assert!(name.ends_with('\u{2026}'));
}

#[test]
fn an_entry_a_program_planted_in_a_folder_the_person_can_write_names_nothing() {
    // `(case, the entry, the subject, the name shown)`: the entry says "Firefox"; the program is
    // not the real one, and says what it is.
    let cases = [
        (
            "the planted entry's launch line is the program's own path",
            planted(app("x", "Firefox", Some("/tmp/evil"), None)),
            "/tmp/evil",
            "evil",
        ),
        (
            "the planted entry's launch line is a bare name",
            planted(app("x", "Firefox", Some("evil"), None)),
            "/usr/bin/evil",
            "evil",
        ),
        (
            "the program is called firefox, in a folder the person can write",
            app("firefox", "Firefox", Some("firefox"), None),
            "/home/ada/bin/firefox",
            "firefox",
        ),
        (
            "a planted entry for a sandboxed app's id",
            planted(app("org.evil.App", "Firefox", None, Some("org.evil.App"))),
            "org.evil.App",
            "org.evil.App",
        ),
    ];
    for (case, entry, who, want) in cases {
        let look = book_of(vec![entry]).look_of(&subject(who));
        assert_eq!(look.name, want, "{case}");
        assert_eq!(look.icon, None, "{case}: no borrowed icon");
    }
    let shown = book_of(vec![planted(app("x", "Firefox", Some("/tmp/evil"), None))])
        .look_of(&subject("/tmp/evil"));
    assert_eq!(shown.place.as_deref(), Some("/tmp"));

    // A planted entry ahead of the system's own does not take its place.
    let both = book_of(vec![
        planted(app("x", "Firefox", Some("/usr/bin/firefox"), None)),
        app("firefox", "Mozilla Firefox", Some("/usr/bin/firefox"), None),
    ]);
    assert_eq!(
        both.look_of(&subject("/usr/bin/firefox")).name,
        "Mozilla Firefox"
    );
}

#[test]
fn a_name_that_hides_or_reorders_text_is_drawn_plainly() {
    let look = book_of(Vec::new()).look_of(&subject("/tmp/ev\u{202E}il\nname"));
    assert_eq!(look.name, "ev\u{FFFD}il\u{FFFD}name");
    let deep = format!("/{}/prog", "d".repeat(100));
    let place = book_of(Vec::new()).look_of(&subject(&deep)).place;
    let place = place.unwrap_or_default();
    assert_eq!(place.chars().count(), 60);
    assert!(place.starts_with('\u{2026}') && place.ends_with('d'));
}
