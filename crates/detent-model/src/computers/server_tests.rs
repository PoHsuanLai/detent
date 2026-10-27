//! The Add a server sheet: what it checks, what Add asks for, and how it ends.

use super::{
    ComputerRow, ComputersEffect as Effect, ComputersFound, ComputersInput as Input,
    ComputersState as State, KeyField, NewServer, ServerDraft, ServerEdit, ServerFault, ServerForm,
    ServerModel, ServerModels, ServerPhase, ServerReach, ServerRefusal, computers_step,
};
use porter_core::{ModelId, SecretText};
use std::io;
use std::path::PathBuf;

fn model(text: &str) -> ModelId {
    ModelId::parse(text).expect("a model id")
}

fn offered() -> Vec<ServerModel> {
    vec![ServerModel {
        id: model("qwen3-8b"),
        label: "Qwen3 8B".to_owned(),
    }]
}

fn ready() -> State {
    let found = ComputersFound {
        computers: Vec::<ComputerRow>::new(),
        guests: vec![],
    };
    computers_step(State::Looking, Input::Found(Ok(found))).0
}

fn sheet_of(state: &State) -> Option<&NewServer> {
    match state {
        State::Ready(ready) => ready.server.as_ref(),
        other => panic!("not ready: {other:?}"),
    }
}

fn form(name: &str, port: &str, socket: Option<&str>) -> ServerForm {
    ServerForm {
        name: name.to_owned(),
        model: Some(model("qwen3-8b")),
        port: port.to_owned(),
        socket: socket.map(PathBuf::from),
        key: KeyField::Nothing,
    }
}

#[test]
fn the_form_says_what_is_wrong_with_it_first() {
    let long = "x".repeat(65);
    let no_model = ServerForm {
        model: None,
        ..form("Lab", "8080", None)
    };
    let cases: Vec<(&str, ServerForm, ServerFault)> = vec![
        ("empty name", form("  ", "8080", None), ServerFault::NoName),
        (
            "long name",
            form(&long, "8080", None),
            ServerFault::LongName,
        ),
        ("no model", no_model, ServerFault::NoModel),
        ("neither", form("Lab", " ", None), ServerFault::NoReach),
        (
            "both",
            form("Lab", "80", Some("/run/s")),
            ServerFault::BothReach,
        ),
        (
            "not a number",
            form("Lab", "http", None),
            ServerFault::BadPort,
        ),
        ("zero", form("Lab", "0", None), ServerFault::BadPort),
        ("too big", form("Lab", "65536", None), ServerFault::BadPort),
    ];
    for (case, form, fault) in cases {
        assert_eq!(form.check(), Err(fault), "{case}");
    }
}

#[test]
fn a_name_of_sixty_four_characters_is_kept_trimmed() {
    let name = "\u{e9}".repeat(64);
    let draft = form(&format!(" {name} "), "8080", None)
        .check()
        .expect("fine");
    assert_eq!(draft.name, name);
}

#[test]
fn a_number_or_a_file_makes_the_reach() {
    let port = form("Lab", " 8080 ", None).check().expect("a port");
    assert_eq!(port.reach, ServerReach::Port(8080));
    let file = form("Lab", "", Some("/run/user/1/s.sock"))
        .check()
        .expect("a file");
    assert_eq!(
        file.reach,
        ServerReach::Socket(PathBuf::from("/run/user/1/s.sock"))
    );
}

fn draft_with(key: KeyField) -> ServerDraft {
    ServerDraft {
        key,
        ..form("Lab", "8080", None).check().expect("fine")
    }
}

#[test]
fn a_key_is_typed_or_read_from_its_file_only_when_asked() {
    let none = draft_with(KeyField::Nothing).secret(|_| panic!("no file"));
    assert_eq!(none, Ok(None));
    let typed = draft_with(KeyField::Typed(SecretText::new(" abc "))).secret(|_| panic!("no file"));
    assert_eq!(typed, Ok(Some(SecretText::new("abc"))));
    let file = draft_with(KeyField::File(PathBuf::from("/k")));
    assert_eq!(
        file.secret(|path| {
            assert_eq!(path, PathBuf::from("/k"));
            Ok("  from-file\n".to_owned())
        }),
        Ok(Some(SecretText::new("from-file")))
    );
    assert_eq!(
        file.secret(|_| Err(io::Error::other("gone"))),
        Err(ServerFault::KeyFileUnreadable)
    );
    assert_eq!(
        file.secret(|_| Ok("\n".to_owned())),
        Err(ServerFault::KeyFileEmpty)
    );
}

