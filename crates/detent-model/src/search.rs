//! Searching the schemas' own words (design/22 section 9.3 item 3: no hand-kept table).

use crate::catalogue::Catalogue;
use crate::page::PageSlug;
use ds_settings::schema::KeyPath;

/// How well a key matches, best first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    /// The whole query is in the key's label.
    Label,
    /// The whole query is in its help line.
    Help,
    /// Every word of the query is somewhere in its label, help or section.
    Words,
}

/// One key a query found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// The page the key is on.
    pub page: PageSlug,
    /// The key.
    pub path: KeyPath,
    /// How well it matched.
    pub rank: Rank,
}

/// What a search box says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Search {
    /// The box is empty.
    Idle,
    /// The keys found, best first; empty when nothing matches.
    Results(Vec<Hit>),
}

impl Search {
    /// The best hit, if any.
    pub fn first(&self) -> Option<&Hit> {
        match self {
            Search::Idle => None,
            Search::Results(hits) => hits.first(),
        }
    }

    /// The hits, none while idle.
    pub fn hits(&self) -> &[Hit] {
        match self {
            Search::Idle => &[],
            Search::Results(hits) => hits,
        }
    }
}

fn rank_of(phrase: &str, words: &[&str], label: &str, help: &str, section: &str) -> Option<Rank> {
    let (label, help, section) = (
        label.to_lowercase(),
        help.to_lowercase(),
        section.to_lowercase(),
    );
    if label.contains(phrase) {
        return Some(Rank::Label);
    }
    if help.contains(phrase) {
        return Some(Rank::Help);
    }
    let everywhere = format!("{label} {help} {section}");
    words
        .iter()
        .all(|word| everywhere.contains(word))
        .then_some(Rank::Words)
}

/// The keys of `catalogue` that `query` finds, best rank first, then in page and schema order.
pub fn search(catalogue: &Catalogue, query: &str) -> Search {
    let phrase = query.trim().to_lowercase();
    if phrase.is_empty() {
        return Search::Idle;
    }
    let words: Vec<&str> = phrase.split_whitespace().collect();
    let mut hits: Vec<Hit> = catalogue
        .entries()
        .filter_map(|(page, entry)| {
            let spec = &entry.spec;
            rank_of(
                &phrase,
                &words,
                &spec.label.0,
                &spec.help.0,
                &spec.section.0,
            )
            .map(|rank| Hit {
                page: page.slug.clone(),
                path: spec.path.clone(),
                rank,
            })
        })
        .collect();
    hits.sort_by_key(|hit| hit.rank);
    Search::Results(hits)
}
