//! Every key of every schema, grouped by page then section (design/22 section 9.3 item 1).

use crate::assistant_model::assistant_model_page;
use crate::bluetooth::bluetooth_page;
use crate::config_file::ConfigFile;
use crate::discovery::ProgramSchema;
use crate::displays::displays_page;
use crate::entry::KeyEntry;
use crate::page::{PageSlug, sidebar_place};
use crate::updates::software_update_page;
use crate::wifi::wifi_page;
use ds_settings::schema::{Exposure, ForeignTables, KeyPath, KeySpec, Page, PageTable};

/// The keys of one section that share an exposure, in schema order.
#[derive(Debug, Clone, PartialEq)]
pub struct SectionModel {
    /// The section's title; empty for keys the schema gave none.
    pub title: String,
    /// Its keys.
    pub keys: Vec<KeyEntry>,
}

/// One page and its sections.
#[derive(Debug, Clone, PartialEq)]
pub struct PageModel {
    /// The page as the schemas name it.
    pub page: Page,
    /// Its sidebar key.
    pub slug: PageSlug,
    /// What the sidebar and the titlebar call it: the declared title of a declared page, else the
    /// page's own name.
    pub title: String,
    /// The sections that hold a basic key, holding only those keys, in the order the schemas
    /// first mention them. A section with no basic key is not here.
    pub sections: Vec<SectionModel>,
    /// The advanced keys of the whole page, grouped by section in the order the schemas first
    /// mention them: one disclosure at the end of the page draws them.
    pub advanced: Vec<SectionModel>,
}

/// All the keys that live in one file, from every schema that names it.
#[derive(Debug, Clone, PartialEq)]
pub struct FileSpec {
    /// The file.
    pub file: ConfigFile,
    /// Its keys, in schema order.
    pub keys: Vec<KeySpec>,
    /// Tables of the file that belong to the programs, not to settings: never reported as
    /// unknown. The union over every program that shares the file.
    pub foreign: ForeignTables,
}

/// The whole app: pages in the sidebar's order (`PageTable::sidebar()`), and the files behind them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Catalogue {
    /// Pages that have at least one key, in sidebar order, and the live pages (Accounts, Privacy,
    /// Assistant Model, Software Update, Wi-Fi, Bluetooth, Displays), which need none.
    pub pages: Vec<PageModel>,
    /// Every settings file the keys read and write.
    pub files: Vec<FileSpec>,
}

impl Catalogue {
    /// The catalogue of `programs`.
    pub fn build(programs: &[ProgramSchema]) -> Catalogue {
        let entries: Vec<KeyEntry> = programs
            .iter()
            .flat_map(|p| {
                p.keys
                    .iter()
                    .map(|k| KeyEntry::new(k.clone(), p.file.clone()))
            })
            .collect();
        Catalogue {
            pages: group_pages(&entries),
            files: group_files(programs),
        }
    }

    /// The page with `slug`.
    pub fn page(&self, slug: &PageSlug) -> Option<&PageModel> {
        self.pages.iter().find(|page| &page.slug == slug)
    }

    /// The file's spec.
    pub fn file(&self, file: &ConfigFile) -> Option<&FileSpec> {
        self.files.iter().find(|spec| &spec.file == file)
    }

    /// The key at `path`, when exactly one key of any schema has that path (two programs that
    /// name the same path make it ambiguous, and neither is returned).
    pub fn entry(&self, path: &KeyPath) -> Option<&KeyEntry> {
        let mut found = self.entries().filter(|(_, e)| &e.spec.path == path);
        match (found.next(), found.next()) {
            (Some((_, entry)), None) => Some(entry),
            _ => None,
        }
    }

    /// Every key on every page, page by page.
    pub fn entries(&self) -> impl Iterator<Item = (&PageModel, &KeyEntry)> {
        self.pages.iter().flat_map(|page| {
            page.sections
                .iter()
                .chain(&page.advanced)
                .flat_map(|section| section.keys.iter())
                .map(move |entry| (page, entry))
        })
    }
}

fn group_files(programs: &[ProgramSchema]) -> Vec<FileSpec> {
    programs
        .iter()
        .fold(Vec::new(), |mut files: Vec<FileSpec>, p| {
            match files.iter_mut().find(|f| f.file == p.file) {
                Some(existing) => {
                    existing.keys.extend(p.keys.iter().cloned());
                    existing.foreign = union(&existing.foreign, &p.foreign);
                }
                None => files.push(FileSpec {
                    file: p.file.clone(),
                    keys: p.keys.clone(),
                    foreign: p.foreign.clone(),
                }),
            }
            files
        })
}

/// The name a person reads for `page`: a declared page's title (Wi-Fi, Software Update), else
/// the page's own label.
fn title_of(table: &PageTable, page: &Page) -> String {
    match page {
        Page::App(_) => table.title_of(page).0,
        fixed => fixed.label().to_owned(),
    }
}

fn group_pages(entries: &[KeyEntry]) -> Vec<PageModel> {
    let table = PageTable::built_in();
    let mut pages: Vec<PageModel> = entries.iter().fold(Vec::new(), |mut pages, entry| {
        let slug = PageSlug::of(&entry.spec.page);
        let at = match pages.iter().position(|p| p.slug == slug) {
            Some(at) => at,
            None => {
                pages.push(PageModel {
                    page: entry.spec.page.clone(),
                    title: title_of(&table, &entry.spec.page),
                    slug,
                    sections: Vec::new(),
                    advanced: Vec::new(),
                });
                pages.len() - 1
            }
        };
        let page = &mut pages[at];
        match entry.spec.exposure {
            Exposure::Basic => place_in_section(&mut page.sections, entry),
            Exposure::Advanced => place_in_section(&mut page.advanced, entry),
        }
        pages
    });
    // Accounts is accountd's live pane, Privacy the compositor's list, Assistant Model inferd's
    // picker row, Software Update the update service's, and Wi-Fi, Bluetooth and Displays the
    // computer's own (no file names any of them), so all seven pages are always there.
    for live in [
        Page::Accounts,
        Page::Privacy,
        assistant_model_page(),
        software_update_page(),
        wifi_page(),
        bluetooth_page(),
        displays_page(),
    ] {
        if !pages.iter().any(|page| page.page == live) {
            pages.push(PageModel {
                slug: PageSlug::of(&live),
                title: title_of(&table, &live),
                page: live,
                sections: Vec::new(),
                advanced: Vec::new(),
            });
        }
    }
    // Quire's table says which pages the sidebar lists and in what order.
    let mut placed: Vec<_> = pages
        .into_iter()
        .filter_map(|model| sidebar_place(&table, &model.page).map(|place| (place, model)))
        .collect();
    placed.sort_by(|a, b| a.0.cmp(&b.0));
    placed.into_iter().map(|(_, model)| model).collect()
}

fn place_in_section(sections: &mut Vec<SectionModel>, entry: &KeyEntry) {
    let title = &entry.spec.section.0;
    match sections.iter_mut().find(|s| &s.title == title) {
        Some(section) => section.keys.push(entry.clone()),
        None => sections.push(SectionModel {
            title: title.clone(),
            keys: vec![entry.clone()],
        }),
    }
}

/// `a` then the tables of `b` it does not already hold.
fn union(a: &ForeignTables, b: &ForeignTables) -> ForeignTables {
    let mut tables = a.0.clone();
    tables.extend(b.0.iter().filter(|t| !a.0.contains(t)).cloned());
    ForeignTables(tables)
}
