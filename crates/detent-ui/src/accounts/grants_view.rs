//! The apps that may use an account: a row each with the app's permission and when it was given,
//! Revoke on a strip that shows under the pointer, and a card that names the services the app
//! uses.

use crate::copy::accounts::grant_when;
use crate::key_row::row_id;
use detent_model::{Account, AccountPermission, ModuleRow};
use dioxus::prelude::*;
use ds::components::app::hover_strip::{ActionId, HoverStrip, StripAction};
use ds::components::lists::list::model::ListStyle;
use ds::components::overlays::hover_card::parts::HoverCardPart;
use ds::components::overlays::hover_card::target::{HoverTarget, TargetElement};
use ds::prelude::*;
use ds::root::common::Common;
use ds::stack::hover_hub::{HoverKey, HoverKind, use_hover_hub};

/// Before a permission's path in the key of its hover target.
const KEY_PREFIX: &str = "grant:";

/// The strip's one action.
const REVOKE: &str = "revoke";

/// Revoke, on the strip of `permission`'s row. It acts inside the press and only while the pane is
/// free.
fn revoke_strip(
    permission: &AccountPermission,
    row: ModuleRow,
    availability: Availability,
    onrevoke: EventHandler<ModuleRow>,
) -> Element {
    let actions = vec![StripAction {
        id: ActionId(REVOKE.to_owned()),
        icon: Icon::X,
        label: crate::copy::account_views::revoke(&permission.label),
        fly: crate::copy::account_views::REVOKE_ASKS_AGAIN_NEXT_TIME.to_owned(),
        onhover: None,
        onclick: EventHandler::new(|_| {}),
    }];
    rsx! {
        HoverStrip {
            actions,
            on_press: move |_: ActionId| {
                if availability == Availability::Enabled {
                    onrevoke.call(row.clone());
                }
            },
        }
    }
}

fn app_item(
    account: &Account,
    permission: &AccountPermission,
    availability: Availability,
    onrevoke: EventHandler<ModuleRow>,
) -> ListItem<String> {
    let strip = revoke_strip(
        permission,
        account.row(&permission.path),
        availability,
        onrevoke,
    );
    let row = rsx! {
        Row {
            title: permission.label.clone(),
            detail: grant_when(permission).map(TextLine::from),
            accessory: Accessory::Slot(strip),
            common: Common { id: Some(row_id("permission", &permission.path.0)), ..Common::default() },
        }
    };
    // Only a permission whose sentence names an app has a card to show.
    let content = match permission.sentence() {
        Some(_) => rsx! {
            HoverTarget {
                hover_key: HoverKey(format!("{KEY_PREFIX}{}", permission.path.0)),
                kind: HoverKind::Side,
                as_: TargetElement::Div,
                {row}
            }
        },
        None => row,
    };
    ListItem::row(permission.path.0.clone(), permission.label.clone(), content)
}

/// The card of the permission whose hover key is `key`: the app, and the services it uses.
fn card_of(account: &Account, key: &HoverKey) -> Option<Vec<HoverCardPart>> {
    let path = key.0.strip_prefix(KEY_PREFIX)?;
    let permission = account.permissions.iter().find(|p| p.path.0 == path)?;
    let app = permission.sentence()?.app;
    let services = account.services_of_app(&app);
    let uses = if services.is_empty() {
        crate::copy::account_views::USES_NO_SERVICE.to_owned()
    } else {
        crate::copy::account_views::uses(services.join(", "))
    };
    Some(vec![HoverCardPart::Title(app), HoverCardPart::Sub(uses)])
}

/// The permissions of `account`, as a list of apps, and the card the pointer resting on one opens.
#[component]
pub(super) fn AppsWithAccess(
    account: Account,
    availability: Availability,
    onrevoke: EventHandler<ModuleRow>,
) -> Element {
    let hub = use_hover_hub();
    let items: Vec<ListItem<String>> = account
        .permissions
        .iter()
        .map(|permission| app_item(&account, permission, availability, onrevoke))
        .collect();
    let open = hub.open().filter(|(key, _)| key.0.starts_with(KEY_PREFIX));
    rsx! {
        div { class: "accounts-apps", "data-accounts": "apps",
            List::<String> {
                label: crate::copy::account_views::APPS_WITH_ACCESS,
                items,
                style: ListStyle::Grouped,
            }
            if let Some((key, _)) = open {
                if let Some(parts) = card_of(&account, &key) {
                    HoverCard { key: "{key.0}", kind: HoverKind::Side, parts, children: rsx! {} }
                }
            }
        }
    }
}
