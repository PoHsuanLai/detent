//! The Hot Corners sheet's helpers from memory: the manifests of the apps "installed", and a file
//! chooser that answers what it is told and writes down every time it was asked. Never a real
//! folder, never the real chooser.

use super::CornerDesk;
use crate::lock::locked;
use detent_model::{HotActionChoice, HotOpenKind, Work, actions_of_all};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Book {
    manifests: Vec<String>,
    next: Mutex<Option<String>>,
    asked: Mutex<Vec<HotOpenKind>>,
}

/// A desk that answers from what it was given.
#[derive(Debug, Clone, Default)]
pub struct FakeCorners(Arc<Book>);

impl FakeCorners {
    /// Apps whose manifests are `texts`.
    pub fn with_manifests(texts: &[&str]) -> FakeCorners {
        FakeCorners(Arc::new(Book {
            manifests: texts.iter().map(|text| (*text).to_owned()).collect(),
            ..Book::default()
        }))
    }

    /// The path the chooser gives the next time it is asked.
    pub fn will_choose(&self, path: &str) {
        *locked(&self.0.next) = Some(path.to_owned());
    }

    /// What the person was asked for, in order.
    pub fn asked(&self) -> Vec<HotOpenKind> {
        locked(&self.0.asked).clone()
    }

    /// The handle the environment takes.
    pub fn desk(&self) -> Arc<dyn CornerDesk> {
        Arc::new(self.clone())
    }
}

impl CornerDesk for FakeCorners {
    fn actions(&self) -> Work<'_, Vec<HotActionChoice>> {
        Box::pin(async { actions_of_all(self.0.manifests.iter().map(String::as_str)) })
    }

    fn choose(&self, kind: HotOpenKind) -> Work<'_, Option<String>> {
        Box::pin(async move {
            locked(&self.0.asked).push(kind);
            locked(&self.0.next).take()
        })
    }
}
