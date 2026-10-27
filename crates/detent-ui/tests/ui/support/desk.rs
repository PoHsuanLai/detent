//! A desk for the engine sheet that lists the engines it is given and writes down what it was
//! asked: never a download, never a folder chooser, never the real computer.
#![allow(dead_code)]

use detent_client::Desk;
use detent_model::{EngineRow, EngineStanding, EngineWay, InstallScope, Work};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// What the sheet asked, and the engines it is shown.
#[derive(Debug, Default)]
struct Book {
    rows: Mutex<Vec<EngineRow>>,
    installs: Mutex<Vec<(String, InstallScope)>>,
    cancels: Mutex<Vec<String>>,
    points: Mutex<Vec<(String, String)>>,
    folder: Mutex<Option<String>>,
    restarts: AtomicUsize,
}

/// The test's side of the desk.
#[derive(Debug, Clone, Default)]
pub struct Fake(Arc<Book>);

impl Fake {
    /// A desk that lists `rows`.
    pub fn with(rows: Vec<EngineRow>) -> Fake {
        let fake = Fake::default();
        *fake.0.rows.lock().expect("rows") = rows;
        fake
    }

    /// The engines the desk lists from now on.
    pub fn show(&self, rows: Vec<EngineRow>) {
        *self.0.rows.lock().expect("rows") = rows;
    }

    /// The folder the chooser gives the person's choice.
    pub fn will_choose(&self, folder: &str) {
        *self.0.folder.lock().expect("folder") = Some(folder.to_owned());
    }

    /// Every download asked for.
    pub fn installs(&self) -> Vec<(String, InstallScope)> {
        self.0.installs.lock().expect("installs").clone()
    }

    /// Every folder counted.
    pub fn points(&self) -> Vec<(String, String)> {
        self.0.points.lock().expect("points").clone()
    }

    /// Every stop asked for.
    pub fn cancels(&self) -> Vec<String> {
        self.0.cancels.lock().expect("cancels").clone()
    }

    /// How many times the assistant was asked to restart.
    pub fn restarts(&self) -> usize {
        self.0.restarts.load(Ordering::SeqCst)
    }

    /// The handle the environment takes.
    pub fn desk(&self) -> Arc<dyn Desk> {
        Arc::new(self.clone())
    }
}

/// The engines of a computer with nothing installed: llama.cpp and vLLM can be downloaded (for
/// everyone or for the person), Strata only pointed at.
pub fn nothing_installed() -> Vec<EngineRow> {
    let download = EngineWay::Download {
        scopes: vec![InstallScope::AllUsers, InstallScope::JustMe],
        size: Some(30_497_889),
    };
    vec![
        EngineRow {
            id: "llama_cpp".to_owned(),
            name: "llama.cpp".to_owned(),
            standing: EngineStanding::NotInstalled,
            way: download.clone(),
        },
        EngineRow {
            id: "vllm".to_owned(),
            name: "vLLM".to_owned(),
            standing: EngineStanding::NotInstalled,
            way: download,
        },
        EngineRow {
            id: "strata".to_owned(),
            name: "Strata".to_owned(),
            standing: EngineStanding::NotInstalled,
            way: EngineWay::Folder,
        },
    ]
}

impl Desk for Fake {
    fn rows(&self) -> Work<'_, Vec<EngineRow>> {
        Box::pin(async move { self.0.rows.lock().expect("rows").clone() })
    }

    fn install(&self, engine: &str, scope: InstallScope) -> Result<(), String> {
        self.0
            .installs
            .lock()
            .expect("installs")
            .push((engine.to_owned(), scope));
        Ok(())
    }

    fn cancel(&self, engine: &str) {
        self.0
            .cancels
            .lock()
            .expect("cancels")
            .push(engine.to_owned());
    }

    fn choose_folder(&self) -> Work<'_, Option<String>> {
        Box::pin(async move { self.0.folder.lock().expect("folder").clone() })
    }

    fn point(&self, engine: &str, folder: &str) -> Work<'_, Result<(), String>> {
        self.0
            .points
            .lock()
            .expect("points")
            .push((engine.to_owned(), folder.to_owned()));
        Box::pin(async { Ok(()) })
    }

    fn restart(&self) -> Work<'_, ()> {
        self.0.restarts.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {})
    }
}
