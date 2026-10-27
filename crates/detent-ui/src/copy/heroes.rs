//! The one sentence under the name on a page that opens with a hero.

use detent_model::Hero;

/// What the page of `hero` is for.
pub(crate) fn line(hero: Hero) -> &'static str {
    match hero {
        Hero::Privacy => "Choose what each app can reach on this computer.",
        Hero::Accounts => "Sign in to your accounts and choose what syncs.",
        Hero::Intelligence => "Pick the assistant, its models and what it remembers.",
    }
}

#[cfg(test)]
mod tests {
    use super::line;
    use detent_model::Hero;

    #[test]
    fn each_hero_has_one_short_sentence() {
        for hero in [Hero::Privacy, Hero::Accounts, Hero::Intelligence] {
            let words = line(hero);
            assert!(
                words.ends_with('.') && words.len() < 70,
                "{hero:?}: {words}"
            );
        }
    }
}
