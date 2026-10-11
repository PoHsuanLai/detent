//! accountd's accounts, read where a page outside the Accounts pane needs them: read when accountd
//! appears, again when it says a row changed, and again when its name changes hands.

use super::drive::is_valued;
use super::reload::{reload, reload_later};
use crate::env::Env;
use crate::live::{Daemon, Followed, follow};
use detent_model::{Absence, AccountsPane, Reads};
use dioxus::prelude::*;
use ds_settings::live::LiveClient;

/// How many times a failed read is tried again.
const RETRIES: usize = 3;

/// Where the reading stands.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Listed {
    /// accountd has not answered yet.
    Looking,
    /// accountd cannot be used, and why.
    Absent(Absence),
    /// The accounts accountd lists now.
    Accounts(AccountsPane),
}

/// The accounts accountd lists, kept current.
pub(crate) fn use_listed() -> Signal<Listed> {
    let env = use_context::<Env>();
    let mut listed = use_signal(|| Listed::Looking);
    let mut client = use_signal(|| None::<LiveClient>);
    let reads = use_signal(Reads::default);
    use_future(move || {
        let bus = env.bus.clone();
        async move {
            follow(
                &bus,
                &Daemon::accountd(),
                is_valued,
                |followed| match followed {
                    Followed::Opened(opened, described) => {
                        client.set(Some(opened));
                        listed.set(Listed::Accounts(AccountsPane::of(&described)));
                    }
                    Followed::Gone(absence) => {
                        client.set(None);
                        listed.set(Listed::Absent(absence));
                    }
                    Followed::Changed(..) => {
                        let Some(opened) = client.peek().clone() else {
                            return;
                        };
                        spawn(async move {
                            let mut found = reload(reads, &opened).await;
                            // A failed read keeps the list and is tried again, a few times.
                            for _ in 0..RETRIES {
                                if !matches!(found, Some(Err(_))) {
                                    break;
                                }
                                found = reload_later(reads, &opened).await;
                            }
                            if let Some(Ok(described)) = found {
                                listed.set(Listed::Accounts(AccountsPane::of(&described)));
                            }
                        });
                    }
                },
            )
            .await;
        }
    });
    listed
}
