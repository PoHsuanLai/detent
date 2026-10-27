//! The rows, words and programs the fake inferd lists.

use super::*;

/// The tiers each fake kind lists.
pub(super) const ROWS: &[(&str, &str, &str)] = &[
    ("llm", "Language", "fast"),
    ("llm", "Language", "balanced"),
    ("llm", "Language", "best"),
    ("computer_use", "Computer use", "balanced"),
];

/// The rows of an inferd that has moved to slots (`ai.model.<slot>.<tier>`): the slot's key
/// segment and the display name its section carries (`Models / Text`).
pub(super) const SLOT_ROWS: &[(&str, &str, &str)] = &[
    ("text", "Text", "fast"),
    ("text", "Text", "balanced"),
    ("text", "Text", "best"),
    ("voice_in", "Speech to text", "fast"),
    ("computer_use", "Computer use", "balanced"),
];

/// The display word of a tier's key segment.
pub(super) fn tier_word(tier: &str) -> &'static str {
    match tier {
        "fast" => "Fast",
        "balanced" => "Balanced",
        _ => "Demanding",
    }
}

/// The words a slot's rows list, and the group each model sits in (slots layout only).
pub(super) const WORDS: [(&str, &str, Option<&str>); 6] = [
    ("", "Catalogue default", None),
    ("auto", "Automatic", None),
    ("local/qwen", "qwen", Some("On this computer")),
    ("anthropic/claude", "claude", Some("Anthropic")),
    ("google/gemini", "gemini", Some("Google")),
    ("local/holo", "holo", Some("On this computer")),
];

/// Where a model of the slots layout runs: the two local ones on this computer through
/// llama.cpp, as porter's inferd says them; the hosted ones in their company's cloud.
fn place_of(word: &str) -> Option<ChoicePlace> {
    let engine = |name: &str| ChoicePlace::ThisComputer {
        engine: Some(EngineName(name.to_owned())),
    };
    let cloud = |company: &str| ChoicePlace::Cloud {
        company: CompanyName(company.to_owned()),
    };
    match word {
        "local/qwen" | "local/holo" => Some(engine("llama.cpp")),
        "anthropic/claude" => Some(cloud("Anthropic")),
        "google/gemini" => Some(cloud("Google")),
        _ => None,
    }
}

/// The places of `words`, for the words that have one.
pub(super) fn places_of(words: &[(&str, &str, Option<&str>)]) -> BTreeMap<ChoiceWord, ChoicePlace> {
    words
        .iter()
        .filter_map(|(word, _, _)| Some((ChoiceWord((*word).to_owned()), place_of(word)?)))
        .collect()
}

/// The engine program rows the fake lists when asked to, with their labels.
pub(super) const PROGRAMS: &[(&str, &str)] = &[
    ("engines.vllm_python", "vLLM's Python"),
    ("engines.llama_server", "llama-server"),
];

/// The key of the text slot's Automatic list.
pub(super) const POOL_KEY: &str = "ai.auto.pool.text";
