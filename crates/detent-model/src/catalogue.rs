//! Every key of every schema, grouped by page then section (design/22 section 9.3 item 1).

use crate::config_file::ConfigFile;
use crate::discovery::ProgramSchema;
use crate::entry::KeyEntry;
use crate::page::{PageSlug, sidebar_order};
use ds_settings::schema::{Exposure, KeySpec, Page};

/// The keys of one section: the ones shown, and the ones behind the section's disclosure.
#[derive(Debug, Clone, PartialEq)]
pub struct SectionModel {
    /// The section's title; empty for keys the schema gave none.
    pub title: String,
    /// Basic keys, in schema order.
    pub basic: Vec<KeyEntry>,
    /// Advanced keys, in schema order.
    pub advanced: Vec<KeyEntry>,
}

/// One page and its sections.
#[derive(Debug, Clone, PartialEq)]
pub struct PageModel {
    /// The page as the schemas name it.
    pub page: Page,
    /// Its sidebar key.
    pub slug: PageSlug,
    /// Sections with a basic key first, then the sections that only hold advanced keys; each
    /// group in the order the schemas first mention them.
    pub sections: Vec<SectionModel>,
}

/// All the keys that live in one file, from every schema that names it.
#[derive(Debug, Clone, PartialEq)]
pub struct FileSpec {
    /// The file.
    pub file: ConfigFile,
    /// Its keys, in schema order.
    pub keys: Vec<KeySpec>,
}

/// The whole app: pages in sidebar order, and the files behind them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Catalogue {
    /// Pages that have at least one key, in sidebar order.
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

    /// Every key on every page, page by page.
    pub fn entries(&self) -> impl Iterator<Item = (&PageModel, &KeyEntry)> {
        self.pages.iter().flat_map(|page| {
            page.sections
                .iter()
                .flat_map(|s| s.basic.iter().chain(&s.advanced))
                .map(move |entry| (page, entry))
        })
    }
}

fn group_files(programs: &[ProgramSchema]) -> Vec<FileSpec> {
    programs
        .iter()
        .fold(Vec::new(), |mut files: Vec<FileSpec>, p| {
            match files.iter_mut().find(|f| f.file == p.file) {
                Some(existing) => existing.keys.extend(p.keys.iter().cloned()),
                None => files.push(FileSpec {
                    file: p.file.clone(),
                    keys: p.keys.clone(),
                }),
            }
            files
        })
}

fn group_pages(entries: &[KeyEntry]) -> Vec<PageModel> {
    let mut pages: Vec<PageModel> = entries.iter().fold(Vec::new(), |mut pages, entry| {
        let slug = PageSlug::of(&entry.spec.page);
        let at = match pages.iter().position(|p| p.slug == slug) {
            Some(at) => at,
            None => {
                pages.push(PageModel {
                    page: entry.spec.page.clone(),
                    slug,
                    sections: Vec::new(),
                });
                pages.len() - 1
            }
        };
        place_in_section(&mut pages[at].sections, entry);
        pages
    });
    pages.sort_by(|a, b| sidebar_order(&a.page, &b.page));
    for page in &mut pages {
        page.sections
            .sort_by_key(|section| section.basic.is_empty());
    }
    pages
}

fn place_in_section(sections: &mut Vec<SectionModel>, entry: &KeyEntry) {
    let title = &entry.spec.section.0;
    let at = match sections.iter().position(|s| &s.title == title) {
        Some(at) => at,
        None => {
            sections.push(SectionModel {
                title: title.clone(),
                basic: Vec::new(),
                advanced: Vec::new(),
            });
            sections.len() - 1
        }
    };
    match entry.spec.exposure {
        Exposure::Basic => sections[at].basic.push(entry.clone()),
        Exposure::Advanced => sections[at].advanced.push(entry.clone()),
    }
}
