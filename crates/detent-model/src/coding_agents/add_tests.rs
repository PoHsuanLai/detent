use super::add::{
    AddForm, FoundAgent, NOTHING_NEW, RemoveQuestion, Request, add_refusal, removal_of,
    remove_refusal, unlisted_found,
};
use super::add_choice::{
    AddButton, Candidate, DOWNLOAD_FAILED, LIST_FAILED, Listing, Presence, Step, download_failure,
    list_failure,
};
use crate::outside::Assistant;
use docket_settings::{
    AddFault, AgentRow, ChoiceFault, Install, ModelState, Offers, SignInState, Source,
};

fn found(id: &str) -> FoundAgent {
    FoundAgent {
        id: id.to_owned(),
        version: "1.2.0".to_owned(),
    }
}

fn row(program: &str, source: Source) -> AgentRow {
    AgentRow {
        program: program.to_owned(),
        label: "Claude Code".to_owned(),
        source,
        command: None,
        model: ModelState::AgentsOwn,
        offers: Offers::NotSeenYet,
        sign_in: SignInState::NotSeenYet,
        way: None,
        rewind: docket_core::Rewind::Docket,
    }
}

fn registry(id: &str) -> Source {
    Source::Registry {
        id: id.to_owned(),
        version: "1.0.0".to_owned(),
        installed: Install::Installed,
    }
}

fn own() -> Source {
    Source::Own {
        installed: Install::Installed,
    }
}

#[test]
fn a_name_is_read_from_the_id() {
    // `(id, name)`.
    const CASES: &[(&str, &str)] = &[
        ("claude-code", "Claude Code"),
        ("gemini", "Gemini"),
        ("a_b.c", "A B C"),
        ("--", ""),
    ];
    for (id, want) in CASES {
        assert_eq!(found(id).name(), *want, "{id}");
    }
}

#[test]
fn only_the_installed_assistants_nobody_lists_are_offered() {
    let rows = [
        row("claude-bin", registry("claude-code")),
        row("mine", own()),
    ];
    let all = [found("claude-code"), found("mine"), found("gemini")];
    let ids: Vec<String> = unlisted_found(&all, &rows)
        .into_iter()
        .map(|f| f.id)
        .collect();
    assert_eq!(ids, ["gemini"]);
}

fn online(id: &str, name: &str, presence: Presence) -> Candidate {
    Candidate {
        id: id.to_owned(),
        name: name.to_owned(),
        description: format!("About {name}."),
        version: "2.0.0".to_owned(),
        presence,
    }
}

fn ids(form: &AddForm) -> Vec<&str> {
    form.entries().iter().map(|one| one.id.as_str()).collect()
}

#[test]
fn the_form_names_the_pick_after_itself_and_checks_the_name() {
    let empty = AddForm::offline(&[], &[]);
    assert_eq!(
        (empty.button(), empty.request()),
        (AddButton::Off, Err(NOTHING_NEW))
    );
    let form = AddForm::offline(&[found("claude-code"), found("gemini")], &[]);
    assert_eq!(form.label(), "Claude Code");
    let form = form.pick("gemini");
    assert_eq!((form.picked_id(), form.label()), ("gemini", "Gemini"));
    let unchanged = form.clone().pick("nobody");
    assert_eq!(unchanged, form, "an id not offered changes nothing");

    // `(typed, the name written, or the sentence it is refused with)`.
    const CASES: &[(&str, Result<&str, &str>)] = &[
        ("  Gem  ", Ok("Gem")),
        ("", Err("Give it a name of up to 64 letters.")),
        ("   ", Err("Give it a name of up to 64 letters.")),
    ];
    for (typed, want) in CASES {
        let asked = form.clone().typed((*typed).to_owned()).request();
        let got = asked.map(|request| match request {
            Request::Add(new) => new.label,
            Request::Install { .. } => String::new(),
        });
        let want = want.map(str::to_owned);
        assert_eq!(got, want, "{typed:?}");
    }
    let Ok(Request::Add(new)) = form.request() else {
        panic!("an installed one is listed, not downloaded");
    };
    assert_eq!(
        (
            new.program.as_str(),
            new.registry.as_str(),
            new.version.as_str()
        ),
        ("gemini", "gemini", "1.2.0")
    );
}

#[test]
fn a_name_docket_cannot_keep_is_refused_before_anything_is_written() {
    let form = AddForm::offline(&[found("odd+name")], &[]);
    assert_eq!(form.request(), Err(add_refusal(AddFault::BadName)));
}

