//! The models inferd lists, read where a place offers them to choose from.

use crate::follow::{Followed, follow_with};
use crate::live::Live;
use crate::pane::{Pane, Pending};
use detent_model::{Ready, picker_row_is_choice as is_choice};
use detent_system::bus::{BusSource, Daemon};

/// inferd's picker rows, none while inferd is not there.
#[derive(Debug, Clone)]
pub struct Models {
    bus: BusSource,
    live: Live<Option<Ready>>,
}

impl Models {
    pub(crate) fn over(bus: BusSource) -> Models {
        Models {
            bus,
            live: Live::default(),
        }
    }

    async fn follow(&self) {
        follow_with(
            &self.bus,
            &Daemon::inferd(),
            is_choice,
            async |followed| match followed {
                Followed::Opened(_, described) => {
                    self.live.update(|_| Some(Ready::of(&described)));
                }
                Followed::Gone(_) => self.live.update(|_| None),
                Followed::Changed(..) => {}
            },
        )
        .await;
    }
}

impl Pane for Models {
    type State = Option<Ready>;

    fn live(&self) -> &Live<Option<Ready>> {
        &self.live
    }

    fn watch(&self) -> Pending {
        let me = self.clone();
        Box::pin(async move { me.follow().await })
    }
}
