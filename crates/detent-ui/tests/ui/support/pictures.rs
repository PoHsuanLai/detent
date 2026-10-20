//! Pictures for the Wallpaper page that lists what it is given and chooses what it is told to,
//! and writes down every time it was asked: never a real folder, never the chooser.
#![allow(dead_code)]

use detent_model::Work;
use detent_ui::Pictures;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Book {
    listed: Mutex<Vec<PathBuf>>,
    next: Mutex<Option<PathBuf>>,
    asked: AtomicUsize,
}

/// The test's side of the pictures.
#[derive(Debug, Clone, Default)]
pub struct Fake(Arc<Book>);

impl Fake {
    /// Pictures that list `paths`.
    pub fn with(paths: &[&str]) -> Fake {
        let fake = Fake::default();
        *fake.0.listed.lock().expect("listed") = paths.iter().map(PathBuf::from).collect();
        fake
    }

    /// The picture the chooser gives the person's choice: it is kept among the listed ones.
    pub fn will_choose(&self, path: &str) {
        *self.0.next.lock().expect("next") = Some(PathBuf::from(path));
    }

    /// How many times the person was asked for a picture.
    pub fn asked(&self) -> usize {
        self.0.asked.load(Ordering::SeqCst)
    }

    /// The handle the environment takes.
    pub fn pictures(&self) -> Arc<dyn Pictures> {
        Arc::new(self.clone())
    }
}

impl Pictures for Fake {
    fn list(&self) -> Work<'_, Vec<PathBuf>> {
        Box::pin(async { self.0.listed.lock().expect("listed").clone() })
    }

    fn add(&self) -> Work<'_, Option<PathBuf>> {
        Box::pin(async {
            self.0.asked.fetch_add(1, Ordering::SeqCst);
            let chosen = self.0.next.lock().expect("next").take()?;
            self.0.listed.lock().expect("listed").push(chosen.clone());
            Some(chosen)
        })
    }
}
