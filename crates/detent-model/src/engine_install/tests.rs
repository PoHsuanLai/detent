use super::*;

fn row(id: &str, name: &str, standing: EngineStanding, way: EngineWay) -> EngineRow {
    EngineRow {
        id: id.to_owned(),
        name: name.to_owned(),
        standing,
        way,
    }
}

fn download(scopes: &[InstallScope], size: Option<u64>) -> EngineWay {
    EngineWay::Download {
        scopes: scopes.to_vec(),
        size,
    }
}

const BOTH: [InstallScope; 2] = [InstallScope::AllUsers, InstallScope::JustMe];

fn llama(standing: EngineStanding) -> EngineRow {
    row(
        "llama_cpp",
        "llama.cpp",
        standing,
        download(&BOTH, Some(30_497_889)),
    )
}

fn strata(standing: EngineStanding) -> EngineRow {
    row("strata", "Strata", standing, EngineWay::Folder)
}

fn state(rows: Vec<EngineRow>) -> EngineSheetState {
    EngineSheetState {
        rows,
        sheet: EngineSheet::Closed,
    }
}

fn step(
    state: EngineSheetState,
    inputs: impl IntoIterator<Item = EngineSheetInput>,
) -> (EngineSheetState, Vec<EngineSheetEffect>) {
    inputs
        .into_iter()
        .fold((state, Vec::new()), |(state, mut said), input| {
            let (next, effects) = engine_sheet_step(state, input);
            said.extend(effects);
            (next, said)
        })
}

fn chose(id: &str) -> EngineSheetInput {
    EngineSheetInput::Chose(id.to_owned())
}

#[test]
fn an_engine_that_is_not_there_is_asked_about_and_one_that_is_ready_is_not() {
    let rows = vec![
        llama(EngineStanding::NotInstalled),
        strata(EngineStanding::Ready),
    ];
    let (asked, effects) = step(state(rows.clone()), [chose("llama_cpp")]);
    assert!(effects.is_empty());
    let words = sheet_words(&asked.sheet).expect("a sheet");
    assert_eq!(words.title, "Download llama.cpp?");
    assert_eq!(
        words.message,
        "Your assistant needs llama.cpp to run models on this computer. It's about 30 MB."
    );
    assert_eq!(words.scopes, BOTH);
    assert_eq!(
        words.scope,
        InstallScope::AllUsers,
        "everyone's is the default"
    );
    assert_eq!(words.primary.map(|b| b.label), Some("Download"));
    assert_eq!(words.secondary.map(|b| b.label), Some("Not Now"));
    let (nothing, _) = step(state(rows), [chose("strata")]);
    assert_eq!(
        nothing.sheet,
        EngineSheet::Closed,
        "a ready engine asks nothing"
    );
    let (unknown, _) = step(nothing, [chose("kokoro")]);
    assert_eq!(unknown.sheet, EngineSheet::Closed);
}

#[test]
fn not_now_leaves_nothing_started_and_the_engines_as_they_were() {
    let rows = vec![llama(EngineStanding::NotInstalled)];
    let (left, effects) = step(
        state(rows.clone()),
        [chose("llama_cpp"), EngineSheetInput::NotNow],
    );
    assert_eq!(left, state(rows));
    assert!(effects.is_empty());
}

#[test]
fn only_the_persons_own_way_is_the_default_when_it_is_the_only_one_and_asks_no_choice() {
    let only = row(
        "vllm",
        "vLLM",
        EngineStanding::NotInstalled,
        download(&[InstallScope::JustMe], Some(4_411_066_545)),
    );
    let (asked, _) = step(state(vec![only]), [chose("vllm")]);
    let words = sheet_words(&asked.sheet).expect("a sheet");
    assert_eq!(
        (words.scope, words.scopes.is_empty()),
        (InstallScope::JustMe, true)
    );
    assert!(
        words.message.ends_with("It's about 4.4 GB."),
        "{}",
        words.message
    );
}

#[test]
fn a_download_follows_its_row_to_done_and_continue_restarts_the_assistant() {
    let rows = vec![llama(EngineStanding::NotInstalled)];
    let (working, effects) = step(
        state(rows),
        [
            chose("llama_cpp"),
            EngineSheetInput::Scope(InstallScope::JustMe),
            EngineSheetInput::Go,
        ],
    );
    assert_eq!(
        effects,
        [EngineSheetEffect::Install {
            engine: "llama_cpp".to_owned(),
            scope: InstallScope::JustMe
        }]
    );
    let (halfway, _) = step(
        working,
        [EngineSheetInput::Rows(vec![llama(
            EngineStanding::Installing(500),
        )])],
    );
    assert_eq!(
        sheet_words(&halfway.sheet).and_then(|w| w.progress),
        Some(500)
    );
    let (done, _) = step(
        halfway,
        [EngineSheetInput::Rows(vec![llama(EngineStanding::Ready)])],
    );
    let words = sheet_words(&done.sheet).expect("a sheet");
    assert_eq!(words.title, "llama.cpp Is Ready");
    assert_eq!(words.primary.map(|b| b.press), Some(Press::Continue));
    let (closed, effects) = step(done, [EngineSheetInput::Continue]);
    assert_eq!(closed.sheet, EngineSheet::Closed);
    assert_eq!(effects, [EngineSheetEffect::Restart]);
}

