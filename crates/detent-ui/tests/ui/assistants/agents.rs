//! Outside assistants: listed by their plain names, added from a sheet and removed after a question.

use super::*;

#[test]
fn an_outside_assistant_is_listed_by_its_plain_name_and_its_model_is_chosen_in_its_entry() {
    let sandbox = Sandbox::new("coding-assistants");
    sandbox.write("docket/agents.toml", AGENTS);
    sandbox.write_data("docket/agents/state/claude-code/offered.toml", OFFERED);
    let mut harness = sandbox.harness(intelligence());
    let group = "[data-assistants=\"coding\"]";
    settle(&mut harness, "the group", |h| h.count(group) == 1);
    let text = harness.text_of(group).unwrap_or_default();
    for want in [
        "Coding assistants",
        "Claude Code",
        "Signed in",
        "The agent\u{2019}s own choice",
    ] {
        assert!(text.contains(want), "{want} in {text}");
    }
    for protocol in ["ACP", "MCP", "protocol"] {
        assert!(!text.contains(protocol), "{protocol} in {text}");
    }
    assert_eq!(sandbox.read("docket/agents.toml").as_deref(), Some(AGENTS));

    let button = format!("{group} .ds-popup .ds-button");
    scroll_into_view(&mut harness, &button);
    click(&mut harness, &button);
    let item = ".ds-menu-item:nth-child(3) .ds-menu-label";
    let at = harness.centre(item).expect("the menu is open");
    harness.send(Input::pointer_move(at));
    click(&mut harness, item);
    let written = sandbox.read("docket/agents.toml").expect("the file");
    assert_eq!(
        written,
        format!("{AGENTS}model = \"opus\"\n"),
        "the model lands in the entry and the comment stays"
    );
}

const INSTALLED: &str =
    "id = \"gemini\"\nversion = \"1.0.0\"\ncommand = \"bin/gemini\"\ncheck = \"declared\"\n";
const CODING_GROUP: &str = "[data-assistants=\"coding\"]";
const SHEET: &str = "[data-system=\"add-coding-assistant\"]";

#[test]
fn an_installed_assistant_is_added_from_a_sheet_that_says_it_uses_the_internet() {
    let sandbox = Sandbox::new("coding-assistants-add");
    sandbox.write("docket/agents.toml", AGENTS);
    sandbox.write_data(
        "docket/agents/installed/gemini/1.0.0/installed.toml",
        INSTALLED,
    );
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the group", |h| h.count(CODING_GROUP) == 1);
    let open = format!("{CODING_GROUP} button[aria-label=\"Add a coding assistant\"]");
    scroll_into_view(&mut harness, &open);
    click_when_still(&mut harness, &open);
    settle(&mut harness, "the sheet", |h| h.count(SHEET) == 1);
    let text = harness.text_of(SHEET).unwrap_or_default();
    for want in [
        "Add a coding assistant",
        "It signs in to its own service and uses the internet.",
    ] {
        assert!(text.contains(want), "{want} in {text}");
    }
    for developer in ["ACP", "registry", "agents.toml", "npm"] {
        assert!(!text.contains(developer), "{developer} in {text}");
    }
    assert_eq!(sandbox.read("docket/agents.toml").as_deref(), Some(AGENTS));

    click(&mut harness, &format!("{SHEET} [aria-label=\"Add\"]"));
    settle(&mut harness, "the sheet closing", |h| h.count(SHEET) == 0);
    let written = sandbox.read("docket/agents.toml").expect("the file");
    for want in [
        "program = \"gemini\"",
        "label = \"Gemini\"",
        "route = \"login\"",
        "network = \"host\"",
    ] {
        assert!(written.contains(want), "{want} in {written}");
    }
    assert!(written.starts_with(AGENTS), "{written}");
}

#[test]
fn an_added_assistant_is_removed_after_a_question_and_a_cancelled_one_stays() {
    let sandbox = Sandbox::new("coding-assistants-remove");
    sandbox.write("docket/agents.toml", AGENTS);
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the group", |h| h.count(CODING_GROUP) == 1);
    let remove = format!("{CODING_GROUP} [aria-label=\"Remove Claude Code\"]");
    scroll_into_view(&mut harness, &remove);
    click_when_still(&mut harness, &remove);
    settle(&mut harness, "the question", |h| h.count(".ds-alert") == 1);
    let text = harness.text_of(".ds-alert").unwrap_or_default();
    for want in [
        "Remove Claude Code?",
        "It won\u{2019}t be offered any more. Its sign-in stays.",
    ] {
        assert!(text.contains(want), "{want} in {text}");
    }
    click(
        &mut harness,
        ".ds-alert .ds-button:not([data-role=\"destructive\"])",
    );
    settle(&mut harness, "the question closing", |h| {
        h.count(".ds-alert") == 0
    });
    assert_eq!(
        sandbox.read("docket/agents.toml").as_deref(),
        Some(AGENTS),
        "cancel changes nothing"
    );

    click_when_still(&mut harness, &remove);
    settle(&mut harness, "the question", |h| h.count(".ds-alert") == 1);
    click(
        &mut harness,
        ".ds-alert .ds-button[data-role=\"destructive\"]",
    );
    settle(&mut harness, "the file", |_| {
        !sandbox
            .read("docket/agents.toml")
            .unwrap_or_default()
            .contains("claude-code")
    });
}
