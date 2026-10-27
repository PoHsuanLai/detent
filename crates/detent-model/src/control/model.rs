//! The one control that draws a key (design/22 section 9.1's fixed table), and which values it
//! accepts.

use super::toggle::TogglePair;
use crate::hot_corners::{HOT_CORNERS_CONTROL, corner_admits};
use crate::range::Range;
use crate::rows::Columns;
use ds_settings::schema::{Control as Named, KeyKind, KeySpec, STANDARD};

/// A kind quire has no component for: shown read-only with an "Edit in file" note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOnly {
    /// A captured key chord: quire has no shortcut-capture field.
    Shortcut,
    /// A list of values: quire has no list editor.
    List,
    /// An action the owning program runs through its live settings module (a button): detent
    /// has no live client yet, so the row is read-only until the Accounts pane's lane wires one.
    Action,
}

/// What draws a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control {
    /// A one-word enum: read-only text.
    Readout(String),
    /// A two-word enum that is on and off: a switch.
    Toggle(TogglePair),
    /// Three to five words, or two that are not on and off: a segmented control.
    Segmented(Vec<String>),
    /// More: a pop-up button.
    Menu(Vec<String>),
    /// A bounded number: a slider.
    Slider(Range),
    /// Free text: a text field.
    Text,
    /// A colour: the swatch row.
    Colour,
    /// A path chosen from a grid of pictures, with "Add Picture…": a text key whose schema
    /// says `control = "picture"`.
    Picture,
    /// What a screen corner does, for each set of modifier keys: a key whose schema says
    /// `control = "corners"`. The Desktop & Dock page draws the four corner keys as one
    /// button that opens the Hot Corners sheet, whatever kind the schema gives them.
    HotCorners,
    /// A choice among things somebody installed (a look): a pop-up button over the words on
    /// offer, `standard` first. Any stored text is held, so the id of an Extension that is gone
    /// is kept and shown as unavailable rather than reset (`KeyEntry::offering` fills the words).
    Provided(Vec<String>),
    /// A list of tables: the rows editor, one cell per column.
    Rows(Columns),
    /// No component: read-only, edited in the file.
    FileOnly(FileOnly),
}

/// Whether the schema asks for the picture grid and the key can hold a path (a control that
/// cannot draw the key's kind is ignored, so a schema never makes a grid of a number).
fn named_picture(spec: &KeySpec) -> bool {
    spec.control
        .as_ref()
        .and_then(Named::named)
        .is_some_and(|named| named == Named::Picture && named.fits(&spec.kind))
}

// The raw name `corners` is read, not `Named::named`.
/// Whether the schema asks for the Hot Corners sheet. The sheet reads the value itself, so the
/// key's kind does not matter.
fn named_hot_corners(spec: &KeySpec) -> bool {
    spec.control
        .as_ref()
        .is_some_and(|name| name.0 == HOT_CORNERS_CONTROL)
}

impl Control {
    /// The control for `spec`'s kind. Total: every [`KeyKind`] arm is written, so a new kind is
    /// a compile error here.
    pub fn of(spec: &KeySpec) -> Control {
        if named_picture(spec) {
            return Control::Picture;
        }
        if named_hot_corners(spec) {
            return Control::HotCorners;
        }
        match &spec.kind {
            KeyKind::Fixed { variant } => Control::Readout(variant.clone()),
            KeyKind::Toggle { variants } => TogglePair::of(variants)
                .map_or_else(|| Control::Segmented(variants.to_vec()), Control::Toggle),
            KeyKind::Segmented { variants } => Control::Segmented(variants.clone()),
            KeyKind::Menu { variants } => Control::Menu(variants.clone()),
            KeyKind::Bounded { .. } => {
                Range::of(&spec.kind, &spec.default).map_or(Control::Text, Control::Slider)
            }
            KeyKind::Text => Control::Text,
            KeyKind::Colour => Control::Colour,
            KeyKind::Shortcut => Control::FileOnly(FileOnly::Shortcut),
            KeyKind::Rows { columns } => Control::Rows(Columns::new(columns.clone())),
            KeyKind::List(_) => Control::FileOnly(FileOnly::List),
            KeyKind::Live { .. } => Control::FileOnly(FileOnly::Action),
            KeyKind::Provided { .. } => Control::Provided(vec![STANDARD.to_owned()]),
        }
    }

    /// `value` if this control can hold it (a number in range, a word of its set, text), else
    /// `None`: the lenient loader then uses the key's default.
    pub fn admit(&self, value: &toml::Value) -> Option<toml::Value> {
        let word_of = |words: &[String]| {
            value
                .as_str()
                .filter(|word| words.iter().any(|w| w == word))
                .map(|_| value.clone())
        };
        match self {
            Control::Readout(word) => word_of(std::slice::from_ref(word)),
            Control::Toggle(pair) => word_of(&[pair.on.clone(), pair.off.clone()]),
            Control::Segmented(words) | Control::Menu(words) => word_of(words),
            Control::Slider(range) => range.admit(value),
            Control::Text | Control::Colour | Control::Picture | Control::Provided(_) => {
                value.is_str().then(|| value.clone())
            }
            Control::HotCorners => corner_admits(value).then(|| value.clone()),
            Control::Rows(columns) => columns.cells(value).map(|_| value.clone()),
            Control::FileOnly(_) => Some(value.clone()),
        }
    }
}
