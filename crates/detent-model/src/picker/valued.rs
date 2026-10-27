//! Which rows of inferd's module the picker reads with their value.

use ds_settings::schema::KeyKind;

/// The rows whose value the picker reads: its menus, and the lists of choices from a menu (the
/// models Automatic chooses among).
pub fn picker_row_is_choice(kind: &KeyKind) -> bool {
    match kind {
        KeyKind::Menu { .. } => true,
        KeyKind::List(inner) => matches!(**inner, KeyKind::Menu { .. }),
        _ => false,
    }
}
