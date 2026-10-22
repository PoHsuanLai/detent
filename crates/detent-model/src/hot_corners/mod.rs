//! The Hot Corners sheet of the Desktop & Dock page: what each screen corner does, as a list of
//! bindings, each for a set of modifier keys. Pure: the sheet draws it and writes the corner
//! keys with the values it makes.

mod actions;
mod binding;
mod corner;
mod forms;
mod keys;
mod menu;
mod rows;
#[cfg(test)]
mod tests;

pub use actions::{
    HotActionChoice, HotActionGroup, HotApp, HotKnown, action_groups, actions_of, actions_of_all,
    apps_matching,
};
pub use binding::{HotBinding, HotDesk, HotDo, HotItem, bindings_of, corner_admits, corner_value};
pub use corner::{Corner, HOT_CORNERS_CONTROL};
pub use forms::{HotOpenForm, HotOpenKind, command_binding, web_address};
pub use keys::{HotKeys, HotModifier};
pub use menu::{
    CornerAsk, CornerChoice, CornerMenuLine, HotPicked, HotShown, corner_label, corner_menu,
    corner_picked, corner_shown,
};
pub use rows::{HotAddFault, add_row, binding_for, remove_row, shown_rows, with_binding};
