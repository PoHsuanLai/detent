//! Which row of inferd's picker the assistant is routed by.

use crate::picker::{ModelRow, PickerState, Ready};
use ds_settings::schema::KeyPath;

/// The assistant's row: the text slot at the balanced tier (the planner asks for that tier),
/// under the name the module gives it now and the one an older module still gives it.
pub(crate) const ASSISTANT_ROWS: [&str; 2] = ["ai.model.text.balanced", "ai.model.llm.balanced"];

/// The assistant's row in `ready`; `None` when the module lists none.
fn assistant_row(ready: &Ready) -> Option<&ModelRow> {
    ASSISTANT_ROWS
        .iter()
        .find_map(|path| ready.row(&KeyPath((*path).to_owned())))
}

/// The assistant's row, when the picker is there and lists one.
pub fn assistant_of(state: &PickerState) -> Option<&ModelRow> {
    match state {
        PickerState::Ready(ready) => assistant_row(ready),
        PickerState::Looking | PickerState::Unavailable(_) => None,
    }
}
