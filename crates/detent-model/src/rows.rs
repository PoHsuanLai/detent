//! A list-of-tables key (design/22 section 9.1, `KeyKind::Rows`) as the rows editor holds it:
//! every row a line of text per column, checked against the column's kind.

use ds_settings::schema::{Column, ColumnKind};
use regex::Regex;

/// The columns of a rows key, in the order the schema gives them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Columns(pub Vec<Column>);

/// Whether `text` is a value of `column`: a word of a choice, or text the whole of which matches
/// the column's pattern. A pattern that is not a regular expression admits any text; the owning
/// program validates the rows again at load.
pub fn cell_fits(column: &Column, text: &str) -> bool {
    match &column.kind {
        ColumnKind::Choice { variants } => variants.iter().any(|word| word == text),
        ColumnKind::Text { pattern: None } => true,
        ColumnKind::Text {
            pattern: Some(pattern),
        } => Regex::new(&format!("^(?:{pattern})$")).is_ok_and(|whole| whole.is_match(text)),
    }
}

impl Columns {
    /// `value`'s rows as text, one string per column, when every row is a table of exactly these
    /// columns holding values that fit; else `None` (the value is shown as written and no edit
    /// could keep what the editor cannot hold).
    pub fn cells(&self, value: &toml::Value) -> Option<Vec<Vec<String>>> {
        value
            .as_array()?
            .iter()
            .map(|row| {
                let table = row.as_table()?;
                (table.len() == self.0.len()).then_some(())?;
                self.0
                    .iter()
                    .map(|column| {
                        let text = table.get(&column.name.0)?.as_str()?;
                        cell_fits(column, text).then(|| text.to_owned())
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
                (row.len() == self.0.len()).then_some(())?;
                self.0
                    .iter()
                    .zip(row)
                    .map(|(column, text)| {
                        cell_fits(column, text)
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
        self.0
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
    use super::{Columns, cell_fits};
    use ds_settings::schema::{Column, ColumnKind, ColumnName, Label};

    fn column(name: &str, kind: ColumnKind) -> Column {
        Column {
            name: ColumnName(name.to_owned()),
            label: Label(name.to_owned()),
            kind,
        }
    }

    fn apps() -> Columns {
        Columns(vec![
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
            assert_eq!(cell_fits(&columns.0[at], text), fits, "{text:?}");
        }
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
