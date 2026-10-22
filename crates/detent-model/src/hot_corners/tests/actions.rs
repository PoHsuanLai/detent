use super::*;

/// A manifest of an app `org.quire.Mail` with the five kinds of action the list sorts.
const MAIL: &str = r#"
vocab = 1
app = "org.quire.Mail"
entities = []

[[actions]]
name = "mail.compose"
label = "Compose"
on = { kind = "nothing" }
effect = "read"
classes = []
undo = "not_undoable"
reach = "offered"
latency = "instant"
result = { kind = "nothing" }
keys = { kind = "none" }
lasting = "no"
dry_run = "none"
params = []

[[actions]]
name = "mail.check"
label = "Check for new mail"
on = { kind = "nothing" }
effect = "read"
classes = []
undo = "not_undoable"
reach = "offered"
latency = "instant"
result = { kind = "nothing" }
keys = { kind = "none" }
lasting = "no"
dry_run = "none"

[[actions.params]]
name = "folder"
label = "Folder"
ty = { kind = "text", v = { max = 100, lines = "one" } }
need = { kind = "optional" }
sink = "inert"

[[actions]]
name = "mail.search"
label = "Search"
on = { kind = "nothing" }
effect = "read"
classes = []
undo = "not_undoable"
reach = "offered"
latency = "instant"
result = { kind = "nothing" }
keys = { kind = "none" }
lasting = "no"
dry_run = "none"

[[actions.params]]
name = "words"
label = "Words"
ty = { kind = "text", v = { max = 100, lines = "one" } }
need = { kind = "required" }
sink = "inert"

[[actions]]
name = "mail.reply"
label = "Reply"
on = { kind = "text" }
effect = "read"
classes = []
undo = "not_undoable"
reach = "offered"
latency = "instant"
result = { kind = "nothing" }
keys = { kind = "none" }
lasting = "no"
dry_run = "none"
params = []

[[actions]]
name = "mail.settings"
label = "Mail settings"
on = { kind = "nothing" }
effect = "read"
classes = []
undo = "not_undoable"
reach = "hidden"
latency = "instant"
result = { kind = "nothing" }
keys = { kind = "none" }
lasting = "no"
dry_run = "none"
params = []
"#;

#[test]
fn the_action_list_leaves_out_what_needs_parameters_a_target_or_the_person() {
    let offered: Vec<(String, String)> = actions_of(MAIL)
        .into_iter()
        .map(|choice| (choice.action, choice.label))
        .collect();
    assert_eq!(
        offered,
        [
            ("compose".to_owned(), "Compose".to_owned()),
            ("check".to_owned(), "Check for new mail".to_owned()),
        ],
        "search needs words, reply needs a text, settings is kept for the person"
    );
    assert_eq!(actions_of("not a manifest"), Vec::new());
    assert_eq!(
        actions_of(&MAIL.replace("vocab = 1", "vocab = 999")),
        Vec::new()
    );
}

#[test]
fn an_app_two_folders_describe_is_listed_once_and_the_list_groups_and_searches() {
    let stale = MAIL.replace("Compose", "Old compose");
    let mut known = known();
    known.actions = actions_of_all([MAIL, stale.as_str()]);
    assert_eq!(known.actions.len(), 2, "the first folder's manifest wins");
    assert_eq!(known.actions[0].label, "Compose");

    let titles = |query: &str| -> Vec<(String, Vec<String>)> {
        action_groups(&known, query)
            .into_iter()
            .map(|group| {
                (
                    group.title,
                    group.actions.into_iter().map(|a| a.label).collect(),
                )
            })
            .collect()
    };
    let all = vec![(
        "Mail".to_owned(),
        vec!["Check for new mail".to_owned(), "Compose".to_owned()],
    )];
    assert_eq!(titles(""), all);
    assert_eq!(
        titles("  MAIL "),
        all,
        "the app's name matches, in any case"
    );
    assert_eq!(
        titles("mail comp"),
        [("Mail".to_owned(), vec!["Compose".to_owned()])],
        "every word must match the line"
    );
    assert_eq!(titles("calendar"), Vec::new());
}
