use super::add::{
    AddForm, FoundAgent, RemoveQuestion, add_refusal, removal_of, remove_refusal, unlisted_found,
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

#[test]
fn the_form_names_the_pick_after_itself_and_checks_the_name() {
    assert!(AddForm::open(Vec::new()).is_none());
    let form = AddForm::open(vec![found("claude-code"), found("gemini")]).expect("choices");
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
        let got = asked.map(|new| new.label);
        let want = want.map(str::to_owned);
        assert_eq!(got, want, "{typed:?}");
    }
    let new = form.request().expect("fine");
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
    let form = AddForm::open(vec![found("odd+name")]).expect("choices");
    assert_eq!(form.request(), Err(add_refusal(AddFault::BadName)));
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
    let form = AddForm::open(vec![found("gemini")]).expect("choices");
    let refused = form.refused(AddFault::Listed);
    assert_eq!(refused.refusal, Some(add_refusal(AddFault::Listed)));
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
