//! The first-run step "Sign in to your accounts": the accounts already signed in, "Add Account…"
//! (the same sheet the Accounts page opens), Skip and Continue. Either button closes the window;
//! the setup flow opens the next step.

use super::avatar_view::{AccountAvatar, AvatarPlace};
use super::watch::{Listed, use_listed};
use crate::copy::account_notices::add_end_line;
use crate::copy::accounts::status_label;
use crate::env::Env;
use crate::key_row::row_id;
use crate::marks::ProviderMarks;
use detent_model::{
    Adding, SetupAction, SetupAvailability, SignedIn, SyncWork, accounts_setup_offer, signed_in,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button::Button;
use ds::components::controls::button_model::Answers;
use ds::components::lists::list::model::ListStyle;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;
use ds::style::icon::family::PlateFamily;
use ds::style::tokens::control_size::ControlSize;
use ds_blitz::{AppHandle, use_app_handle};

/// Said once, at the top.
const HEADING: &str = "Sign in to Your Accounts";

/// What the step says while nothing is signed in.
const NOTHING_YET: &str =
    "Nothing is signed in yet. Add an account and the apps you allow can use it.";

/// The line under an account that needs a new sign-in.
fn standing_line(row: &SignedIn) -> String {
    row.provider.as_ref().map_or_else(
        || status_label(&row.status),
        |provider| provider.name.clone(),
    )
}

/// The accounts signed in, one row each.
fn rows_of(rows: &[SignedIn], marks: &ProviderMarks) -> Vec<ListItem<String>> {
    rows.iter()
        .map(|row| {
            let content = rsx! {
                div { class: "account-row-body",
                    AccountAvatar {
                        label: row.label.clone(),
                        provider: row.provider.clone(),
                        marks: marks.clone(),
                        place: AvatarPlace::Row,
                        work: SyncWork::Resting,
                    }
                    div { class: "account-row-words",
                        Label { text: row.label.clone(), role: LabelRole::Primary }
                        Label { text: standing_line(row), role: LabelRole::Secondary, style: LabelStyle::Footnote }
                    }
                }
            };
            ListItem::row(
                row.key.0.clone(),
                row.label.clone(),
                rsx! {
                    Row {
                        title: row.label.clone(),
                        content,
                        common: Common {
                            id: Some(row_id("signed-in", &row.key.0)),
                            ..Common::default()
                        },
                    }
                },
            )
        })
        .collect()
}

/// The step, under the frame.
#[component]
pub(crate) fn AccountsSetup() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let app = use_app_handle();
    let listed = use_listed();
    let adding = use_signal(|| Adding::Idle);
    let rows = match listed() {
        Listed::Accounts(pane) => signed_in(&pane),
        Listed::Looking | Listed::Absent(_) => Vec::new(),
    };
    let skip = availability_of(accounts_setup_offer(&rows, SetupAction::Skip));
    let next = availability_of(accounts_setup_offer(&rows, SetupAction::Continue));
    let add = match adding() {
        Adding::Idle => Availability::Enabled,
        Adding::SheetOpen => Availability::Disabled,
    };
    let items = rows_of(&rows, &env.marks);
    let empty = rows.is_empty();
    let accounts = env.accounts.clone();
    let (skip_app, next_app) = (app.clone(), app);
    rsx! {
        div { class: "accounts-setup", "data-accounts": "setup",
            IconView {
                source: IconSource::Glyph(Icon::Key),
                size: IconSize::Tile48,
                plate: Some(PlateFamily::Blue),
            }
            Label { text: HEADING, role: LabelRole::Primary, style: LabelStyle::Title }
            div { class: "accounts-setup-card",
                if empty {
                    Label { text: NOTHING_YET, role: LabelRole::Secondary }
                } else {
                    List::<String> {
                        label: "Signed in",
                        items,
                        style: ListStyle::Grouped,
                    }
                }
            }
            Button {
                common: Common { aria_label: Some("Add Account\u{2026}".to_owned()), ..Common::default() },
                label: "Add Account\u{2026}",
                size: ControlSize::Large,
                availability: add,
                onclick: move |_| {
                    let (accounts, mut adding) = (accounts.clone(), adding);
                    adding.set(Adding::SheetOpen);
                    spawn(async move {
                        let end = accounts.add_account().await;
                        adding.set(Adding::Idle);
                        if let Some(line) = add_end_line(&end) {
                            toasts.push(line.to_owned(), None);
                        }
                    });
                },
            }
            div { class: "accounts-setup-buttons",
                Button {
                    label: "Skip",
                    answers: Answers::Escape,
                    size: ControlSize::Large,
                    availability: skip,
                    common: Common { extra_class: ExtraClass::parse("accounts-skip").ok(), ..Common::default() },
                    onclick: move |_| leave(skip_app.as_ref()),
                }
                Button {
                    label: "Continue",
                    answers: Answers::Return,
                    size: ControlSize::Large,
                    availability: next,
                    common: Common { extra_class: ExtraClass::parse("accounts-continue").ok(), ..Common::default() },
                    onclick: move |_| leave(next_app.as_ref()),
                }
            }
        }
    }
}

/// A button that can be pressed now is enabled.
fn availability_of(availability: SetupAvailability) -> Availability {
    match availability {
        SetupAvailability::Now => Availability::Enabled,
        SetupAvailability::NotYet => Availability::Disabled,
    }
}

/// Closes the window. `app` is the running app, absent in the harness, where there is no window
/// to close.
fn leave(app: Option<&AppHandle>) {
    if let Some(app) = app {
        // The loop having ended already is the same as the window being closed.
        let _ended = app.quit();
    }
}
