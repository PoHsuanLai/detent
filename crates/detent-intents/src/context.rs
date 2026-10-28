//! What Settings answers for the seams ds fills in other apps: it reports no particular place
//! and declines a summon (the window has no prompt field).

use docket_core::{
    ContextScope, ContextSnapshot, Here, Selection, SummonAnswer, SummonOrigin, SummonSerial,
    TextTarget, Visible, WindowPrivacy,
};
use docket_provider::{ContextSource, SummonTarget};
use porter_core::{AppName, Count};

use crate::answer::own;

/// The context of a Settings window: it is open and nowhere in particular.
#[derive(Debug, Clone)]
pub(crate) struct SettingsContext(pub AppName);

impl ContextSource for SettingsContext {
    fn snapshot(&self, _: ContextScope) -> ContextSnapshot {
        ContextSnapshot {
            app: self.0.clone(),
            window: own(&self.0, "Settings"),
            here: Here::Nowhere,
            selection: Selection::Nothing,
            visible: Visible {
                kind: None,
                items: Vec::new(),
                total: Count(0),
            },
            text_target: TextTarget::None,
            privacy: WindowPrivacy::Normal,
        }
    }
}

impl SummonTarget for SettingsContext {
    fn summon(&self, _: SummonSerial, _: SummonOrigin) -> SummonAnswer {
        SummonAnswer::Declined
    }
}
