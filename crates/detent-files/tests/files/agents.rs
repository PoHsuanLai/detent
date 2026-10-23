//! The outside agents in docket's `agents.toml`: read with what each last offered, the menus the
//! row draws, and a choice written into its entry with every other line kept.

use detent_files::{
    ConfigHome, DataHome, add_agent, installed_agents, load_agents, remove_agent, save_agent_choice,
};
use detent_model::{
    AGENTS_OWN, AddForm, model_choice, model_menu, unlisted_found, way_choice, way_menu,
};
use detent_testkit::Scratch;
use docket_settings::{AddFault, ChoiceFault};

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

fn install(data: &DataHome, id: &str, version: &str) {
    let dir = data
        .0
        .join("docket/agents/installed")
        .join(id)
        .join(version);
    std::fs::create_dir_all(&dir).expect("mkdir");
    let record = format!(
        "id = \"{id}\"\nversion = \"{version}\"\ncommand = \"bin/{id}\"\ncheck = \"declared\"\n"
    );
    std::fs::write(dir.join("installed.toml"), record).expect("record");
}

#[test]
fn installed_assistants_are_found_by_their_records_and_the_listed_ones_are_left_out() {
    let (_scratch, config, data) = homes("agents-installed");
    assert!(installed_agents(&data).is_empty(), "nothing installed yet");
    install(&data, "gemini", "0.9.0");
    install(&data, "gemini", "1.0.0");
    install(&data, "claude-code", "1.0.0");
    // A folder with no record is a half-made install, not an assistant.
    std::fs::create_dir_all(data.0.join("docket/agents/installed/half/2.0.0")).expect("mkdir");
    let found: Vec<(String, String)> = installed_agents(&data)
        .into_iter()
        .map(|f| (f.id, f.version))
        .collect();
    assert_eq!(
        found,
        [
            ("claude-code".to_owned(), "1.0.0".to_owned()),
            ("gemini".to_owned(), "1.0.0".to_owned())
        ]
    );
    let rows = load_agents(&config, &data).rows;
    let new: Vec<String> = unlisted_found(&installed_agents(&data), &rows)
        .into_iter()
        .map(|f| f.id)
        .collect();
    assert_eq!(new, ["gemini"], "claude-code is listed already");
}

#[test]
fn an_assistant_added_from_the_installed_ones_can_be_removed_again() {
    let (scratch, config, data) = homes("agents-add-remove");
    install(&data, "gemini", "1.0.0");
    let rows = load_agents(&config, &data).rows;
    let form = AddForm::open(unlisted_found(&installed_agents(&data), &rows)).expect("one new");
    let new = form.typed("My Gemini".to_owned()).request().expect("fine");
    add_agent(&config, &new).expect("added");
    let path = scratch.join("config/docket/agents.toml");
    let text = std::fs::read_to_string(&path).expect("file");
    assert!(text.starts_with("# mine\n"), "the comment stays: {text}");
    for want in [
        "route = \"login\"",
        "network = \"host\"",
        "version = \"1.0.0\"",
    ] {
        assert!(text.contains(want), "{want} in {text}");
    }
    let after = load_agents(&config, &data);
    assert_eq!(after.get("gemini").expect("listed").label, "My Gemini");
    assert_eq!(add_agent(&config, &new), Err(AddFault::Listed));

    remove_agent(&config, "gemini").expect("removed");
    let back = std::fs::read_to_string(&path).expect("file");
    assert!(!back.contains("gemini"), "{back}");
    assert!(back.starts_with("# mine\n"), "the comment stays: {back}");
    assert_eq!(load_agents(&config, &data).rows.len(), 2, "the others stay");
    assert_eq!(
        remove_agent(&config, "gemini"),
        Err(ChoiceFault::NoSuchAgent)
    );
    assert!(
        data.0.join("docket/agents/installed/gemini/1.0.0").is_dir(),
        "the installed files stay"
    );
}

#[test]
fn adding_creates_the_file_when_there_is_none() {
    let scratch = Scratch::under(TMP, "agents-add-fresh");
    let config = ConfigHome(scratch.join("config"));
    let data = DataHome(scratch.join("data"));
    install(&data, "gemini", "1.0.0");
    let form = AddForm::open(installed_agents(&data)).expect("one");
    add_agent(&config, &form.request().expect("fine")).expect("added");
    assert!(load_agents(&config, &data).get("gemini").is_some());
}
