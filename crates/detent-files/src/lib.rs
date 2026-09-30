//! Reading and writing the settings files the schemas describe (design/22 section 2): a lenient
//! read in which a bad value costs only its own key, unknown keys reported and never kept, and
//! an atomic temp-and-rename write. The programs' own directory watches apply what is written;
//! nothing here talks to a program.

mod error;
mod load;
mod save;
mod unknown;
mod values;

pub use error::FilesError;
pub use load::load;
pub use save::save;
pub use values::{ConfigHome, FileValues};
