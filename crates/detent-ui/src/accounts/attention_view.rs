//! The banner at the head of an account's screen when it needs the person: its sign-in ended, or
//! the provider is about to end it. Its button runs the account's Sign in again row.

use super::adapter::ActionButton;
use detent_model::{Account, Attention, ModuleRow};
use dioxus::prelude::*;
use ds::components::overlays::inline_banner::InlineBanner;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};
use ds_settings::schema::ActionWeight;

/// What the banner's button says; the account's name stands behind it for a screen reader.
const SIGN_IN: &str = "Sign In\u{2026}";

/// The banner for `attention` on `account`; nothing when the account has no Sign in again row to
/// run. `onsign_in` is given that row.
#[component]
pub(super) fn AttentionBanner(
    account: Account,
    attention: Attention,
    availability: Availability,
    onsign_in: EventHandler<ModuleRow>,
) -> Element {
    let Some((path, _)) = account.reauth.clone() else {
        return rsx! {};
    };
    let severity = match attention {
        Attention::SignInEnded => Severity::Warn,
        Attention::SignInEnding { .. } => Severity::Info,
    };
    let row = account.row(&path);
    rsx! {
        InlineBanner {
            severity,
            text: TextLine::from(attention.words(&account)),
            common: Common {
                data: DataName::parse("accounts")
                    .map(|name| vec![DataAttr::new(name, "attention")])
                    .unwrap_or_default(),
                ..Common::default()
            },
            actions: rsx! {
                ActionButton {
                    label: SIGN_IN.to_owned(),
                    subject: account.label.clone(),
                    weight: ActionWeight::Plain,
                    availability,
                    onpress: move |()| onsign_in.call(row.clone()),
                }
            },
        }
    }
}
