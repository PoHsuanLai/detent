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

fn state(rows: Vec<EngineRow>) -> EngineOfferState {
    EngineOfferState {
        rows,
        offer: EngineOffer::Idle,
    }
}

fn step(
    state: EngineOfferState,
    inputs: impl IntoIterator<Item = EngineOfferInput>,
) -> (EngineOfferState, Vec<EngineOfferEffect>) {
    inputs
        .into_iter()
        .fold((state, Vec::new()), |(state, mut said), input| {
            let (next, effects) = engine_offer_step(state, input);
            said.extend(effects);
            (next, said)
        })
}

fn chose(id: &str) -> EngineOfferInput {
    EngineOfferInput::Chose(id.to_owned())
}

#[test]
fn an_engine_that_is_not_there_is_asked_about_and_one_that_is_ready_is_not() {
    let rows = vec![
        llama(EngineStanding::NotInstalled),
        strata(EngineStanding::Ready),
    ];
    let (asked, effects) = step(state(rows.clone()), [chose("llama_cpp")]);
    assert!(effects.is_empty());
    let brief = engine_brief(&asked.offer).expect("an offer");
    assert_eq!(brief.engine.name, "llama.cpp");
    assert_eq!(
        brief.matter,
        EngineMatter::Download {
            replacing: Replacing::No,
            size: Some(30_497_889)
        }
    );
    assert_eq!(brief.scopes, BOTH);
    assert_eq!(
        brief.scope,
        InstallScope::AllUsers,
        "everyone's is the default"
    );
    assert_eq!(brief.primary, Some(EngineAction::Download));
    assert_eq!(brief.secondary, Some(EngineAction::NotNow));
    let (nothing, _) = step(state(rows), [chose("strata")]);
    assert_eq!(
        nothing.offer,
        EngineOffer::Idle,
        "a ready engine asks nothing"
    );
    let (unknown, _) = step(nothing, [chose("kokoro")]);
    assert_eq!(unknown.offer, EngineOffer::Idle);
}

