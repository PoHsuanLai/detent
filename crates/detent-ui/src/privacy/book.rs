//! The installed apps' desktop entries, scanned once when a pane that names programs is shown.

use crate::env::Env;
use detent_files::scan_programs;
use detent_model::ProgramBook;
use dioxus::prelude::*;

/// The book of installed apps, empty until the scan has read the folders. A program is named from
/// it only when a system entry vouches for it (`ProgramBook::look_of`).
pub(crate) fn use_program_book() -> Signal<ProgramBook> {
    let env = use_context::<Env>();
    let mut book = use_signal(ProgramBook::default);
    use_future({
        let dirs = env.application_dirs.clone();
        let rule = env.entry_rule.clone();
        move || {
            let dirs = dirs.clone();
            let rule = rule.clone();
            async move {
                if let Ok(found) =
                    tokio::task::spawn_blocking(move || scan_programs(&dirs, &rule)).await
                {
                    book.set(found);
                }
            }
        }
    });
    book
}
