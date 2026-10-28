//! The one sentence under the name on a page that opens with a hero.

use detent_model::PageKind;

/// What the page of `kind` is for; empty for a page that does not open with a hero.
pub(crate) fn line(kind: PageKind) -> &'static str {
    match kind {
        PageKind::Privacy => "Choose what each app can reach on this computer.",
        PageKind::Accounts => "Sign in to your accounts and choose what syncs.",
        PageKind::Intelligence => "Pick the assistant, its models and what it remembers.",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::line;
    use detent_model::PageKind;

    #[test]
    fn each_hero_has_one_short_sentence() {
        for kind in [
            PageKind::Privacy,
            PageKind::Accounts,
            PageKind::Intelligence,
        ] {
            let words = line(kind);
            assert!(
                words.ends_with('.') && words.len() < 70,
                "{kind:?}: {words}"
            );
        }
    }
}
