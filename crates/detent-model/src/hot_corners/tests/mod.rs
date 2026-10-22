use super::{
    Corner, CornerAsk, CornerChoice, CornerMenuLine, HotActionChoice, HotApp, HotBinding, HotDesk,
    HotDo, HotItem, HotKeys, HotKnown, HotModifier, HotOpenForm, HotOpenKind, HotPicked,
    action_groups, actions_of, actions_of_all, add_row, apps_matching, binding_for, bindings_of,
    command_binding, corner_admits, corner_label, corner_menu, corner_picked, corner_shown,
    corner_value, remove_row, shown_rows, web_address, with_binding,
};
use toml::Value;

/// The value of the one key `v` in the TOML `text` (`v = ...`).
fn value(text: &str) -> Value {
    let table: toml::Table = text.parse().expect("test TOML");
    table["v"].clone()
}

fn keys(modifiers: &[HotModifier]) -> HotKeys {
    HotKeys::of(modifiers.iter().copied())
}

fn bind(held: &[HotModifier], does: HotDo) -> HotBinding {
    HotBinding {
        keys: keys(held),
        does,
    }
}

fn command(text: &str) -> HotDo {
    HotDo::Command(text.to_owned())
}

fn known() -> HotKnown {
    HotKnown {
        apps: vec![HotApp {
            id: "org.quire.Mail".to_owned(),
            name: "Mail".to_owned(),
        }],
        actions: vec![HotActionChoice {
            app: "org.quire.Mail".to_owned(),
            action: "compose".to_owned(),
            label: "Compose".to_owned(),
        }],
    }
}

fn lock() -> HotDo {
    HotDo::Desk(HotDesk::Lock)
}

mod actions;
mod forms;
mod menu;
mod rows;
mod values;
