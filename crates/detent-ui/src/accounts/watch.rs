//! The accounts the modules list, read where a page outside the Accounts pane needs them: read
//! when a module appears, again when it says a row changed, and again when its name changes
//! hands. A module that is not there leaves only its part out.

use super::drive::is_valued;
use super::feeds::Feeds;
use super::reload::{reload, reload_later};
use crate::env::Env;
use crate::live::{BusSource, Daemon, Followed, follow};
use detent_model::{Absence, AccountsPane, Described, ModuleId, Sources};
use dioxus::prelude::*;

/// How many times a failed read is tried again.
const RETRIES: usize = 3;

/// Where the reading stands.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Listed {
    /// No module has answered yet.
    Looking,
    /// No module can be used, and why.
    Absent(Absence),
    /// The accounts the modules that answered list now.
    Accounts(AccountsPane),
}

impl Listed {
    fn of(sources: &Sources) -> Listed {
        match (sources.pane(), sources.absence()) {
            (Some(pane), _) => Listed::Accounts(pane),
            (None, Some(absence)) => Listed::Absent(absence),
            (None, None) => Listed::Looking,
        }
    }
}

/// The accounts the modules list, kept current.
pub(crate) fn use_listed() -> Signal<Listed> {
    let env = use_context::<Env>();
    let listed = use_signal(|| Listed::Looking);
    let sources = use_signal(Sources::default);
    let feeds = Feeds::new();
    use_future({
        let bus = env.bus.clone();
        move || watch(ModuleId::Accountd, bus.clone(), sources, listed, feeds)
    });
    use_future({
        let bus = env.bus.clone();
        move || watch(ModuleId::Inferd, bus.clone(), sources, listed, feeds)
    });
    use_future({
        let bus = env.bus.clone();
        move || watch(ModuleId::Syncd, bus.clone(), sources, listed, feeds)
    });
    listed
}

/// Takes in what `module` found and says what the modules make now.
fn hear(
    mut sources: Signal<Sources>,
    mut listed: Signal<Listed>,
    module: ModuleId,
    found: Result<Described, Absence>,
) {
    let mut now = sources.peek().clone();
    now.hear(module, found);
    listed.set(Listed::of(&now));
    sources.set(now);
}

async fn watch(
    module: ModuleId,
    bus: BusSource,
    sources: Signal<Sources>,
    listed: Signal<Listed>,
    feeds: Feeds,
) {
    let mut client = feeds.of(module).client;
    let reads = feeds.of(module).reads;
    follow(&bus, &Daemon::of(module), is_valued, |followed| {
        match followed {
            Followed::Opened(opened, described) => {
                client.set(Some(opened));
                hear(sources, listed, module, Ok(described));
            }
            Followed::Gone(absence) => {
                client.set(None);
                hear(sources, listed, module, Err(absence));
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
                        hear(sources, listed, module, Ok(described));
                    }
                });
            }
        }
    })
    .await;
}
