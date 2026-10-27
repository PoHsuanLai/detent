//! The page over a fake publication: the rows casement lists, as they are named and typed. The
//! fixture is here; the panes, the changes and the names are in the files beside it.

mod pane;
mod programs;
mod step;

use super::*;
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
        ends: None,
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
        row.note = Some(Note("Set by your administrator".to_owned()));
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
