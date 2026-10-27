//! The models inferd lists, read where a place offers them to choose from.

use crate::env::Env;
use crate::live::{Followed, follow};
use detent_model::Ready;
use detent_model::picker_row_is_choice as is_choice;
use detent_system::bus::Daemon;
use dioxus::prelude::*;

/// inferd's picker rows, none while inferd is not there.
pub(super) fn use_picker() -> Signal<Option<Ready>> {
    let env = use_context::<Env>();
    let mut ready = use_signal(|| None::<Ready>);
    use_future(move || {
        let bus = env.bus.clone();
        async move {
            follow(
                &bus,
                &Daemon::inferd(),
                is_choice,
                |followed| match followed {
                    Followed::Opened(_, described) => ready.set(Some(Ready::of(&described))),
                    Followed::Gone(_) => ready.set(None),
                    Followed::Changed(..) => {}
                },
            )
            .await;
        }
    });
    ready
}
