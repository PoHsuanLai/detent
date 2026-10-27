//! Which rows of the account modules are read with their value.

use ds_settings::schema::KeyKind;

/// The rows whose value is read with the rows: the services (on or off) and the client ids.
pub fn accounts_row_is_valued(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Toggle { .. } | KeyKind::Text)
}