#[test]
fn a_blank_typed_key_is_no_key_and_a_file_replaces_a_typed_key() {
    let typed = ServerForm::default().edited(ServerEdit::KeyText(SecretText::new("k")));
    assert_eq!(typed.key, KeyField::Typed(SecretText::new("k")));
    let file = typed
        .clone()
        .edited(ServerEdit::KeyFile(Some(PathBuf::from("/k"))));
    assert_eq!(file.key, KeyField::File(PathBuf::from("/k")));
    let blank = typed.edited(ServerEdit::KeyText(SecretText::new("  ")));
    assert_eq!(blank.key, KeyField::Nothing);
}

fn opened() -> State {
    let (state, effects) = computers_step(ready(), Input::OpenServer);
    assert_eq!(effects, vec![Effect::LoadServerModels]);
    computers_step(state, Input::ServerModels(offered())).0
}

fn typed(state: State, edits: Vec<ServerEdit>) -> State {
    edits.into_iter().fold(state, |state, edit| {
        computers_step(state, Input::ServerEdit(edit)).0
    })
}

#[test]
fn the_sheet_opens_looking_and_picks_the_first_model_offered() {
    let (looking, _) = computers_step(ready(), Input::OpenServer);
    assert_eq!(
        sheet_of(&looking).map(|s| &s.models),
        Some(&ServerModels::Looking)
    );
    let sheet = sheet_of(&opened()).cloned().expect("open");
    assert_eq!(sheet.form.model, Some(model("qwen3-8b")));
    assert_eq!(sheet.models, ServerModels::Offered(offered()));
}

#[test]
fn add_with_a_bad_sheet_tells_why_and_asks_for_nothing() {
    let (state, effects) = computers_step(opened(), Input::AddServer);
    assert!(effects.is_empty());
    let sheet = sheet_of(&state).expect("still open");
    assert_eq!(sheet.fault, Some(ServerRefusal::Form(ServerFault::NoName)));
    assert_eq!(sheet.phase, ServerPhase::Editing);
}

#[test]
fn add_asks_for_the_server_and_waits() {
    let state = typed(
        opened(),
        vec![
            ServerEdit::Name("Lab".to_owned()),
            ServerEdit::Port("8080".to_owned()),
            ServerEdit::KeyText(SecretText::new("k")),
        ],
    );
    let (state, effects) = computers_step(state, Input::AddServer);
    let want = ServerDraft {
        name: "Lab".to_owned(),
        model: model("qwen3-8b"),
        reach: ServerReach::Port(8080),
        key: KeyField::Typed(SecretText::new("k")),
    };
    assert_eq!(effects, vec![Effect::AddServer(want)]);
    assert_eq!(sheet_of(&state).map(|s| s.phase), Some(ServerPhase::Adding));
    // Nothing is taken meanwhile: not an edit, not Cancel, not a second Add.
    let (again, more) = computers_step(state.clone(), Input::AddServer);
    assert!(more.is_empty());
    assert_eq!(again, state);
    assert_eq!(
        typed(state.clone(), vec![ServerEdit::Name("x".to_owned())]),
        state
    );
    assert_eq!(computers_step(state.clone(), Input::CloseServer).0, state);
}

fn adding() -> State {
    let state = typed(
        opened(),
        vec![
            ServerEdit::Name("Lab".to_owned()),
            ServerEdit::Port("8080".to_owned()),
        ],
    );
    computers_step(state, Input::AddServer).0
}

#[test]
fn a_refusal_stays_on_the_sheet_in_the_services_words() {
    let (state, effects) = computers_step(
        adding(),
        Input::ServerAdded(Err(ServerRefusal::Service("Taken.".to_owned()))),
    );
    assert!(effects.is_empty());
    let sheet = sheet_of(&state).expect("open");
    assert_eq!(
        sheet.fault,
        Some(ServerRefusal::Service("Taken.".to_owned()))
    );
    assert_eq!(sheet.phase, ServerPhase::Editing);
    assert_eq!(sheet.form.name, "Lab");
}

#[test]
fn success_closes_the_sheet_and_reads_the_list() {
    let (state, effects) = computers_step(adding(), Input::ServerAdded(Ok(())));
    assert_eq!(sheet_of(&state), None);
    assert_eq!(effects, vec![Effect::Reload]);
}

#[test]
fn cancel_closes_an_idle_sheet_and_a_late_answer_is_dropped() {
    let (closed, _) = computers_step(opened(), Input::CloseServer);
    assert_eq!(sheet_of(&closed), None);
    let (late, effects) = computers_step(
        closed,
        Input::ServerAdded(Err(ServerRefusal::Service("x".to_owned()))),
    );
    assert_eq!(sheet_of(&late), None);
    assert!(effects.is_empty());
}

#[test]
fn reading_the_list_again_keeps_the_open_sheet() {
    let found = ComputersFound {
        computers: vec![],
        guests: vec![],
    };
    let (kept, _) = computers_step(opened(), Input::Found(Ok(found)));
    assert!(sheet_of(&kept).is_some());
}