#[test]
fn a_failed_download_says_why_and_try_again_asks_again_and_a_stop_closes_it() {
    let (working, _) = step(
        state(vec![llama(EngineStanding::NotInstalled)]),
        [chose("llama_cpp"), EngineSheetInput::Go],
    );
    let (failed, _) = step(
        working.clone(),
        [EngineSheetInput::Rows(vec![llama(EngineStanding::Failed(
            "The download didn't finish.".to_owned(),
        ))])],
    );
    let words = sheet_words(&failed.sheet).expect("a sheet");
    assert_eq!(words.title, "Couldn't Download llama.cpp");
    assert_eq!(words.message, "The download didn't finish.");
    assert_eq!(words.primary.map(|b| b.label), Some("Try Again"));
    let (again, effects) = step(failed.clone(), [EngineSheetInput::Go]);
    assert!(effects.is_empty());
    assert!(matches!(again.sheet, EngineSheet::Asking { .. }));
    let (left, _) = step(failed, [EngineSheetInput::NotNow]);
    assert_eq!(left.sheet, EngineSheet::Closed);

    let (stopping, effects) = step(working, [EngineSheetInput::Stop]);
    assert_eq!(effects, [EngineSheetEffect::Cancel("llama_cpp".to_owned())]);
    let (stopped, _) = step(
        stopping,
        [EngineSheetInput::Rows(vec![llama(
            EngineStanding::NotInstalled,
        )])],
    );
    assert_eq!(stopped.sheet, EngineSheet::Closed, "a stop says nothing");
    let (refused, _) = step(
        state(vec![llama(EngineStanding::NotInstalled)]),
        [
            chose("llama_cpp"),
            EngineSheetInput::Go,
            EngineSheetInput::NotStarted("Nothing can be installed.".to_owned()),
        ],
    );
    assert!(matches!(refused.sheet, EngineSheet::Failed { .. }));
}

#[test]
fn an_install_that_was_changed_asks_to_download_again() {
    let (asked, _) = step(
        state(vec![llama(EngineStanding::Changed)]),
        [chose("llama_cpp")],
    );
    let words = sheet_words(&asked.sheet).expect("a sheet");
    assert_eq!(words.title, "Download llama.cpp Again?");
    assert!(
        words
            .message
            .starts_with("llama.cpp was changed since it was installed")
    );
}

#[test]
fn an_engine_with_no_download_asks_for_a_folder_and_counts_the_one_chosen() {
    let (asked, _) = step(
        state(vec![strata(EngineStanding::NotInstalled)]),
        [chose("strata")],
    );
    let words = sheet_words(&asked.sheet).expect("a sheet");
    assert_eq!(words.title, "Strata Isn't Available to Download");
    assert_eq!(
        words.primary.map(|b| b.label),
        Some("Choose Folder\u{2026}")
    );
    let (chosen, effects) = step(asked.clone(), [EngineSheetInput::Go]);
    assert_eq!(effects, [EngineSheetEffect::ChooseFolder]);
    let (closed_chooser, effects) = step(chosen, [EngineSheetInput::Folder(None)]);
    assert!(effects.is_empty() && matches!(closed_chooser.sheet, EngineSheet::Asking { .. }));
    let (pointing, effects) = step(
        closed_chooser,
        [EngineSheetInput::Folder(Some(
            "/mnt/data/strata".to_owned(),
        ))],
    );
    assert_eq!(
        effects,
        [EngineSheetEffect::Point {
            engine: "strata".to_owned(),
            folder: "/mnt/data/strata".to_owned()
        }]
    );
    // A look at the engines while it is checked does not close the sheet.
    let (still, _) = step(
        pointing.clone(),
        [EngineSheetInput::Rows(vec![strata(
            EngineStanding::NotInstalled,
        )])],
    );
    assert!(matches!(still.sheet, EngineSheet::Pointing { .. }));
    let (done, _) = step(pointing.clone(), [EngineSheetInput::Pointed(Ok(()))]);
    assert!(matches!(done.sheet, EngineSheet::Done { .. }));
    let (failed, _) = step(
        pointing,
        [EngineSheetInput::Pointed(Err(
            "That folder doesn't hold Strata.".to_owned(),
        ))],
    );
    assert!(matches!(failed.sheet, EngineSheet::Failed { .. }));

    let nothing = row(
        "kokoro",
        "Kokoro",
        EngineStanding::NotInstalled,
        EngineWay::Nothing,
    );
    let (told, _) = step(state(vec![nothing]), [chose("kokoro")]);
    let words = sheet_words(&told.sheet).expect("a sheet");
    assert_eq!(
        words.message,
        "There is no download of Kokoro for this computer."
    );
    assert_eq!(words.primary.map(|b| b.press), Some(Press::NotNow));
}

#[test]
fn sizes_are_said_in_whole_megabytes_and_tenths_of_a_gigabyte() {
    for (bytes, said) in [
        (17_851_611, "18 MB"),
        (30_497_889, "30 MB"),
        (440_236_547, "440 MB"),
        (996_000_000, "1.0 GB"),
        (4_411_066_545, "4.4 GB"),
        (1, "1 MB"),
    ] {
        assert_eq!(size_words(bytes), said, "{bytes}");
    }
}

#[test]
fn no_word_on_the_sheet_is_a_developer_word_or_a_path() {
    let rows = vec![
        llama(EngineStanding::NotInstalled),
        strata(EngineStanding::NotInstalled),
    ];
    for id in ["llama_cpp", "strata"] {
        let (asked, _) = step(state(rows.clone()), [chose(id)]);
        let words = sheet_words(&asked.sheet).expect("a sheet");
        let said = format!("{} {}", words.title, words.message).to_lowercase();
        for banned in [
            "pkexec", "polkit", "sha", "http", "tar", "/", "cli", "api", "mcp",
        ] {
            assert!(!said.split_whitespace().any(|w| w == banned), "{said}");
        }
        assert!(!said.contains('/'), "{said}");
    }
}
