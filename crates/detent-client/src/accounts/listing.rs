//! The accounts the modules list, read where a page outside the Accounts pane needs them: read
//! when a module appears, again when it says a row changed, and again when its name changes
//! hands. A module that is not there leaves only its part out.

use super::feeds::Feeds;
use crate::follow::{Followed, follow_with};
use crate::live::Live;
use crate::pane::{Pane, Pending};
use detent_model::{Absence, AccountsPane, Described, ModuleId, Sources, accounts_row_is_valued};
use detent_system::bus::Daemon;
use detent_system::modules::ModulesLink;
use std::sync::{Arc, Mutex, PoisonError};

/// How many times a failed read is tried again.
const RETRIES: usize = 3;

/// Where the reading stands.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Listed {
    /// No module has answered yet.
    #[default]
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
#[derive(Debug, Clone)]
pub struct Listing {
    modules: Arc<dyn ModulesLink>,
    feeds: Feeds,
    sources: Arc<Mutex<Sources>>,
    live: Live<Listed>,
}

impl Listing {
    pub(crate) fn over(modules: Arc<dyn ModulesLink>) -> Listing {
        Listing {
            modules,
            feeds: Feeds::default(),
            sources: Arc::default(),
            live: Live::default(),
        }
    }

    /// Takes in what `module` found and says what the modules make now.
    fn hear(&self, module: ModuleId, found: Result<Described, Absence>) {
        let mut sources = self.sources.lock().unwrap_or_else(PoisonError::into_inner);
        sources.hear(module, found);
        let now = Listed::of(&sources);
        drop(sources);
        self.live.update(|_| now);
    }

    async fn follow_all(&self) {
        tokio::join!(
            self.follow(ModuleId::Accountd),
            self.follow(ModuleId::Inferd),
            self.follow(ModuleId::Syncd),
        );
    }

    async fn follow(&self, module: ModuleId) {
        let feed = self.feeds.of(module);
        follow_with(
            &*self.modules,
            &Daemon::of(module),
            accounts_row_is_valued,
            async |followed| match followed {
                Followed::Opened(opened, described) => {
                    feed.module.put(Some(opened));
                    self.hear(module, Ok(described));
                }
                Followed::Gone(absence) => {
                    feed.module.put(None);
                    self.hear(module, Err(absence));
                }
                Followed::Changed(..) => {
                    let Some(opened) = feed.module.get() else {
                        return;
                    };
                    let mut found = feed.reload(&opened).await;
                    // A failed read keeps the list and is tried again, a few times.
                    for _ in 0..RETRIES {
                        if !matches!(found, Some(Err(_))) {
                            break;
                        }
                        found = feed.reload_later(&opened).await;
                    }
                    if let Some(Ok(described)) = found {
                        self.hear(module, Ok(described));
                    }
                }
            },
        )
        .await;
    }
}

impl Pane for Listing {
    type State = Listed;

    fn live(&self) -> &Live<Listed> {
        &self.live
    }

    fn watch(&self) -> Pending {
        let me = self.clone();
        Box::pin(async move { me.follow_all().await })
    }
}
