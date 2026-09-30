//! A scratch directory beside the build output, never in the person's own home or `/tmp`.

use std::path::{Path, PathBuf};

/// A fresh, empty directory that is removed when it drops.
#[derive(Debug)]
pub struct Scratch(PathBuf);

impl Scratch {
    /// A new empty directory `base/<name>`; `base` is the calling test's
    /// `env!("CARGO_TARGET_TMPDIR")`, and `name` is unique per test.
    ///
    /// # Panics
    /// If the directory cannot be created: a test that cannot get a scratch directory has
    /// nothing to run in.
    pub fn under(base: &str, name: &str) -> Scratch {
        let dir = Path::new(base).join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("scratch {}: {e}", dir.display()));
        Scratch(dir)
    }

    /// The directory.
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// `relative` inside the directory.
    pub fn join(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
