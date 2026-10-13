//! What the pane holds of each module it follows: the client its rows are set and run through,
//! and which of its reads run. Each module has its own, so one that is slow or gone holds
//! nothing up.

use detent_model::{ModuleId, Reads};
use dioxus::prelude::*;
use ds_settings::live::LiveClient;

/// One module's client (none while it is not there) and its reads.
#[derive(Clone, Copy)]
pub(super) struct Feed {
    /// The client of the module, while it is on the bus.
    pub client: Signal<Option<LiveClient>>,
    /// Which reads of the module run.
    pub reads: Signal<Reads>,
}

fn feed() -> Feed {
    Feed {
        client: use_signal(|| None),
        reads: use_signal(Reads::default),
    }
}

/// The three modules' feeds.
#[derive(Clone, Copy)]
pub(super) struct Feeds {
    accountd: Feed,
    inferd: Feed,
    syncd: Feed,
}

impl Feeds {
    /// Makes the feeds; call it once, in the component that follows the modules.
    pub fn new() -> Feeds {
        Feeds {
            accountd: feed(),
            inferd: feed(),
            syncd: feed(),
        }
    }

    /// The feed of `module`.
    pub fn of(self, module: ModuleId) -> Feed {
        match module {
            ModuleId::Accountd => self.accountd,
            ModuleId::Inferd => self.inferd,
            ModuleId::Syncd => self.syncd,
        }
    }
}
