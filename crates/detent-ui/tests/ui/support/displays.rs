//! The compositor's displays from memory: nothing here reaches a compositor. It writes down what
//! the page asked and moves its own standing the way the real one would.

use detent_model::{
    Absence, DisplayChange, DisplayEdit, DisplayMode, DisplaysStatus, ModePreference, Work, mode_at,
};
use detent_ui::{DisplaysLink, Ticker, Ticks};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Shared {
    standing: Mutex<Result<DisplaysStatus, Absence>>,
    asked: Mutex<Vec<DisplayChange>>,
    refusal: Mutex<Option<String>>,
    listeners: Mutex<Vec<Ticker>>,
}

/// Displays that answer from what they were given.
#[derive(Debug, Clone)]
pub struct Displays(Arc<Shared>);

impl Displays {
    /// Displays in this standing.
    pub fn answering(status: DisplaysStatus) -> Displays {
        Displays::standing(Ok(status))
    }

    /// No compositor, for this reason.
    pub fn missing(why: Absence) -> Displays {
        Displays::standing(Err(why))
    }

    fn standing(standing: Result<DisplaysStatus, Absence>) -> Displays {
        Displays(Arc::new(Shared {
            standing: Mutex::new(standing),
            asked: Mutex::default(),
            refusal: Mutex::default(),
            listeners: Mutex::default(),
        }))
    }

    /// The next changes are refused, in these words.
    pub fn refuses(&self, words: &str) {
        *self.0.refusal.lock().expect("lock") = Some(words.to_owned());
    }

    /// What was asked of it, in order.
    pub fn asked(&self) -> Vec<DisplayChange> {
        self.0.asked.lock().expect("lock").clone()
    }

    fn tick(&self) {
        self.0
            .listeners
            .lock()
            .expect("lock")
            .retain(|listener| listener.tick().is_ok());
    }
}

impl DisplaysLink for Displays {
    fn read(&self) -> Work<'_, Result<DisplaysStatus, Absence>> {
        let standing = self.0.standing.lock().expect("lock").clone();
        Box::pin(std::future::ready(standing))
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Absence>> {
        let (listener, ticks) = Ticks::channel();
        self.0.listeners.lock().expect("lock").push(listener);
        Box::pin(std::future::ready(Ok(ticks)))
    }

    fn apply(&self, change: DisplayChange) -> Work<'_, Result<(), String>> {
        self.0.asked.lock().expect("lock").push(change.clone());
        if let Some(words) = self.0.refusal.lock().expect("lock").clone() {
            return Box::pin(std::future::ready(Err(words)));
        }
        if let Ok(status) = &mut *self.0.standing.lock().expect("lock") {
            for display in &mut status.displays {
                if display.id != change.output {
                    continue;
                }
                match &change.edit {
                    DisplayEdit::Scale(scale) => display.scale = *scale,
                    DisplayEdit::Rotate(rotation) => display.rotation = *rotation,
                    DisplayEdit::Move(at) => display.position = *at,
                    DisplayEdit::Size(size) => {
                        display.current = mode_at(&display.modes, display.current, *size);
                    }
                    DisplayEdit::Refresh(rate) => {
                        display.current = display.current.map(|mode| DisplayMode {
                            refresh: *rate,
                            preferred: ModePreference::No,
                            ..mode
                        });
                    }
                }
            }
        }
        self.tick();
        Box::pin(std::future::ready(Ok(())))
    }
}
