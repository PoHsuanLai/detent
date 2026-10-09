//! The assistants that sign themselves in (Claude Code, Antigravity): one row each, with how it
//! stands and the way to its account, where it is signed in. Apart from the built-in assistant's
//! places; the two are never one list.

use crate::accounts::{AccountLink, OpenButton};
use detent_model::{Account, AccountsPane};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use porter_core::sheet::ProviderGroup;

/// The group's title.
const TITLE: &str = "Other assistants";

/// The rows of the assistants `pane` lists, nothing when it lists none.
#[component]
pub(super) fn ExternalAssistants(pane: AccountsPane) -> Element {
    let link = use_context::<AccountLink>();
    let assistants: Vec<Account> = pane.in_group(ProviderGroup::Agent).cloned().collect();
    if assistants.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "assistants", "data-assistants": "others",
            FieldGroup { title: Some(TITLE.to_owned()),
                for assistant in assistants {
                    FieldRow {
                        key: "{assistant.key.0}",
                        label: TextLine::from(assistant.label.clone()),
                        Label { text: assistant.status.label(), role: LabelRole::Secondary, style: LabelStyle::Footnote }
                        OpenButton {
                            label: if assistant.reauth.is_some() { "Sign in" } else { "Manage" },
                            subject: assistant.label.clone(),
                            onpress: {
                                let key = assistant.key.clone();
                                move |()| link.open(key.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}
