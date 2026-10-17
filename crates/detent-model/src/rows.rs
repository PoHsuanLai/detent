//! A list-of-tables key (design/22 section 9.1, `KeyKind::Rows`) as the rows editor holds it:
//! every row a line of text per column, checked against the column's kind.

use ds_settings::schema::{Column, ColumnKind};
use regex::Regex;
use std::sync::Arc;

/// The columns of a rows key, in the order the schema gives them, with each pattern compiled
/// once here and not each time a cell is checked.
#[derive(Debug, Clone)]
pub struct Columns {
    columns: Vec<Column>,
    /// For each column, the whole-text expression of its pattern; none for a column without one
    /// and for a pattern that is not a regular expression.
    patterns: Arc<[Option<Regex>]>,
}

impl PartialEq for Columns {
    /// Two sets of columns are the same when the schema's columns are.
    fn eq(&self, other: &Columns) -> bool {
        self.columns == other.columns
    }
}

impl Eq for Columns {}

/// The expression that matches the whole of a text matching `pattern`; none when `pattern` is not
/// a regular expression.
fn whole(pattern: &str) -> Option<Regex> {
    Regex::new(&format!("^(?:{pattern})$")).ok()
}

impl Columns {
    /// The columns `columns`, their patterns compiled.
    pub fn new(columns: Vec<Column>) -> Columns {
        let patterns = columns
            .iter()
            .map(|column| match &column.kind {
                ColumnKind::Text {
                    pattern: Some(pattern),
                } => whole(pattern),
                ColumnKind::Text { pattern: None } | ColumnKind::Choice { .. } => None,
            })
            .collect();
        Columns { columns, patterns }
    }

    /// The columns, in order.
    pub fn all(&self) -> &[Column] {
        &self.columns
    }

    /// Whether `text` is a value of the column at `at`: a word of a choice, or text the whole of
    /// which matches the column's pattern. A pattern that is not a regular expression admits
    /// nothing (the schema is wrong; a cell is not stored against it), and there is no such
    /// column for an `at` past the last.
    pub fn fits(&self, at: usize, text: &str) -> bool {
        let Some(column) = self.columns.get(at) else {
            return false;
        };
        match &column.kind {
            ColumnKind::Choice { variants } => variants.iter().any(|word| word == text),
            ColumnKind::Text { pattern: None } => true,
            ColumnKind::Text { pattern: Some(_) } => self
                .patterns
                .get(at)
                .and_then(Option::as_ref)
                .is_some_and(|pattern| pattern.is_match(text)),
        }
    }

    /// `value`'s rows as text, one string per column, when every row is a table of exactly these
    /// columns holding values that fit; else `None` (the value is shown as written and no edit
    /// could keep what the editor cannot hold).
    pub fn cells(&self, value: &toml::Value) -> Option<Vec<Vec<String>>> {
        value
            .as_array()?
            .iter()
            .map(|row| {
                let table = row.as_table()?;
                (table.len() == self.columns.len()).then_some(())?;
                self.columns
                    .iter()
                    .enumerate()
                    .map(|(at, column)| {
                        let text = table.get(&column.name.0)?.as_str()?;
                        self.fits(at, text).then(|| text.to_owned())
                    })
                    .collect()
            })
            .collect()
    }

    /// The rows `cells` hold as a value to store, when every cell fits its column.
    pub fn value_of(&self, cells: &[Vec<String>]) -> Option<toml::Value> {
        cells
            .iter()
            .map(|row| {
                (row.len() == self.columns.len()).then_some(())?;
                self.columns
                    .iter()
                    .zip(row)
                    .enumerate()
                    .map(|(at, (column, text))| {
                        self.fits(at, text)
                            .then(|| (column.name.0.clone(), toml::Value::String(text.clone())))
                    })
                    .collect::<Option<toml::Table>>()
                    .map(toml::Value::Table)
            })
            .collect::<Option<Vec<_>>>()
            .map(toml::Value::Array)
    }

    /// A new row's starting cells: empty text, and a choice's first word.
    pub fn blank(&self) -> Vec<String> {
        self.columns
            .iter()
            .map(|column| match &column.kind {
                ColumnKind::Choice { variants } => variants.first().cloned().unwrap_or_default(),
                ColumnKind::Text { .. } => String::new(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Columns;
    use ds_settings::schema::{Column, ColumnKind, ColumnName, Label};

    fn column(name: &str, kind: ColumnKind) -> Column {
        Column {
            name: ColumnName(name.to_owned()),
            label: Label(name.to_owned()),
            kind,
        }
    }

    fn apps() -> Columns {
        Columns::new(vec![
            column(
                "app",
                ColumnKind::Text {
                    pattern: Some("[a-z]+(\\.[a-z]+)+".to_owned()),
                },
            ),
            column(
                "allow",
                ColumnKind::Choice {
                    variants: vec!["ask".to_owned(), "always".to_owned()],
                },
            ),
        ])
    }

    fn rows(text: &str) -> toml::Value {
        text.parse::<toml::Table>().unwrap()["r"].clone()
    }

    #[test]
    fn a_cell_must_match_its_pattern_whole_and_its_choice_exactly() {
        let columns = apps();
        let cases = [
            (0, "org.gnome.Nautilus", false),
            (0, "org.gnome", true),
            (0, "org.gnome extra", false),
            (0, "", false),
            (1, "ask", true),
            (1, "Ask", false),
        ];
        for (at, text, fits) in cases {
            assert_eq!(columns.fits(at, text), fits, "{text:?}");
        }
    }

    #[test]
    fn a_pattern_that_is_not_a_regular_expression_admits_nothing_and_a_missing_column_too() {
        let columns = Columns::new(vec![column(
            "app",
            ColumnKind::Text {
                pattern: Some("([a-z".to_owned()),
            },
        )]);
        assert!(!columns.fits(0, "abc"));
        assert!(!columns.fits(0, ""));
        assert!(!apps().fits(2, "ask"));
    }

    #[test]
    fn rows_become_cells_and_cells_become_the_same_rows() {
        let columns = apps();
        let value = rows(
            "r = [{ app = \"org.a\", allow = \"ask\" }, { app = \"org.b\", allow = \"always\" }]",
        );
        let cells = columns.cells(&value).expect("rows that fit");
        assert_eq!(cells, [["org.a", "ask"], ["org.b", "always"]]);
        assert_eq!(columns.value_of(&cells), Some(value));
    }

    #[test]
    fn rows_the_editor_cannot_hold_are_not_taken() {
        let columns = apps();
        for (case, text) in [
            (
                "an unknown word",
                "r = [{ app = \"org.a\", allow = \"maybe\" }]",
            ),
            (
                "a cell too many",
                "r = [{ app = \"org.a\", allow = \"ask\", why = \"x\" }]",
            ),
            ("a cell missing", "r = [{ app = \"org.a\" }]"),
            ("a bad name", "r = [{ app = \"nope\", allow = \"ask\" }]"),
            ("not a list", "r = \"org.a\""),
        ] {
            assert_eq!(columns.cells(&rows(text)), None, "{case}");
        }
    }

    #[test]
    fn a_blank_row_is_not_a_value_until_it_is_filled_in() {
        let columns = apps();
        let blank = columns.blank();
        assert_eq!(blank, ["", "ask"]);
        assert_eq!(columns.value_of(&[blank]), None);
    }
}
