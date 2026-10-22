//! The Hot Corners sheet's helpers for a test: the manifests of the apps "installed", and a
//! file chooser that answers what it is told and writes down every time it was asked. Never a
//! real folder, never the real chooser.
#![allow(dead_code)]

use detent_model::{HotActionChoice, HotOpenKind, Work, actions_of_all};
use detent_ui::CornerDesk;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Book {
    manifests: Vec<String>,
    next: Mutex<Option<String>>,
    asked: Mutex<Vec<HotOpenKind>>,
}

/// The test's side of the sheet's helpers.
#[derive(Debug, Clone, Default)]
pub struct Fake(Arc<Book>);

impl Fake {
    /// Apps whose manifests are `texts`.
    pub fn with_manifests(texts: &[&str]) -> Fake {
        Fake(Arc::new(Book {
            manifests: texts.iter().map(|text| (*text).to_owned()).collect(),
            ..Book::default()
        }))
    }

    /// The path the chooser gives the next time it is asked.
    pub fn will_choose(&self, path: &str) {
        *self.0.next.lock().expect("next") = Some(path.to_owned());
    }

    /// What the person was asked for, in order.
    pub fn asked(&self) -> Vec<HotOpenKind> {
        self.0.asked.lock().expect("asked").clone()
    }

    /// The handle the environment takes.
    pub fn desk(&self) -> Arc<dyn CornerDesk> {
        Arc::new(self.clone())
    }
}

impl CornerDesk for Fake {
    fn actions(&self) -> Work<'_, Vec<HotActionChoice>> {
        Box::pin(async { actions_of_all(self.0.manifests.iter().map(String::as_str)) })
    }

    fn choose(&self, kind: HotOpenKind) -> Work<'_, Option<String>> {
        Box::pin(async move {
            self.0.asked.lock().expect("asked").push(kind);
            self.0.next.lock().expect("next").take()
        })
    }
}
