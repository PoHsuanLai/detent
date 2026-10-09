//! The outside agents in docket's `agents.toml`: read with what each last offered, the menus the
//! row draws, and a choice written into its entry with every other line kept.

use detent_files::{ConfigHome, DataHome, load_agents, save_agent_choice};
use detent_model::{AGENTS_OWN, model_choice, model_menu, way_choice, way_menu};
use detent_testkit::Scratch;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

const AGENTS: &str = "# mine\n[[agent]]\nprogram = \"claude-code\"\nlabel = \"Claude Code\"\nregistry = \"claude-code\"\nversion = \"1.0.0\"\nmodel = \"haiku-old\"\n\n[[agent]]\nprogram = \"mine\"\n";

const OFFERED: &str = "seen = 1\nstart = \"needs_sign_in\"\n\n[[model]]\nid = \"sonnet\"\nname = \"Sonnet 5.5\"\n\n[[model]]\nid = \"opus\"\nname = \"Opus\"\n\n[[way]]\nid = \"login\"\nname = \"Sign in with your account\"\n";

fn homes(name: &str) -> (Scratch, ConfigHome, DataHome) {
    let scratch = Scratch::under(TMP, name);
    let config = scratch.join("config");
    let data = scratch.join("data");
    std::fs::create_dir_all(config.join("docket")).expect("mkdir");
    std::fs::write(config.join("docket/agents.toml"), AGENTS).expect("agents");
    let state = data.join("docket/agents/state/claude-code");
    std::fs::create_dir_all(&state).expect("mkdir");
    std::fs::write(state.join("offered.toml"), OFFERED).expect("offered");
    (scratch, ConfigHome(config), DataHome(data))
}

fn words(entries: &[detent_model::Entry]) -> Vec<(&str, &str)> {
    entries
        .iter()
        .map(|e| (e.word.as_str(), e.name.as_str()))
        .collect()
}

#[test]
fn the_rows_and_menus_are_what_the_agent_offered_and_what_the_file_names() {
    let (_scratch, config, data) = homes("agents-menus");
    let agents = load_agents(&config, &data);
    let names: Vec<&str> = agents.rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(names, ["Claude Code", "mine"]);
    let claude = agents.get("claude-code").expect("listed");
    assert_eq!(
        words(&model_menu(claude)),
        [
            (AGENTS_OWN, "The agent\u{2019}s own choice"),
            ("sonnet", "Sonnet 5.5"),
            ("opus", "Opus"),
            ("haiku-old", "haiku-old"),
        ],
        "what is written is never hidden"
    );
    assert_eq!(
        words(&way_menu(claude)),
        [("login", "Sign in with your account")]
    );
    let mine = agents.get("mine").expect("listed");
    assert_eq!(
        model_menu(mine).len(),
        1,
        "an agent that never started offers no model"
    );
    assert!(way_menu(mine).is_empty());
}

#[test]
fn a_choice_is_written_into_its_entry_and_nothing_else_moves() {
    let (scratch, config, data) = homes("agents-write");
    save_agent_choice(&config, "claude-code", &model_choice("opus")).expect("saved");
    save_agent_choice(&config, "claude-code", &way_choice("login")).expect("saved");
    let text = std::fs::read_to_string(scratch.join("config/docket/agents.toml")).expect("file");
    assert!(text.starts_with("# mine\n"), "{text}");
    assert!(text.contains("program = \"mine\""), "{text}");
    let after = load_agents(&config, &data);
    let claude = after.get("claude-code").expect("listed");
    assert_eq!(claude.way.as_deref(), Some("login"));
    assert_eq!(
        words(&model_menu(claude)).len(),
        3,
        "opus is offered, so no extra entry"
    );
    save_agent_choice(&config, "claude-code", &model_choice(AGENTS_OWN)).expect("saved");
    let cleared = load_agents(&config, &data);
    assert_eq!(
        cleared.get("claude-code").expect("listed").model,
        docket_settings::ModelState::AgentsOwn
    );
    assert!(save_agent_choice(&config, "nobody", &model_choice("opus")).is_err());
}