#[test]
fn not_now_leaves_nothing_started_and_the_engines_as_they_were() {
    let rows = vec![llama(EngineStanding::NotInstalled)];
    let (left, effects) = step(
        state(rows.clone()),
        [chose("llama_cpp"), EngineOfferInput::NotNow],
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
    let brief = engine_brief(&asked.offer).expect("an offer");
    assert_eq!(brief.scope, InstallScope::JustMe);
    assert!(brief.scopes.is_empty(), "one way asks no choice");
    assert_eq!(
        brief.matter,
        EngineMatter::Download {
            replacing: Replacing::No,
            size: Some(4_411_066_545)
        }
    );
}

#[test]
fn a_download_follows_its_row_to_done_and_continue_restarts_the_assistant() {
    let rows = vec![llama(EngineStanding::NotInstalled)];
    let (working, effects) = step(
        state(rows),
        [
            chose("llama_cpp"),
            EngineOfferInput::Scope(InstallScope::JustMe),
            EngineOfferInput::Go,
        ],
    );
    assert_eq!(
        effects,
        [EngineOfferEffect::Install {
            engine: "llama_cpp".to_owned(),
            scope: InstallScope::JustMe
        }]
    );
    let (halfway, _) = step(
        working,
        [EngineOfferInput::Rows(vec![llama(
            EngineStanding::Installing(500),
        )])],
    );
    assert_eq!(
        engine_brief(&halfway.offer).and_then(|w| w.progress),
        Some(500)
    );
    let (done, _) = step(
        halfway,
        [EngineOfferInput::Rows(vec![llama(EngineStanding::Ready)])],
    );
    let brief = engine_brief(&done.offer).expect("an offer");
    assert_eq!(brief.matter, EngineMatter::Ready);
    assert_eq!(brief.primary, Some(EngineAction::Continue));
    let (closed, effects) = step(done, [EngineOfferInput::Continue]);
    assert_eq!(closed.offer, EngineOffer::Idle);
    assert_eq!(effects, [EngineOfferEffect::Restart]);
}

#[test]
fn a_failed_download_says_why_and_try_again_asks_again_and_a_stop_closes_it() {
    let (working, _) = step(
        state(vec![llama(EngineStanding::NotInstalled)]),
        [chose("llama_cpp"), EngineOfferInput::Go],
    );
    let (failed, _) = step(
        working.clone(),
        [EngineOfferInput::Rows(vec![llama(EngineStanding::Failed(
            EngineFailure::Offline,
        ))])],
    );
    let brief = engine_brief(&failed.offer).expect("an offer");
    assert_eq!(
        brief.matter,
        EngineMatter::Failed {
            reason: EngineFailure::Offline
        }
    );
    assert_eq!(brief.primary, Some(EngineAction::TryAgain));
    let (again, effects) = step(failed.clone(), [EngineOfferInput::Go]);
    assert!(effects.is_empty());
    assert!(matches!(again.offer, EngineOffer::Asking { .. }));
    let (left, _) = step(failed, [EngineOfferInput::NotNow]);
    assert_eq!(left.offer, EngineOffer::Idle);

    let (stopping, effects) = step(working, [EngineOfferInput::Stop]);
    assert_eq!(effects, [EngineOfferEffect::Cancel("llama_cpp".to_owned())]);
    let (stopped, _) = step(
        stopping,
        [EngineOfferInput::Rows(vec![llama(
            EngineStanding::NotInstalled,
        )])],
    );
    assert_eq!(stopped.offer, EngineOffer::Idle, "a stop says nothing");
    let (refused, _) = step(
        state(vec![llama(EngineStanding::NotInstalled)]),
        [
            chose("llama_cpp"),
            EngineOfferInput::Go,
            EngineOfferInput::NotStarted(EngineFailure::Unsupported),
        ],
    );
    assert!(matches!(refused.offer, EngineOffer::Failed { .. }));
}

#[test]
fn an_install_that_was_changed_asks_to_download_again() {
    let (asked, _) = step(
        state(vec![llama(EngineStanding::Changed)]),
        [chose("llama_cpp")],
    );
    let brief = engine_brief(&asked.offer).expect("an offer");
    assert_eq!(
        brief.matter,
        EngineMatter::Download {
            replacing: Replacing::Yes,
            size: Some(30_497_889)
        }
    );
}

#[test]
fn an_engine_with_no_download_asks_for_a_folder_and_counts_the_one_chosen() {
    let (asked, _) = step(
        state(vec![strata(EngineStanding::NotInstalled)]),
        [chose("strata")],
    );
    let brief = engine_brief(&asked.offer).expect("an offer");
    assert_eq!(brief.matter, EngineMatter::Folder);
    assert_eq!(brief.primary, Some(EngineAction::ChooseFolder));
    let (chosen, effects) = step(asked.clone(), [EngineOfferInput::Go]);
    assert_eq!(effects, [EngineOfferEffect::ChooseFolder]);
    let (closed_chooser, effects) = step(chosen, [EngineOfferInput::Folder(None)]);
    assert!(effects.is_empty() && matches!(closed_chooser.offer, EngineOffer::Asking { .. }));
    let (pointing, effects) = step(
        closed_chooser,
        [EngineOfferInput::Folder(Some(
            "/mnt/data/strata".to_owned(),
        ))],
    );
    assert_eq!(
        effects,
        [EngineOfferEffect::Point {
            engine: "strata".to_owned(),
            folder: "/mnt/data/strata".to_owned()
        }]
    );
    // A look at the engines while it is checked does not close the offer.
    let (still, _) = step(
        pointing.clone(),
        [EngineOfferInput::Rows(vec![strata(
            EngineStanding::NotInstalled,
        )])],
    );
    assert!(matches!(still.offer, EngineOffer::Pointing { .. }));
    let (done, _) = step(pointing.clone(), [EngineOfferInput::Pointed(Ok(()))]);
    assert!(matches!(done.offer, EngineOffer::Done { .. }));
    let (failed, _) = step(
        pointing,
        [EngineOfferInput::Pointed(Err(EngineFailure::NotThere))],
    );
    assert!(matches!(failed.offer, EngineOffer::Failed { .. }));

    let nothing = row(
        "kokoro",
        "Kokoro",
        EngineStanding::NotInstalled,
        EngineWay::Nothing,
    );
    let (told, _) = step(state(vec![nothing]), [chose("kokoro")]);
    let brief = engine_brief(&told.offer).expect("an offer");
    assert_eq!(brief.matter, EngineMatter::Unavailable);
    assert_eq!(brief.primary, Some(EngineAction::Acknowledge));
    assert_eq!(
        brief.primary.map(EngineAction::input),
        Some(EngineOfferInput::NotNow)
    );
}
