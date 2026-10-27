//! The assistants pane's words.

/// The menu line for "the agent's own choice": no model is written.
pub(crate) fn agents_own() -> &'static str {
    "The agent\u{2019}s own choice"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_agents_own_choice_is_named() {
        assert_eq!(agents_own(), "The agent\u{2019}s own choice");
    }
}
