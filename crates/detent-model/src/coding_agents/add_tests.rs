use super::add::{
    AddForm, AddRefusal, FoundAgent, RemoveQuestion, Request, removal_of, unlisted_found,
};
use super::add_choice::{AddAction, Candidate, Listing, Presence, Step};
use crate::outside::Assistant;
use docket_settings::{AddFault, AgentRow, Install, ModelState, Offers, SignInState, Source};

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
        (AddAction::Off, Err(AddRefusal::NothingNew))
    );
    let form = AddForm::offline(&[found("claude-code"), found("gemini")], &[]);
    assert_eq!(form.label(), "Claude Code");
    let form = form.pick("gemini");
    assert_eq!((form.picked_id(), form.label()), ("gemini", "Gemini"));
    let unchanged = form.clone().pick("nobody");
    assert_eq!(unchanged, form, "an id not offered changes nothing");

    // `(typed, the name written, or the refusal it meets)`.
    let bad_label = Err(AddRefusal::Fault(AddFault::BadLabel));
    let cases = [
        ("  Gem  ", Ok("Gem")),
        ("", bad_label.clone()),
        ("   ", bad_label),
    ];
    for (typed, want) in cases {
        let asked = form.clone().typed(typed.to_owned()).request();
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
    assert_eq!(form.request(), Err(AddRefusal::Fault(AddFault::BadName)));
}

#[test]
fn the_sheet_waits_for_the_list_then_offers_it_without_what_is_added() {
    let rows = [row("claude-bin", registry("claude-code"))];
    let form = AddForm::loading(&[found("gemini")], &rows);
    assert_eq!(form.listing(), &Listing::Loading);
    assert_eq!(
        form.button(),
        AddAction::Off,
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
    assert_eq!((form.label(), form.button()), ("Codex", AddAction::Ready));
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
        &Listing::Failed(Some("no route to the host".to_owned()))
    );
    assert_eq!(
        (ids(&form), form.button()),
        (vec!["gemini"], AddAction::Ready)
    );
    let silent = AddForm::loading(&[], &[]).unreachable(" \n");
    assert_eq!(silent.listing(), &Listing::Failed(None));
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

    assert_eq!((form.step(), form.download_of()), (Step::Idle, None));
    let form = form.installing();
    assert_eq!(
        (form.step(), form.button()),
        (Step::Starting, AddAction::Busy)
    );
    assert_eq!(
        form.download_of(),
        None,
        "nothing is said before the download begins"
    );
    let form = form.downloading();
    assert_eq!(form.step(), Step::Downloading);
    assert_eq!(
        form.download_of().map(|one| one.name.as_str()),
        Some("Codex")
    );
    assert_eq!(form.clone().pick("nobody"), form);
}

#[test]
fn a_download_that_fails_keeps_the_sheet_open_with_what_was_said() {
    let form = AddForm::loading(&[], &[])
        .catalogue(vec![online("codex", "Codex", Presence::Online)])
        .installing()
        .downloading()
        .install_failed("the archive did not match its checksum\n");
    assert_eq!((form.step(), form.button()), (Step::Idle, AddAction::Ready));
    assert_eq!(
        form.refusal,
        Some(AddRefusal::DownloadSaid(
            "the archive did not match its checksum".to_owned()
        ))
    );
    let silent = AddForm::loading(&[], &[])
        .catalogue(vec![online("codex", "Codex", Presence::Online)])
        .install_failed("");
    assert_eq!(silent.refusal, Some(AddRefusal::DownloadFailed));
    assert_eq!(form.typed("x".to_owned()).refusal, None, "typing clears it");
}

#[test]
fn a_refused_write_is_kept_until_the_name_is_typed_again() {
    let form = AddForm::offline(&[found("gemini")], &[]);
    let refused = form.refused(AddFault::Listed);
    assert_eq!(refused.refusal, Some(AddRefusal::Fault(AddFault::Listed)));
    assert_eq!(
        refused.typed("x".to_owned()).refusal,
        None,
        "typing clears it"
    );
}

#[test]
fn only_an_added_assistant_can_be_removed() {
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
    assert_eq!(removal_of(&of(own())), None);
    let account_only = Assistant {
        name: "x".to_owned(),
        agent: None,
        account: None,
    };
    assert_eq!(removal_of(&account_only), None);
}
