//! "Programs for local models": inferd's text rows, which name the program of an engine. A path
//! is set through the module; the module's refusal, in its own words, stays with the row. The
//! steps are `detent-model`'s `programs_step`.

use crate::follow::{Followed, Slot, follow_with};
use crate::live::Live;
use crate::pane::{Machine, Pending, pane};
use detent_model::{ProgramsEffect, ProgramsInput, ProgramsPane, program_rows, programs_step};
use detent_system::bus::{BusSource, Daemon};
use detent_system::picker::PickerLink;
use ds_settings::schema::KeyKind;
use std::sync::Arc;

/// The rows of inferd's module whose value the program rows read: its text rows.
fn is_text(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Text)
}

/// The programs' handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct EnginePrograms {
    bus: BusSource,
    picker: Arc<dyn PickerLink>,
    module: Slot,
    live: Live<ProgramsPane>,
}

pane!(EnginePrograms, ProgramsPane, follow);

impl EnginePrograms {
    pub(crate) fn over(bus: BusSource, picker: Arc<dyn PickerLink>) -> EnginePrograms {
        EnginePrograms {
            bus,
            picker,
            module: Slot::default(),
            live: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: ProgramsInput) -> Pending {
        self.dispatch(input)
    }

    /// Follows inferd's module: its text rows with their values, and each value it puts in force.
    async fn follow(&self) {
        follow_with(
            &self.bus,
            &Daemon::inferd(),
            is_text,
            async |followed| match followed {
                Followed::Opened(opened, described) => {
                    let rows = program_rows(&described);
                    self.dispatch(ProgramsInput::Found(Ok(rows))).await;
                    self.module.put(Some(opened));
                }
                Followed::Gone(absence) => {
                    self.dispatch(ProgramsInput::Found(Err(absence))).await;
                    self.module.put(None);
                }
                Followed::Changed(path, toml::Value::String(word)) => {
                    self.dispatch(ProgramsInput::Changed(path, word)).await;
                }
                Followed::Changed(..) => {}
            },
        )
        .await;
    }
}

impl Machine for EnginePrograms {
    type Input = ProgramsInput;
    type Effect = ProgramsEffect;

    fn step(state: ProgramsPane, input: ProgramsInput) -> (ProgramsPane, Vec<ProgramsEffect>) {
        programs_step(state, input)
    }

    async fn run(&self, effect: ProgramsEffect) -> Vec<ProgramsInput> {
        let ProgramsEffect::Set(path, value) = effect;
        let Some(opened) = self.module.get() else {
            return Vec::new();
        };
        let ended = match self.picker.set(&opened, &path, &value).await {
            Ok(()) => ProgramsInput::Applied(path, value),
            Err(why) => ProgramsInput::Refused(path, why),
        };
        vec![ended]
    }
}
