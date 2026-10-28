//! The lending switch's world: a shipped file in a scratch folder, and a service manager that
//! writes down what it was asked and never touches the real one.

use porter_client::{UnitFailure, UnitManager};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// What the shipped file holds in these tests.
pub const SHIPPED: &str = "[Service]\nIPAddressAllow=100.64.0.0/10\n";

/// A manager that records its calls, in order, and can be told to refuse the restart.
#[derive(Debug, Clone, Default)]
pub struct Units {
    calls: Arc<Mutex<Vec<String>>>,
    refuse_restart: Arc<AtomicBool>,
}

impl Units {
    /// The calls made so far, oldest first.
    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("the log").clone()
    }

    /// From now on the restart fails.
    pub fn refuse_restart(&self) {
        self.refuse_restart.store(true, Ordering::SeqCst);
    }

    fn note(&self, call: String) {
        self.calls.lock().expect("the log").push(call);
    }
}

impl UnitManager for Units {
    async fn reload(&self) -> Result<(), UnitFailure> {
        self.note("reload".to_owned());
        Ok(())
    }

    async fn restart(&self, unit: &'static str) -> Result<(), UnitFailure> {
        self.note(format!("restart {}", unit));
        if self.refuse_restart.load(Ordering::SeqCst) {
            Err(UnitFailure("refused by the test".to_owned()))
        } else {
            Ok(())
        }
    }
}