#[test]
fn the_sheet_waits_for_the_list_then_offers_it_without_what_is_added() {
    let rows = [row("claude-bin", registry("claude-code"))];
    let form = AddForm::loading(&[found("gemini")], &rows);
    assert_eq!(form.listing(), &Listing::Loading);
    assert_eq!(
        form.button(),
        AddButton::Off,
        "nothing to add while it loads"
    );

    let offers = vec![
        online("claude-code", "Claude Code", Presence::Installed),
        online("codex", "Codex", Presence::Online),
        online("gemini", "Gemini CLI", Presence::Installed),
    ];
    let form = form.catalogue(offers.clone());
    assert_eq!(form.listing(), &Listing::Listed);
    assert_eq!(
        ids(&form),
        ["codex", "gemini"],
        "the added one is not offered"
    );
    assert_eq!((form.label(), form.button()), ("Codex", AddButton::Ready));
    assert_eq!(Presence::Installed.mark(), Some("Installed"));
    assert_eq!(Presence::Online.mark(), None);
    assert_eq!(
        form.clone().catalogue(Vec::new()),
        form,
        "a list that comes late changes nothing"
    );

    // An installed assistant the list does not know stays offered, after the listed ones.
    let extra = AddForm::loading(&[found("mine")], &[]).catalogue(offers);
    assert_eq!(ids(&extra), ["claude-code", "codex", "gemini", "mine"]);
}

#[test]
fn a_list_that_cannot_be_fetched_says_why_and_offers_the_installed_ones() {
    let form = AddForm::loading(&[found("gemini")], &[]).unreachable("no route to the host\n");
    assert_eq!(
        form.listing(),
        &Listing::Failed(format!("{LIST_FAILED} no route to the host"))
    );
    assert_eq!(
        (ids(&form), form.button()),
        (vec!["gemini"], AddButton::Ready)
    );
    assert_eq!(list_failure(" \n"), LIST_FAILED);
    let late = form.clone().unreachable("again");
    assert_eq!(late, form, "only a form still loading takes it");
}

#[test]
fn an_assistant_not_installed_is_downloaded_first_and_the_sheet_says_so() {
    let form =
        AddForm::loading(&[], &[]).catalogue(vec![online("codex", "Codex", Presence::Online)]);
    let Ok(Request::Install { id, version, then }) = form.request() else {
        panic!("a download comes first");
    };
    assert_eq!((id.as_str(), version.as_str()), ("codex", "2.0.0"));
    assert_eq!(
        (then.program.as_str(), then.label.as_str()),
        ("codex", "Codex")
    );

    assert_eq!((form.step(), form.download_line()), (Step::Idle, None));
    let form = form.installing();
    assert_eq!(
        (form.step(), form.button()),
        (Step::Starting, AddButton::Busy)
    );
    assert_eq!(
        form.download_line(),
        None,
        "nothing is said before the download begins"
    );
    let form = form.downloading();
    assert_eq!(form.step(), Step::Downloading);
    assert_eq!(
        form.download_line().as_deref(),
        Some("Downloading Codex\u{2026}")
    );
    assert_eq!(form.clone().pick("nobody"), form);
}

#[test]
fn a_download_that_fails_keeps_the_sheet_open_with_the_sentence() {
    let form = AddForm::loading(&[], &[])
        .catalogue(vec![online("codex", "Codex", Presence::Online)])
        .installing()
        .downloading()
        .install_failed("the archive did not match its checksum\n");
    assert_eq!((form.step(), form.button()), (Step::Idle, AddButton::Ready));
    assert_eq!(
        form.refusal.as_deref(),
        Some("the archive did not match its checksum")
    );
    assert_eq!(download_failure(""), DOWNLOAD_FAILED);
    assert_eq!(form.typed("x".to_owned()).refusal, None, "typing clears it");
}

#[test]
fn every_refusal_has_a_sentence_without_developer_words() {
    let adds = [
        AddFault::Unreadable,
        AddFault::Listed,
        AddFault::BadName,
        AddFault::BadLabel,
        AddFault::Io,
    ];
    let removes = [
        ChoiceFault::NoSuchAgent,
        ChoiceFault::Unreadable,
        ChoiceFault::BadModel,
        ChoiceFault::BadWay,
        ChoiceFault::Io,
    ];
    let sentences = adds
        .into_iter()
        .map(add_refusal)
        .chain(removes.into_iter().map(remove_refusal));
    for words in sentences {
        assert!(words.ends_with('.'), "{words}");
        for bad in ["agents.toml", "ACP", "registry", "TOML"] {
            assert!(!words.contains(bad), "{bad} in {words}");
        }
    }
    let form = AddForm::offline(&[found("gemini")], &[]);
    let refused = form.refused(AddFault::Listed);
    assert_eq!(
        refused.refusal.as_deref(),
        Some(add_refusal(AddFault::Listed))
    );
    assert_eq!(
        refused.typed("x".to_owned()).refusal,
        None,
        "typing clears it"
    );
}

#[test]
fn only_an_added_assistant_can_be_removed_and_the_question_says_what_stays() {
    let of = |source| Assistant {
        name: "Claude Code".to_owned(),
        agent: Some(row("claude-code", source)),
        account: None,
    };
    let question = removal_of(&of(registry("claude-code"))).expect("added ones go");
    assert_eq!(
        question,
        RemoveQuestion {
            program: "claude-code".to_owned(),
            label: "Claude Code".to_owned()
        }
    );
    assert_eq!(question.title(), "Remove Claude Code?");
    assert_eq!(
        question.message(),
        "It won\u{2019}t be offered any more. Its sign-in stays."
    );
    assert_eq!(removal_of(&of(own())), None);
    let account_only = Assistant {
        name: "x".to_owned(),
        agent: None,
        account: None,
    };
    assert_eq!(removal_of(&account_only), None);
}
