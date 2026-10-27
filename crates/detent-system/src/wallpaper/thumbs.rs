//! The small pictures of the Wallpaper page, kept where every program on the desktop looks for them.
//!
//! A tile shows its wallpaper at 72 logical pixels; decoding the original (a 5120 by 2880 picture is
//! sixty megabytes of pixels) for that would stall the page. So each tile draws a small copy from the
//! freedesktop thumbnail cache, `<cache home>/thumbnails/large/<md5 of the file's address>.png`, the
//! cache a file manager and sill's launcher share (`anyview_platform::portable::FreedesktopThumbnails`).
//! A copy made for this very version of the file is reused; otherwise the original is decoded once,
//! scaled to 256 pixels on its longer side (twice a tile at the display's usual scale) and stored.

use anyview_core::{
    ByteLen, FileHead, FilePath, Input, Peek, PeekBudget, PixelArea, Resize, SniffStep, sniff,
};
use anyview_fs::OnDisk;
use anyview_image::{ImageError, RasterPeek, resized};
use anyview_platform::portable::FreedesktopThumbnails;
use anyview_platform::{Env, PlatformError, ThumbPixels, ThumbSize, ThumbnailCache, file_uri};
use detent_files::CacheHome;
use md5::{Digest, Md5};
use std::path::{Path, PathBuf};

/// The size kept: 256 pixels on the longer side.
const SIZE: ThumbSize = ThumbSize::Large;

/// The folder of that size under `thumbnails`, as the specification names it.
const SIZE_FOLDER: &str = "large";

/// The biggest original read, so a stray giant file cannot fill memory.
const MOST_BYTES: ByteLen = ByteLen(128 * 1024 * 1024);

/// What decoding may spend: pixels enough for the scaled copy, which is cut from a reduced decode.
const BUDGET: PeekBudget = PeekBudget::DEFAULT.with_pixels(PixelArea(512 * 512));

/// Why a wallpaper has no small picture.
#[derive(Debug, thiserror::Error)]
pub enum ThumbFault {
    /// The path is not an absolute one, so the cache has no name for it.
    #[error("the picture's path is not absolute")]
    NotAbsolute,
    /// The file could not be read.
    #[error("the picture could not be read: {0}")]
    Unreadable(#[source] std::io::Error),
    /// The file is larger than a wallpaper is read at.
    #[error("the picture is too large to read")]
    TooLarge,
    /// The file is not a picture of a kind the viewer reads.
    #[error("the file is not a picture")]
    NotAPicture,
    /// The picture could not be decoded.
    #[error("the picture could not be decoded: {0}")]
    Decode(#[from] ImageError),
    /// The scaled picture holds no pixels.
    #[error("the picture is empty")]
    Empty,
    /// The cache refused the small picture.
    #[error("the small picture could not be kept: {0}")]
    Store(#[from] PlatformError),
}

/// The thumbnail cache of the person's cache home.
#[derive(Debug, Clone)]
pub struct Thumbnails {
    cache: FreedesktopThumbnails,
    root: PathBuf,
}

impl Thumbnails {
    /// The cache under `home`: `<home>/thumbnails`.
    pub fn under(home: &CacheHome) -> Thumbnails {
        // The platform crate reads only the cache directory of its `Env`; the rest is isolated and
        // refuses to reach a bus or start a program.
        let mut env = Env::isolated(&home.0);
        env.dirs.cache = home.0.clone();
        Thumbnails {
            cache: FreedesktopThumbnails::new(&env),
            root: home.0.join("thumbnails"),
        }
    }

    /// The folder the small pictures are drawn from, which the window may read.
    pub fn folder(&self) -> PathBuf {
        self.root.join(SIZE_FOLDER)
    }

    /// The small picture of the file at `source`: the one the cache holds for this version of it, or
    /// one made now and kept. Blocking, for a worker thread.
    pub fn of(&self, source: &Path) -> Result<PathBuf, ThumbFault> {
        let file = FilePath::new(source).map_err(|_| ThumbFault::NotAbsolute)?;
        let input = file.on_disk();
        let stamp = input.stamp();
        let kept = self.cache.lookup(&file, &stamp, SIZE);
        if matches!(kept, Ok(Some(_))) {
            return Ok(self.file_of(&file));
        }
        let pixels = small_picture(&input)?;
        self.cache.store(&file, &stamp, SIZE, &pixels)?;
        Ok(self.file_of(&file))
    }

    /// Where the small picture of `file` is: the MD5 of the file's address, as the specification
    /// names it (the name the cache's writer gives).
    fn file_of(&self, file: &FilePath) -> PathBuf {
        let name: String = Md5::digest(file_uri(file).as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        self.folder().join(format!("{name}.png"))
    }
}

/// The original of `input`, decoded and scaled to the size kept.
fn small_picture(input: &Input) -> Result<ThumbPixels, ThumbFault> {
    if input.bytes().len().0 > MOST_BYTES.0 {
        return Err(ThumbFault::TooLarge);
    }
    let head = input
        .bytes()
        .read_range(0..FileHead::MAX.0)
        .map_err(ThumbFault::Unreadable)?;
    let SniffStep::Done(sniffed) = sniff(&FileHead::new(&head), input.name()) else {
        return Err(ThumbFault::NotAPicture);
    };
    let peeked = RasterPeek::peek(input, &sniffed, &BUDGET)?;
    let small = resized(&peeked.picture, Resize::LongEdge(SIZE.edge()));
    ThumbPixels::new(small.size(), small.into_bytes()).ok_or(ThumbFault::Empty)
}
