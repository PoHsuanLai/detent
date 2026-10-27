//! Pictures for the Wallpaper page that lists what it is given and chooses what it is told to,
//! and writes down every time it was asked: never a real folder, never the chooser.

use detent_client::Pictures;
use detent_model::{AddOutcome, Picture, PictureFrom, Work};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Book {
    listed: Mutex<Vec<Picture>>,
    unmakeable: Mutex<Vec<PathBuf>>,
    made: Mutex<Vec<PathBuf>>,
    next: Mutex<Option<AddOutcome>>,
    asked: AtomicUsize,
}

/// The test's side of the pictures.
#[derive(Debug, Clone, Default)]
pub struct Fake(Arc<Book>);

impl Fake {
    /// Pictures that list `paths`.
    pub fn with(paths: &[&str]) -> Fake {
        let fake = Fake::default();
        *fake.0.listed.lock().expect("listed") = paths
            .iter()
            .map(|path| Picture::file(PathBuf::from(path)))
            .collect();
        fake
    }

    /// A picture the person added before, listed under the added ones.
    pub fn and_added(self, path: &str) -> Fake {
        self.0
            .listed
            .lock()
            .expect("listed")
            .push(Picture::file(PathBuf::from(path)).placed(PictureFrom::Added));
        self
    }

    /// A picture that is one file named `title`, drawn small from `preview`.
    pub fn and_packaged(self, path: &str, title: &str, preview: &str) -> Fake {
        self.0
            .listed
            .lock()
            .expect("listed")
            .push(Picture::packaged(
                PathBuf::from(path),
                title.to_owned(),
                Some(PathBuf::from(preview)),
            ));
        self
    }

    /// The file whose small picture cannot be made.
    pub fn cannot_shrink(self, path: &str) -> Fake {
        self.0
            .unmakeable
            .lock()
            .expect("unmakeable")
            .push(PathBuf::from(path));
        self
    }

    /// The files a small picture was asked for, in order.
    pub fn made(&self) -> Vec<PathBuf> {
        self.0.made.lock().expect("made").clone()
    }

    /// Where the small picture of the file `source` is: the same name under the cache.
    pub fn small_of(source: &str) -> String {
        let name = source.rsplit('/').next().unwrap_or(source);
        let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
        format!("/cache/thumbnails/large/{stem}.png")
    }

    /// The picture the chooser gives the person's choice: it is kept among the added ones.
    pub fn will_choose(&self, path: &str) {
        *self.0.next.lock().expect("next") = Some(AddOutcome::Added(PathBuf::from(path)));
    }

    /// The way the next try to add a picture ends, when it does not add one.
    pub fn will_end(&self, outcome: AddOutcome) {
        *self.0.next.lock().expect("next") = Some(outcome);
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
    fn list(&self) -> Work<'_, Vec<Picture>> {
        Box::pin(async { self.0.listed.lock().expect("listed").clone() })
    }

    fn add(&self) -> Work<'_, AddOutcome> {
        Box::pin(async {
            self.0.asked.fetch_add(1, Ordering::SeqCst);
            let ended = self
                .0
                .next
                .lock()
                .expect("next")
                .take()
                .unwrap_or(AddOutcome::Closed);
            if let AddOutcome::Added(chosen) = &ended {
                self.0
                    .listed
                    .lock()
                    .expect("listed")
                    .push(Picture::file(chosen.clone()).placed(PictureFrom::Added));
            }
            ended
        })
    }

    fn thumbnail(&self, source: PathBuf) -> Work<'_, Option<PathBuf>> {
        Box::pin(async move {
            self.0.made.lock().expect("made").push(source.clone());
            if self
                .0
                .unmakeable
                .lock()
                .expect("unmakeable")
                .contains(&source)
            {
                return None;
            }
            Some(PathBuf::from(Fake::small_of(&source.to_string_lossy())))
        })
    }

    fn roots(&self) -> Vec<PathBuf> {
        Vec::new()
    }
}
